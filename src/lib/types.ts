export interface Project {
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

export interface Task {
  id: string;
  subject: string;
  description: string;
  status: "pending" | "in_progress" | "done" | "blocked" | "cancelled";
  priority: "critical" | "high" | "normal" | "low";
  assignee?: string;
  skill?: string;
  mission_id?: string;
  project: string;
  labels: string[];
  cost_usd: number;
  created_at: string;
  updated_at?: string;
}

export interface Mission {
  id: string;
  name: string;
  description: string;
  status: "pending" | "in_progress" | "done" | "cancelled";
  project: string;
  skill?: string;
  schedule?: string;
  task_count: number;
  done_count: number;
  created_at: string;
}

export interface AuditEntry {
  id: number;
  timestamp: string;
  project: string;
  decision_type: string;
  summary: string;
  agent?: string;
  task_id?: string;
  metadata?: Record<string, unknown>;
}

export interface DaemonStatus {
  running: boolean;
  uptime_secs: number;
  projects: number;
  active_workers: number;
  total_cost_usd: number;
  cron_jobs: number;
}

export interface DashboardStats {
  active_workers: number;
  total_cost_today: number;
  tasks_completed_24h: number;
  projects_tracked: number;
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
