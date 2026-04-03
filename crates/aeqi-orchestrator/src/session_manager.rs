//! Session Manager — holds running agent sessions in memory.
//!
//! Each running session is a spawned `Agent::run()` task with a perpetual input
//! channel. Messages are injected via `input_tx`, responses collected via
//! `ChatStreamSender` broadcast. Sessions persist until explicitly closed (which
//! drops the input channel, causing the agent loop to exit).
//!
//! Two kinds of sessions:
//! - **Permanent**: one per agent, always alive, IS the agent's identity
//! - **Spawned**: created by triggers, skills, or users — persistent until closed

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tracing::{debug, info, warn};

use aeqi_core::AgentResult;
use aeqi_core::chat_stream::{ChatStreamEvent, ChatStreamSender};

/// A running agent session — the in-memory handle to a live agent loop.
pub struct RunningSession {
    pub session_id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub input_tx: mpsc::UnboundedSender<String>,
    pub stream_sender: ChatStreamSender,
    pub cancel_token: Arc<std::sync::atomic::AtomicBool>,
    pub join_handle: tokio::task::JoinHandle<anyhow::Result<AgentResult>>,
    pub chat_id: i64,
}

impl RunningSession {
    /// Send a message and wait for the agent's response.
    ///
    /// Subscribes to the stream, pushes the message, collects TextDelta events
    /// until a Complete event arrives. Returns the accumulated response text
    /// and token counts.
    pub async fn send_and_wait(&self, message: &str) -> anyhow::Result<SessionResponse> {
        // Subscribe BEFORE pushing so we don't miss events.
        let mut rx = self.stream_sender.subscribe();

        // Push message into the agent loop.
        self.input_tx
            .send(message.to_string())
            .map_err(|_| anyhow::anyhow!("session closed — agent loop exited"))?;

        // Collect response.
        let mut text = String::new();
        let mut iterations = 0u32;
        let mut prompt_tokens = 0u32;
        let mut completion_tokens = 0u32;

        loop {
            match tokio::time::timeout(std::time::Duration::from_secs(300), rx.recv()).await {
                Ok(Ok(event)) => match event {
                    ChatStreamEvent::TextDelta { text: delta } => {
                        text.push_str(&delta);
                    }
                    ChatStreamEvent::Complete {
                        total_prompt_tokens,
                        total_completion_tokens,
                        iterations: iters,
                        ..
                    } => {
                        prompt_tokens = total_prompt_tokens;
                        completion_tokens = total_completion_tokens;
                        iterations = iters;
                        break;
                    }
                    _ => {
                        // TurnStart, ToolStart, ToolComplete, etc. — skip.
                    }
                },
                Ok(Err(tokio::sync::broadcast::error::RecvError::Lagged(n))) => {
                    warn!(lagged = n, "stream subscriber lagged — some events lost");
                }
                Ok(Err(tokio::sync::broadcast::error::RecvError::Closed)) => {
                    // Agent loop ended without a Complete event.
                    break;
                }
                Err(_) => {
                    return Err(anyhow::anyhow!("session response timed out (300s)"));
                }
            }
        }

        Ok(SessionResponse {
            text,
            iterations,
            prompt_tokens,
            completion_tokens,
        })
    }

    /// Check if the agent loop is still running.
    pub fn is_alive(&self) -> bool {
        !self.join_handle.is_finished()
    }
}

/// Response from a session send.
pub struct SessionResponse {
    pub text: String,
    pub iterations: u32,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

/// Manages all running agent sessions in the daemon.
pub struct SessionManager {
    sessions: Mutex<HashMap<String, RunningSession>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    /// Register a running session.
    pub async fn register(&self, session: RunningSession) {
        let session_id = session.session_id.clone();
        let agent_name = session.agent_name.clone();
        info!(session_id = %session_id, agent = %agent_name, "session registered");
        self.sessions.lock().await.insert(session_id, session);
    }

    /// Get a reference to a running session for sending messages.
    /// Returns None if session doesn't exist or agent loop has exited.
    pub async fn get(&self, session_id: &str) -> Option<()> {
        let sessions = self.sessions.lock().await;
        sessions
            .get(session_id)
            .and_then(|s| if s.is_alive() { Some(()) } else { None })
    }

    /// Send a message to a running session and wait for the response.
    pub async fn send(&self, session_id: &str, message: &str) -> anyhow::Result<SessionResponse> {
        let sessions = self.sessions.lock().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| anyhow::anyhow!("session '{}' not running", session_id))?;

        if !session.is_alive() {
            return Err(anyhow::anyhow!(
                "session '{}' agent loop has exited",
                session_id
            ));
        }

        session.send_and_wait(message).await
    }

    /// Subscribe to a session's stream for real-time events.
    pub async fn subscribe(
        &self,
        session_id: &str,
    ) -> Option<tokio::sync::broadcast::Receiver<ChatStreamEvent>> {
        let sessions = self.sessions.lock().await;
        sessions
            .get(session_id)
            .map(|s| s.stream_sender.subscribe())
    }

    /// Inject a message into a running session without waiting for the response.
    /// Returns a broadcast receiver for streaming events. The caller reads events
    /// from the receiver until Complete arrives.
    pub async fn send_streaming(
        &self,
        session_id: &str,
        message: &str,
    ) -> anyhow::Result<tokio::sync::broadcast::Receiver<ChatStreamEvent>> {
        let sessions = self.sessions.lock().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| anyhow::anyhow!("session '{}' not running", session_id))?;

        if !session.is_alive() {
            return Err(anyhow::anyhow!(
                "session '{}' agent loop has exited",
                session_id
            ));
        }

        // Subscribe BEFORE pushing so we don't miss events.
        let rx = session.stream_sender.subscribe();

        session
            .input_tx
            .send(message.to_string())
            .map_err(|_| anyhow::anyhow!("session closed — agent loop exited"))?;

        Ok(rx)
    }

    /// Remove and shut down a session. Drops input_tx which causes the agent
    /// loop to exit at the next await point.
    pub async fn close(&self, session_id: &str) -> bool {
        let removed = self.sessions.lock().await.remove(session_id);
        if let Some(session) = removed {
            info!(
                session_id = %session_id,
                agent = %session.agent_name,
                "session closed — dropping input channel"
            );
            // Drop input_tx — agent loop sees None from recv() and exits.
            drop(session.input_tx);
            // Cancel token as backup.
            session
                .cancel_token
                .store(true, std::sync::atomic::Ordering::Relaxed);
            true
        } else {
            debug!(session_id = %session_id, "close: session not found (already stopped?)");
            false
        }
    }

    /// Reap dead sessions (agent loops that exited on their own).
    pub async fn reap_dead(&self) {
        let mut sessions = self.sessions.lock().await;
        let dead: Vec<String> = sessions
            .iter()
            .filter(|(_, s)| !s.is_alive())
            .map(|(id, _)| id.clone())
            .collect();

        for id in &dead {
            sessions.remove(id);
        }

        if !dead.is_empty() {
            info!(count = dead.len(), "reaped dead sessions: {:?}", dead);
        }
    }

    /// List all running session IDs.
    pub async fn list_running(&self) -> Vec<String> {
        self.sessions.lock().await.keys().cloned().collect()
    }

    /// Check if a session is running.
    pub async fn is_running(&self, session_id: &str) -> bool {
        let sessions = self.sessions.lock().await;
        sessions.get(session_id).is_some_and(|s| s.is_alive())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
