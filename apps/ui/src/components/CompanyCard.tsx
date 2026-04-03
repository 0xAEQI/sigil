import { Link } from "react-router-dom";
import { ProgressBar } from "@/components/ui";
import type { Company } from "@/lib/types";

interface CompanyCardProps {
  company: Company;
}

export default function CompanyCard({ company }: CompanyCardProps) {
  const total =
    company.stats.pending +
    company.stats.active +
    company.stats.done +
    company.stats.failed;
  const completionPct = total > 0 ? (company.stats.done / total) * 100 : 0;

  return (
    <Link to={`/companies/${company.name}`} className="project-card">
      <div className="project-card-header">
        <div>
          <div className="project-name-row">
            <code className="project-prefix">{company.prefix}</code>
            <span className="project-name">{company.name}</span>
          </div>
          {company.description && (
            <p className="project-description">{company.description}</p>
          )}
        </div>
      </div>

      <div className="project-team">
        <span className="project-team-label">Team:</span>
        <span className="project-leader">{company.team.leader}</span>
        {company.team.advisors.map((a) => (
          <span key={a} className="project-advisor">
            {a}
          </span>
        ))}
      </div>

      <div className="project-stats-bar">
        <ProgressBar value={completionPct} label={`${company.stats.done}/${total} tasks`} />
      </div>

      <div className="project-stat-row">
        <div className="project-stat">
          <span className="project-stat-count project-stat-pending">
            {company.stats.pending}
          </span>
          <span className="project-stat-label">pending</span>
        </div>
        <div className="project-stat">
          <span className="project-stat-count project-stat-active">
            {company.stats.active}
          </span>
          <span className="project-stat-label">active</span>
        </div>
        <div className="project-stat">
          <span className="project-stat-count project-stat-done">
            {company.stats.done}
          </span>
          <span className="project-stat-label">done</span>
        </div>
        <div className="project-stat">
          <span className="project-stat-count project-stat-failed">
            {company.stats.failed}
          </span>
          <span className="project-stat-label">failed</span>
        </div>
      </div>
    </Link>
  );
}
