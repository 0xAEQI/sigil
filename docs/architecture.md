# Architecture

AEQI is an agent runtime and orchestration engine in Rust. 10 crates, 70k lines, 613 tests.

## System Map

```
Operator
  ├─ CLI (aeqi)           commands, TUI chat, MCP server, daemon control
  ├─ Web UI (apps/ui)     React 19, sessions, dashboard, agent control plane
  └─ API (aeqi-web)       Axum REST + WebSocket, JWT auth
          │
          ▼
    Daemon (aeqi-orchestrator)
          │
          ├─ SessionManager       persistent agent sessions (perpetual + spawned)
          ├─ WorkerPool           task-based agent execution per company
          ├─ CompanyRegistry      companies, worker pools, cost ledger, metrics
          ├─ AgentRegistry        persistent agent identities (UUID, system prompt, org tree)
          ├─ DispatchBus          agent-to-agent messaging with ACK/retry
          ├─ Notes                inter-agent coordination surface with TTL
          ├─ SessionStore         SQLite chat/session transcript persistence
          ├─ TriggerStore         cron/event/webhook triggers
          ├─ CostLedger           per-agent, per-company cost tracking
          ├─ EventBroadcaster     real-time execution events
          └─ Middleware (9 layers) guardrails, graph, loop detection, compression,
                                   budget, cost, memory refresh, clarification, safety net
          │
          ▼
    Providers (aeqi-providers)    OpenRouter, Anthropic, Ollama
```

## Crates

| Crate | Purpose |
|-------|---------|
| `aeqi-core` | Agent loop, config, identity, traits, session types, chat stream |
| `aeqi-orchestrator` | Daemon, worker pools, middleware, sessions, dispatch, cost, notes |
| `aeqi-tools` | Shell, file I/O, grep, glob, git worktree, web, tasks |
| `aeqi-providers` | LLM providers with retry, fallback, cost estimation |
| `aeqi-memory` | SQLite + FTS5 + vector search, hybrid ranking, MMR, chunking |
| `aeqi-tasks` | JSONL task DAG with dependency inference |
| `aeqi-web` | Axum HTTP API + WebSocket streaming |
| `aeqi-gates` | Telegram, Discord, Slack channel integrations |
| `aeqi-graph` | Code intelligence — tree-sitter indexing, symbol graph, impact analysis |
| `aeqi-cli` | CLI binary, TUI, MCP server, daemon/web commands |

## Primitives

**Agent** — persistent identity with stable UUID, entity-scoped memory, department membership, system prompt from template. Stored in AgentRegistry (SQLite).

**Session** — continuous execution context. One permanent per agent (always alive), plus spawned sessions for triggers/skills/users. Agent loop runs with `SessionType::Perpetual` — waits for input after each response, compacts when full, never exits until closed.

**Company** — an operating unit with its own repo, task store, memory DB, worker pool, team, departments, and budget. Configured in `[[companies]]` in aeqi.toml.

**Department** — org hierarchy within a company. Agents belong to departments. Departments have managers and escalation chains.

**Task** — a unit of work assigned to an agent. JSONL-persisted, DAG dependencies, priority, status lifecycle (pending → in_progress → done/blocked/cancelled).

## Two Execution Paths

**Sessions (interactive)** — user/trigger opens a session on an agent. Messages flow in via `session_send`. Agent has full tool access, memory, streaming. Context persists across messages. Used by the web UI and API.

**Workers (task-based)** — the patrol loop assigns pending tasks to agents. WorkerPool spawns `AgentWorker` per task with pre-loaded context (identity, notes, org context, skill prompt, middleware chain). Worker runs to completion, returns structured outcome.

Both paths use the same `Agent::run()` loop, same tools, same providers.

## Agent Loop

```
System prompt (identity + primers)
  + User message
  + Initial memory recall (domain + entity scopes)
  │
  ▼
┌─ LLM call (with all tool specs) ─────────────────┐
│  Response: text and/or tool calls                 │
│  ├─ TextDelta → stream to UI                      │
│  ├─ ToolStart → execute tool                      │
│  │   └─ ToolComplete → add result to messages     │
│  ├─ Mid-loop memory recall (every N tools)        │
│  └─ EndTurn → wait for next input (perpetual)     │
│              or exit (async)                       │
│                                                   │
│  Compaction: snip → microcompact → LLM summary    │
│  when context exceeds threshold                   │
└───────────────────────────────────────────────────┘
```

## Tools Available to Agents

All tools are self-describing via `tool.spec()` — the LLM sees name, description, and JSON schema on every call. No primer needed to list them.

**Base tools:** shell, read_file, write_file, edit_file, list_dir, grep, glob, web_fetch, web_search, git_worktree, task_create, task_close, task_show, task_update, task_dep, task_ready

**Orchestration tools:** aeqi_recall (memory search), aeqi_remember (memory store), aeqi_notes (coordination), aeqi_delegate (agent/department delegation), aeqi_graph (code intelligence), task_detail, task_cancel, task_reprioritize, transcript_search, usage_stats

## Memory

3 scopes: **Domain** (company-wide), **Entity** (per-agent UUID), **System** (cross-company).

SQLite + FTS5 for keyword search. Vector embeddings for semantic search. Hybrid ranking: `keyword_weight * BM25 + vector_weight * cosine`. Temporal decay with 30-day half-life. MMR for diversity. Chunking at 400 tokens with 80 overlap.

## Config

Single file: `config/aeqi.toml`

```toml
[aeqi]
name = "my-instance"
shared_primer = """..."""       # injected into all agent sessions

[[companies]]
name = "mycompany"
prefix = "mc"
repo = "/path/to/repo"
model = "provider/model-name"
primer = """..."""              # company-specific context
team.leader = "engineer"
team.agents = ["engineer", "reviewer"]

[[companies.departments]]
name = "engineering"
lead = "engineer"
agents = ["engineer", "reviewer"]
```

Agent templates in `agents/{name}/agent.toml` with frontmatter (display_name, model, capabilities, triggers) and `[prompt].system` for the full system prompt.

## IPC

Unix socket at `~/.aeqi/rm.sock`. JSON-line protocol. 54 commands covering status, tasks, sessions, memory, notes, agents, departments, triggers, cost, audit, approvals, webhooks.

Streaming mode: `session_send` with `"stream": true` writes multiple JSON lines (TextDelta, ToolStart, ToolComplete, Complete) before closing.

## Web API

49 routes behind JWT auth. WebSocket at `/api/chat/stream` for real-time session streaming. Public webhook endpoint for trigger firing with HMAC-SHA256 verification.
