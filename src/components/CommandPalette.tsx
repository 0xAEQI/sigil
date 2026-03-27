import { useState, useEffect, useRef, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { api } from "@/lib/api";

interface PaletteItem {
  id: string;
  label: string;
  hint?: string;
  section: string;
  action: () => void;
}

export default function CommandPalette({ open, onClose }: { open: boolean; onClose: () => void }) {
  const [query, setQuery] = useState("");
  const [items, setItems] = useState<PaletteItem[]>([]);
  const [selected, setSelected] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const navigate = useNavigate();

  const go = useCallback((path: string) => { navigate(path); onClose(); }, [navigate, onClose]);

  useEffect(() => {
    if (!open) { setQuery(""); setSelected(0); return; }
    inputRef.current?.focus();

    // Build items from API data
    const buildItems = async () => {
      const navItems: PaletteItem[] = [
        { id: "nav-chat", label: "Chat", hint: "Home", section: "Navigate", action: () => go("/") },
        { id: "nav-dashboard", label: "Dashboard", hint: "Command center", section: "Navigate", action: () => go("/dashboard") },
        { id: "nav-tasks", label: "Tasks", hint: "View all tasks", section: "Navigate", action: () => go("/tasks") },
        { id: "nav-missions", label: "Missions", hint: "Mission tracker", section: "Navigate", action: () => go("/missions") },
        { id: "nav-knowledge", label: "Knowledge", hint: "Browse memories", section: "Navigate", action: () => go("/knowledge") },
        { id: "nav-skills", label: "Skills", hint: "Skill catalog", section: "Navigate", action: () => go("/skills") },
        { id: "nav-ops", label: "Operations", hint: "Crons & watchdogs", section: "Navigate", action: () => go("/operations") },
        { id: "nav-cost", label: "Cost", hint: "Budget tracking", section: "Navigate", action: () => go("/cost") },
        { id: "nav-audit", label: "Audit", hint: "Decision trail", section: "Navigate", action: () => go("/audit") },
        { id: "nav-settings", label: "Settings", hint: "Configuration", section: "Navigate", action: () => go("/settings") },
      ];

      try {
        const [projectsData, agentsData] = await Promise.all([
          api.getProjects().catch(() => ({ projects: [] })),
          api.getAgents().catch(() => ({ agents: [] })),
        ]);

        const projectItems: PaletteItem[] = (projectsData.projects || []).map((p: any) => ({
          id: `proj-${p.name}`,
          label: p.name,
          hint: `${p.open_tasks || 0} open tasks`,
          section: "Projects",
          action: () => go(`/projects/${p.name}`),
        }));

        const agentItems: PaletteItem[] = (agentsData.agents || []).map((a: any) => ({
          id: `agent-${a.name}`,
          label: a.name,
          hint: a.role,
          section: "Agents",
          action: () => go(`/agents/${a.name}`),
        }));

        setItems([...navItems, ...projectItems, ...agentItems]);
      } catch {
        setItems(navItems);
      }
    };
    buildItems();
  }, [open, go]);

  const filtered = query
    ? items.filter((item) =>
        item.label.toLowerCase().includes(query.toLowerCase()) ||
        (item.hint || "").toLowerCase().includes(query.toLowerCase())
      )
    : items;

  useEffect(() => { setSelected(0); }, [query]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") { onClose(); return; }
    if (e.key === "ArrowDown") { e.preventDefault(); setSelected((s) => Math.min(s + 1, filtered.length - 1)); }
    if (e.key === "ArrowUp") { e.preventDefault(); setSelected((s) => Math.max(s - 1, 0)); }
    if (e.key === "Enter" && filtered[selected]) { filtered[selected].action(); }
  };

  if (!open) return null;

  // Group by section
  const sections: Record<string, PaletteItem[]> = {};
  filtered.forEach((item) => {
    if (!sections[item.section]) sections[item.section] = [];
    sections[item.section].push(item);
  });

  let globalIndex = 0;

  return (
    <div className="palette-overlay" onClick={onClose}>
      <div className="palette" onClick={(e) => e.stopPropagation()}>
        <input
          ref={inputRef}
          className="palette-input"
          placeholder="Where to?"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={handleKeyDown}
        />
        <div className="palette-results">
          {Object.entries(sections).map(([section, sectionItems]) => (
            <div key={section}>
              <div className="palette-section">{section}</div>
              {sectionItems.map((item) => {
                const idx = globalIndex++;
                return (
                  <div
                    key={item.id}
                    className={`palette-item ${idx === selected ? "palette-item-active" : ""}`}
                    onClick={item.action}
                    onMouseEnter={() => setSelected(idx)}
                  >
                    <span className="palette-item-label">{item.label}</span>
                    {item.hint && <span className="palette-item-hint">{item.hint}</span>}
                  </div>
                );
              })}
            </div>
          ))}
          {filtered.length === 0 && (
            <div className="palette-empty">No results</div>
          )}
        </div>
      </div>
    </div>
  );
}
