import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import Header from "@/components/Header";
import EmptyState from "@/components/EmptyState";
import StatusBadge from "@/components/StatusBadge";
import { api } from "@/lib/api";

export default function ProjectsPage() {
  const [projects, setProjects] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.getProjects().then((data) => {
      setProjects(data.projects || []);
      setLoading(false);
    }).catch(() => setLoading(false));
  }, []);

  return (
    <>
      <Header title="Projects" />
      {loading ? (
        <div className="loading">Loading projects...</div>
      ) : projects.length === 0 ? (
        <EmptyState title="No projects" description="No projects registered." />
      ) : (
        <div className="cards-grid">
          {projects.map((p: any) => {
            const total = p.total_tasks || 0;
            const done = p.done_tasks || 0;
            const pct = total > 0 ? (done / total) * 100 : 0;

            return (
              <Link key={p.name} to={`/projects/${p.name}`} className="project-card">
                <div className="project-name">{p.name}</div>
                <div className="project-desc">
                  {p.prefix && <code style={{ color: "var(--accent)", marginRight: "var(--space-2)" }}>{p.prefix}</code>}
                  {p.active_missions > 0 && `${p.active_missions} active missions`}
                </div>

                {p.team && (
                  <div className="project-team">
                    <span className="team-tag">{p.team.leader}</span>
                    {(p.team.agents || []).slice(0, 3).map((a: string) => (
                      <span key={a} className="team-tag" style={{ opacity: 0.6 }}>{a}</span>
                    ))}
                  </div>
                )}

                <div className="project-progress">
                  <div className="progress-bar-bg">
                    <div className="progress-bar-fill" style={{ width: `${pct}%` }} />
                  </div>
                </div>

                <div className="project-stats">
                  <span style={{ color: "var(--text-muted)" }}>{p.pending_tasks || 0} pending</span>
                  <span style={{ color: "var(--info)" }}>{p.in_progress_tasks || 0} active</span>
                  <span style={{ color: "var(--success)" }}>{done} done</span>
                  <span style={{ color: "var(--text-muted)" }}>{total} total</span>
                </div>
              </Link>
            );
          })}
        </div>
      )}
    </>
  );
}
