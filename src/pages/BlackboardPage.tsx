import { useEffect, useState } from "react";
import Header from "@/components/Header";
import EmptyState from "@/components/EmptyState";
import { api } from "@/lib/api";

export default function BlackboardPage() {
  const [entries, setEntries] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [projectFilter, setProjectFilter] = useState("");

  useEffect(() => {
    setLoading(true);
    const params: any = { limit: 50 };
    if (projectFilter) params.project = projectFilter;
    api.getBlackboard(params).then((data) => {
      setEntries(data.entries || []);
      setLoading(false);
    }).catch(() => setLoading(false));
  }, [projectFilter]);

  return (
    <>
      <Header title="Blackboard" />

      <div className="filters">
        <input
          className="filter-input"
          placeholder="Filter by project..."
          value={projectFilter}
          onChange={(e) => setProjectFilter(e.target.value)}
        />
        <span style={{ fontSize: "var(--font-size-xs)", color: "var(--text-muted)", alignSelf: "center" }}>
          {entries.length} entries
        </span>
      </div>

      {loading ? (
        <div className="loading">Loading blackboard...</div>
      ) : entries.length === 0 ? (
        <EmptyState title="No entries" description="The blackboard is empty." />
      ) : (
        <div>
          {entries.map((entry: any, i: number) => (
            <div key={i} className="blackboard-entry">
              <div className="blackboard-key">{entry.key}</div>
              <div className="blackboard-content">{entry.content}</div>
              <div className="blackboard-meta">
                <span>Agent: {entry.agent}</span>
                <span>Project: {entry.project}</span>
                {entry.tags?.length > 0 && <span>Tags: {entry.tags.join(", ")}</span>}
                <span>{new Date(entry.created_at).toLocaleString()}</span>
              </div>
            </div>
          ))}
        </div>
      )}
    </>
  );
}
