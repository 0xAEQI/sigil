import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useChatStore } from "@/store/chat";
import { api } from "@/lib/api";
import type { PersistentAgent, Department } from "@/lib/types";

interface DeptGroup {
  dept: Department;
  agents: PersistentAgent[];
}

export default function AgentNav() {
  const navigate = useNavigate();
  const channel = useChatStore((s) => s.channel);
  const selectedAgent = useChatStore((s) => s.selectedAgent);
  const setSelectedAgent = useChatStore((s) => s.setSelectedAgent);
  const [agents, setAgents] = useState<PersistentAgent[]>([]);
  const [departments, setDepartments] = useState<Department[]>([]);

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

  // Group agents by department
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

  return (
    <nav className="agent-nav">
      <div
        className={`agent-row scope-header${!selectedAgent ? " active" : ""}`}
        onClick={() => { setSelectedAgent(null); navigate("/"); }}
      >
        {scopeName}
      </div>

      <div className="agent-nav-sep" />

      {/* Department groups */}
      {deptGroups.map((group) => (
        <div key={group.dept.id} className="dept-group">
          <div className="dept-name">{group.dept.name}</div>
          {group.agents.map((agent) => (
            <div
              key={agent.id}
              className={`agent-row dept-agent${selectedAgent === agent.name ? " active" : ""}`}
              onClick={() => { setSelectedAgent(agent.name); navigate("/"); }}
            >
              {agent.display_name || agent.name}
            </div>
          ))}
        </div>
      ))}

      {/* Root agents (no department) */}
      {rootAgents.map((agent) => (
        <div
          key={agent.id}
          className={`agent-row${selectedAgent === agent.name ? " active" : ""}`}
          onClick={() => { setSelectedAgent(agent.name); navigate("/"); }}
        >
          {agent.display_name || agent.name}
        </div>
      ))}

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
