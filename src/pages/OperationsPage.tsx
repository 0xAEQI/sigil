import { useEffect, useState } from "react";
import Header from "@/components/Header";
import AuditEntryComponent from "@/components/AuditEntry";
import { api } from "@/lib/api";

export default function OperationsPage() {
  const [crons, setCrons] = useState<any[]>([]);
  const [watchdogs, setWatchdogs] = useState<any[]>([]);
  const [audit, setAudit] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [tab, setTab] = useState<"crons" | "watchdogs" | "activity">("crons");

  useEffect(() => {
    Promise.all([
      api.getCrons().then((d) => setCrons(d.jobs || [])),
      api.getWatchdogs().then((d) => setWatchdogs(d.rules || [])),
      api.getAudit({ last: 30 }).then((d) => setAudit(d.events || [])),
    ])
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <div className="loading">Loading operations...</div>;

  return (
    <>
      <Header title="Operations" />

      {/* Summary stats */}
      <div className="hero-stats" style={{ marginBottom: "var(--space-6)" }}>
        <div className="hero-stat">
          <div className="hero-stat-value">{crons.length}</div>
          <div className="hero-stat-label">Cron Jobs</div>
        </div>
        <div className="hero-stat-divider" />
        <div className="hero-stat">
          <div className="hero-stat-value">{watchdogs.filter((w: any) => w.enabled !== false).length}</div>
          <div className="hero-stat-label">Active Watchdogs</div>
        </div>
        <div className="hero-stat-divider" />
        <div className="hero-stat">
          <div className="hero-stat-value">{audit.length}</div>
          <div className="hero-stat-label">Recent Events</div>
        </div>
      </div>

      {/* Tabs */}
      <div style={{ display: "flex", gap: "var(--space-1)", marginBottom: "var(--space-4)" }}>
        {(["crons", "watchdogs", "activity"] as const).map((t) => (
          <button
            key={t}
            className={`btn ${tab === t ? "btn-primary" : ""}`}
            onClick={() => setTab(t)}
          >
            {t === "crons" ? `Scheduled (${crons.length})` : t === "watchdogs" ? `Watchdogs (${watchdogs.length})` : `Activity (${audit.length})`}
          </button>
        ))}
      </div>

      {/* Crons Tab */}
      {tab === "crons" && (
        <div>
          {crons.length === 0 ? (
            <div className="dash-empty">No cron jobs configured</div>
          ) : (
            crons.map((job: any, i: number) => (
              <div key={i} className="ops-card">
                <div className="ops-card-header">
                  <div className="ops-card-title">
                    <span className="ops-card-icon">
                      <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
                        <circle cx="7" cy="7" r="6" />
                        <path d="M7 3v4l2.5 1.5" />
                      </svg>
                    </span>
                    {job.name}
                  </div>
                  <code className="ops-card-schedule">{job.schedule}</code>
                </div>
                <div className="ops-card-body">
                  <div className="ops-card-meta">
                    <span className="ops-card-project">{job.project}</span>
                    {job.last_run && (
                      <span className="ops-card-last-run">
                        Last: {new Date(job.last_run).toLocaleString("en-US", { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" })}
                      </span>
                    )}
                  </div>
                  <div className="ops-card-prompt">{job.prompt.slice(0, 120)}{job.prompt.length > 120 ? "..." : ""}</div>
                </div>
              </div>
            ))
          )}
        </div>
      )}

      {/* Watchdogs Tab */}
      {tab === "watchdogs" && (
        <div>
          {watchdogs.length === 0 ? (
            <div className="dash-empty">No watchdog rules configured</div>
          ) : (
            watchdogs.map((rule: any, i: number) => {
              const condition = rule.condition || {};
              const action = rule.action || {};
              const actionType = Object.keys(action)[0] || "unknown";
              const actionDetail = action[actionType];
              const isEnabled = rule.enabled !== false;

              return (
                <div key={i} className={`ops-card ${!isEnabled ? "ops-card-disabled" : ""}`}>
                  <div className="ops-card-header">
                    <div className="ops-card-title">
                      <span className="ops-card-icon">
                        <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke={isEnabled ? "var(--warning)" : "var(--text-muted)"} strokeWidth="1.5">
                          <path d="M7 1v4M7 9v1M2 13h10l-1.5-4H3.5L2 13z" />
                          <circle cx="7" cy="6" r="3" />
                        </svg>
                      </span>
                      {rule.name}
                    </div>
                    <span className={`ops-card-status ${isEnabled ? "ops-card-status-active" : "ops-card-status-inactive"}`}>
                      {isEnabled ? "active" : "disabled"}
                    </span>
                  </div>
                  <div className="ops-card-body">
                    <div className="ops-card-condition">
                      <span className="ops-card-label">Trigger:</span>
                      <code>{condition.event || "—"}</code>
                      {condition.project_filter && <span className="ops-card-project">{condition.project_filter}</span>}
                      {condition.count_threshold && <span>≥ {condition.count_threshold}x</span>}
                      {condition.window_secs && <span>in {Math.round(condition.window_secs / 60)}min</span>}
                    </div>
                    <div className="ops-card-action">
                      <span className="ops-card-label">Action:</span>
                      <span>{actionType}</span>
                      {actionDetail?.message && <span className="ops-card-message">"{actionDetail.message}"</span>}
                    </div>
                    <div className="ops-card-meta">
                      <span>Cooldown: {Math.round((rule.cooldown_secs || 3600) / 60)}min</span>
                    </div>
                  </div>
                </div>
              );
            })
          )}
        </div>
      )}

      {/* Activity Tab */}
      {tab === "activity" && (
        <div className="column-section">
          <div className="column-section-body">
            {audit.length === 0 ? (
              <div className="dash-empty">No recent activity</div>
            ) : (
              audit.map((entry: any, i: number) => (
                <AuditEntryComponent key={i} entry={entry} />
              ))
            )}
          </div>
        </div>
      )}
    </>
  );
}
