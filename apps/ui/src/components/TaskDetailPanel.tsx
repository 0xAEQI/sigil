import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import StatusBadge from "./StatusBadge";
import Tabs from "./ui/Tabs";
import { PRIORITY_COLORS } from "@/lib/constants";
import { api } from "@/lib/api";
import {
  runtimeLabel,
  summarizeTaskRuntime,
  extractRuntime,
  extractOutcome,
} from "@/lib/runtime";
import {
  checkpointsToTimeline,
  auditToTimeline,
  mergeTimelines,
} from "@/lib/events";
import type { TimelineItem } from "@/lib/events";

interface TaskDetailPanelProps {
  task: any;
  allTasks: any[];
  onClose: () => void;
  onSelectTask: (task: any) => void;
}

function formatDate(dateStr?: string | null): string {
  if (!dateStr) return "\u2014";
  const d = new Date(dateStr);
  return d.toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function formatTimelineTime(dateStr: string): string {
  const d = new Date(dateStr);
  return d.toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function OverviewTab({ task }: { task: any }) {
  const runtime = extractRuntime(task);
  const outcome = extractOutcome(task);
  const phase = runtime?.session?.phase;
  const runtimeSummary = summarizeTaskRuntime(runtime, task.closed_reason);
  const label = runtimeLabel(runtime);

  return (
    <div className="task-detail-body">
      {/* Subject */}
      <h3 style={{ margin: "0 0 8px", fontSize: "var(--font-size-lg)", fontWeight: 600, color: "var(--text-primary)" }}>
        {task.subject}
      </h3>

      {/* Description */}
      {task.description && (
        <p style={{ margin: "0 0 20px", fontSize: "var(--font-size-sm)", color: "var(--text-secondary)", lineHeight: 1.5 }}>
          {task.description}
        </p>
      )}

      {/* Fields grid */}
      <div className="task-detail-section">
        <div className="task-detail-section-title">Details</div>
        <div className="task-detail-grid">
          <div>
            <div className="task-detail-field-label">Status</div>
            <div className="task-detail-field-value"><StatusBadge status={task.status} size="sm" /></div>
          </div>
          <div>
            <div className="task-detail-field-label">Priority</div>
            <div className="task-detail-field-value" style={{ color: PRIORITY_COLORS[task.priority] || "var(--text-primary)" }}>
              {task.priority}
            </div>
          </div>
          <div>
            <div className="task-detail-field-label">Company</div>
            <div className="task-detail-field-value">{task.company || "\u2014"}</div>
          </div>
          <div>
            <div className="task-detail-field-label">Assignee</div>
            <div className="task-detail-field-value">{task.assignee || "\u2014"}</div>
          </div>
          <div>
            <div className="task-detail-field-label">Agent ID</div>
            <div className="task-detail-field-value">
              {task.agent_id ? <code>{task.agent_id}</code> : "\u2014"}
            </div>
          </div>
          <div>
            <div className="task-detail-field-label">Locked By</div>
            <div className="task-detail-field-value">{task.locked_by || "\u2014"}</div>
          </div>
          <div>
            <div className="task-detail-field-label">Retry Count</div>
            <div className="task-detail-field-value">{task.retry_count ?? 0}</div>
          </div>
          <div>
            <div className="task-detail-field-label">Cost</div>
            <div className="task-detail-field-value">
              {task.cost_usd != null ? `$${task.cost_usd.toFixed(2)}` : "\u2014"}
            </div>
          </div>
        </div>
      </div>

      {/* Timestamps */}
      <div className="task-detail-section">
        <div className="task-detail-section-title">Timestamps</div>
        <div className="task-detail-grid">
          <div>
            <div className="task-detail-field-label">Created</div>
            <div className="task-detail-field-value">{formatDate(task.created_at)}</div>
          </div>
          <div>
            <div className="task-detail-field-label">Updated</div>
            <div className="task-detail-field-value">{formatDate(task.updated_at)}</div>
          </div>
          {task.closed_at && (
            <div>
              <div className="task-detail-field-label">Closed</div>
              <div className="task-detail-field-value">{formatDate(task.closed_at)}</div>
            </div>
          )}
        </div>
      </div>

      {/* Runtime */}
      {(phase || label || runtimeSummary) && (
        <div className="task-detail-section">
          <div className="task-detail-section-title">Runtime</div>
          {phase && (
            <span className={`task-phase-pill task-phase-${phase}`}>
              {phase}
            </span>
          )}
          {runtimeSummary && (
            <p style={{ margin: "8px 0 0", fontSize: "var(--font-size-sm)", color: "var(--text-secondary)" }}>
              {runtimeSummary}
            </p>
          )}
        </div>
      )}

      {/* Outcome with verification */}
      {outcome && (
        <div className="task-detail-section">
          <div className="task-detail-section-title">Outcome</div>
          <div style={{ fontSize: "var(--font-size-sm)", color: "var(--text-secondary)" }}>
            <strong>{outcome.kind}</strong>: {outcome.summary}
            {outcome.reason && <p style={{ margin: "4px 0 0", color: "var(--text-muted)" }}>{outcome.reason}</p>}
            {outcome.next_action && <p style={{ margin: "4px 0 0", color: "var(--text-muted)" }}>Next: {outcome.next_action}</p>}
          </div>
        </div>
      )}

      {/* Verification from runtime */}
      {runtime?.outcome?.verification && (
        <div className={`task-verification${runtime.outcome.verification.approved ? " task-verification-approved" : ""}`}>
          <div style={{ fontSize: "var(--font-size-sm)", color: "var(--text-secondary)" }}>
            <strong>Verification:</strong> {runtime.outcome.verification.approved ? "Approved" : "Not approved"}
            {runtime.outcome.verification.confidence != null && (
              <span style={{ marginLeft: 8, color: "var(--text-muted)" }}>
                Confidence: {(runtime.outcome.verification.confidence * 100).toFixed(0)}%
              </span>
            )}
          </div>
          {runtime.outcome.verification.warnings && runtime.outcome.verification.warnings.length > 0 && (
            <div className="task-verification-warnings">
              {runtime.outcome.verification.warnings.map((w, i) => (
                <div key={i}>{w}</div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function TimelineTab({ task }: { task: any }) {
  const [items, setItems] = useState<TimelineItem[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);

    api.getAuditForTask(task.id).then((data) => {
      if (cancelled) return;
      const auditItems = auditToTimeline(data.entries || []);
      const cpItems = task.checkpoints
        ? checkpointsToTimeline(task.checkpoints, task.id)
        : [];
      setItems(mergeTimelines(auditItems, cpItems));
      setLoading(false);
    }).catch(() => {
      if (!cancelled) setLoading(false);
    });

    return () => { cancelled = true; };
  }, [task.id, task.checkpoints]);

  if (loading) {
    return (
      <div className="task-detail-body" style={{ color: "var(--text-muted)", fontSize: "var(--font-size-sm)" }}>
        Loading timeline...
      </div>
    );
  }

  if (items.length === 0) {
    return (
      <div className="task-detail-body" style={{ color: "var(--text-muted)", fontSize: "var(--font-size-sm)" }}>
        No timeline events yet.
      </div>
    );
  }

  return (
    <div className="task-detail-body">
      <div className="task-timeline">
        {items.map((item, i) => (
          <div key={item.id} className="task-timeline-item">
            <div className="task-timeline-rail">
              <div className={`task-timeline-dot task-timeline-dot-${item.type}`} />
              {i < items.length - 1 && <div className="task-timeline-line" />}
            </div>
            <div className="task-timeline-content">
              <div className="task-timeline-header">
                <span className="task-timeline-type">{item.type.replace(/_/g, " ")}</span>
                <span className="task-timeline-time">{formatTimelineTime(item.timestamp)}</span>
              </div>
              {item.summary && (
                <div className="task-timeline-summary">{item.summary}</div>
              )}
              <div className="task-timeline-meta">
                {item.agent && <span>{item.agent}</span>}
                {item.checkpoint?.cost_usd != null && (
                  <span>${item.checkpoint.cost_usd.toFixed(2)}</span>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Session link */}
      <div style={{ marginTop: 16 }}>
        <Link
          to={`/sessions?task=${task.id}`}
          style={{ fontSize: "var(--font-size-xs)", color: "var(--accent)" }}
        >
          View session transcript
        </Link>
      </div>
    </div>
  );
}

function DependenciesTab({ task, allTasks, onSelectTask }: { task: any; allTasks: any[]; onSelectTask: (t: any) => void }) {
  const dependsOn: string[] = task.depends_on || [];
  const blocks: string[] = task.blocks || [];

  if (dependsOn.length === 0 && blocks.length === 0) {
    return (
      <div className="task-detail-body">
        <div className="task-dep-empty">No dependencies</div>
      </div>
    );
  }

  const renderChips = (ids: string[]) => (
    <div className="task-dep-list">
      {ids.map((id) => {
        const found = allTasks.find((t) => t.id === id);
        return (
          <div
            key={id}
            className="task-dep-chip"
            onClick={() => found && onSelectTask(found)}
          >
            <span className="task-dep-chip-id">{id}</span>
            <span className="task-dep-chip-subject">{found?.subject || "Unknown"}</span>
            {found && <StatusBadge status={found.status} size="sm" />}
          </div>
        );
      })}
    </div>
  );

  return (
    <div className="task-detail-body">
      {dependsOn.length > 0 && (
        <div className="task-detail-section">
          <div className="task-detail-section-title">Depends On</div>
          {renderChips(dependsOn)}
        </div>
      )}
      {blocks.length > 0 && (
        <div className="task-detail-section">
          <div className="task-detail-section-title">Blocks</div>
          {renderChips(blocks)}
        </div>
      )}
    </div>
  );
}

export default function TaskDetailPanel({ task, allTasks, onClose, onSelectTask }: TaskDetailPanelProps) {
  return (
    <>
      <div className="task-detail-header">
        <span className="task-detail-title">
          <code style={{ marginRight: 8, color: "var(--text-muted)" }}>{task.id}</code>
          Task Details
        </span>
        <button className="task-detail-close" onClick={onClose}>&times;</button>
      </div>
      <Tabs
        tabs={[
          {
            id: "overview",
            label: "Overview",
            content: <OverviewTab task={task} />,
          },
          {
            id: "timeline",
            label: "Timeline",
            content: <TimelineTab task={task} />,
          },
          {
            id: "dependencies",
            label: "Dependencies",
            count: (task.depends_on?.length || 0) + (task.blocks?.length || 0),
            content: (
              <DependenciesTab
                task={task}
                allTasks={allTasks}
                onSelectTask={onSelectTask}
              />
            ),
          },
        ]}
        defaultTab="overview"
      />
    </>
  );
}
