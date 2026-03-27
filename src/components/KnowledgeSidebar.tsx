import { useEffect, useState, useRef } from "react";
import { api } from "@/lib/api";

const CATEGORY_COLORS: Record<string, string> = {
  fact: "#3b82f6",
  procedure: "#8b5cf6",
  preference: "#f59e0b",
  context: "#06b6d4",
  evergreen: "#22c55e",
};

interface KnowledgeItem {
  id: string;
  key: string;
  content: string;
  source: "memory" | "blackboard";
  category?: string;
  scope?: string;
  tags?: string[];
  created_at: string;
  project: string;
}

export default function KnowledgeSidebar({ project }: { project: string }) {
  const [items, setItems] = useState<KnowledgeItem[]>([]);
  const [search, setSearch] = useState("");
  const [expanded, setExpanded] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const fetchKnowledge = (query?: string) => {
    setLoading(true);
    api.getChannelKnowledge({
      project,
      query: query || undefined,
      limit: 20,
    }).then((d) => {
      setItems(d.items || []);
      setLoading(false);
    }).catch(() => setLoading(false));
  };

  useEffect(() => {
    setSearch("");
    setExpanded(null);
    fetchKnowledge();
  }, [project]);

  const handleSearch = (value: string) => {
    setSearch(value);
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => {
      fetchKnowledge(value);
    }, 300);
  };

  return (
    <div className="knowledge-sidebar">
      <div className="knowledge-sidebar-header">
        <span className="knowledge-sidebar-title">Knowledge</span>
        <span className="knowledge-sidebar-count">{items.length}</span>
      </div>

      <div className="knowledge-sidebar-search">
        <input
          className="knowledge-search-input"
          placeholder="Search..."
          value={search}
          onChange={(e) => handleSearch(e.target.value)}
        />
      </div>

      <div className="knowledge-sidebar-list">
        {loading ? (
          <div className="knowledge-sidebar-empty">Loading...</div>
        ) : items.length === 0 ? (
          <div className="knowledge-sidebar-empty">
            {search ? "No results" : "No knowledge yet"}
          </div>
        ) : (
          items.map((item) => (
            <div
              key={item.id}
              className={`knowledge-item ${expanded === item.id ? "knowledge-item-expanded" : ""}`}
              onClick={() => setExpanded(expanded === item.id ? null : item.id)}
            >
              <div className="knowledge-item-header">
                <span
                  className="knowledge-item-source"
                  style={{
                    color: item.source === "memory" ? "var(--info)" : "var(--accent)",
                  }}
                >
                  {item.source === "memory" ? "M" : "B"}
                </span>
                <span className="knowledge-item-key">{item.key}</span>
                {item.category && (
                  <span
                    className="knowledge-item-cat"
                    style={{ color: CATEGORY_COLORS[item.category] || "var(--text-muted)" }}
                  >
                    {item.category}
                  </span>
                )}
              </div>
              {expanded === item.id ? (
                <div className="knowledge-item-content">{item.content}</div>
              ) : (
                <div className="knowledge-item-preview">
                  {item.content.slice(0, 80)}{item.content.length > 80 ? "..." : ""}
                </div>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
