import { useEffect, useState, useRef } from "react";
import { useSearchParams } from "react-router-dom";
import { api } from "@/lib/api";
import { useChatStore } from "@/store/chat";

interface Session {
  id: string;
  subject: string;
  status: string;
  agent: string;
  project: string;
  skill?: string;
  created_at?: string;
  updated_at?: string;
}

interface Message {
  role: string;
  content: string;
  timestamp: string;
}

function timeAgo(ts: string): string {
  const diff = Date.now() - new Date(ts).getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return "now";
  if (mins < 60) return `${mins}m`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h`;
  return `${Math.floor(hrs / 24)}d`;
}

function statusDot(status: string) {
  if (status === "InProgress" || status === "in_progress") return "● ";
  if (status === "Done" || status === "done") return "○ ";
  if (status === "Blocked" || status === "blocked") return "◌ ";
  return "◌ ";
}

export default function SessionsPage() {
  const [searchParams] = useSearchParams();
  const agentFilter = searchParams.get("agent");
  const selectedAgent = useChatStore((s) => s.selectedAgent);
  const scope = agentFilter || selectedAgent;

  const [sessions, setSessions] = useState<Session[]>([]);
  const [activeSession, setActiveSession] = useState<string | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [sending, setSending] = useState(false);
  const messagesEnd = useRef<HTMLDivElement>(null);

  // Load sessions from tasks
  useEffect(() => {
    api.getTasks({}).then((d: any) => {
      let tasks = (d.tasks || []).map((t: any) => ({
        id: t.id,
        subject: t.subject,
        status: t.status,
        agent: t.assignee || t.agent_id || "—",
        project: t.project || "—",
        skill: t.skill,
        created_at: t.created_at,
        updated_at: t.updated_at,
      }));

      // Filter by agent scope
      if (scope && !scope.startsWith("dept:")) {
        tasks = tasks.filter((t: Session) =>
          t.agent.toLowerCase().includes(scope.toLowerCase())
        );
      }

      // Sort: running first, then by time
      tasks.sort((a: Session, b: Session) => {
        const aActive = a.status === "InProgress" || a.status === "in_progress";
        const bActive = b.status === "InProgress" || b.status === "in_progress";
        if (aActive && !bActive) return -1;
        if (!aActive && bActive) return 1;
        return (b.updated_at || b.created_at || "").localeCompare(a.updated_at || a.created_at || "");
      });

      setSessions(tasks);
    }).catch(() => {});
  }, [scope]);

  // Load transcript for active session
  useEffect(() => {
    if (!activeSession) return;
    api.getChatHistory({ chat_id: undefined, project: undefined, limit: 100 })
      .then((d: any) => {
        // Filter messages that match this task's transcript channel
        setMessages(d.messages || []);
        setTimeout(() => messagesEnd.current?.scrollIntoView({ behavior: "smooth" }), 100);
      })
      .catch(() => setMessages([]));
  }, [activeSession]);

  // Send message to session
  const handleSend = async () => {
    if (!input.trim() || sending) return;
    setSending(true);
    try {
      await api.chatFull({
        message: input,
        sender: "operator",
      });
      setInput("");
    } catch { /* ignore */ }
    setSending(false);
  };

  // ── Session list ──
  if (!activeSession) {
    return (
      <div className="sessions-page">
        <div className="sessions-header">
          <h2 className="sessions-title">Sessions</h2>
          <p className="sessions-meta">
            {sessions.length} session{sessions.length !== 1 ? "s" : ""}
            {scope ? ` · ${scope}` : ""}
          </p>
        </div>

        <div className="sessions-list">
          {sessions.map((s) => (
            <div
              key={s.id}
              className="session-row"
              onClick={() => setActiveSession(s.id)}
            >
              <div className="session-row-main">
                <span className="session-row-status">{statusDot(s.status)}</span>
                <span className="session-row-subject">{s.subject}</span>
              </div>
              <div className="session-row-meta">
                <span className="session-row-agent">{s.agent}</span>
                {s.skill && <span className="session-row-skill">{s.skill}</span>}
                {s.created_at && <span className="session-row-time">{timeAgo(s.created_at)}</span>}
              </div>
            </div>
          ))}

          {sessions.length === 0 && (
            <div className="sessions-empty">No sessions{scope ? ` for ${scope}` : ""}</div>
          )}
        </div>
      </div>
    );
  }

  // ── Session detail ──
  const session = sessions.find((s) => s.id === activeSession);

  return (
    <div className="session-detail">
      <div className="session-detail-header">
        <button className="session-back" onClick={() => setActiveSession(null)}>←</button>
        <div className="session-detail-info">
          <span className="session-detail-subject">{session?.subject || activeSession}</span>
          <span className="session-detail-meta">
            {session?.agent} · {session?.status} {session?.skill ? `· ${session.skill}` : ""}
          </span>
        </div>
      </div>

      <div className="session-messages">
        {messages.length === 0 && (
          <div className="sessions-empty">No transcript available</div>
        )}
        {messages.map((msg, i) => (
          <div key={i} className={`session-msg session-msg-${msg.role}`}>
            <span className="session-msg-role">{msg.role}</span>
            <pre className="session-msg-content">{msg.content}</pre>
          </div>
        ))}
        <div ref={messagesEnd} />
      </div>

      <div className="session-input-wrap">
        <input
          className="session-input"
          type="text"
          placeholder="Send a message..."
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleSend()}
          disabled={sending}
        />
      </div>
    </div>
  );
}
