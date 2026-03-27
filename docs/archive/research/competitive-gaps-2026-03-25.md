# Competitive Gaps — Code-Level Analysis

Date: 2026-03-25
Sources: Local clones of hermes-agent, supermemory, deer-flow

## The Three Gaps to Close

### Gap 1: Live Streaming (beat Deer Flow)

**Current Sigil:** Polling-based. `chat_poll` every 5 seconds. No real-time feedback.

**Deer Flow pattern:** WebSocket streaming with 4 event channels:
- `messages-tuple`: incremental AI message deltas
- `values`: full state snapshots
- Custom events: `task_started`, `task_running`, `task_completed`
- Subagent message capture during `astream()` — not after

**What to build:**
1. WebSocket event stream from daemon → web (sigil-web already has WebSocket infra)
2. Event protocol: `WorkerEvent` enum with task lifecycle events
3. Worker emits events during execution (between tool calls)
4. Frontend subscribes via WebSocket, renders live progress
5. Replace chat_poll with push-based completion notification
6. Subtask cards showing live AI messages from workers

**Impact:** Transforms Sigil from "check back later" to "watch it work in real-time."

### Gap 2: Trust & Approval Rails (beat Hermes)

**Current Sigil:** Guardrails middleware blocks dangerous patterns, but no user approval flow.

**Hermes pattern:** 3-level approval system:
- `once`: execute immediately, no record
- `session`: approve for current session only
- `always`: permanent allowlist in config
- Smart mode: LLM assesses risk before prompting user
- 55 regex patterns across 8 categories
- Per-session approval state with thread-safe tracking

**What to build:**
1. `ApprovalPolicy` in sigil-orchestrator: dangerous patterns + approval state
2. `ApprovalState`: session-scoped + permanent allowlists (DashMap)
3. When guardrails middleware detects dangerous op → emit approval request event
4. UI shows approval dialog: [Allow Once] [Allow Session] [Allow Always] [Deny]
5. Daemon IPC commands: `approve`, `deny` with scope
6. Permanent approvals persisted to config

**Impact:** Sigil becomes trustworthy for autonomous operation. Users can confidently let it run overnight.

### Gap 3: Memory as Product (beat Supermemory)

**Current Sigil:** Deep memory internals (graph, dedup, hotness, hierarchy, retrieval scoring). But invisible externally.

**Supermemory pattern:**
- Profile API: `GET /v3/profile` returns `{static: [...], dynamic: [...]}`
- Conversation ingestion: `POST /v4/conversations` accepts full threads
- MCP tools: `memory` (save/forget), `recall`, `context` prompt
- Graph API: `getGraphBounds()` + `getGraphViewport()` for spatial visualization
- Container tags: first-class multi-tenancy scoping
- Memory versioning with parent-child chains

**What to build:**
1. Profile service: classify memories as STATIC vs DYNAMIC, expose via API
2. Conversation ingestion endpoint: accept thread → extract facts async
3. MCP server: expose memory/recall/context as MCP tools
4. Graph query API: return memory nodes + edges + positions for visualization
5. Memory graph UI component: D3 force-directed layout in the dashboard
6. Container tags: use project name as container, support cross-project queries

**Impact:** Sigil's memory goes from "internal strength" to "visible, consumable, inspectable product feature."

## Priority Order

1. **Live streaming** — highest UX impact, most visible improvement
2. **Trust rails** — required for autonomous operation (the core product promise)
3. **Memory product** — differentiator that compounds over time

## Also Noted (from Hermes)

Lower priority but valuable:
- Provider registry with declarative auth types (OAuth, API key, external process)
- Session store with schema versioning and reasoning preservation
- Deterministic session keys from message source
- Self-registering tool registry with composable toolsets
- Secret type with validation and redaction
