import { Link } from "react-router-dom";
import type { Project } from "@/lib/types";

interface ProjectCardProps {
  project: Project;
}

export default function ProjectCard({ project }: ProjectCardProps) {
  const total =
    project.stats.pending +
    project.stats.active +
    project.stats.done +
    project.stats.failed;
  const completionPct = total > 0 ? (project.stats.done / total) * 100 : 0;

  return (
    <Link to={`/projects/${project.name}`} className="project-card">
      <div className="project-card-header">
        <div>
          <div className="project-name-row">
            <code className="project-prefix">{project.prefix}</code>
            <span className="project-name">{project.name}</span>
          </div>
          {project.description && (
            <p className="project-description">{project.description}</p>
          )}
        </div>
      </div>

      <div className="project-team">
        <span className="project-team-label">Team:</span>
        <span className="project-leader">{project.team.leader}</span>
        {project.team.advisors.map((a) => (
          <span key={a} className="project-advisor">
            {a}
          </span>
        ))}
      </div>

      <div className="project-stats-bar">
        <div className="project-progress-bar">
          <div
            className="project-progress-fill"
            style={{ width: `${completionPct}%` }}
          />
        </div>
        <span className="project-progress-text">
          {project.stats.done}/{total} tasks
        </span>
      </div>

      <div className="project-stat-row">
        <div className="project-stat">
          <span className="project-stat-count project-stat-pending">
            {project.stats.pending}
          </span>
          <span className="project-stat-label">pending</span>
        </div>
        <div className="project-stat">
          <span className="project-stat-count project-stat-active">
            {project.stats.active}
          </span>
          <span className="project-stat-label">active</span>
        </div>
        <div className="project-stat">
          <span className="project-stat-count project-stat-done">
            {project.stats.done}
          </span>
          <span className="project-stat-label">done</span>
        </div>
        <div className="project-stat">
          <span className="project-stat-count project-stat-failed">
            {project.stats.failed}
          </span>
          <span className="project-stat-label">failed</span>
        </div>
      </div>
    </Link>
  );
}
