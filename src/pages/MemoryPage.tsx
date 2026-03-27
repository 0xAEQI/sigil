import { useEffect, useState } from "react";
import Header from "@/components/Header";
import EmptyState from "@/components/EmptyState";
import { api } from "@/lib/api";

const CATEGORY_COLORS: Record<string, string> = {
  fact: "var(--info)",
  decision: "var(--accent)",
  preference: "var(--warning)",
  insight: "var(--success)",
};

const SCOPE_LABELS: Record<string, string> = {
  domain: "Domain",
  system: "System",
  self: "Self",
  personal: "Personal",
  session: "Session",
};

export default function MemoryPage() {
  const [projectList, setProjectList] = useState<any[]>([]);
  const [selectedProject, setSelectedProject] = useState<string>("");
  const [memories, setMemories] = useState<any[]>([]);
  const [search, setSearch] = useState("");
  const [loading, setLoading] = useState(true);

  // Load projects with memory counts.
  useEffect(() => {
    api.getMemories().then((d) => {
      setProjectList(d.projects || []);
      // Auto-select first project with memories.
      const first = (d.projects || []).find((p: any) => p.count > 0);
      if (first) setSelectedProject(first.project);
      setLoading(false);
    }).catch(() => setLoading(false));
  }, []);

  // Load memories for selected project.
  useEffect(() => {
    if (!selectedProject) return;
    setLoading(true);
    api.getMemories({
      project: selectedProject,
      query: search || undefined,
      limit: 100,
    }).then((d) => {
      setMemories(d.memories || []);
      setLoading(false);
    }).catch(() => setLoading(false));
  }, [selectedProject, search]);

  return (
    <>
      <Header title="Memory" />

      {/* Project selector + search */}
      <div className="filters">
        <select
          className="filter-select"
          value={selectedProject}
          onChange={(e) => setSelectedProject(e.target.value)}
        >
          <option value="">Select project...</option>
          {projectList.map((p: any) => (
            <option key={p.project} value={p.project}>
              {p.project} ({p.count} memories)
            </option>
          ))}
        </select>
        <input
          className="filter-input"
          style={{ flex: 1 }}
          placeholder="Search memories..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <span style={{ fontSize: "var(--font-size-xs)", color: "var(--text-muted)", alignSelf: "center" }}>
          {memories.length} results
        </span>
      </div>

      {!selectedProject ? (
        <EmptyState title="Select a project" description="Choose a project to view its agent memories." />
      ) : loading ? (
        <div className="loading">Loading memories...</div>
      ) : memories.length === 0 ? (
        <EmptyState title="No memories" description={search ? "No memories match your search." : "No memories stored yet for this project."} />
      ) : (
        <div>
          {memories.map((m: any) => (
            <div key={m.id} className="memory-entry">
              <div className="memory-header">
                <code className="memory-key">{m.key}</code>
                <div className="memory-tags">
                  <span className="memory-category" style={{ color: CATEGORY_COLORS[m.category] || "var(--text-muted)" }}>
                    {m.category}
                  </span>
                  <span className="memory-scope">
                    {SCOPE_LABELS[m.scope] || m.scope}
                  </span>
                </div>
              </div>
              <div className="memory-content">{m.content}</div>
              <div className="memory-meta">
                {m.entity_id && <span>Entity: {m.entity_id}</span>}
                <span>{new Date(m.created_at).toLocaleString("en-US", { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" })}</span>
              </div>
            </div>
          ))}
        </div>
      )}
    </>
  );
}
