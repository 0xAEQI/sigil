import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useChatStore } from "@/store/chat";
import { api } from "@/lib/api";
import type { PersistentAgent, Department } from "@/lib/types";

interface DeptGroup {
  dept: Department;
  agents: PersistentAgent[];
}

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

export default function AgentNav() {
  const navigate = useNavigate();
  const channel = useChatStore((s) => s.channel);
  const selectedAgent = useChatStore((s) => s.selectedAgent);
  const setSelectedAgent = useChatStore((s) => s.setSelectedAgent);
  const [agents, setAgents] = useState<PersistentAgent[]>([]);
  const [departments, setDepartments] = useState<Department[]>([]);
  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});

  useEffect(() => {
    const load = () => {
      api.getAgents().then((d: any) => {
        const list = d.agents || d.registry || [];
        setAgents(list.filter((a: PersistentAgent) => a.status === "Active" || a.status === "active"));
      }).catch(() => {});

      api.getDepartments?.().then((d: any) => {
        setDepartments(d.departments || []);
      }).catch(() => {});
    };
    load();
    const interval = setInterval(load, 20000);
    return () => clearInterval(interval);
  }, [channel]);

  const filtered = channel
    ? agents.filter((a) => a.project === channel || !a.project)
    : agents.filter((a) => !a.project);

  const deptGroups: DeptGroup[] = departments
    .filter((d) => !channel || d.project === channel)
    .map((dept) => ({
      dept,
      agents: filtered.filter((a) => a.department_id === dept.id),
    }))
    .filter((g) => g.agents.length > 0);

  const deptAgentIds = new Set(deptGroups.flatMap((g) => g.agents.map((a) => a.id)));
  const rootAgents = filtered.filter((a) => !deptAgentIds.has(a.id));

  const scopeName = channel || "AEQI";

  const toggleDept = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setCollapsed((prev) => ({ ...prev, [id]: !prev[id] }));
  };

  return (
    <nav className="agent-nav">
      <div
        className={`agent-row scope-header${!selectedAgent ? " active" : ""}`}
        onClick={() => { setSelectedAgent(null); navigate("/"); }}
      >
        {scopeName}
      </div>

      <div className="agent-nav-sep" />

      {/* Root agents (no department) — above departments */}
      {rootAgents.map((agent) => (
        <div
          key={agent.id}
          className={`agent-row${selectedAgent === agent.name ? " active" : ""}`}
          onClick={() => { setSelectedAgent(agent.name); navigate("/"); }}
        >
          {agent.display_name || agent.name}
        </div>
      ))}

      {/* Department groups */}
      {deptGroups.map((group) => {
        const isCollapsed = collapsed[group.dept.id] ?? false;
        const isDeptActive = selectedAgent === `dept:${group.dept.id}`;

        return (
          <div key={group.dept.id} className="dept-group">
            <div
              className={`dept-name${isDeptActive ? " active" : ""}`}
              onClick={() => { setSelectedAgent(`dept:${group.dept.id}`); navigate(`/departments/${group.dept.id}`); }}
            >
              <span className="dept-name-label">{group.dept.name}</span>
              <span className="dept-chevron" onClick={(e) => toggleDept(group.dept.id, e)}>
                <Chevron expanded={!isCollapsed} />
              </span>
            </div>
            {!isCollapsed && group.agents.map((agent) => (
              <div
                key={agent.id}
                className={`agent-row dept-agent${selectedAgent === agent.name ? " active" : ""}`}
                onClick={() => { setSelectedAgent(agent.name); navigate("/"); }}
              >
                {agent.display_name || agent.name}
              </div>
            ))}
          </div>
        );
      })}

      <div className="agent-nav-add" onClick={() => navigate("/agents")}>+</div>

      <div
        className="agent-nav-footer"
        onClick={() => navigate("/settings")}
      >
        Settings
      </div>
    </nav>
  );
}
