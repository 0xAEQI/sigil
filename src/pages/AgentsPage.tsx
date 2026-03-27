import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import Header from "@/components/Header";
import EmptyState from "@/components/EmptyState";
import { api } from "@/lib/api";

export default function AgentsPage() {
  const [agents, setAgents] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.getAgents().then((data) => {
      setAgents(data.agents || []);
      setLoading(false);
    }).catch(() => setLoading(false));
  }, []);

  return (
    <>
      <Header title="Agents" />
      {loading ? (
        <div className="loading">Loading agents...</div>
      ) : agents.length === 0 ? (
        <EmptyState title="No agents" description="No agents discovered. Check agents/ directory." />
      ) : (
        <div className="cards-grid">
          {agents.map((a: any) => (
            <Link key={a.name} to={`/agents/${a.name}`} className="agent-card">
              <div className="agent-header">
                <div style={{ display: "flex", alignItems: "center", gap: "var(--space-2)" }}>
                  <div className="dash-agent-avatar">{a.name[0].toUpperCase()}</div>
                  <div>
                    <span className="agent-name">{a.name}</span>
                    <span className="agent-prefix">{a.prefix}</span>
                  </div>
                </div>
                <span className="agent-role">{a.role}</span>
              </div>
              {a.model && <div className="agent-model">{a.model}</div>}
              <div className="agent-expertise">
                {(a.expertise || []).map((e: string) => (
                  <span key={e} className="expertise-tag">{e}</span>
                ))}
              </div>
              {a.expertise_scores && a.expertise_scores.length > 0 && (
                <div className="agent-stats">
                  {a.expertise_scores.slice(0, 3).map((s: any, i: number) => (
                    <span key={i} style={{ fontSize: "var(--font-size-xs)", color: "var(--text-muted)" }}>
                      {(s.success_rate * 100).toFixed(0)}% ({s.total_tasks} tasks)
                    </span>
                  ))}
                </div>
              )}
            </Link>
          ))}
        </div>
      )}
    </>
  );
}
