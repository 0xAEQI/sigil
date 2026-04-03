import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useChatStore } from "@/store/chat";
import { useDaemonStore } from "@/store/daemon";
import type { AgentRef, PersistentAgent, Department } from "@/lib/types";

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

interface DeptNode {
  dept: Department;
  agents: PersistentAgent[];
  children: DeptNode[];
}

function buildTree(
  departments: Department[],
  agents: PersistentAgent[],
  parentId: string | null
): DeptNode[] {
  return departments
    .filter((d) => (d.parent_id || null) === parentId)
    .map((dept) => ({
      dept,
      agents: agents.filter((a) => a.department_id === dept.id),
      children: buildTree(departments, agents, dept.id),
    }))
    .filter((n) => n.agents.length > 0 || n.children.length > 0);
}

function DeptGroupView({
  node,
  depth,
  selectedAgent,
  collapsed,
  onSelectAgent,
  onSelectDept,
  onToggle,
}: {
  node: DeptNode;
  depth: number;
  selectedAgent: AgentRef | null;
  collapsed: Record<string, boolean>;
  onSelectAgent: (agent: AgentRef) => void;
  onSelectDept: (id: string, name: string) => void;
  onToggle: (id: string, e: React.MouseEvent) => void;
}) {
  const isCollapsed = collapsed[node.dept.id] ?? false;
  const isDeptActive = selectedAgent?.id === `dept:${node.dept.id}`;

  // Each depth level gets slightly more bg opacity
  const bg = `rgba(255,255,255,${0.02 + depth * 0.01})`;

  return (
    <div className="dept-group" style={{ background: bg }}>
      <div
        className={`dept-name${isDeptActive ? " active" : ""}`}
        onClick={() => onSelectDept(node.dept.id, node.dept.name)}
      >
        <span className="dept-name-label">{node.dept.name}</span>
        <span className="dept-chevron" onClick={(e) => onToggle(node.dept.id, e)}>
          <Chevron expanded={!isCollapsed} />
        </span>
      </div>
      {!isCollapsed && (
        <>
          {node.agents.map((agent) => {
            const label = agent.display_name || agent.name;
            return (
              <div
                key={agent.id}
                className={`agent-row dept-agent${selectedAgent?.id === agent.id ? " active" : ""}`}
                onClick={() => onSelectAgent({ id: agent.id, name: agent.name, display_name: agent.display_name, project: agent.project, model: agent.model })}
              >
                {label}
              </div>
            );
          })}
          {node.children.map((child) => (
            <DeptGroupView
              key={child.dept.id}
              node={child}
              depth={depth + 1}
              selectedAgent={selectedAgent}
              collapsed={collapsed}
              onSelectAgent={onSelectAgent}
              onSelectDept={onSelectDept}
              onToggle={onToggle}
            />
          ))}
        </>
      )}
    </div>
  );
}

export default function AgentNav() {
  const navigate = useNavigate();
  const channel = useChatStore((s) => s.channel);
  const selectedAgent = useChatStore((s) => s.selectedAgent);
  const setSelectedAgent = useChatStore((s) => s.setSelectedAgent);
  const allAgents = useDaemonStore((s) => s.agents);
  const allDepartments = useDaemonStore((s) => s.departments);
  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});

  const filtered = channel
    ? allAgents.filter((a) => (a.status === "Active" || a.status === "active") && (a.project === channel || !a.project))
    : allAgents.filter((a) => (a.status === "Active" || a.status === "active") && !a.project);

  const filteredDepts = allDepartments.filter((d) => !channel || d.project === channel);
  const tree = buildTree(filteredDepts, filtered, null);

  // Root agents: not in any department
  const allDeptAgentIds = new Set<string>();
  const collectIds = (nodes: DeptNode[]) => {
    for (const n of nodes) {
      n.agents.forEach((a) => allDeptAgentIds.add(a.id));
      collectIds(n.children);
    }
  };
  collectIds(tree);
  const rootAgents = filtered.filter((a) => !allDeptAgentIds.has(a.id));

  const scopeName = channel || "AEQI";

  const toggleDept = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setCollapsed((prev) => ({ ...prev, [id]: !prev[id] }));
  };

  const currentPath = () => {
    const p = window.location.pathname;
    return p === "/login" ? "/" : p;
  };

  const handleSelectAgent = (agent: AgentRef) => {
    setSelectedAgent(agent);
    navigate(`${currentPath()}?agent=${encodeURIComponent(agent.name)}`);
  };

  const handleSelectDept = (id: string, name: string) => {
    setSelectedAgent({ id: `dept:${id}`, name });
    navigate(`${currentPath()}?dept=${encodeURIComponent(name)}`);
  };

  return (
    <nav className="agent-nav">
      {/* Agents list */}
      <div className="agent-nav-panel">
        <div className="agent-nav-panel-title">Agents</div>
        <div className="agent-nav-panel-add" onClick={() => navigate("/agents")}>+</div>

        {rootAgents.map((agent) => {
          const label = agent.display_name || agent.name;
          return (
            <div
              key={agent.id}
              className={`agent-row${selectedAgent?.id === agent.id ? " active" : ""}`}
              onClick={() => handleSelectAgent({ id: agent.id, name: agent.name, display_name: agent.display_name, project: agent.project, model: agent.model })}
            >
              {label}
            </div>
          );
        })}

        {tree.map((node) => (
          <DeptGroupView
            key={node.dept.id}
            node={node}
            depth={0}
            selectedAgent={selectedAgent}
            collapsed={collapsed}
            onSelectAgent={handleSelectAgent}
            onSelectDept={handleSelectDept}
            onToggle={toggleDept}
          />
        ))}
      </div>

    </nav>
  );
}
