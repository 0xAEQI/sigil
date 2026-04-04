import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useChatStore } from "@/store/chat";
import { useDaemonStore } from "@/store/daemon";
import type { Agent, AgentRef } from "@/lib/types";

function Chevron({ expanded }: { expanded: boolean }) {
  return (
    <svg
      width="14"
      height="14"
      viewBox="0 0 14 14"
      fill="none"
      style={{
        transform: expanded ? "rotate(90deg)" : "rotate(0deg)",
        transition: "transform 0.15s ease",
        flexShrink: 0,
      }}
    >
      <path
        d="M5 3.5L8.5 7L5 10.5"
        stroke="currentColor"
        strokeWidth="1.2"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

interface AgentNode {
  id: string;
  name: string;
  display_name?: string;
  status: string;
  model?: string;
  children: AgentNode[];
}

function buildAgentTree(agents: Agent[]): AgentNode[] {
  const byId = new Map<string, Agent>();
  for (const a of agents) byId.set(a.id, a);

  const childrenMap = new Map<string, Agent[]>();
  const roots: Agent[] = [];

  for (const a of agents) {
    if (a.parent_id && byId.has(a.parent_id)) {
      const existing = childrenMap.get(a.parent_id) || [];
      existing.push(a);
      childrenMap.set(a.parent_id, existing);
    } else {
      roots.push(a);
    }
  }

  function toNode(agent: Agent): AgentNode {
    const kids = childrenMap.get(agent.id) || [];
    return {
      id: agent.id,
      name: agent.name,
      display_name: agent.display_name,
      status: agent.status,
      model: agent.model,
      children: kids.map(toNode),
    };
  }

  return roots.map(toNode);
}

function statusColor(status: string): string {
  const s = status.toLowerCase();
  if (s === "active" || s === "working" || s === "running") return "var(--success)";
  if (s === "paused" || s === "idle") return "var(--text-muted)";
  if (s === "error" || s === "failed") return "var(--error)";
  return "var(--text-muted)";
}

function AgentNodeView({
  node,
  depth,
  selectedAgent,
  collapsed,
  onSelectAgent,
  onToggle,
}: {
  node: AgentNode;
  depth: number;
  selectedAgent: AgentRef | null;
  collapsed: Record<string, boolean>;
  onSelectAgent: (agent: AgentRef) => void;
  onToggle: (id: string, e: React.MouseEvent) => void;
}) {
  const isActive = selectedAgent?.id === node.id;
  const hasChildren = node.children.length > 0;
  const isCollapsed = collapsed[node.id] ?? false;
  const label = node.display_name || node.name;

  return (
    <div className="agent-tree-node">
      <div
        className={`agent-row${isActive ? " active" : ""}`}
        style={{ paddingLeft: `${8 + depth * 12}px` }}
        onClick={() =>
          onSelectAgent({
            id: node.id,
            name: node.name,
            display_name: node.display_name,
            model: node.model,
          })
        }
      >
        {hasChildren ? (
          <span
            className="agent-tree-toggle"
            onClick={(e) => onToggle(node.id, e)}
          >
            <Chevron expanded={!isCollapsed} />
          </span>
        ) : (
          <span className="agent-tree-spacer" />
        )}
        <span
          className="agent-dot"
          style={{ background: statusColor(node.status) }}
        />
        <span className="agent-row-label">{label}</span>
      </div>
      {hasChildren && !isCollapsed && (
        <div className="agent-tree-children">
          {node.children.map((child) => (
            <AgentNodeView
              key={child.id}
              node={child}
              depth={depth + 1}
              selectedAgent={selectedAgent}
              collapsed={collapsed}
              onSelectAgent={onSelectAgent}
              onToggle={onToggle}
            />
          ))}
        </div>
      )}
    </div>
  );
}

export default function AgentTree() {
  const navigate = useNavigate();
  const selectedAgent = useChatStore((s) => s.selectedAgent);
  const setSelectedAgent = useChatStore((s) => s.setSelectedAgent);
  const allAgents = useDaemonStore((s) => s.agents);
  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});

  const tree = buildAgentTree(allAgents);

  const toggleNode = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setCollapsed((prev) => ({ ...prev, [id]: !prev[id] }));
  };

  const handleSelectAgent = (agent: AgentRef) => {
    setSelectedAgent(agent);
    navigate(`/?agent=${encodeURIComponent(agent.id)}`);
  };

  const handleClearSelection = () => {
    setSelectedAgent(null);
    navigate("/");
  };

  return (
    <nav className="agent-tree">
      <div className="agent-tree-list">
        {tree.map((node) => (
          <AgentNodeView
            key={node.id}
            node={node}
            depth={0}
            selectedAgent={selectedAgent}
            collapsed={collapsed}
            onSelectAgent={handleSelectAgent}
            onToggle={toggleNode}
          />
        ))}

        {allAgents.length === 0 && (
          <div className="agent-tree-empty">No agents yet</div>
        )}
      </div>
    </nav>
  );
}
