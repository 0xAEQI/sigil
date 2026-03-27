import { useEffect, useState } from "react";
import Header from "@/components/Header";
import AuditEntryComponent from "@/components/AuditEntry";
import EmptyState from "@/components/EmptyState";
import { api } from "@/lib/api";

export default function AuditPage() {
  const [events, setEvents] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [projectFilter, setProjectFilter] = useState("");
  const [limit, setLimit] = useState(50);

  useEffect(() => {
    setLoading(true);
    const params: any = { last: limit };
    if (projectFilter) params.project = projectFilter;
    api.getAudit(params).then((data) => {
      setEvents(data.events || []);
      setLoading(false);
    }).catch(() => setLoading(false));
  }, [projectFilter, limit]);

  return (
    <>
      <Header title="Audit Trail" />

      <div className="filters">
        <input
          className="filter-input"
          placeholder="Filter by project..."
          value={projectFilter}
          onChange={(e) => setProjectFilter(e.target.value)}
        />
        <select
          className="filter-select"
          value={limit}
          onChange={(e) => setLimit(Number(e.target.value))}
        >
          <option value={20}>Last 20</option>
          <option value={50}>Last 50</option>
          <option value={100}>Last 100</option>
        </select>
        <span style={{ fontSize: "var(--font-size-xs)", color: "var(--text-muted)", alignSelf: "center" }}>
          {events.length} events
        </span>
      </div>

      {loading ? (
        <div className="loading">Loading audit trail...</div>
      ) : events.length === 0 ? (
        <EmptyState title="No audit events" description="No decision events recorded yet." />
      ) : (
        <div className="column-section">
          <div className="column-section-body">
            {events.map((entry: any, i: number) => (
              <AuditEntryComponent key={i} entry={entry} />
            ))}
          </div>
        </div>
      )}
    </>
  );
}
