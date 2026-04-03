import { useEffect } from "react";
import { useDaemonStore } from "@/store/daemon";
import { runtimeLabel } from "@/lib/runtime";
import Header from "@/components/Header";

function timeAgo(ts: string | undefined | null): string {
  if (!ts) return "";
  const diff = Date.now() - new Date(ts).getTime();
  if (diff < 0) return "now";
  const sec = Math.floor(diff / 1000);
  if (sec < 60) return `${sec}s`;
  const min = Math.floor(sec / 60);
  if (min < 60) return `${min}m`;
  const hr = Math.floor(min / 60);
  if (hr < 24) return `${hr}h`;
  const d = Math.floor(hr / 24);
  return `${d}d`;
}

function formatUsd(n: number): string {
  return `$${n.toFixed(2)}`;
}

export default function DashboardPage() {
  const status = useDaemonStore((s) => s.status);
  const tasks = useDaemonStore((s) => s.tasks);
  const agents = useDaemonStore((s) => s.agents);
  const cost = useDaemonStore((s) => s.cost);
  const audit = useDaemonStore((s) => s.audit);
  const fetchAll = useDaemonStore((s) => s.fetchAll);

  useEffect(() => { fetchAll(); }, [fetchAll]);

  const pendingTasks = tasks.filter((t: any) => t.status === "pending");
  const activeTasks = tasks.filter((t: any) => t.status === "in_progress");
  const blockedTasks = tasks.filter((t: any) => t.status === "blocked");
  const doneTasks = tasks
    .filter((t: any) => t.status === "done")
    .sort((a: any, b: any) =>
      new Date(b.updated_at || b.created_at).getTime() -
      new Date(a.updated_at || a.created_at).getTime()
    )
    .slice(0, 5);
  const activeWorkers = status?.active_workers ?? activeTasks.length;
  const spent = cost?.spent_today_usd ?? 0;
  const budget = cost?.daily_budget_usd ?? 10;
  const remaining = Math.max(0, budget - spent);
  const pct = budget > 0 ? Math.min(100, (spent / budget) * 100) : 0;

  const rawPerProject = cost?.per_project;
  const perProject = Array.isArray(rawPerProject) ? rawPerProject : [];

  return (
    <div className="page-content">
      <Header title="Dashboard" />

      {/* Hero stats bar */}
      <div className="dash-hero">
        <div className="dash-hero-stat">
          <div className="dash-hero-value">{activeWorkers}</div>
          <div className="dash-hero-label">Active Workers</div>
        </div>
        <div className="dash-hero-stat">
          <div className="dash-hero-value">{pendingTasks.length}</div>
          <div className="dash-hero-label">Pending Tasks</div>
        </div>
        <div className="dash-hero-stat">
          <div className={`dash-hero-value${blockedTasks.length > 0 ? " dash-hero-value-warning" : ""}`}>
            {blockedTasks.length}
          </div>
          <div className="dash-hero-label">Blocked</div>
        </div>
        <div className="dash-hero-stat">
          <div className="dash-hero-value">{formatUsd(spent)}</div>
          <div className="dash-hero-label">Daily Cost</div>
        </div>
        <div className="dash-hero-stat">
          <div className={`dash-hero-value${remaining > 0 ? " dash-hero-value-success" : " dash-hero-value-error"}`}>
            {formatUsd(remaining)}
          </div>
          <div className="dash-hero-label">Budget Remaining</div>
        </div>
      </div>

      {/* Budget utilization bar */}
      <div className="dash-budget">
        <div className="dash-budget-header">
          <span className="dash-budget-title">Budget Utilization</span>
          <span className="dash-budget-numbers">
            {formatUsd(spent)} / {formatUsd(budget)} ({pct.toFixed(0)}%)
          </span>
        </div>
        <div className="dash-budget-track">
          <div className="dash-budget-fill" style={{ width: `${pct}%` }} />
        </div>
      </div>

      {/* Two column grid */}
      <div className="dash-grid">
        <div className="dash-col">
          {/* Active Work panel */}
          <div className="dash-panel">
            <div className="dash-panel-header">
              <span className="dash-panel-title">Active Work</span>
            </div>
            {activeTasks.length === 0 ? (
              <div className="dash-panel-empty">No active tasks</div>
            ) : (
              activeTasks.map((t: any) => {
                const phase = runtimeLabel(t.runtime);
                return (
                  <div key={t.id} className="dash-active-row">
                    <span className="dash-active-agent">
                      {t.assignee || t.agent || "—"}
                    </span>
                    <span className="dash-active-subject">{t.subject}</span>
                    {phase && <span className="dash-active-phase">{phase}</span>}
                    <span className="dash-done-time">
                      {timeAgo(t.started_at || t.updated_at || t.created_at)}
                    </span>
                  </div>
                );
              })
            )}
          </div>

          {/* Blocked Work panel */}
          <div className="dash-panel">
            <div className="dash-panel-header">
              <span className="dash-panel-title">Blocked</span>
            </div>
            {blockedTasks.length === 0 ? (
              <div className="dash-panel-empty">Nothing blocked</div>
            ) : (
              blockedTasks.map((t: any) => (
                <div key={t.id} className="dash-blocked-row">
                  <span className="dash-blocked-subject">
                    {t.id} — {t.subject}
                  </span>
                  {t.blocked_reason && (
                    <span className="dash-blocked-reason">{t.blocked_reason}</span>
                  )}
                </div>
              ))
            )}
          </div>

          {/* Recently Completed */}
          <div className="dash-panel">
            <div className="dash-panel-header">
              <span className="dash-panel-title">Recently Completed</span>
            </div>
            {doneTasks.length === 0 ? (
              <div className="dash-panel-empty">No completed tasks</div>
            ) : (
              doneTasks.map((t: any) => (
                <div key={t.id} className="dash-done-row">
                  <span className="dash-done-subject">{t.subject}</span>
                  <span className="dash-done-time">
                    {timeAgo(t.updated_at || t.created_at)}
                  </span>
                </div>
              ))
            )}
          </div>
        </div>

        <div className="dash-col">
          {/* Cost breakdown */}
          <div className="dash-panel">
            <div className="dash-panel-header">
              <span className="dash-panel-title">Cost by Company</span>
            </div>
            {perProject.length === 0 ? (
              <div className="dash-panel-empty">No cost data</div>
            ) : (
              perProject.map((p: any) => (
                <div key={p.name || p.company} className="dash-cost-row">
                  <span className="dash-cost-company">{p.name || p.company}</span>
                  <span className="dash-cost-amount">{formatUsd(p.spent ?? p.cost ?? 0)}</span>
                </div>
              ))
            )}
          </div>

          {/* Activity Feed */}
          <div className="dash-panel">
            <div className="dash-panel-header">
              <span className="dash-panel-title">Activity</span>
            </div>
            {audit.length === 0 ? (
              <div className="dash-panel-empty">No recent activity</div>
            ) : (
              audit.slice(0, 15).map((e: any, i: number) => (
                <div key={e.id || i} className="dash-audit-row">
                  <span className="dash-audit-time">
                    {timeAgo(e.timestamp || e.created_at)}
                  </span>
                  <span className="dash-audit-agent">
                    {e.agent || e.actor || "—"}
                  </span>
                  <span className="dash-audit-summary">
                    {e.summary || e.reasoning || e.description || e.decision_type || "—"}
                  </span>
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
