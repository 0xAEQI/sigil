import { useState, useRef, useEffect, useCallback, memo } from "react";
import { api, ApiError } from "@/lib/api";
import type { ThreadEvent } from "@/lib/types";
import { useChatStore } from "@/store/chat";
import { useWebSocket, type WorkerEvent } from "@/hooks/useWebSocket";

interface BubbleMessage {
  kind: "message";
  id: string;
  role: "user" | "assistant";
  content: string;
  timestamp: Date;
  data?: Record<string, unknown>;
  status?: "sending" | "sent" | "error";
}

interface NoticeMessage {
  kind: "notice";
  id: string;
  title: string;
  content: string;
  timestamp: Date;
  tone: "accent" | "success" | "warning" | "error" | "neutral";
  meta?: string;
}

type RenderMessage = BubbleMessage | NoticeMessage;

let msgId = 0;
function nextId() {
  return `msg-${Date.now()}-${++msgId}`;
}

function timeLabel(date: Date): string {
  return date.toLocaleTimeString("en-US", { hour: "2-digit", minute: "2-digit" });
}

function shouldShowTime(current: BubbleMessage, prev?: RenderMessage): boolean {
  if (!prev || prev.kind !== "message") return true;
  if (prev.role !== current.role) return true;
  return current.timestamp.getTime() - prev.timestamp.getTime() > 120000;
}

function formatApiError(error: unknown): string {
  if (error instanceof ApiError) return error.message;
  if (error instanceof Error) return error.message;
  return "Request failed";
}

function parseChannelScope(channel: string | null): {
  project?: string;
  department?: string;
  channelName?: string;
} {
  if (!channel) {
    return { channelName: "sigil" };
  }
  const [project, department] = channel.split("/");
  if (!project) return {};
  return {
    project,
    department: department || undefined,
    channelName: channel,
  };
}

function eventTitle(event: ThreadEvent): string {
  switch (event.event_type) {
    case "task_created":
      return "Task created";
    case "task_released":
      return "Scheduled";
    case "task_completed":
      return "Completed";
    case "task_blocked":
      return "Blocked";
    case "task_cancelled":
      return "Cancelled";
    case "task_timed_out":
      return "Timed out";
    case "task_slow":
      return "Still working";
    case "council_pending":
      return "Council pending";
    case "council_started":
      return "Consulting advisors";
    case "council_ready":
      return "Council attached";
    case "council_advice":
      return `${event.role} advice`;
    case "task_closed":
      return "Task closed";
    case "knowledge_stored":
      return "Knowledge stored";
    default:
      return event.event_type.replaceAll("_", " ");
  }
}

function eventTone(event: ThreadEvent): NoticeMessage["tone"] {
  switch (event.event_type) {
    case "task_completed":
    case "council_ready":
    case "knowledge_stored":
      return "success";
    case "task_blocked":
    case "task_cancelled":
    case "task_timed_out":
      return "error";
    case "task_slow":
      return "warning";
    case "task_created":
    case "task_released":
    case "council_pending":
    case "council_started":
    case "council_advice":
    case "task_closed":
      return "accent";
    default:
      return "neutral";
  }
}

function eventMeta(event: ThreadEvent): string | undefined {
  const parts: string[] = [];
  const taskId = event.metadata?.task_id;
  const project = event.metadata?.project;
  const advisor = event.metadata?.advisor;
  const status = event.metadata?.status;

  if (typeof taskId === "string") parts.push(taskId);
  if (typeof project === "string") parts.push(project);
  if (typeof advisor === "string") parts.push(advisor);
  if (typeof status === "string") parts.push(status);

  return parts.length > 0 ? parts.join(" • ") : undefined;
}

function mapTimeline(events: ThreadEvent[]): RenderMessage[] {
  return events.map((event) => {
    const timestamp = new Date(event.timestamp);
    if (event.event_type === "message") {
      return {
        kind: "message",
        id: `event-${event.id}`,
        role: event.role === "User" ? "user" : "assistant",
        content: event.content,
        timestamp,
      } satisfies BubbleMessage;
    }

    return {
      kind: "notice",
      id: `event-${event.id}`,
      title: eventTitle(event),
      content: event.content,
      timestamp,
      tone: eventTone(event),
      meta: eventMeta(event),
    } satisfies NoticeMessage;
  });
}

// ── Message bubble ──
const ChatBubble = memo(function ChatBubble({
  msg,
  showTime,
  onCopy,
}: {
  msg: BubbleMessage;
  showTime: boolean;
  onCopy: (text: string) => void;
}) {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    onCopy(msg.content);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  const isUser = msg.role === "user";
  const costInfo =
    msg.data &&
    typeof msg.data === "object" &&
    "cost" in msg.data &&
    msg.data.cost &&
    typeof msg.data.cost === "object"
      ? (msg.data.cost as Record<string, unknown>)
      : null;

  return (
    <div className={`c-row ${isUser ? "c-row-user" : "c-row-assistant"}`}>
      {!isUser && (
        <div className="c-avatar">
          <svg
            width="14"
            height="14"
            viewBox="0 0 16 16"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.3"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path d="M3 5l5-3 5 3" />
            <path d="M3 5v6l5 3 5-3V5" />
            <path d="M8 8v6" />
            <circle cx="8" cy="8" r="1.5" fill="currentColor" stroke="none" />
          </svg>
        </div>
      )}
      <div className={`c-bubble ${isUser ? "c-bubble-user" : "c-bubble-assistant"}`}>
        <div className="c-text">{msg.content}</div>
        {costInfo && (
          <div className="c-meta-cost">
            ${Number(costInfo.spent ?? 0).toFixed(3)} / ${Number(costInfo.budget ?? 0).toFixed(2)}
          </div>
        )}
        <div className="c-bubble-footer">
          {showTime && <span className="c-time">{timeLabel(msg.timestamp)}</span>}
          {msg.status === "sending" && <span className="c-status">Sending...</span>}
          {msg.status === "sent" && <span className="c-status">Queued</span>}
          {msg.status === "error" && <span className="c-status c-status-error">Failed</span>}
        </div>
        {!isUser && (
          <button className="c-action" onClick={handleCopy} title="Copy">
            {copied ? (
              <svg
                width="12"
                height="12"
                viewBox="0 0 12 12"
                fill="none"
                stroke="var(--success)"
                strokeWidth="1.5"
              >
                <path d="M2 6l3 3 5-5" />
              </svg>
            ) : (
              <svg
                width="12"
                height="12"
                viewBox="0 0 12 12"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.5"
              >
                <rect x="4" y="4" width="6" height="6" rx="1" />
                <path d="M2 8V2h6" />
              </svg>
            )}
          </button>
        )}
      </div>
    </div>
  );
});

function NoticeRow({ msg }: { msg: NoticeMessage }) {
  return (
    <div className="c-notice-row">
      <div className={`c-notice c-notice-${msg.tone}`}>
        <div className="c-notice-header">
          <span className="c-notice-title">{msg.title}</span>
          <span className="c-notice-time">{timeLabel(msg.timestamp)}</span>
        </div>
        <div className="c-notice-body">{msg.content}</div>
        {msg.meta && <div className="c-notice-meta">{msg.meta}</div>}
      </div>
    </div>
  );
}

// ── Typing indicator ──
function TypingIndicator() {
  return (
    <div className="c-row c-row-assistant">
      <div className="c-avatar">
        <svg
          width="14"
          height="14"
          viewBox="0 0 16 16"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.3"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <path d="M3 5l5-3 5 3" />
          <path d="M3 5v6l5 3 5-3V5" />
          <path d="M8 8v6" />
          <circle cx="8" cy="8" r="1.5" fill="currentColor" stroke="none" />
        </svg>
      </div>
      <div className="c-bubble c-bubble-assistant c-bubble-typing">
        <span className="c-dot" />
        <span className="c-dot" />
        <span className="c-dot" />
      </div>
    </div>
  );
}

// ── Empty state ──
function EmptyChat({ onSuggestion }: { onSuggestion: (value: string) => void }) {
  return (
    <div className="c-empty">
      <div className="c-empty-avatar">
        <svg
          width="24"
          height="24"
          viewBox="0 0 16 16"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.1"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <path d="M3 5l5-3 5 3" />
          <path d="M3 5v6l5 3 5-3V5" />
          <path d="M8 8v6" />
          <circle cx="8" cy="8" r="1.5" fill="currentColor" stroke="none" />
        </svg>
      </div>
      <h2 className="c-empty-title">What needs to happen?</h2>
      <p className="c-empty-hint">
        This thread now follows the daemon&apos;s typed timeline, not a local message cache.
      </p>
      <div className="c-empty-suggestions">
        {[
          "What is the status right now?",
          "Create a task to audit the patrol loop.",
          "What happened overnight?",
        ].map((suggestion) => (
          <button
            key={suggestion}
            className="c-suggestion"
            onClick={() => onSuggestion(suggestion)}
          >
            {suggestion}
          </button>
        ))}
      </div>
    </div>
  );
}

// ── Scroll-to-bottom ──
function ScrollAnchor({ show, onClick }: { show: boolean; onClick: () => void }) {
  if (!show) return null;
  return (
    <button className="c-scroll-btn" onClick={onClick}>
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M7 2v10M3 8l4 4 4-4" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
    </button>
  );
}

export default function ChatPage() {
  const channel = useChatStore((s) => s.channel);
  const thread = useChatStore((s) => s.threads[channel || "__global__"]);
  const getOrCreateThread = useChatStore((s) => s.getOrCreateThread);
  const updateThread = useChatStore((s) => s.updateThread);

  const [timelineEvents, setTimelineEvents] = useState<ThreadEvent[]>([]);
  const [pendingMessages, setPendingMessages] = useState<BubbleMessage[]>([]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [loadingTimeline, setLoadingTimeline] = useState(false);
  const [showScroll, setShowScroll] = useState(false);
  const [dragOver, setDragOver] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const messagesRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const isAtBottom = useRef(true);
  const refreshInFlight = useRef(false);

  const scope = parseChannelScope(channel);

  useEffect(() => {
    getOrCreateThread(channel);
  }, [channel, getOrCreateThread]);

  useEffect(() => {
    setPendingMessages([]);
    setError(null);
  }, [channel]);

  const refreshTimeline = useCallback(async (silent = false) => {
    if (!thread || refreshInFlight.current) return;
    refreshInFlight.current = true;
    if (!silent) setLoadingTimeline(true);

    try {
      const response = await api.chatTimeline({
        chatId: thread.chatId,
        project: scope.project,
        department: scope.department,
        channelName: scope.channelName,
        limit: 200,
      });

      if (typeof response.chat_id === "number" && response.chat_id !== thread.chatId) {
        updateThread(channel, { chatId: response.chat_id });
      }

      setTimelineEvents(Array.isArray(response.events) ? response.events : []);
      setPendingMessages((prev) => prev.filter((msg) => msg.status === "error"));
      setError(null);
    } catch (caughtError) {
      if (!silent) {
        setError(formatApiError(caughtError));
      }
    } finally {
      refreshInFlight.current = false;
      setLoadingTimeline(false);
    }
  }, [channel, thread, updateThread]);

  useEffect(() => {
    if (!thread) return;
    void refreshTimeline();
  }, [thread, refreshTimeline]);

  useEffect(() => {
    if (!thread) return;
    const interval = window.setInterval(() => {
      void refreshTimeline(true);
    }, 2000);
    return () => window.clearInterval(interval);
  }, [thread, refreshTimeline]);

  useEffect(() => {
    if (isAtBottom.current && messagesRef.current) {
      messagesRef.current.scrollTop = messagesRef.current.scrollHeight;
    }
  }, [timelineEvents, pendingMessages, loading]);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const handleScroll = useCallback(() => {
    if (!messagesRef.current) return;
    const { scrollTop, scrollHeight, clientHeight } = messagesRef.current;
    const atBottom = scrollHeight - scrollTop - clientHeight < 60;
    isAtBottom.current = atBottom;
    setShowScroll(!atBottom);
  }, []);

  const scrollToBottom = useCallback(() => {
    if (messagesRef.current) {
      messagesRef.current.scrollTo({ top: messagesRef.current.scrollHeight, behavior: "smooth" });
    }
  }, []);

  const handleInputChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setInput(e.target.value);
    const el = e.target;
    el.style.height = "auto";
    el.style.height = `${Math.min(el.scrollHeight, 160)}px`;
  };

  const copyToClipboard = useCallback((text: string) => {
    navigator.clipboard.writeText(text).catch(() => {});
  }, []);

  const sendMessage = async () => {
    const msg = input.trim();
    if (!msg || loading || !thread) return;

    const optimisticId = nextId();
    const optimisticMessage: BubbleMessage = {
      kind: "message",
      id: optimisticId,
      role: "user",
      content: msg,
      timestamp: new Date(),
      status: "sending",
    };

    setPendingMessages((prev) => [...prev, optimisticMessage]);
    setInput("");
    setError(null);
    setLoading(true);
    if (inputRef.current) inputRef.current.style.height = "auto";

    try {
      const response = await api.chatFull({
        message: msg,
        project: scope.project,
        department: scope.department,
        channelName: scope.channelName,
        chatId: thread.chatId,
        sender: "operator",
      });

      setPendingMessages((prev) =>
        prev.map((entry) =>
          entry.id === optimisticId ? { ...entry, status: "sent" } : entry,
        ),
      );

      if (typeof response.chat_id === "number" && response.chat_id !== thread.chatId) {
        updateThread(channel, { chatId: response.chat_id });
      }

      await refreshTimeline(true);
    } catch (caughtError) {
      setPendingMessages((prev) =>
        prev.map((entry) =>
          entry.id === optimisticId ? { ...entry, status: "error" } : entry,
        ),
      );
      setError(formatApiError(caughtError));
    } finally {
      setLoading(false);
      inputRef.current?.focus();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      void sendMessage();
    }
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
    // TODO: file upload
  };

  const { events: workerEvents } = useWebSocket();
  const activeWorkerEvents = (() => {
    const byTask = new Map<string, WorkerEvent>();
    for (const event of workerEvents) {
      if (scope.project && event.project && event.project !== scope.project) continue;
      if (event.task_id && (event.event_type === "TaskStarted" || event.event_type === "Progress")) {
        byTask.set(event.task_id, event);
      }
      if (event.task_id && (event.event_type === "TaskCompleted" || event.event_type === "TaskFailed")) {
        byTask.delete(event.task_id);
      }
    }
    return Array.from(byTask.values());
  })();

  const timelineMessages = mapTimeline(timelineEvents);
  const messages: RenderMessage[] = [...timelineMessages, ...pendingMessages];
  const hasMessages = messages.length > 0;
  const canSend = input.trim().length > 0 && !loading;

  return (
    <div
      className={`c-page ${dragOver ? "c-page-dragover" : ""}`}
      onDragOver={(e) => { e.preventDefault(); setDragOver(true); }}
      onDragLeave={() => setDragOver(false)}
      onDrop={handleDrop}
    >
      {error && (
        <div className="c-error-banner">
          <span className="c-error-banner-label">Error</span>
          <span>{error}</span>
        </div>
      )}

      <div className="c-messages" ref={messagesRef} onScroll={handleScroll}>
        {!hasMessages && !loading && !loadingTimeline && (
          <EmptyChat onSuggestion={(value) => setInput(value)} />
        )}
        {messages.map((msg, i) =>
          msg.kind === "message" ? (
            <ChatBubble
              key={msg.id}
              msg={msg}
              showTime={shouldShowTime(msg, messages[i - 1])}
              onCopy={copyToClipboard}
            />
          ) : (
            <NoticeRow key={msg.id} msg={msg} />
          ),
        )}
        {(loading || loadingTimeline) && <TypingIndicator />}
      </div>

      {activeWorkerEvents.length > 0 && (
        <div className="c-worker-bar">
          {activeWorkerEvents.map((event) => (
            <div key={`${event.task_id}-${event.event_type}`} className="c-worker-event">
              <span className="c-worker-dot" />
              <span className="c-worker-agent">{event.agent}</span>
              <span className="c-worker-task">{event.task_id}</span>
              {event.turns != null && <span className="c-worker-turns">{event.turns} turns</span>}
              {event.cost_usd != null && (
                <span className="c-worker-cost">${event.cost_usd.toFixed(3)}</span>
              )}
            </div>
          ))}
        </div>
      )}

      <ScrollAnchor show={showScroll} onClick={scrollToBottom} />

      <div className="c-composer">
        <div className="c-composer-inner">
          {channel && <span className="c-composer-ctx">#{channel.split("/").pop()}</span>}
          <textarea
            ref={inputRef}
            className="c-textarea"
            placeholder={channel ? `Message #${channel.split("/").pop()}...` : "What needs to happen?"}
            value={input}
            onChange={handleInputChange}
            onKeyDown={handleKeyDown}
            rows={1}
          />
          <div className="c-composer-actions">
            <span className="c-composer-hint">
              {input.length > 0 ? <kbd>Enter</kbd> : <kbd>Shift+Enter</kbd>}
            </span>
            <button
              className={`c-send ${canSend ? "c-send-ready" : ""} ${loading ? "c-send-loading" : ""}`}
              onClick={() => void sendMessage()}
              disabled={!canSend}
            >
              {loading ? (
                <svg className="c-send-spinner" width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="2">
                  <circle cx="8" cy="8" r="6" strokeDasharray="28" strokeDashoffset="8" strokeLinecap="round" />
                </svg>
              ) : (
                <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M2 8h12M10 4l4 4-4 4" strokeLinecap="round" strokeLinejoin="round" />
                </svg>
              )}
            </button>
          </div>
        </div>
      </div>

      {dragOver && (
        <div className="c-drop-overlay">
          <div className="c-drop-label">Drop files here</div>
        </div>
      )}
    </div>
  );
}
