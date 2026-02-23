use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::fate::FateStore;
use crate::pulse::Pulse;
use crate::whisper::WhisperBus;
use crate::registry::DomainRegistry;

/// The Summoner: background process that runs the DomainRegistry patrol loop,
/// pulses, and cron jobs.
pub struct Summoner {
    pub registry: Arc<DomainRegistry>,
    pub whisper_bus: Arc<WhisperBus>,
    pub patrol_interval_secs: u64,
    pub pulses: Vec<Pulse>,
    pub fate_store: Option<Arc<Mutex<FateStore>>>,
    pub pid_file: Option<PathBuf>,
    pub socket_path: Option<PathBuf>,
    running: Arc<std::sync::atomic::AtomicBool>,
    config_reloaded: Arc<std::sync::atomic::AtomicBool>,
}

impl Summoner {
    pub fn new(registry: Arc<DomainRegistry>, whisper_bus: Arc<WhisperBus>) -> Self {
        Self {
            registry,
            whisper_bus,
            patrol_interval_secs: 30,
            pulses: Vec::new(),
            fate_store: None,
            pid_file: None,
            socket_path: None,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            config_reloaded: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Add a pulse to the daemon.
    pub fn add_pulse(&mut self, pulse: Pulse) {
        self.pulses.push(pulse);
    }

    /// Set the cron store for scheduled jobs.
    pub fn set_fate_store(&mut self, store: FateStore) {
        self.fate_store = Some(Arc::new(Mutex::new(store)));
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
        if let Ok(content) = std::fs::read_to_string(pid_path)
            && let Ok(pid) = content.trim().parse::<u32>() {
                // Check if process exists.
                return Path::new(&format!("/proc/{pid}")).exists();
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
                    let registry = self.registry.clone();
                    let whisper_bus = self.whisper_bus.clone();
                    let pulse_count = self.pulses.len();
                    let fate_store = self.fate_store.clone();
                    let running = self.running.clone();
                    info!(path = %sock_path.display(), "IPC socket listening");
                    tokio::spawn(async move {
                        Self::socket_accept_loop(
                            listener, registry, whisper_bus,
                            pulse_count, fate_store, running,
                        ).await;
                    });
                }
                Err(e) => {
                    warn!(error = %e, path = %sock_path.display(), "failed to bind IPC socket");
                }
            }
        }

        // Load persisted state from disk.
        match self.whisper_bus.load().await {
            Ok(n) if n > 0 => info!(count = n, "loaded persisted whispers"),
            Ok(_) => {}
            Err(e) => warn!(error = %e, "failed to load whisper bus"),
        }
        match self.registry.cost_ledger.load() {
            Ok(n) if n > 0 => info!(count = n, "loaded persisted cost entries"),
            Ok(_) => {}
            Err(e) => warn!(error = %e, "failed to load cost ledger"),
        }

        info!(
            pulses = self.pulses.len(),
            cron = self.fate_store.is_some(),
            "daemon started"
        );

        while self.running.load(std::sync::atomic::Ordering::SeqCst) {
            // 1. Patrol cycle: reap finished workers, assign + launch new ones (non-blocking).
            if let Err(e) = self.registry.patrol_all().await {
                warn!(error = %e, "patrol cycle failed");
            }

            // 2. Run due pulses.
            for pulse in self.pulses.iter_mut() {
                if pulse.is_due() {
                    match pulse.run().await {
                        Ok(result) => {
                            info!(rig = %pulse.domain_name, "pulse completed");
                            let _ = result;
                        }
                        Err(e) => {
                            warn!(rig = %pulse.domain_name, error = %e, "pulse failed");
                        }
                    }
                }
            }

            // 3. Run due cron jobs.
            if let Some(ref fate_store) = self.fate_store {
                let due_jobs = {
                    let store = fate_store.lock().await;
                    store.due_jobs()
                        .into_iter()
                        .map(|j| (j.name.clone(), j.rig.clone(), j.prompt.clone(), j.isolated))
                        .collect::<Vec<_>>()
                };

                for (name, rig, prompt, _isolated) in due_jobs {
                    info!(name = %name, rig = %rig, "cron job triggered");

                    match self.registry.assign(&rig, &format!("[cron] {name}"), &prompt).await {
                        Ok(bead) => {
                            info!(bead = %bead.id, "cron job created bead");
                        }
                        Err(e) => {
                            warn!(name = %name, error = %e, "cron job failed to create bead");
                        }
                    }

                    let mut store = fate_store.lock().await;
                    let _ = store.mark_run(&name);
                }

                // Cleanup completed one-shots.
                let mut store = fate_store.lock().await;
                let _ = store.cleanup_oneshots();
            }

            // 4. Check for config reload signal (SIGHUP).
            if self.config_reloaded.swap(false, std::sync::atomic::Ordering::SeqCst) {
                info!("config reload requested (SIGHUP received)");
                match realm_core::config::RealmConfig::discover() {
                    Ok((_new_config, _path)) => {
                        info!("config reloaded successfully via SIGHUP");
                    }
                    Err(e) => {
                        warn!(error = %e, "failed to reload config, keeping current");
                    }
                }
            }

            // 5. Periodic persistence: save whisper bus + cost ledger every patrol.
            if let Err(e) = self.whisper_bus.save().await {
                warn!(error = %e, "failed to save whisper bus");
            }
            if let Err(e) = self.registry.cost_ledger.save() {
                warn!(error = %e, "failed to save cost ledger");
            }

            // 6. Update daily cost gauge.
            let (spent, _, _) = self.registry.cost_ledger.budget_status();
            self.registry.metrics.daily_cost_usd.set(spent);
            let pending_whispers = self.whisper_bus.pending_count();
            self.registry.metrics.whisper_queue_depth.set(pending_whispers as f64);

            // 7. Prune old cost entries (older than 7 days) every cycle.
            self.registry.cost_ledger.prune_old();

            // Sleep until next patrol — wakes instantly on new bead, or after interval.
            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_secs(self.patrol_interval_secs)) => {},
                _ = self.registry.wake.notified() => {
                    debug!("woken by new bead");
                },
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
        registry: Arc<DomainRegistry>,
        whisper_bus: Arc<WhisperBus>,
        pulse_count: usize,
        fate_store: Option<Arc<Mutex<FateStore>>>,
        running: Arc<std::sync::atomic::AtomicBool>,
    ) {
        loop {
            if !running.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            match listener.accept().await {
                Ok((stream, _)) => {
                    let registry = registry.clone();
                    let whisper_bus = whisper_bus.clone();
                    let fate_store = fate_store.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_socket_connection(
                            stream, registry, whisper_bus,
                            pulse_count, fate_store,
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
    #[cfg(unix)]
    async fn handle_socket_connection(
        stream: tokio::net::UnixStream,
        registry: Arc<DomainRegistry>,
        whisper_bus: Arc<WhisperBus>,
        pulse_count: usize,
        fate_store: Option<Arc<Mutex<FateStore>>>,
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
                    let domain_names = registry.domain_names().await;
                    let worker_count = registry.total_max_spirits().await;
                    let mail_count = whisper_bus.pending_count();
                    let cron_count = if let Some(ref cs) = fate_store {
                        cs.lock().await.jobs.len()
                    } else {
                        0
                    };

                    let (spent, budget, remaining) = registry.cost_ledger.budget_status();
                    let domain_budgets = registry.cost_ledger.all_domain_budget_statuses();
                    let domain_budget_info: serde_json::Map<String, serde_json::Value> = domain_budgets
                        .into_iter()
                        .map(|(name, (spent, budget, remaining))| {
                            (name, serde_json::json!({
                                "spent_usd": spent,
                                "budget_usd": budget,
                                "remaining_usd": remaining,
                            }))
                        })
                        .collect();

                    serde_json::json!({
                        "ok": true,
                        "rigs": domain_names,
                        "domain_count": domain_names.len(),
                        "max_workers": worker_count,
                        "pulses": pulse_count,
                        "cron_jobs": cron_count,
                        "pending_mail": mail_count,
                        "cost_today_usd": spent,
                        "daily_budget_usd": budget,
                        "budget_remaining_usd": remaining,
                        "domain_budgets": domain_budget_info,
                    })
                }

                "rigs" => {
                    let domains = registry.domains_info().await;
                    serde_json::json!({"ok": true, "domains": domains})
                }

                "mail" => {
                    let messages = whisper_bus.drain();
                    let msgs: Vec<serde_json::Value> = messages.iter().map(|m| {
                        serde_json::json!({
                            "from": m.from,
                            "to": m.to,
                            "subject": m.kind.subject_tag(),
                            "body": m.kind.body_text(),
                        })
                    }).collect();
                    serde_json::json!({"ok": true, "messages": msgs})
                }

                "metrics" => {
                    let text = registry.metrics.render();
                    serde_json::json!({"ok": true, "metrics": text})
                }

                "cost" => {
                    let (spent, budget, remaining) = registry.cost_ledger.budget_status();
                    let report = registry.cost_ledger.daily_report();
                    let domain_budgets = registry.cost_ledger.all_domain_budget_statuses();
                    let domain_budget_info: serde_json::Map<String, serde_json::Value> = domain_budgets
                        .into_iter()
                        .map(|(name, (spent, budget, remaining))| {
                            (name, serde_json::json!({
                                "spent_usd": spent,
                                "budget_usd": budget,
                                "remaining_usd": remaining,
                            }))
                        })
                        .collect();
                    serde_json::json!({
                        "ok": true,
                        "spent_today_usd": spent,
                        "daily_budget_usd": budget,
                        "remaining_usd": remaining,
                        "per_domain": report,
                        "domain_budgets": domain_budget_info,
                    })
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
