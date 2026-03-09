use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::schedule::ScheduleStore;
use crate::heartbeat::Heartbeat;
use crate::reflection::Reflection;
use crate::session_tracker::SessionTracker;
use crate::message::DispatchBus;
use crate::lifecycle::LifecycleEngine;
use crate::registry::ProjectRegistry;
use crate::watchdog::WatchdogEngine;

const ACK_RETRY_AGE_SECS: u64 = 60;

/// The Daemon: background process that runs the ProjectRegistry patrol loop,
/// pulses, and cron jobs.
pub struct Daemon {
    pub registry: Arc<ProjectRegistry>,
    pub dispatch_bus: Arc<DispatchBus>,
    pub patrol_interval_secs: u64,
    pub pulses: Vec<Heartbeat>,
    pub reflections: Vec<Reflection>,
    pub lifecycle: Option<LifecycleEngine>,
    pub cron_store: Option<Arc<Mutex<ScheduleStore>>>,
    pub watchdog: Option<WatchdogEngine>,
    pub pid_file: Option<PathBuf>,
    pub socket_path: Option<PathBuf>,
    session_tracker_shutdown: Option<Arc<tokio::sync::Notify>>,
    running: Arc<std::sync::atomic::AtomicBool>,
    config_reloaded: Arc<std::sync::atomic::AtomicBool>,
    shutdown_notify: Arc<tokio::sync::Notify>,
}

impl Daemon {
    pub fn new(registry: Arc<ProjectRegistry>, dispatch_bus: Arc<DispatchBus>) -> Self {
        Self {
            registry,
            dispatch_bus,
            patrol_interval_secs: 30,
            pulses: Vec::new(),
            reflections: Vec::new(),
            lifecycle: None,
            cron_store: None,
            watchdog: None,
            pid_file: None,
            socket_path: None,
            session_tracker_shutdown: None,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            config_reloaded: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            shutdown_notify: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// Add a heartbeat to the daemon.
    pub fn add_heartbeat(&mut self, heartbeat: Heartbeat) {
        self.pulses.push(heartbeat);
    }

    /// Add a reflection cycle to the daemon.
    pub fn add_reflection(&mut self, reflection: Reflection) {
        self.reflections.push(reflection);
    }

    /// Start the session tracker in a dedicated tokio::spawn.
    /// Returns the shutdown Notify so it can be stopped later.
    pub fn start_session_tracker(&mut self, tracker: SessionTracker) {
        let shutdown = Arc::new(tokio::sync::Notify::new());
        let shutdown_clone = shutdown.clone();
        tokio::spawn(async move {
            tracker.run(shutdown_clone).await;
        });
        self.session_tracker_shutdown = Some(shutdown);
        info!("session tracker launched");
    }

    /// Stop the session tracker if running.
    pub fn stop_session_tracker(&mut self) {
        if let Some(notify) = self.session_tracker_shutdown.take() {
            notify.notify_waiters();
            info!("session tracker stopped");
        }
    }

    /// Set the lifecycle engine for autonomous agent processes.
    pub fn set_lifecycle(&mut self, engine: LifecycleEngine) {
        self.lifecycle = Some(engine);
    }

    /// Set the cron store for scheduled jobs.
    pub fn set_cron_store(&mut self, store: ScheduleStore) {
        self.cron_store = Some(Arc::new(Mutex::new(store)));
    }

    /// Set the watchdog engine for event-driven automation.
    pub fn set_watchdog(&mut self, engine: WatchdogEngine) {
        self.watchdog = Some(engine);
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

        let running = self.running.clone();
        let shutdown_notify = self.shutdown_notify.clone();
        tokio::spawn(async move {
            if let Ok(()) = tokio::signal::ctrl_c().await {
                info!("received Ctrl+C, shutting down...");
                running.store(false, std::sync::atomic::Ordering::SeqCst);
                shutdown_notify.notify_waiters();
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

        // Set up SIGTERM handler for graceful shutdown (e.g. `rm daemon stop`, Docker, systemd).
        #[cfg(unix)]
        {
            let running = self.running.clone();
            let shutdown_notify = self.shutdown_notify.clone();
            tokio::spawn(async move {
                let mut signal = tokio::signal::unix::signal(
                    tokio::signal::unix::SignalKind::terminate(),
                ).expect("failed to register SIGTERM handler");
                signal.recv().await;
                info!("received SIGTERM, shutting down...");
                running.store(false, std::sync::atomic::Ordering::SeqCst);
                shutdown_notify.notify_waiters();
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
                    let dispatch_bus = self.dispatch_bus.clone();
                    let pulse_count = self.pulses.len();
                    let cron_store = self.cron_store.clone();
                    let running = self.running.clone();
                    info!(path = %sock_path.display(), "IPC socket listening");
                    tokio::spawn(async move {
                        Self::socket_accept_loop(
                            listener, registry, dispatch_bus,
                            pulse_count, cron_store, running,
                        ).await;
                    });
                }
                Err(e) => {
                    warn!(error = %e, path = %sock_path.display(), "failed to bind IPC socket");
                }
            }
        }

        // Load persisted state from disk.
        match self.dispatch_bus.load().await {
            Ok(n) if n > 0 => info!(count = n, "loaded persisted dispatches"),
            Ok(_) => {}
            Err(e) => warn!(error = %e, "failed to load dispatch bus"),
        }
        match self.registry.cost_ledger.load() {
            Ok(n) if n > 0 => info!(count = n, "loaded persisted cost entries"),
            Ok(_) => {}
            Err(e) => warn!(error = %e, "failed to load cost ledger"),
        }

        info!(
            pulses = self.pulses.len(),
            cron = self.cron_store.is_some(),
            "daemon started"
        );

        while self.running.load(std::sync::atomic::Ordering::SeqCst) {
            // 1. Patrol cycle: reap finished workers, assign + launch new ones (non-blocking).
            if let Err(e) = self.registry.patrol_all().await {
                warn!(error = %e, "patrol cycle failed");
            }

            // 2. Run due heartbeats.
            for heartbeat in self.pulses.iter_mut() {
                if heartbeat.is_due() {
                    match heartbeat.run().await {
                        Ok(result) => {
                            info!(project = %heartbeat.project_name, "heartbeat completed");
                            let _ = result;
                        }
                        Err(e) => {
                            warn!(project = %heartbeat.project_name, error = %e, "heartbeat failed");
                        }
                    }
                }
            }

            // 3. Run due reflections (self-examination of identity files).
            for reflection in self.reflections.iter_mut() {
                if reflection.is_due() {
                    match reflection.run().await {
                        Ok(result) => {
                            info!(project = %reflection.project_name, result = %result, "reflection completed");
                        }
                        Err(e) => {
                            warn!(project = %reflection.project_name, error = %e, "reflection failed");
                        }
                    }
                }
            }

            // 4. Run due lifecycle processes (autonomous agent evolution).
            if let Some(ref mut lifecycle) = self.lifecycle {
                for result in lifecycle.tick().await {
                    if let Some(ref err) = result.error {
                        warn!(agent=%result.agent, process=%result.process, error=%err, "lifecycle failed");
                    } else {
                        info!(agent=%result.agent, process=%result.process, summary=%result.summary,
                            cost_usd=%result.cost_usd, "lifecycle completed");
                    }
                }
            }

            // 5. Run due cron jobs.
            if let Some(ref cron_store) = self.cron_store {
                let due_jobs = {
                    let store = cron_store.lock().await;
                    store.due_jobs()
                        .into_iter()
                        .map(|j| (j.name.clone(), j.project.clone(), j.prompt.clone(), j.isolated))
                        .collect::<Vec<_>>()
                };

                for (name, project, prompt, _isolated) in due_jobs {
                    info!(name = %name, project = %project, "cron job triggered");

                    match self.registry.assign(&project, &format!("[cron] {name}"), &prompt).await {
                        Ok(task) => {
                            info!(task = %task.id, "cron job created task");
                        }
                        Err(e) => {
                            warn!(name = %name, error = %e, "cron job failed to create task");
                        }
                    }

                    let mut store = cron_store.lock().await;
                    let _ = store.mark_run(&name);
                }

                // Cleanup completed one-shots.
                let mut store = cron_store.lock().await;
                let _ = store.cleanup_oneshots();
            }

            // 6. Check for config reload signal (SIGHUP).
            if self.config_reloaded.swap(false, std::sync::atomic::Ordering::SeqCst) {
                info!("config reload requested (SIGHUP received)");
                match sigil_core::config::SigilConfig::discover() {
                    Ok((new_config, path)) => {
                        // Apply runtime-safe fields from the reloaded config.

                        // (a) Global daily budget.
                        self.registry.cost_ledger.set_daily_budget(new_config.security.max_cost_per_day_usd);

                        // (b) Per-project budgets + worker counts + orchestrator params.
                        let orch = &new_config.orchestrator;
                        for pcfg in &new_config.projects {
                            if let Some(budget) = pcfg.max_cost_per_day_usd {
                                self.registry.cost_ledger.set_project_budget(&pcfg.name, budget);
                            }

                            // Update supervisor parameters.
                            if let Some(sup) = self.registry.get_supervisor(&pcfg.name).await {
                                let mut s = sup.lock().await;
                                s.max_workers = pcfg.max_workers;

                                // Apply orchestrator config (per-project override or global).
                                let proj_orch = pcfg.orchestrator.as_ref().unwrap_or(orch);
                                s.max_resolution_attempts = proj_orch.max_resolution_attempts;
                                s.max_description_chars = proj_orch.max_description_chars;
                                s.max_task_retries = proj_orch.max_task_retries;

                                debug!(
                                    project = %pcfg.name,
                                    max_workers = s.max_workers,
                                    max_retries = s.max_task_retries,
                                    "supervisor config updated via SIGHUP"
                                );
                            }
                        }

                        // (c) Patrol interval.
                        if let Some(interval) = new_config.sigil.patrol_interval_secs {
                            self.patrol_interval_secs = interval;
                        }

                        info!(path = %path.display(), "config reloaded and applied via SIGHUP");
                    }
                    Err(e) => {
                        warn!(error = %e, "failed to reload config, keeping current");
                    }
                }
            }

            // 7. Periodic persistence: save dispatch bus + cost ledger every patrol.
            if let Err(e) = self.dispatch_bus.save().await {
                warn!(error = %e, "failed to save dispatch bus");
            }
            if let Err(e) = self.registry.cost_ledger.save() {
                warn!(error = %e, "failed to save cost ledger");
            }

            // 8. Surface dispatch retries / dead letters for critical mail.
            let retried = self.dispatch_bus.retry_unacked(ACK_RETRY_AGE_SECS).await;
            for dispatch in &retried {
                warn!(
                    to = %dispatch.to,
                    subject = %dispatch.kind.subject_tag(),
                    retry = dispatch.retry_count,
                    "retrying unacknowledged dispatch"
                );
            }
            let dead_letters = self.dispatch_bus.dead_letters().await;
            for dispatch in &dead_letters {
                warn!(
                    to = %dispatch.to,
                    subject = %dispatch.kind.subject_tag(),
                    retries = dispatch.retry_count,
                    "dispatch moved to dead-letter state"
                );
            }

            // 9. Update daily cost gauge.
            let (spent, _, _) = self.registry.cost_ledger.budget_status();
            self.registry.metrics.daily_cost_usd.set(spent);
            let pending_dispatches = self.dispatch_bus.pending_count();
            self.registry.metrics.dispatch_queue_depth.set(pending_dispatches as f64);

            // 10. Prune old cost entries (older than 7 days) every cycle.
            self.registry.cost_ledger.prune_old();

            // 11. Prune expired blackboard entries.
            if let Some(ref bb) = self.registry.blackboard
                && let Err(e) = bb.prune_expired()
            {
                warn!(error = %e, "failed to prune blackboard");
            }

            // 12. Evaluate watchdog rules.
            if let Some(ref mut watchdog) = self.watchdog
                && let Some(ref audit) = self.registry.audit_log
            {
                let (spent, budget, _) = self.registry.cost_ledger.budget_status();
                let budget_pct = if budget > 0.0 { Some(spent / budget) } else { None };
                let fired = watchdog.evaluate(audit, budget_pct);
                for (name, _action) in &fired {
                    info!(rule = %name, "watchdog rule fired");
                }
            }

            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_secs(self.patrol_interval_secs)) => {},
                _ = self.registry.wake.notified() => {
                    debug!("woken by new task");
                },
                _ = self.shutdown_notify.notified() => break,
            }
        }

        self.stop_session_tracker();
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
        registry: Arc<ProjectRegistry>,
        dispatch_bus: Arc<DispatchBus>,
        pulse_count: usize,
        cron_store: Option<Arc<Mutex<ScheduleStore>>>,
        running: Arc<std::sync::atomic::AtomicBool>,
    ) {
        loop {
            if !running.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            match listener.accept().await {
                Ok((stream, _)) => {
                    let registry = registry.clone();
                    let dispatch_bus = dispatch_bus.clone();
                    let cron_store = cron_store.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_socket_connection(
                            stream, registry, dispatch_bus,
                            pulse_count, cron_store,
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
        registry: Arc<ProjectRegistry>,
        dispatch_bus: Arc<DispatchBus>,
        pulse_count: usize,
        cron_store: Option<Arc<Mutex<ScheduleStore>>>,
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
                    let project_names: Vec<String> = registry.project_names().await;
                    let worker_count = registry.total_max_workers().await;
                    let mail_count = dispatch_bus.pending_count();
                    let cron_count = if let Some(ref cs) = cron_store {
                        cs.lock().await.jobs.len()
                    } else {
                        0
                    };

                    let (spent, budget, remaining) = registry.cost_ledger.budget_status();
                    let project_budgets = registry.cost_ledger.all_project_budget_statuses();
                    let project_budget_info: serde_json::Map<String, serde_json::Value> = project_budgets
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
                        "projects": project_names,
                        "project_count": project_names.len(),
                        "max_workers": worker_count,
                        "pulses": pulse_count,
                        "cron_jobs": cron_count,
                        "pending_mail": mail_count,
                        "cost_today_usd": spent,
                        "daily_budget_usd": budget,
                        "budget_remaining_usd": remaining,
                        "project_budgets": project_budget_info,
                    })
                }

                "projects" => {
                    let projects = registry.projects_info().await;
                    serde_json::json!({"ok": true, "projects": projects})
                }

                "mail" => {
                    let messages = dispatch_bus.drain();
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
                    let project_budgets = registry.cost_ledger.all_project_budget_statuses();
                    let project_budget_info: serde_json::Map<String, serde_json::Value> = project_budgets
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
                        "per_project": report,
                        "project_budgets": project_budget_info,
                    })
                }

                "audit" => {
                    let project_filter = request.get("project").and_then(|v| v.as_str());
                    let last = request.get("last").and_then(|v| v.as_u64()).unwrap_or(20) as u32;
                    match &registry.audit_log {
                        Some(audit) => {
                            let events = if let Some(proj) = project_filter {
                                audit.query_by_project(proj).unwrap_or_default()
                            } else {
                                audit.query_recent(last).unwrap_or_default()
                            };
                            let items: Vec<serde_json::Value> = events.iter().map(|e| {
                                serde_json::json!({
                                    "timestamp": e.timestamp.to_rfc3339(),
                                    "project": e.project,
                                    "decision_type": e.decision_type.to_string(),
                                    "task_id": e.task_id,
                                    "agent": e.agent,
                                    "reasoning": e.reasoning,
                                })
                            }).collect();
                            serde_json::json!({"ok": true, "events": items})
                        }
                        None => serde_json::json!({"ok": false, "error": "audit log not initialized"}),
                    }
                }

                "blackboard" => {
                    let project_filter = request.get("project").and_then(|v| v.as_str()).unwrap_or("*");
                    let limit = request.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as u32;
                    match &registry.blackboard {
                        Some(bb) => {
                            let entries = bb.list_project(project_filter, limit).unwrap_or_default();
                            let items: Vec<serde_json::Value> = entries.iter().map(|e| {
                                serde_json::json!({
                                    "key": e.key,
                                    "content": e.content,
                                    "agent": e.agent,
                                    "project": e.project,
                                    "tags": e.tags,
                                    "created_at": e.created_at.to_rfc3339(),
                                    "expires_at": e.expires_at.to_rfc3339(),
                                })
                            }).collect();
                            serde_json::json!({"ok": true, "entries": items})
                        }
                        None => serde_json::json!({"ok": false, "error": "blackboard not initialized"}),
                    }
                }

                "expertise" => {
                    let domain = request.get("domain").and_then(|v| v.as_str()).unwrap_or("general");
                    match &registry.expertise_ledger {
                        Some(ledger) => {
                            let scores = ledger.rank_for_domain(domain).unwrap_or_default();
                            let items: Vec<serde_json::Value> = scores.iter().map(|s| {
                                serde_json::json!({
                                    "agent": s.agent_name,
                                    "success_rate": s.success_rate,
                                    "avg_cost": s.avg_cost,
                                    "total_tasks": s.total_tasks,
                                    "confidence": s.confidence,
                                })
                            }).collect();
                            serde_json::json!({"ok": true, "scores": items})
                        }
                        None => serde_json::json!({"ok": false, "error": "expertise ledger not initialized"}),
                    }
                }

                _ => serde_json::json!({"ok": false, "error": format!("unknown command: {cmd}")}),
            };

            let mut resp_bytes = serde_json::to_vec(&response)?;
            resp_bytes.push(b'\n');
            writer.write_all(&resp_bytes).await?;
        }

        Ok(())
    }

    pub fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
        self.shutdown_notify.notify_waiters();
    }

    /// Check if daemon is running.
    pub fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::SeqCst)
    }
}
