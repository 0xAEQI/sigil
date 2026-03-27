import { Link } from "react-router-dom";
import StatusBadge from "./StatusBadge";
import { ROLE_LABELS } from "@/lib/constants";
import type { Agent } from "@/lib/types";

interface AgentCardProps {
  agent: Agent;
  compact?: boolean;
}

export default function AgentCard({ agent, compact = false }: AgentCardProps) {
  if (compact) {
    return (
      <Link to={`/agents/${agent.name}`} className="agent-card-compact">
        <div className="agent-card-compact-header">
          <div className="agent-name-row">
            <code className="agent-prefix">{agent.prefix}</code>
            <span className="agent-name">{agent.name}</span>
          </div>
          <StatusBadge status={agent.status} size="sm" />
        </div>
        {agent.current_task && (
          <div className="agent-current-task">
            <span className="agent-task-label">Working on</span>
            <code className="agent-task-id">{agent.current_task}</code>
          </div>
        )}
      </Link>
    );
  }

  return (
    <Link to={`/agents/${agent.name}`} className="agent-card">
      <div className="agent-card-header">
        <div>
          <div className="agent-name-row">
            <code className="agent-prefix">{agent.prefix}</code>
            <span className="agent-name">{agent.name}</span>
          </div>
          <div className="agent-role-model">
            <span className="agent-role">{ROLE_LABELS[agent.role]}</span>
            <span className="agent-model-sep">&middot;</span>
            <span className="agent-model">{agent.model}</span>
          </div>
        </div>
        <StatusBadge status={agent.status} />
      </div>

      <div className="agent-expertise">
        {agent.expertise.map((skill) => (
          <span key={skill} className="expertise-tag">
            {skill}
          </span>
        ))}
      </div>

      {agent.current_task && (
        <div className="agent-current-task">
          <span className="agent-task-label">Working on</span>
          <code className="agent-task-id">{agent.current_task}</code>
        </div>
      )}

      <div className="agent-stats">
        <div className="agent-stat">
          <span className="agent-stat-value">{agent.stats.completed}</span>
          <span className="agent-stat-label">completed</span>
        </div>
        <div className="agent-stat">
          <span className="agent-stat-value">{agent.stats.failed}</span>
          <span className="agent-stat-label">failed</span>
        </div>
        <div className="agent-stat">
          <span className="agent-stat-value">
            ${agent.stats.avg_cost_usd.toFixed(3)}
          </span>
          <span className="agent-stat-label">avg cost</span>
        </div>
      </div>
    </Link>
  );
}
