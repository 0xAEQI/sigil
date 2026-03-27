import { STATUS_COLORS, STATUS_BG_COLORS } from "@/lib/constants";

interface StatusBadgeProps {
  status: string;
  size?: "sm" | "md";
}

const DISPLAY_LABELS: Record<string, string> = {
  idle: "Idle",
  working: "Working",
  offline: "Offline",
  pending: "Pending",
  in_progress: "In Progress",
  done: "Done",
  blocked: "Blocked",
  cancelled: "Cancelled",
  failed: "Failed",
};

export default function StatusBadge({ status, size = "md" }: StatusBadgeProps) {
  return (
    <span
      className={`status-badge status-badge-${size}`}
      style={{
        color: STATUS_COLORS[status] || "var(--text-secondary)",
        backgroundColor: STATUS_BG_COLORS[status] || "rgba(136, 136, 160, 0.1)",
        borderColor: STATUS_COLORS[status] || "var(--text-secondary)",
      }}
    >
      <span
        className="status-dot"
        style={{
          backgroundColor: STATUS_COLORS[status] || "var(--text-secondary)",
        }}
      />
      {DISPLAY_LABELS[status] || status}
    </span>
  );
}
