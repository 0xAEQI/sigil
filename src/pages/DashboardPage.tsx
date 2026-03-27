import { useEffect, useState, useRef } from "react";
import { Link } from "react-router-dom";
import { api } from "@/lib/api";

interface DashboardData {
  status: any;
  recent_audit: any[];
  cost: any;
}

function formatTime(ts: string): string {
  const d = new Date(ts);
  return d.toLocaleTimeString("en-US", { hour: "2-digit", minute: "2-digit", second: "2-digit" });
}

function AnimatedNumber({ value, prefix = "", suffix = "", decimals = 0 }: { value: number; prefix?: string; suffix?: string; decimals?: number }) {
  const [display, setDisplay] = useState(0);
  const ref = useRef<number>(0);

  useEffect(() => {
    const start = ref.current;
    const end = value;
    const duration = 800;
    const startTime = Date.now();

    const animate = () => {
      const elapsed = Date.now() - startTime;
      const progress = Math.min(elapsed / duration, 1);
      const eased = 1 - Math.pow(1 - progress, 3);
      const current = start + (end - start) * eased;
      setDisplay(current);
      ref.current = current;
      if (progress < 1) requestAnimationFrame(animate);
    };
    animate();
  }, [value]);

  return <>{prefix}{display.toFixed(decimals)}{suffix}</>;
}

const DECISION_COLORS: Record<string, string> = {
  route_decision: "#3b82f6",
  task_assigned: "#b08d57",
  task_started: "#3b82f6",
  task_completed: "#22c55e",
  task_failed: "#ef4444",
  mission_created: "#b08d57",
  mission_decomposed: "#b08d57",
  preflight_pass: "#22c55e",
  preflight_reject: "#ef4444",
  expertise_update: "#8b5cf6",
  failure_analyzed: "#f59e0b",
  watchdog_triggered: "#f59e0b",
  blackboard_post: "#06b6d4",
  reroute: "#f59e0b",
};

export default function DashboardPage() {
  const [data, setData] = useState<DashboardData | null>(null);
  const [tasks, setTasks] = useState<any[]>([]);
  const [projects, setProjects] = useState<any[]>([]);
  const [agents, setAgents] = useState<any[]>([]);
  const [brief, setBrief] = useState<string | null>(null);
  const [rateLimit, setRateLimit] = useState<any>(null);
  const [now, setNow] = useState(new Date());

  useEffect(() => {
    const fetchAll = () => {
      api.getDashboard().then(setData).catch(() => {});
      api.getTasks({ status: "in_progress" }).then((d) => setTasks(d.tasks || [])).catch(() => {});
      api.getProjects().then((d) => setProjects(d.projects || [])).catch(() => {});
      api.getAgents().then((d) => setAgents(d.agents || [])).catch(() => {});
      api.getRateLimit().then((d) => setRateLimit(d.rate_limit)).catch(() => {});
    };
    fetchAll();
    api.getBrief().then((d) => setBrief(d.brief || null)).catch(() => {});
    const interval = setInterval(fetchAll, 10000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    const t = setInterval(() => setNow(new Date()), 1000);
    return () => clearInterval(t);
  }, []);

  const status = data?.status;
  const cost = data?.cost;
  const audit = data?.recent_audit || [];
  const budgetPct = cost ? (cost.spent_today_usd / (cost.daily_budget_usd || 1)) * 100 : 0;

  // Projects with tasks are real projects
  const realProjects = projects.filter((p: any) => p.total_tasks > 0);

  return (
    <div className="dashboard">
      {/* Header */}
      <div className="dash-header">
        <div>
          <h1 className="dash-title">Command Center</h1>
          <p className="dash-subtitle">{now.toLocaleDateString("en-US", { weekday: "long", month: "long", day: "numeric", year: "numeric" })}</p>
        </div>
        <div className="dash-clock">
          <span className="clock-time">{now.toLocaleTimeString("en-US", { hour: "2-digit", minute: "2-digit", second: "2-digit" })}</span>
          <span className={`clock-dot ${status?.ok ? "clock-dot-live" : ""}`} />
        </div>
      </div>

      {/* Daily Brief */}
      {brief && (
        <div className="dash-brief">
          <div className="dash-brief-avatar">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M3 5l5-3 5 3" /><path d="M3 5v6l5 3 5-3V5" /><path d="M8 8v6" /><circle cx="8" cy="8" r="1.5" fill="currentColor" stroke="none" />
            </svg>
          </div>
          <pre className="dash-brief-text">{brief}</pre>
        </div>
      )}

      {/* Hero Stats */}
      <div className="hero-stats">
        <div className="hero-stat">
          <div className="hero-stat-value">
            {status ? <AnimatedNumber value={realProjects.length} /> : "—"}
          </div>
          <div className="hero-stat-label">Projects</div>
        </div>
        <div className="hero-stat-divider" />
        <div className="hero-stat">
          <div className="hero-stat-value">
            {status ? <AnimatedNumber value={status.max_workers} /> : "—"}
          </div>
          <div className="hero-stat-label">Workers</div>
        </div>
        <div className="hero-stat-divider" />
        <div className="hero-stat">
          <div className="hero-stat-value">
            {status ? <AnimatedNumber value={status.cron_jobs} /> : "—"}
          </div>
          <div className="hero-stat-label">Cron Jobs</div>
        </div>
        <div className="hero-stat-divider" />
        <div className="hero-stat">
          <div className="hero-stat-value hero-stat-cost">
            {cost ? <AnimatedNumber value={cost.spent_today_usd} prefix="$" decimals={3} /> : "—"}
          </div>
          <div className="hero-stat-label">Cost Today</div>
        </div>
        <div className="hero-stat-divider" />
        <div className="hero-stat">
          <div className="hero-stat-value">
            {status ? <AnimatedNumber value={status.pending_mail} /> : "—"}
          </div>
          <div className="hero-stat-label">Pending</div>
        </div>
      </div>

      {/* Budget Bar */}
      {cost && (
        <div className="dash-budget">
          <div className="dash-budget-header">
            <span className="dash-budget-title">Daily Budget</span>
            <span className="dash-budget-numbers">
              ${Number(cost.spent_today_usd).toFixed(3)} / ${Number(cost.daily_budget_usd).toFixed(2)}
            </span>
          </div>
          <div className="dash-budget-track">
            <div
              className="dash-budget-fill"
              style={{ width: `${Math.min(budgetPct, 100)}%` }}
            />
            <div className="dash-budget-markers">
              <span className="dash-budget-marker" style={{ left: "25%" }} />
              <span className="dash-budget-marker" style={{ left: "50%" }} />
              <span className="dash-budget-marker" style={{ left: "75%" }} />
            </div>
          </div>
          <div className="dash-budget-footer">
            <span>{budgetPct.toFixed(1)}% used</span>
            <span>${Number(cost.remaining_usd).toFixed(3)} remaining</span>
          </div>
        </div>
      )}

      {/* Main Grid */}
      {/* Rate Limit */}
      {rateLimit && (
        <div className="dash-budget" style={{ marginBottom: "var(--space-6)", borderLeftColor: rateLimit.status === "allowed" ? "var(--success)" : "var(--warning)" }}>
          <div className="dash-budget-header">
            <span className="dash-budget-title">Claude Code — {rateLimit.rate_limit_type || "5h window"}</span>
            <span className="dash-budget-numbers" style={{ color: rateLimit.status === "allowed" ? "var(--success)" : "var(--warning)" }}>
              {rateLimit.status}
            </span>
          </div>
          {rateLimit.resets_at > 0 && (
            <div className="dash-budget-footer">
              <span>Resets: {new Date(rateLimit.resets_at * 1000).toLocaleTimeString("en-US", { hour: "2-digit", minute: "2-digit" })}</span>
              <span>Updated: {rateLimit.updated_at ? new Date(rateLimit.updated_at).toLocaleTimeString("en-US", { hour: "2-digit", minute: "2-digit" }) : "—"}</span>
            </div>
          )}
        </div>
      )}

      <div className="dash-grid">
        {/* Left Column — Projects + Active Tasks */}
        <div className="dash-col">
          {/* Projects */}
          <div className="dash-panel">
            <div className="dash-panel-header">
              <span className="dash-panel-title">Projects</span>
              <Link to="/projects" className="dash-panel-link">View all</Link>
            </div>
            <div className="dash-projects">
              {realProjects.map((p: any) => (
                <Link key={p.name} to={`/projects/${p.name}`} className="dash-project">
                  <div className="dash-project-dot" />
                  <span className="dash-project-name">{p.name}</span>
                  <span style={{ fontSize: "var(--font-size-xs)", color: "var(--text-muted)" }}>
                    {p.open_tasks || 0} open
                  </span>
                  <span className="dash-project-badge">active</span>
                </Link>
              ))}
            </div>
          </div>

          {/* Active Tasks */}
          <div className="dash-panel">
            <div className="dash-panel-header">
              <span className="dash-panel-title">Active Work</span>
              <Link to="/tasks" className="dash-panel-link">{tasks.length} in progress</Link>
            </div>
            <div className="dash-tasks">
              {tasks.length === 0 ? (
                <div className="dash-empty">No tasks currently in progress</div>
              ) : (
                tasks.slice(0, 8).map((t: any) => (
                  <div key={t.id} className="dash-task">
                    <code className="dash-task-id">{t.id}</code>
                    <span className="dash-task-subject">{t.subject}</span>
                    <span className="dash-task-project">{t.project}</span>
                  </div>
                ))
              )}
            </div>
          </div>

          {/* Agents */}
          <div className="dash-panel">
            <div className="dash-panel-header">
              <span className="dash-panel-title">Agents</span>
              <Link to="/agents" className="dash-panel-link">{agents.length} registered</Link>
            </div>
            <div className="dash-agents">
              {agents.map((a: any) => (
                <div key={a.name} className="dash-agent">
                  <div className="dash-agent-avatar">{a.name[0].toUpperCase()}</div>
                  <span className="dash-agent-name">{a.name}</span>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Right Column — Live Activity Feed */}
        <div className="dash-col">
          <div className="dash-panel dash-panel-feed">
            <div className="dash-panel-header">
              <span className="dash-panel-title">
                <span className="feed-dot" />
                Live Activity
              </span>
              <Link to="/audit" className="dash-panel-link">Full trail</Link>
            </div>
            <div className="dash-feed">
              {audit.map((event: any, i: number) => (
                <div key={i} className="feed-item" style={{ animationDelay: `${i * 50}ms` }}>
                  <div className="feed-item-line">
                    <span
                      className="feed-item-dot"
                      style={{ background: DECISION_COLORS[event.decision_type] || "#666" }}
                    />
                    <div className="feed-item-content">
                      <div className="feed-item-header">
                        <span className="feed-item-type">{event.decision_type.replace(/_/g, " ")}</span>
                        <span className="feed-item-time">{formatTime(event.timestamp)}</span>
                      </div>
                      <div className="feed-item-detail">
                        {event.reasoning}
                      </div>
                      <div className="feed-item-meta">
                        <span className="feed-item-project">{event.project}</span>
                        {event.agent && <span className="feed-item-agent">{event.agent}</span>}
                        {event.task_id && <code className="feed-item-task">{event.task_id}</code>}
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Dispatch Health */}
          {status?.dispatch_health && (
            <div className="dash-panel">
              <div className="dash-panel-header">
                <span className="dash-panel-title">Dispatch Health</span>
              </div>
              <div className="dash-dispatch">
                <div className="dispatch-stat">
                  <span className="dispatch-value">{status.dispatch_health.unread}</span>
                  <span className="dispatch-label">unread</span>
                </div>
                <div className="dispatch-stat">
                  <span className="dispatch-value">{status.dispatch_health.awaiting_ack}</span>
                  <span className="dispatch-label">awaiting</span>
                </div>
                <div className="dispatch-stat">
                  <span className="dispatch-value">{status.dispatch_health.retrying_delivery}</span>
                  <span className="dispatch-label">retrying</span>
                </div>
                <div className="dispatch-stat">
                  <span className="dispatch-value dispatch-value-warn">{status.dispatch_health.dead_letters}</span>
                  <span className="dispatch-label">dead</span>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
