# Sigil

AI agent orchestration framework in Rust. 9 crates, 433 tests, 28 CLI commands.

## Crates

| Crate | Path | Purpose |
|-------|------|---------|
| `sigil` | `sigil-cli/` | CLI binary and command handlers |
| `sigil-core` | `crates/sigil-core/` | Config (SigilConfig, WebConfig, DepartmentConfig), traits, agent loop, identity, secrets |
| `sigil-orchestrator` | `crates/sigil-orchestrator/` | Daemon, Supervisor, AgentWorker, ChatEngine, ConversationStore, DispatchBus, Audit, Expertise, Blackboard, Watchdog, Preflight, Decomposition, FailureAnalysis, Lifecycle, Middleware Chain, Verification Pipeline, Escalation, Notes, Proactive Engine, Skill Promotion |
| `sigil-web` | `crates/sigil-web/` | Axum REST API + WebSocket server (JWT auth, IPC proxy to daemon) |
| `sigil-tasks` | `crates/sigil-tasks/` | Task DAG (JSONL), missions, dependency inference |
| `sigil-memory` | `crates/sigil-memory/` | SQLite+FTS5, vector search, hybrid ranking, memory graph (relationships, dedup, hotness), hierarchical L0/L1/L2, intelligent retrieval (query planning, multi-signal scoring), lifecycle management, debounced writes |
| `sigil-providers` | `crates/sigil-providers/` | OpenRouter, Anthropic, Ollama + cost estimation |
| `sigil-gates` | `crates/sigil-gates/` | Telegram, Discord, Slack channels |
| `sigil-tools` | `crates/sigil-tools/` | Shell, file, git, tasks, delegate, skills |

## Message Flow

```
User message (Web / Telegram)
    ↓
ChatEngine (orchestrator/src/chat_engine.rs)
    ├─ QUICK PATH: intent detection (create task, close task, note, status)
    │   → Immediate response from daemon data
    └─ FULL PATH: complex work
        → Creates task → Supervisor assigns to worker
        → Worker runs Claude Code → Outcome parsed
        → ChatEngine polls completion → Response delivered
```

### Key execution chain:
1. ChatEngine detects intent or creates task via `registry.assign()`
2. Supervisor patrol picks up task → expertise routing → preflight assessment → spawn worker
3. AgentWorker builds context (memory + blackboard + checkpoints + skill prompt)
4. Worker executes via Claude Code subprocess or internal agent loop
5. Outcome: DONE (close task), BLOCKED (escalate), HANDOFF (re-queue), FAILED (analyze + retry)
6. Reflection extracts insights → stored in memory SQLite
7. ChatEngine detects completion → delivers response to channel

## Quality Bar

```bash
cargo test --workspace    # 433 tests
cargo clippy --workspace --all-targets -- -D warnings
```

## Runtime

- `sigil daemon start` — long-running orchestration plane (systemd: sigil-daemon.service)
- `sigil web start` — Axum REST API on :8400 (systemd: sigil-web.service)
- `sigil run` — one-shot agent execution
- IPC via Unix socket at `~/.sigil/rm.sock` (JSON-line protocol)
- Claude Code execution: shells out to external `claude` binary

## IPC Commands

**Read:** ping, status, readiness, projects, tasks, missions, agents, audit, blackboard, expertise, cost, crons, watchdogs, brief
**Write:** create_task, close_task, post_blackboard
**Chat:** chat (quick), chat_full (agent execution), chat_poll (completion), chat_history, chat_channels

## Important Directories

- `config/sigil.toml` — master config (projects, agents, providers, orchestrator, watchdogs, web)
- `agents/{name}/` — agent identity (agent.toml, PERSONA.md, IDENTITY.md)
- `agents/rei/` — system leader (Rei, 零, The Living Sigil)
- `projects/{name}/` — project config (project.toml, skills, .tasks/)
- `projects/shared/` — shared skills, pipelines
- `~/.sigil/` — daemon state (audit.db, blackboard.db, expertise.db, dispatches.db, memory.db, cost_ledger.jsonl, rm.sock)

## Lock Architecture (CRITICAL)

IPC handlers use `try_lock()` on task boards — return partial data rather than blocking when patrol holds locks. Never use `.lock().await` in IPC read paths.

- `list_project_summaries()`: snapshots project list first, releases RwLocks, then try_lock each task board
- `tasks`/`missions` IPC: `try_lock()` per project, skip if locked
- Write commands (`create_task`, `close_task`): use `.lock().await` (must wait for consistency)

## Config Structure

```toml
[sigil]           # System name, data_dir
[web]             # bind, cors_origins, auth_secret
[providers.*]     # OpenRouter, Anthropic, Ollama
[security]        # autonomy, budget limits
[memory]          # SQLite backend, embedding config
[team]            # System leader (rei), router model
[orchestrator]    # Expertise routing, preflight, decomposition, retry
[[watchdogs]]     # Event-driven alert rules
[[projects]]      # Each project: name, prefix, repo, team, departments, missions
```

## Extension Points

- New tool: implement `Tool` trait, wire into builder
- New provider: implement `Provider` trait
- New channel: implement `Channel` trait, register in daemon startup
- New IPC command: add match arm in `daemon.rs` handle_socket_connection
- New web route: add to `sigil-web/src/routes/mod.rs`
- New department: add `[[projects.departments]]` to sigil.toml
