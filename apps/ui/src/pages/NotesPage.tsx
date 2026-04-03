import { useEffect, useState } from "react";
import Header from "@/components/Header";
import { DataState } from "@/components/ui";
import { api } from "@/lib/api";

export default function NotesPage() {
  const [entries, setEntries] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [companyFilter, setCompanyFilter] = useState("");

  useEffect(() => {
    setLoading(true);
    const params: any = { limit: 50 };
    if (companyFilter) params.company = companyFilter;
    api.getNotes(params).then((data) => {
      setEntries(data.entries || []);
      setLoading(false);
    }).catch(() => setLoading(false));
  }, [companyFilter]);

  return (
    <>
      <Header title="Notes" />

      <div className="filters">
        <input
          className="filter-input"
          placeholder="Filter by company..."
          value={companyFilter}
          onChange={(e) => setCompanyFilter(e.target.value)}
        />
        <span style={{ fontSize: "var(--font-size-xs)", color: "var(--text-muted)", alignSelf: "center" }}>
          {entries.length} entries
        </span>
      </div>

      <DataState loading={loading} empty={entries.length === 0} emptyTitle="No notes" emptyDescription="No notes found." loadingText="Loading notes...">
        <div>
          {entries.map((entry: any, i: number) => (
            <div key={i} className="note-entry">
              <div className="note-key">{entry.key}</div>
              <div className="note-content">{entry.content}</div>
              <div className="note-meta">
                <span>Agent: {entry.agent}</span>
                <span>Company: {entry.company}</span>
                {entry.tags?.length > 0 && <span>Tags: {entry.tags.join(", ")}</span>}
                <span>{new Date(entry.created_at).toLocaleString()}</span>
              </div>
            </div>
          ))}
        </div>
      </DataState>
    </>
  );
}
