use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::cron::CronStore;
use crate::familiar::Familiar;
use crate::heartbeat::Heartbeat;
use crate::mail::MailBus;

/// The Daemon: background process that runs the Familiar + all Witnesses + Heartbeats + Cron.
pub struct Daemon {
    pub familiar: Arc<Mutex<Familiar>>,
    pub mail_bus: Arc<MailBus>,
    pub patrol_interval_secs: u64,
    pub heartbeats: Vec<Heartbeat>,
    pub cron_store: Option<Arc<Mutex<CronStore>>>,
    pub pid_file: Option<PathBuf>,
    pub socket_path: Option<PathBuf>,
    running: Arc<std::sync::atomic::AtomicBool>,
    config_reloaded: Arc<std::sync::atomic::AtomicBool>,
}

impl Daemon {
    pub fn new(familiar: Familiar, mail_bus: Arc<MailBus>) -> Self {
        Self {
            familiar: Arc::new(Mutex::new(familiar)),
            mail_bus,
            patrol_interval_secs: 60,
            heartbeats: Vec::new(),
            cron_store: None,
            pid_file: None,
            socket_path: None,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            config_reloaded: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Add a heartbeat to the daemon.
    pub fn add_heartbeat(&mut self, heartbeat: Heartbeat) {
        self.heartbeats.push(heartbeat);
    }

    /// Set the cron store for scheduled jobs.
    pub fn set_cron_store(&mut self, store: CronStore) {
        self.cron_store = Some(Arc::new(Mutex::new(store)));
    }

    /// Set a PID file path (written on start, removed on stop).
    pub fn set_pid_file(&mut self, path: PathBuf) {
        self.pid_file = Some(path);
    }

    /// Set a Unix socket path for IPC.
    pub fn set_socket_path(&mut self, path: PathBuf) {
        self.socket_path = Some(path);
    }

    /// Write PID file.
    fn write_pid_file(&self) -> Result<()> {
        if let Some(ref path) = self.pid_file {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, std::process::id().to_string())?;
        }
        Ok(())
    }

    /// Remove PID file.
    fn remove_pid_file(&self) {
        if let Some(ref path) = self.pid_file {
            let _ = std::fs::remove_file(path);
        }
    }

    /// Check if a daemon is already running by reading the PID file.
    pub fn is_running_from_pid(pid_path: &Path) -> bool {
        if let Ok(content) = std::fs::read_to_string(pid_path) {
            if let Ok(pid) = content.trim().parse::<u32>() {
                // Check if process exists.
                return Path::new(&format!("/proc/{pid}")).exists();
            }
        }
        false
    }

    /// Start the daemon loop with graceful shutdown on Ctrl+C.
    pub async fn run(&mut self) -> Result<()> {
        self.running
            .store(true, std::sync::atomic::Ordering::SeqCst);

        self.write_pid_file()?;

        // Set up Ctrl+C handler.
        let running = self.running.clone();
        tokio::spawn(async move {
            if let Ok(()) = tokio::signal::ctrl_c().await {
                info!("received Ctrl+C, shutting down...");
                running.store(false, std::sync::atomic::Ordering::SeqCst);
            }
        });

        // Set up SIGHUP handler for config reload.
        #[cfg(unix)]
        {
            let config_reloaded = self.config_reloaded.clone();
            tokio::spawn(async move {
                let mut signal = tokio::signal::unix::signal(
                    tokio::signal::unix::SignalKind::hangup(),
                ).expect("failed to register SIGHUP handler");
                loop {
                    signal.recv().await;
                    info!("received SIGHUP, flagging config reload");
                    config_reloaded.store(true, std::sync::atomic::Ordering::SeqCst);
                }
            });
        }

        // Start Unix socket listener for IPC queries.
        #[cfg(unix)]
        if let Some(ref sock_path) = self.socket_path {
            // Remove stale socket file.
            let _ = std::fs::remove_file(sock_path);
            if let Some(parent) = sock_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match tokio::net::UnixListener::bind(sock_path) {
                Ok(listener) => {
                    let familiar = self.familiar.clone();
                    let mail_bus = self.mail_bus.clone();
                    let heartbeat_count = self.heartbeats.len();
                    let cron_store = self.cron_store.clone();
                    let running = self.running.clone();
                    info!(path = %sock_path.display(), "IPC socket listening");
                    tokio::spawn(async move {
                        Self::socket_accept_loop(
                            listener, familiar, mail_bus,
                            heartbeat_count, cron_store, running,
                        ).await;
                    });
                }
                Err(e) => {
                    warn!(error = %e, path = %sock_path.display(), "failed to bind IPC socket");
                }
            }
        }

        info!(
            heartbeats = self.heartbeats.len(),
            cron = self.cron_store.is_some(),
            "daemon started"
        );

        while self.running.load(std::sync::atomic::Ordering::SeqCst) {
            // 1. Patrol cycle (familiar + witnesses).
            {
                let mut familiar = self.familiar.lock().await;
                if let Err(e) = familiar.patrol().await {
                    warn!(error = %e, "patrol cycle failed");
                }

                // Execute any ready workers.
                let executed = familiar.execute_all().await;
                if executed > 0 {
                    info!(workers = executed, "executed workers");
                }
            }

            // 2. Run due heartbeats.
            for heartbeat in self.heartbeats.iter_mut() {
                if heartbeat.is_due() {
                    match heartbeat.run().await {
                        Ok(result) => {
                            info!(rig = %heartbeat.rig_name, "heartbeat completed");
                            let _ = result; // Result already logged/mailed by heartbeat.
                        }
                        Err(e) => {
                            warn!(rig = %heartbeat.rig_name, error = %e, "heartbeat failed");
                        }
                    }
                }
            }

            // 3. Run due cron jobs.
            if let Some(ref cron_store) = self.cron_store {
                let due_jobs = {
                    let store = cron_store.lock().await;
                    store.due_jobs()
                        .into_iter()
                        .map(|j| (j.name.clone(), j.rig.clone(), j.prompt.clone(), j.isolated))
                        .collect::<Vec<_>>()
                };

                for (name, rig, prompt, _isolated) in due_jobs {
                    info!(name = %name, rig = %rig, "cron job triggered");

                    // Assign the cron job's prompt as a bead.
                    {
                        let mut familiar = self.familiar.lock().await;
                        match familiar.assign(&rig, &format!("[cron] {name}"), &prompt).await {
                            Ok(bead) => {
                                info!(bead = %bead.id, "cron job created bead");
                            }
                            Err(e) => {
                                warn!(name = %name, error = %e, "cron job failed to create bead");
                            }
                        }
                    }

                    // Mark the job as run.
                    let mut store = cron_store.lock().await;
                    let _ = store.mark_run(&name);
                }

                // Cleanup completed one-shots.
                let mut store = cron_store.lock().await;
                let _ = store.cleanup_oneshots();
            }

            // 4. Check for config reload signal (SIGHUP).
            if self.config_reloaded.swap(false, std::sync::atomic::Ordering::SeqCst) {
                info!("config reload requested (SIGHUP received)");
                // The caller should re-read config and update state as needed.
                // For now we log it; full hot-reload requires rebuilding rigs/witnesses.
            }

            // Sleep until next patrol (interruptible).
            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_secs(self.patrol_interval_secs)) => {},
                _ = async {
                    while self.running.load(std::sync::atomic::Ordering::SeqCst) {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                } => break,
            }
        }

        self.remove_pid_file();
        self.remove_socket_file();
        info!("daemon stopped");
        Ok(())
    }

    /// Remove Unix socket file.
    fn remove_socket_file(&self) {
        if let Some(ref path) = self.socket_path {
            let _ = std::fs::remove_file(path);
        }
    }

    /// Accept loop for Unix socket IPC connections.
    #[cfg(unix)]
    async fn socket_accept_loop(
        listener: tokio::net::UnixListener,
        familiar: Arc<Mutex<Familiar>>,
        mail_bus: Arc<MailBus>,
        heartbeat_count: usize,
        cron_store: Option<Arc<Mutex<CronStore>>>,
        running: Arc<std::sync::atomic::AtomicBool>,
    ) {
        loop {
            if !running.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            match listener.accept().await {
                Ok((stream, _)) => {
                    let familiar = familiar.clone();
                    let mail_bus = mail_bus.clone();
                    let cron_store = cron_store.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_socket_connection(
                            stream, familiar, mail_bus,
                            heartbeat_count, cron_store,
                        ).await {
                            debug!(error = %e, "IPC connection error");
                        }
                    });
                }
                Err(e) => {
                    warn!(error = %e, "IPC accept error");
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Handle a single IPC connection. Protocol: one JSON line in, one JSON line out.
    ///
    /// Supported commands:
    ///   {"cmd": "status"} → rig summary, mail count, heartbeats, cron jobs
    ///   {"cmd": "ping"} → {"ok": true, "pong": true}
    ///   {"cmd": "mail"} → pending mail messages
    ///   {"cmd": "rigs"} → list of registered rigs
    #[cfg(unix)]
    async fn handle_socket_connection(
        stream: tokio::net::UnixStream,
        familiar: Arc<Mutex<Familiar>>,
        mail_bus: Arc<MailBus>,
        heartbeat_count: usize,
        cron_store: Option<Arc<Mutex<CronStore>>>,
    ) -> Result<()> {
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        while let Some(line) = lines.next_line().await? {
            let request: serde_json::Value = serde_json::from_str(&line)
                .unwrap_or_else(|_| serde_json::json!({"cmd": "unknown"}));

            let cmd = request.get("cmd").and_then(|v| v.as_str()).unwrap_or("unknown");

            let response = match cmd {
                "ping" => serde_json::json!({"ok": true, "pong": true}),

                "status" => {
                    let fam = familiar.lock().await;
                    let rig_names: Vec<String> = fam.rigs.values().map(|r| r.name.clone()).collect();
                    let worker_count: u32 = fam.rigs.values().map(|r| r.max_workers).sum();
                    let mail_count = mail_bus.pending_count();
                    let cron_count = if let Some(ref cs) = cron_store {
                        cs.lock().await.jobs.len()
                    } else {
                        0
                    };

                    serde_json::json!({
                        "ok": true,
                        "rigs": rig_names,
                        "rig_count": rig_names.len(),
                        "max_workers": worker_count,
                        "heartbeats": heartbeat_count,
                        "cron_jobs": cron_count,
                        "pending_mail": mail_count,
                    })
                }

                "rigs" => {
                    let fam = familiar.lock().await;
                    let rigs: Vec<serde_json::Value> = fam.rigs.values().map(|r| {
                        serde_json::json!({
                            "name": r.name,
                            "prefix": r.prefix,
                            "model": r.model,
                            "max_workers": r.max_workers,
                        })
                    }).collect();
                    serde_json::json!({"ok": true, "rigs": rigs})
                }

                "mail" => {
                    let messages = mail_bus.drain();
                    let msgs: Vec<serde_json::Value> = messages.iter().map(|m| {
                        serde_json::json!({
                            "from": m.from,
                            "to": m.to,
                            "subject": m.subject,
                            "body": m.body,
                        })
                    }).collect();
                    serde_json::json!({"ok": true, "messages": msgs})
                }

                _ => serde_json::json!({"ok": false, "error": format!("unknown command: {cmd}")}),
            };

            let mut resp_bytes = serde_json::to_vec(&response)?;
            resp_bytes.push(b'\n');
            writer.write_all(&resp_bytes).await?;
        }

        Ok(())
    }

    /// Stop the daemon.
    pub fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    /// Check if daemon is running.
    pub fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::SeqCst)
    }
}
