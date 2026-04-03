import type { TaskRuntime } from "./runtime";

export interface Company {
  name: string;
  prefix: string;
  repo: string;
  description?: string;
  team: { leader: string; advisors: string[] };
  stats: { pending: number; active: number; done: number; failed: number };
  missions: Mission[];
}

export interface Agent {
  name: string;
  prefix: string;
  model: string;
  role: "leader" | "advisor" | "worker";
  status: "idle" | "working" | "offline";
  expertise: string[];
  current_task?: string;
  stats: { completed: number; failed: number; avg_cost_usd: number };
}

export interface Checkpoint {
  timestamp: string;
  worker: string;
  progress: string;
  cost_usd: number;
  turns_used: number;
}

export interface TaskOutcome {
  kind: string;      // "done", "blocked", "failed", "handoff"
  summary: string;
  reason?: string;
  next_action?: string;
}

export interface Task {
  id: string;
  subject: string;
  description: string;
  status: "pending" | "in_progress" | "done" | "blocked" | "cancelled";
  priority: "critical" | "high" | "normal" | "low";
  assignee?: string;
  agent_id?: string;
  skill?: string;
  mission_id?: string;
  company: string;
  labels: string[];
  cost_usd: number;
  created_at: string;
  updated_at?: string;
  closed_at?: string;
  closed_reason?: string;
  checkpoints?: Checkpoint[];
  depends_on?: string[];
  blocks?: string[];
  acceptance_criteria?: string;
  retry_count?: number;
  locked_by?: string;
  locked_at?: string;
  metadata?: Record<string, unknown>;
  runtime?: TaskRuntime;
}

export interface Mission {
  id: string;
  name: string;
  description: string;
  status: "pending" | "in_progress" | "done" | "cancelled";
  company: string;
  skill?: string;
  schedule?: string;
  task_count: number;
  done_count: number;
  created_at: string;
}

export interface AgentRef {
  id: string;
  name: string;
  display_name?: string;
  project?: string;
  model?: string;
}

export interface PersistentAgent {
  id: string;
  name: string;
  display_name?: string;
  template: string;
  project?: string;
  department_id?: string;
  model?: string;
  capabilities: string[];
  status: string;
  created_at: string;
  session_id?: string;
  color?: string;
  avatar?: string;
}

export interface Department {
  id: string;
  name: string;
  project?: string;
  manager_id?: string;
  parent_id?: string;
  created_at: string;
}

export interface AuditEntry {
  id: number;
  timestamp: string;
  company: string;
  decision_type: string;
  summary: string;
  agent?: string;
  task_id?: string;
  metadata?: Record<string, unknown>;
}

export interface DaemonStatus {
  running: boolean;
  uptime_secs: number;
  companies: number;
  active_workers: number;
  total_cost_usd: number;
  cron_jobs: number;
}

export interface DashboardStats {
  active_workers: number;
  total_cost_today: number;
  tasks_completed_24h: number;
  companies_tracked: number;
  recent_activity: AuditEntry[];
  active_agents: Agent[];
}

export interface ThreadEvent {
  id: number;
  chat_id: number;
  event_type: string;
  role: string;
  content: string;
  timestamp: string;
  source?: string | null;
  metadata?: Record<string, unknown> | null;
}

export interface ChatThreadState {
  chatId?: number;
}

export type TriggerType =
  | { Schedule: { expr: string } }
  | { Once: { at: string } }
  | { Event: { pattern: string; cooldown_secs: number } }
  | { Webhook: { public_id: string } };

export interface Trigger {
  id: string;
  agent_id: string;
  name: string;
  trigger_type: TriggerType;
  skill: string;
  enabled: boolean;
  max_budget_usd?: number;
  created_at: string;
  last_fired?: string;
  fire_count: number;
  total_cost_usd: number;
}
