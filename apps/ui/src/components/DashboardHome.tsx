import { useEffect } from "react";
import { useDaemonStore } from "@/store/daemon";
import { useChatStore } from "@/store/chat";
import { runtimeLabel } from "@/lib/runtime";
import { timeAgo } from "@/lib/format";
import type { Agent } from "@/lib/types";

function formatUsd(n: number): string {
  return `$${n.toFixed(2)}`;
}

function formatTokens(usd: number): string {
  const tokens = usd * 1_000_000;
  if (tokens >= 1_000_000) return `${(tokens / 1_000_000).toFixed(1)}M`;
  if (tokens >= 1_000) return `${(tokens / 1_000).toFixed(0)}K`;
  return `${Math.round(tokens)}`;
}

function statusColor(status: string): string {
  const s = status.toLowerCase();
  if (s === "active" || s === "working" || s === "running") return "var(--success)";
  if (s === "paused" || s === "idle") return "var(--text-muted)";
  if (s === "error" || s === "failed") return "var(--error)";
  return "var(--text-muted)";
}

export default function DashboardHome() {
  const status = useDaemonStore((s) => s.status);
  const quests = useDaemonStore((s) => s.quests);
  const agents = useDaemonStore((s) => s.agents);
  const cost = useDaemonStore((s) => s.cost);
  const events = useDaemonStore((s) => s.events);
  const fetchAll = useDaemonStore((s) => s.fetchAll);
  const setSelectedAgent = useChatStore((s) => s.setSelectedAgent);

  useEffect(() => { fetchAll(); }, [fetchAll]);

  const activeQuests = quests.filter((q: any) => q.status === "in_progress");
  const blockedQuests = quests.filter((q: any) => q.status === "blocked");
  const spent = cost?.spent_today_usd ?? 0;
  const budget = cost?.daily_budget_usd ?? 10;
  const pct = budget > 0 ? Math.min(100, (spent / budget) * 100) : 0;
  const activeAgentCount = agents.filter((a: any) => {
    const s = (a.status || "").toLowerCase();
    return s === "active" || s === "working" || s === "running";
  }).length;

  const handleAgentClick = (agent: Agent) => {
    setSelectedAgent({
      id: agent.id,
      name: agent.name,
      display_name: agent.display_name,
      model: agent.model,
    });
  };

  return (
    <div className="dash-home">
      {/* Title */}
      <div className="dash-home-header">
        <h1 className="dash-home-title">aeqi<span className="dash-home-dot">.ai</span></h1>
        {agents.length === 0 ? (
          <div className="dash-home-welcome">
            <p className="dash-home-subtitle">The agent runtime.</p>
            <div className="dash-home-discover">
              <div className="dash-discover-item">
                <span className="dash-discover-icon">⚡</span>
                <div>
                  <strong>Prompts</strong>
                  <span>Composable instructions that define what agents know and do</span>
                </div>
              </div>
              <div className="dash-discover-item">
                <span className="dash-discover-icon">◆</span>
                <div>
                  <strong>Quests</strong>
                  <span>Units of work tracked through your agent pipeline</span>
                </div>
              </div>
              <div className="dash-discover-item">
                <span className="dash-discover-icon">✦</span>
                <div>
                  <strong>Agents</strong>
                  <span>Autonomous entities that research, plan, implement, and verify</span>
                </div>
              </div>
              <div className="dash-discover-item">
                <span className="dash-discover-icon">◉</span>
                <div>
                  <strong>Insights</strong>
                  <span>Knowledge your agents accumulate and share across sessions</span>
                </div>
              </div>
            </div>
          </div>
        ) : (
          <p className="dash-home-subtitle">Select an agent to start a session</p>
        )}
      </div>

      {/* Quick stats */}
      <div className="dash-home-stats">
        <div className="dash-stat">
          <span className="dash-stat-value">{activeAgentCount}</span>
          <span className="dash-stat-label">Agents Online</span>
        </div>
        <div className="dash-stat">
          <span className="dash-stat-value">{activeQuests.length}</span>
          <span className="dash-stat-label">Active Quests</span>
        </div>
        <div className="dash-stat">
          <span className={`dash-stat-value${blockedQuests.length > 0 ? " dash-stat-warn" : ""}`}>
            {blockedQuests.length}
          </span>
          <span className="dash-stat-label">Blocked</span>
        </div>
        <div className="dash-stat">
          <span className="dash-stat-value">{formatTokens(spent)} tok</span>
          <span className="dash-stat-label">Tokens Today</span>
        </div>
      </div>

      {/* Budget bar */}
      {budget > 0 && (
        <div className="dash-home-budget">
          <div className="dash-home-budget-header">
            <span>Token budget</span>
            <span>{formatTokens(spent)} / {formatTokens(budget)}</span>
          </div>
          <div className="dash-home-budget-track">
            <div className="dash-home-budget-fill" style={{ width: `${pct}%` }} />
          </div>
        </div>
      )}

      {/* Agent grid -- click to open session */}
      <div className="dash-home-section">
        <div className="dash-home-section-title">Agents</div>
        <div className="dash-agent-grid">
          {agents.map((agent) => (
            <div
              key={agent.id}
              className="dash-agent-card"
              onClick={() => handleAgentClick(agent)}
            >
              <div className="dash-agent-card-header">
                <span className="dash-agent-dot" style={{ background: statusColor(agent.status) }} />
                <span className="dash-agent-name">{agent.display_name || agent.name}</span>
              </div>
              {agent.model && <span className="dash-agent-model">{agent.model}</span>}
              <span className="dash-agent-status">{agent.status}</span>
            </div>
          ))}
          {agents.length === 0 && (
            <div className="dash-home-empty">No agents registered</div>
          )}
        </div>
      </div>

      {/* Active quests */}
      {activeQuests.length > 0 && (
        <div className="dash-home-section">
          <div className="dash-home-section-title">Active Quests</div>
          <div className="dash-quest-list">
            {activeQuests.map((q: any) => (
              <div key={q.id} className="dash-quest-row">
                <span className="dash-quest-agent">{q.assignee || q.agent || "\u2014"}</span>
                <span className="dash-quest-subject">{q.subject}</span>
                {runtimeLabel(q.runtime) && <span className="dash-quest-phase">{runtimeLabel(q.runtime)}</span>}
                <span className="dash-quest-time">{timeAgo(q.started_at || q.updated_at)}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Recent activity */}
      {events.length > 0 && (
        <div className="dash-home-section">
          <div className="dash-home-section-title">Recent Activity</div>
          <div className="dash-activity-list">
            {events.slice(0, 12).map((e: any, i: number) => (
              <div key={e.id || i} className="dash-activity-row">
                <span className="dash-activity-time">{timeAgo(e.timestamp || e.created_at)}</span>
                <span className="dash-activity-agent">{e.agent || e.actor || "\u2014"}</span>
                <span className="dash-activity-summary">
                  {e.summary || e.reasoning || e.description || e.decision_type || "\u2014"}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
