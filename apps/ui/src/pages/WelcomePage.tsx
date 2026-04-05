import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useUIStore } from "@/store/ui";
import "@/styles/welcome.css";

export default function WelcomePage() {
  const navigate = useNavigate();
  const activeWorkspace = useUIStore((s) => s.activeWorkspace);
  const setActiveWorkspace = useUIStore((s) => s.setActiveWorkspace);

  const [editingName, setEditingName] = useState(false);
  const [nameDraft, setNameDraft] = useState(activeWorkspace);
  const [editingTagline, setEditingTagline] = useState(false);
  const [tagline, setTagline] = useState(
    () => localStorage.getItem("aeqi_workspace_tagline") || "The agent runtime.",
  );
  const [taglineDraft, setTaglineDraft] = useState(tagline);

  const saveName = () => {
    if (nameDraft.trim()) {
      setActiveWorkspace(nameDraft.trim());
    }
    setEditingName(false);
  };

  const saveTagline = () => {
    const val = taglineDraft.trim() || "The agent runtime.";
    setTagline(val);
    localStorage.setItem("aeqi_workspace_tagline", val);
    setEditingTagline(false);
  };

  const items = [
    { icon: "✦", name: "Agents", desc: "Autonomous entities that research, plan, implement, and verify.", route: "/agents" },
    { icon: "⚡", name: "Events", desc: "Real-time activity stream. Decisions, messages, and approvals.", route: "/events" },
    { icon: "◆", name: "Quests", desc: "Units of work tracked through your agent pipeline.", route: "/quests" },
    { icon: "◉", name: "Insights", desc: "Knowledge your agents accumulate and share across sessions.", route: "/insights" },
    { icon: "◫", name: "Company", desc: "Team, settings, and configuration for this workspace.", route: "/company" },
    { icon: "▤", name: "Drive", desc: "Files, prompts, agent templates, and artifacts.", route: "/drive" },
    { icon: "⊞", name: "Apps", desc: "Integrations, MCP tools, and third-party connections.", route: "/apps" },
  ];

  return (
    <div className="welcome">
      <div className="welcome-hero">
        {editingName ? (
          <input
            className="welcome-title-input"
            value={nameDraft}
            onChange={(e) => setNameDraft(e.target.value)}
            onBlur={saveName}
            onKeyDown={(e) => {
              if (e.key === "Enter") saveName();
              if (e.key === "Escape") { setEditingName(false); setNameDraft(activeWorkspace); }
            }}
            autoFocus
          />
        ) : (
          <h1
            className="welcome-title welcome-editable"
            onClick={() => { setEditingName(true); setNameDraft(activeWorkspace); }}
            title="Click to rename workspace"
          >
            {activeWorkspace || "aeqi"}<span className="welcome-dot">.ai</span>
          </h1>
        )}

        {editingTagline ? (
          <input
            className="welcome-tagline-input"
            value={taglineDraft}
            onChange={(e) => setTaglineDraft(e.target.value)}
            onBlur={saveTagline}
            onKeyDown={(e) => {
              if (e.key === "Enter") saveTagline();
              if (e.key === "Escape") { setEditingTagline(false); setTaglineDraft(tagline); }
            }}
            autoFocus
          />
        ) : (
          <p
            className="welcome-tagline welcome-editable"
            onClick={() => { setEditingTagline(true); setTaglineDraft(tagline); }}
            title="Click to edit tagline"
          >
            {tagline}
          </p>
        )}
      </div>

      <div className="welcome-grid">
        {items.map((item) => (
          <div key={item.name} className="welcome-card" onClick={() => navigate(item.route)}>
            <span className="welcome-card-icon">{item.icon}</span>
            <div className="welcome-card-body">
              <h3>{item.name}</h3>
              <p>{item.desc}</p>
            </div>
            <span className="welcome-card-arrow">&rarr;</span>
          </div>
        ))}
      </div>

      <div className="welcome-footer">
        <p>Select an agent in the sidebar to start a session.</p>
      </div>
    </div>
  );
}
