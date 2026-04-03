import { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";
import Header from "@/components/Header";
import StatusBadge from "@/components/StatusBadge";
import MissionCard from "@/components/MissionCard";
import AuditEntryComponent from "@/components/AuditEntry";
import { HeroStats, Panel, DetailField, TagList, ProgressBar, Tabs } from "@/components/ui";
import { PRIORITY_COLORS } from "@/lib/constants";
import { api } from "@/lib/api";
import { runtimeLabel, summarizeTaskRuntime } from "@/lib/runtime";

export default function CompanyDetailPage() {
  const { name } = useParams<{ name: string }>();
  const [company, setCompany] = useState<any>(null);
  const [tasks, setTasks] = useState<any[]>([]);
  const [missions, setMissions] = useState<any[]>([]);
  const [audit, setAudit] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!name) return;
    setLoading(true);

    Promise.all([
      api.getCompanies().then((d) => {
        const p = (d.companies || []).find((p: any) => p.name === name);
        setCompany(p || null);
      }),
      api.getTasks({ company: name }).then((d) => setTasks(d.tasks || [])),
      api.getMissions({ company: name }).then((d) => setMissions(d.missions || [])),
      api.getAudit({ company: name, last: 30 }).then((d) => setAudit(d.events || [])),
    ])
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [name]);

  if (loading) return <div className="loading">Loading company...</div>;
  if (!company) return <div className="loading">Company not found</div>;

  const pendingTasks = tasks.filter((t) => t.status === "pending");
  const activeTasks = tasks.filter((t) => t.status === "in_progress");
  const doneTasks = tasks.filter((t) => t.status === "done");
  const total = tasks.length;
  const donePct = total > 0 ? (doneTasks.length / total) * 100 : 0;

  return (
    <>
      <Header
        title={company.name}
        breadcrumbs={[
          { label: "Companies", href: "/companies" },
          { label: company.name },
        ]}
      />

      {/* Hero Stats */}
      <HeroStats stats={[
        { value: total, label: "Total Tasks" },
        { value: pendingTasks.length, label: "Pending", color: "muted" },
        { value: activeTasks.length, label: "In Progress", color: "info" },
        { value: doneTasks.length, label: "Done", color: "success" },
        { value: missions.length, label: "Missions" },
      ]} />

      {/* Company Info */}
      <div className="detail-grid">
        <div className="detail-sidebar">
          {/* Info Panel */}
          <Panel variant="detail" title="Company Info">
            <DetailField label="Prefix"><code>{company.prefix}</code></DetailField>
            {company.team && (
              <>
                <DetailField label="Team Leader">{company.team.leader}</DetailField>
                <DetailField label="Team">
                  <TagList items={company.team.agents || []} />
                </DetailField>
              </>
            )}
            <DetailField label="Progress">
              <ProgressBar value={donePct} label={`${donePct.toFixed(0)}% complete`} />
            </DetailField>
          </Panel>
        </div>

        {/* Main Content */}
        <div className="detail-main">
          <Tabs tabs={[
            {
              id: "tasks",
              label: "Tasks",
              count: tasks.length,
              content: (
                <div className="task-table">
                  {tasks.length === 0 ? (
                    <div className="dash-empty">No tasks in this company</div>
                  ) : (
                    tasks.slice(0, 50).map((task: any) => {
                      const label = runtimeLabel(task.runtime);
                      const detail = summarizeTaskRuntime(task.runtime, task.closed_reason);

                      return (
                        <div key={task.id} className="task-row">
                          <span
                            className="task-priority-bar"
                            style={{ backgroundColor: PRIORITY_COLORS[task.priority] || "var(--text-primary)" }}
                          />
                          <code className="task-id">{task.id}</code>
                          <div className="task-row-detail">
                            <span className="task-subject">{task.subject}</span>
                            {(label || detail) && (
                              <span className="task-runtime">
                                {[label, detail].filter(Boolean).join(" • ")}
                              </span>
                            )}
                          </div>
                          <div className="task-meta">
                            <StatusBadge status={task.status} size="sm" />
                            <span>{task.assignee || "—"}</span>
                          </div>
                        </div>
                      );
                    })
                  )}
                  {tasks.length > 50 && (
                    <div className="dash-empty">
                      <Link to={`/tasks?company=${name}`}>View all {tasks.length} tasks</Link>
                    </div>
                  )}
                </div>
              ),
            },
            {
              id: "missions",
              label: "Missions",
              count: missions.length,
              content: (
                <div>
                  {missions.length === 0 ? (
                    <div className="dash-empty">No missions in this company</div>
                  ) : (
                    <div className="cards-grid">
                      {missions.map((m: any) => (
                        <MissionCard key={m.id} mission={m} />
                      ))}
                    </div>
                  )}
                </div>
              ),
            },
            {
              id: "audit",
              label: "Audit",
              count: audit.length,
              content: (
                <div className="column-section">
                  <div className="column-section-body">
                    {audit.length === 0 ? (
                      <div className="dash-empty">No audit events for this company</div>
                    ) : (
                      audit.map((entry: any, i: number) => (
                        <AuditEntryComponent key={i} entry={entry} />
                      ))
                    )}
                  </div>
                </div>
              ),
            },
          ]} />
        </div>
      </div>
    </>
  );
}
