import { useEffect, useState } from "react";
import Header from "@/components/Header";
import AuditEntryComponent from "@/components/AuditEntry";
import { DataState } from "@/components/ui";
import { api } from "@/lib/api";

export default function AuditPage() {
  const [events, setEvents] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [companyFilter, setCompanyFilter] = useState("");
  const [limit, setLimit] = useState(50);

  useEffect(() => {
    setLoading(true);
    const params: any = { last: limit };
    if (companyFilter) params.company = companyFilter;
    api.getAudit(params).then((data) => {
      setEvents(data.events || []);
      setLoading(false);
    }).catch(() => setLoading(false));
  }, [companyFilter, limit]);

  return (
    <>
      <Header title="Audit Trail" />

      <div className="filters">
        <input
          className="filter-input"
          placeholder="Filter by company..."
          value={companyFilter}
          onChange={(e) => setCompanyFilter(e.target.value)}
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

      <DataState loading={loading} empty={events.length === 0} emptyTitle="No events" emptyDescription="No audit events found." loadingText="Loading audit trail...">
        <div className="column-section">
          <div className="column-section-body">
            {events.map((entry: any, i: number) => (
              <AuditEntryComponent key={i} entry={entry} />
            ))}
          </div>
        </div>
      </DataState>
    </>
  );
}
