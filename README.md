# Sigil

Recursive multi-agent orchestration framework in Rust. A single 7MB binary (`sigil`) that coordinates autonomous AI agents across isolated projects — each with its own repository, task DAG, memory, identity, and worker pool.

**Workers are orchestrators.** Unlike flat agent frameworks, Sigil workers run as full Claude Code instances with unrestricted tool access, including the Task tool for spawning sub-agents. This creates a recursive execution tree where any worker can become a coordinator.

```
User (Human)
    |
    +-- Team of Agents (advisors with domain expertise)
    |         |
    |      Lead Agent (orchestrator)
    |         |
    |   +-----+------+----------+
    |   |            |           |
    | Supervisor  Supervisor  Supervisor
    | (proj-a)   (proj-b)    (proj-c)
    |   |            |           |
    | Workers     Workers     Workers     <- full Claude Code instances
    |   |                                 <- can spawn sub-agents via Task tool
    |   +-- Sub-agent swarm
    |
    +-- Dispatch Bus (inter-agent messaging)
    +-- Cost Ledger (per-project budget enforcement)
    +-- Metrics (Prometheus-compatible)
    +-- Schedule (cron jobs)
```

## Quick Start

```bash
# Build
cargo build --release    # ~7MB with LTO + strip

# Initialize
sigil init

# Set API key
sigil secrets set OPENROUTER_API_KEY sk-or-...

# Run a one-shot agent
sigil run "list files in current directory"

# Assign work to a project
sigil assign "fix the login bug" --project my-project --priority high

# Start the daemon
sigil daemon start

# Check status
sigil status
```

## Architecture

### Execution Hierarchy

| Layer | Role | Model |
|-------|------|-------|
| 0 | Human operator | — |
| 1 | Council (multi-agent advisory) | Gemini Flash (router) |
| 2 | Lead Agent (orchestrator) | Claude Opus |
| 3 | Supervisors (per-project) | Control plane |
| 4 | Workers (Claude Code executors) | Claude Sonnet |
| 5 | Sub-agents (spawned by workers) | Inherited |

### Crate Map

| Crate | Path | Purpose |
|-------|------|---------|
| `sigil` | `sigil-cli/` | CLI binary — 22+ commands |
| `sigil-core` | `crates/sigil-core/` | Traits, config, agent loop, security, identity |
| `sigil-tasks` | `crates/sigil-tasks/` | Git-native task DAG (JSONL, hierarchical IDs) |
| `sigil-orchestrator` | `crates/sigil-orchestrator/` | Router, Supervisor, Worker, Daemon, Dispatch, Ledger, Metrics, Council, Templates, Lifecycle |
| `sigil-memory` | `crates/sigil-memory/` | SQLite + FTS5, vector search, hybrid ranking, chunking |
| `sigil-providers` | `crates/sigil-providers/` | OpenRouter, Anthropic, Ollama + cost estimation |
| `sigil-gates` | `crates/sigil-gates/` | Telegram, Discord, Slack channels |
| `sigil-tools` | `crates/sigil-tools/` | Shell, file, git, tasks, delegate, skills |

### Key Systems

**Cost Ledger** — Per-project + global daily budget enforcement with JSONL persistence. Supervisors check `can_afford_project()` before spawning workers.

**Worker Checkpoints** — External git state capture. On timeout or handoff, the supervisor captures `git status`, last commit, branch — not self-reported by the agent. Successor workers receive checkpoint context.

**Lifecycle Engine** — Autonomous processes: memory reflection, personality evolution, proactive project scanning, creative ideation. Gated by agent bond level, costs ~$0.03/day.

**Dispatch Bus** — Indexed inter-agent messaging with O(1) recipient lookup, TTL expiry, bounded queues, SQLite WAL persistence.

**Context Budget** — Per-layer character limits with checkpoint summarization. Prevents context window overflow.

**Prometheus Metrics** — Zero-dependency text exposition format. Per-project breakdowns: tasks completed, workers active, cost USD, patrol cycle time.

### Escalation Chain

```
Worker BLOCKED → Project resolver (1 attempt) → Project Leader → System Leader → Human
```

## Configuration

`config/sigil.toml` (see `config/sigil.example.toml` for a full example):

```toml
[sigil]
name = "my-project"
data_dir = "~/.sigil"

[providers.openrouter]
api_key = "${OPENROUTER_API_KEY}"
default_model = "minimax/minimax-m2.5"

[security]
autonomy = "supervised"
max_cost_per_day_usd = 10.0

[team]
leader = "my-agent"
router_model = "google/gemini-2.0-flash-001"

[[projects]]
name = "my-project"
prefix = "mp"
repo = "/path/to/repo"
model = "claude-sonnet-4-6"
max_workers = 2
execution_mode = "claude_code"
```

Agents are auto-discovered from `agents/{name}/agent.toml` files on disk. See `agents/README.md`.

## CLI Commands

```
sigil init                              Initialize Sigil
sigil run "prompt" [--project NAME]     One-shot agent execution
sigil status                            System-wide status
sigil doctor [--fix]                    Diagnostics

sigil assign "task" --project NAME      Create a task
sigil ready [--project NAME]            Show unblocked tasks
sigil close ID [--reason "..."]         Close a task

sigil daemon start|stop|status          Daemon management
sigil council "question"                Multi-agent advisory

sigil mission create|list|status        Mission management
sigil team [--project NAME]             Show team assignments

sigil recall "query" [--project NAME]   Search memory
sigil secrets set|get|list|delete       Encrypted secret store
sigil cost [--project NAME]             Cost breakdown
```

## Design Principles

1. **Zero Framework Cognition** — All decisions delegated to the LLM. Rust code is a thin, safe, deterministic shell.

2. **Workers ARE Orchestrators** — Every worker runs as a full Claude Code instance with Task tool access. Any worker can spawn sub-agents, creating recursive execution trees.

3. **Observe, Don't Trust** — Checkpoints are captured externally via git inspection. Agents are unreliable self-reporters; git is the source of truth.

4. **Budget-Gated Execution** — No worker spawns without passing `can_afford_project()`. Per-project budgets + global daily caps prevent runaway costs.

5. **Trait-Driven Swappability** — Provider, Tool, Memory, Observer, Channel — all traits. Swap LLM providers, messaging channels, or memory backends without touching core.

6. **Bootstrap Files Not Config** — PERSONA.md, IDENTITY.md, AGENTS.md — human-readable, git-versioned, agent-editable via reflection.

## Documentation

- [Architecture Deep Dive](docs/architecture.md)
- [Council System](docs/council.md)
- [Claude Code Integration](docs/claude-code-integration.md)
- [Sigil Reference](docs/SIGIL.md)

## Build

```bash
cargo build                     # Dev
cargo build --release           # Release (7MB, LTO + strip)
cargo test                      # 145 tests across 8 crates
cargo clippy                    # Lint (zero warnings)
```

## License

MIT
