import { useEffect, useState } from "react";
import Header from "@/components/Header";
import StatusBadge from "@/components/StatusBadge";
import TaskDetailPanel from "@/components/TaskDetailPanel";
import { DataState } from "@/components/ui";
import { PRIORITY_COLORS } from "@/lib/constants";
import { api } from "@/lib/api";
import { runtimeLabel, summarizeTaskRuntime } from "@/lib/runtime";

export default function TasksPage() {
  const [tasks, setTasks] = useState<any[]>([]);
  const [companies, setCompanies] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [statusFilter, setStatusFilter] = useState("");
  const [companyFilter, setCompanyFilter] = useState("");
  const [showForm, setShowForm] = useState(false);
  const [newTask, setNewTask] = useState({ company: "", subject: "", description: "" });
  const [creating, setCreating] = useState(false);
  const [selectedTask, setSelectedTask] = useState<any | null>(null);

  const fetchTasks = () => {
    setLoading(true);
    const params: any = {};
    if (statusFilter) params.status = statusFilter;
    if (companyFilter) params.company = companyFilter;
    api.getTasks(params).then((data) => {
      setTasks(data.tasks || []);
      setLoading(false);
    }).catch(() => setLoading(false));
  };

  useEffect(() => { fetchTasks(); }, [statusFilter, companyFilter]);

  useEffect(() => {
    api.getCompanies().then((data) => setCompanies(data.companies || [])).catch(() => {});
  }, []);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newTask.company || !newTask.subject) return;
    setCreating(true);
    try {
      await api.createTask(newTask);
      setNewTask({ company: "", subject: "", description: "" });
      setShowForm(false);
      fetchTasks();
    } catch {
      // ignore
    }
    setCreating(false);
  };

  const handleClose = async (taskId: string) => {
    await api.closeTask(taskId);
    fetchTasks();
  };

  return (
    <div className="tasks-split">
      <div className="tasks-list-pane">
        <Header
          title="Tasks"
          actions={
            <button className="btn btn-primary" onClick={() => setShowForm(!showForm)}>
              {showForm ? "Cancel" : "+ New Task"}
            </button>
          }
        />

        {showForm && (
          <form className="dash-panel form-panel" onSubmit={handleCreate}>
            <div className="form-row">
              <select
                className="filter-select"
                value={newTask.company}
                onChange={(e) => setNewTask({ ...newTask, company: e.target.value })}
                required
              >
                <option value="">Select company...</option>
                {companies.map((p: any) => (
                  <option key={p.name} value={p.name}>{p.name}</option>
                ))}
              </select>
              <input
                className="filter-input flex-1"
                placeholder="Task subject..."
                value={newTask.subject}
                onChange={(e) => setNewTask({ ...newTask, subject: e.target.value })}
                required
              />
            </div>
            <textarea
              className="filter-input form-textarea"
              placeholder="Description (optional)..."
              value={newTask.description}
              onChange={(e) => setNewTask({ ...newTask, description: e.target.value })}
            />
            <button className="btn btn-primary" type="submit" disabled={creating}>
              {creating ? "Creating..." : "Create Task"}
            </button>
          </form>
        )}

        <div className="filters">
          <select
            className="filter-select"
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value)}
          >
            <option value="">All statuses</option>
            <option value="pending">Pending</option>
            <option value="in_progress">In Progress</option>
            <option value="done">Done</option>
            <option value="blocked">Blocked</option>
            <option value="cancelled">Cancelled</option>
          </select>
          <select
            className="filter-select"
            value={companyFilter}
            onChange={(e) => setCompanyFilter(e.target.value)}
          >
            <option value="">All companies</option>
            {companies.map((p: any) => (
              <option key={p.name} value={p.name}>{p.name}</option>
            ))}
          </select>
          <span className="filter-count">
            {tasks.length} tasks
          </span>
        </div>

        <DataState loading={loading} empty={tasks.length === 0} emptyTitle="No tasks" emptyDescription="No tasks match the current filters." loadingText="Loading tasks...">
          <div className="task-table">
            {tasks.map((task: any) => {
              const label = runtimeLabel(task.runtime);
              const detail = summarizeTaskRuntime(task.runtime, task.closed_reason);
              const isSelected = selectedTask?.id === task.id;

              return (
                <div
                  key={task.id}
                  className={`task-row${isSelected ? " selected" : ""}`}
                  onClick={() => setSelectedTask(isSelected ? null : task)}
                >
                  <span
                    className="task-priority-bar"
                    style={{ backgroundColor: PRIORITY_COLORS[task.priority] || "var(--text-primary)" }}
                  />
                  <code className="task-id">{task.id}</code>
                  <div className="task-row-detail">
                    <span className="task-subject">{task.subject}</span>
                    {(label || detail) && (
                      <span className="task-runtime">
                        {[label, detail].filter(Boolean).join(" \u2022 ")}
                      </span>
                    )}
                  </div>
                  <div className="task-meta">
                    <StatusBadge status={task.status} size="sm" />
                    <span>{task.assignee || "\u2014"}</span>
                    <span>{task.company}</span>
                    {task.status !== "done" && task.status !== "cancelled" && (
                      <button
                        className="btn btn-2xs"
                        onClick={(e) => { e.stopPropagation(); handleClose(task.id); }}
                      >
                        close
                      </button>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        </DataState>
      </div>

      {selectedTask && (
        <div className="task-detail-pane">
          <TaskDetailPanel
            task={selectedTask}
            allTasks={tasks}
            onClose={() => setSelectedTask(null)}
            onSelectTask={(t) => setSelectedTask(t)}
          />
        </div>
      )}
    </div>
  );
}
