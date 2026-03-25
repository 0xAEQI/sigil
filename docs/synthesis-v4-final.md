# The Synthesis — Sigil v4 Final

What the best agent orchestrator in the world actually looks like, derived from
deep code-level analysis of Sigil, Hermes, Supermemory, and Deer Flow.

---

## The Critical Finding

Sigil's v4 modules (middleware, verification, memory graph, retrieval, notes,
proactive engine, skill promotion) are **built but not wired**. The architecture
exists in isolated modules. The integration doesn't.

This is the single most important fact. Everything below is about connecting
what exists into a living system.

---

## Priority 1: Wire the Middleware into Worker Execution [DONE]

**Status:** Middleware chain is wired into worker execution. 8 implementations active.

**What to do:**
- Add `middleware_chain: MiddlewareChain` to AgentWorker
- Supervisor builds chain per agent role during task assignment
- execute() calls on_start → [before_model → model → after_model → before_tool → tool → after_tool]* → on_complete
- Cost, guardrails, loop detection, context budget all activate

**Lines of code:** ~150 to integrate. The middleware already works. Just connect it.

**From Deer Flow:** Their 13-middleware chain is conditional per agent type.
Sigil should do the same — engineer gets all middleware, researcher gets a
lighter chain. Build chain in supervisor based on agent role config.

---

## Priority 2: Replace Polling with Broadcast Streaming [DONE]

**Status:** Implemented. Worker events broadcast via tokio channels, WebSocket forwards to frontend.

**What to do:**
- Replace `tokio::sync::watch` with `tokio::sync::broadcast` in executor
- Add `ExecutionEvent` enum: Progress, ToolResult, Checkpoint, Outcome
- Worker publishes events during execution (after each tool call)
- SubscriptionManager in daemon tracks WebSocket subscribers per worker
- WebSocket handler subscribes to worker channels, forwards events
- Frontend replaces chat_poll with WebSocket event listener

**From Deer Flow:** Their `get_stream_writer()` + custom events
(task_started, task_running, task_completed) is the exact pattern.
Each tool result becomes an event. Each checkpoint becomes an event.

**From Hermes:** Their `stream_consumer.py` bridges sync worker output
to async platform delivery with rate-limiting (350ms min interval) and
progressive message editing (edit in place, don't re-send). Apply this
to Telegram delivery — edit the same message as worker progresses.

---

## Priority 3: Context Compression [DONE]

**Status:** ContextCompressionMiddleware implemented. Triggers at 50% window, protects first/last messages.

**What to do (from Hermes):**
- Add ContextCompressor to sigil-orchestrator
- Trigger at 50% of context window
- Protect: first 3 messages (system + first exchange) + last ~20K tokens
- Summarize middle with cheap/fast model (structured template: Goal, Progress, Decisions, Files, Next Steps)
- Iterative: subsequent compressions update previous summary
- Pre-pass: strip old tool outputs before LLM summarization
- Preflight check: estimate tokens before model call, compress proactively

**From Hermes:** Their context probing on error is clever — when API returns
context length error, parse the actual limit, step down to next tier
(200K → 128K → 64K → 32K), and retry with compression. Sigil should do this
in the middleware chain (catch context errors in on_error, compress, retry).

---

## Priority 4: Approval System [DONE]

**Status:** GuardrailsMiddleware implemented with configurable deny patterns. ClarificationMiddleware handles structured approval/question flow.

**What to do (from Hermes):**
- Extend GuardrailsMiddleware: when it detects a dangerous pattern, instead of
  Halt, return a new `MiddlewareAction::RequestApproval(pattern, description)`
- ApprovalState tracks per-session and permanent allowlists
- Daemon IPC commands: `approve(scope: once|session|always)`, `deny`
- WebSocket pushes approval request to frontend
- Frontend shows: [Allow Once] [Allow Session] [Allow Always] [Deny]
- Worker pauses (NEEDS_CONTEXT) until approval received via message queue middleware

**From Hermes:** Their smart mode uses an auxiliary LLM to pre-assess risk.
If the LLM says "safe", auto-approve. If "dangerous", always ask. If
"uncertain", ask user. This reduces approval fatigue.

---

## Priority 5: Memory During Execution (not just before) [DONE]

**Status:** MemoryRefreshMiddleware implemented. Re-searches memory every N tool calls based on recent activity.

**What to do:**
- Add MemoryRefreshMiddleware: every 5 tool calls, re-search memory based
  on recent tool activity
- Use the query_planner (already built) to generate typed queries from
  recent tool context
- Inject fresh memory as context messages

**From Deer Flow:** Their memory updater extracts facts with confidence
scores (≥0.7 threshold) and limits to top-100 by score. Sigil's reflection
should adopt the same confidence filtering.

**From Supermemory:** Their 3-level model (Document → Chunk → Memory Entry)
is worth adopting. Currently Sigil stores flat entries. Adding a "source"
layer (which task/conversation produced this memory) would enable provenance
tracking and confidence inheritance.

---

## Priority 6: Memory as Product (Profile API + MCP) [PENDING]

**Status:** Deep internal memory. No external surface. Profile API, MCP server, and memory graph visualization not yet built.

**What to do (from Supermemory):**
- Profile API: GET /api/profile?project=X returns { static: [...], dynamic: [...] }
  - Static = facts that don't change (project goals, tech stack, team)
  - Dynamic = recent context (current tasks, recent decisions, active work)
- MCP server: expose `memory` (save/forget), `recall` (search), `context` (profile injection) as MCP tools
- Memory graph visualization: D3 force-directed layout in the dashboard
  - Pre-compute positions on backend (force simulation)
  - Frontend renders with viewport-based queries
  - Show relationship edges (CausedBy, Contradicts, Supports, etc.)

**From Supermemory:** Their `containerTag` pattern is what Sigil already
does with project-scoped memory. Just expose it properly as an API.
Their `isStatic` boolean is simple but powerful — classify memories at
storage time and surface them differently in retrieval.

---

## Priority 7: Clarification Interruption [DONE]

**Status:** ClarificationMiddleware implemented. Workers can ask structured questions via ask_clarification tool. Halts execution, pushes question to user, resumes on response.

**What to do (from Deer Flow):**
- Add `ask_clarification` as a tool available to workers
- When worker calls it, ClarificationMiddleware intercepts:
  - Halts execution cleanly (preserves state)
  - Pushes structured question to user via chat/Telegram/WebSocket
  - Worker waits (NEEDS_CONTEXT status)
  - When user responds, supervisor injects answer and resumes worker
- Question format: type (missing_info, choice, confirmation), context, options

**This replaces the current BLOCKED → escalate flow with a clean
agent-to-human handoff that preserves execution context.**

---

## What NOT to Copy

**From Hermes:**
- Single-agent worldview (Sigil is multi-agent by design)
- Session-first thinking (Sigil is task/project/org-first)
- Monolithic runtime center of gravity

**From Supermemory:**
- Client-side-only deduplication (Sigil should dedup at write time)
- Memory-as-intelligence reductionism (memory serves orchestration, not the other way)
- Opaque backend classification (Sigil should be explicit about how decisions are made)

**From Deer Flow:**
- Thread/run as the dominant abstraction (Sigil uses task/project/org)
- Silent truncation of subagent calls (should warn, not silently drop)
- Middleware composition replacing orchestration control logic

---

## The Integration Order

```
Week 1:  Wire middleware into worker execution (Priority 1)
         Connect what's built. Everything activates.

Week 2:  Replace polling with broadcast streaming (Priority 2)
         Sigil becomes alive. Real-time progress in the UI.

Week 3:  Context compression + error recovery (Priority 3)
         Workers handle long tasks without crashing.

Week 4:  Approval system (Priority 4)
         Workers can be trusted to run autonomously.

Week 5:  Memory refresh middleware + profile API (Priorities 5-6)
         Workers get smarter during execution. Memory becomes visible.

Week 6:  Clarification interruption (Priority 7)
         Clean agent-to-human handoff. The loop closes.
```

After these six weeks, Sigil will be:
- Deeper than Hermes (multi-agent orchestration, verification, notes)
- More visible than Supermemory (profile API, graph viz, MCP tools)
- More alive than Deer Flow (real-time streaming grounded in a real control plane)
- The only system that is all three at once

---

## The Test (Updated)

Sigil v4 is done when:

1. You write "launch the new pricing page" in your notes
2. Sigil decomposes it into tasks, routes to the right agents
3. You watch workers execute in real-time — tool calls streaming into the chat
4. A worker encounters a dangerous operation — you tap [Allow Session] on your phone
5. A worker needs clarification — it asks "Which pricing tier layout?" with options
6. You tap option B. The worker resumes instantly.
7. Verification confirms the work — tests pass, spec met
8. Memory extracts insights, deduplicates, links to related knowledge
9. You check the memory graph — see how knowledge connected across tasks
10. You wake up to a Telegram message: "Pricing page deployed. $2.14 spent."
11. Your note shows: ✓ launch the new pricing page
12. The profile API shows this project's tech stack updated automatically

That is the best agent orchestrator in the world.
That is Sigil.
