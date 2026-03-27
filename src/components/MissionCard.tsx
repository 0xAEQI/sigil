import StatusBadge from "./StatusBadge";
import type { Mission } from "@/lib/types";

interface MissionCardProps {
  mission: Mission;
}

function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export default function MissionCard({ mission }: MissionCardProps) {
  const progressPct =
    mission.task_count > 0
      ? (mission.done_count / mission.task_count) * 100
      : 0;

  return (
    <div className="mission-card">
      <div className="mission-card-header">
        <div className="mission-title-row">
          <code className="mission-id">{mission.id}</code>
          <span className="mission-name">{mission.name}</span>
        </div>
        <StatusBadge status={mission.status} size="sm" />
      </div>

      <p className="mission-description">{mission.description}</p>

      <div className="mission-meta">
        <span className="mission-project">{mission.project}</span>
        {mission.skill && (
          <span className="mission-skill">{mission.skill}</span>
        )}
        {mission.schedule && (
          <span className="mission-schedule">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.5">
              <circle cx="6" cy="6" r="5" />
              <path d="M6 3v3l2 1" />
            </svg>
            {mission.schedule}
          </span>
        )}
      </div>

      <div className="mission-progress">
        <div className="mission-progress-bar">
          <div
            className="mission-progress-fill"
            style={{ width: `${progressPct}%` }}
          />
        </div>
        <span className="mission-progress-text">
          {mission.done_count}/{mission.task_count} tasks
        </span>
      </div>

      <div className="mission-footer">
        <span className="mission-date">{formatDate(mission.created_at)}</span>
      </div>
    </div>
  );
}
