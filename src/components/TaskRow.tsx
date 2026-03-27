import { Link } from "react-router-dom";
import StatusBadge from "./StatusBadge";
import { PRIORITY_COLORS } from "@/lib/constants";
import type { Task } from "@/lib/types";

interface TaskRowProps {
  task: Task;
}

function timeAgo(dateStr: string): string {
  const now = new Date();
  const date = new Date(dateStr);
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffDays > 0) return `${diffDays}d ago`;
  if (diffHours > 0) return `${diffHours}h ago`;
  if (diffMins > 0) return `${diffMins}m ago`;
  return "just now";
}

export default function TaskRow({ task }: TaskRowProps) {
  return (
    <div className="task-row">
      <div className="task-row-id">
        <span
          className="task-priority-indicator"
          style={{
            backgroundColor:
              PRIORITY_COLORS[task.priority] || "var(--text-primary)",
          }}
        />
        <code className="task-id-code">{task.id}</code>
      </div>

      <div className="task-row-subject">
        <span className="task-subject">{task.subject}</span>
        {task.labels.length > 0 && (
          <div className="task-labels">
            {task.labels.map((label) => (
              <span key={label} className="task-label">
                {label}
              </span>
            ))}
          </div>
        )}
      </div>

      <div className="task-row-meta">
        <StatusBadge status={task.status} size="sm" />

        {task.assignee ? (
          <Link
            to={`/agents/${task.assignee}`}
            className="task-assignee"
          >
            {task.assignee}
          </Link>
        ) : (
          <span className="task-unassigned">unassigned</span>
        )}

        <Link to={`/projects/${task.project}`} className="task-project">
          {task.project}
        </Link>

        {task.cost_usd > 0 && (
          <span className="task-cost">
            ${task.cost_usd.toFixed(2)}
          </span>
        )}

        <span className="task-time">
          {timeAgo(task.updated_at || task.created_at)}
        </span>
      </div>
    </div>
  );
}
