# Sigil — Agent Orchestration Framework

Recursive multi-agent orchestration framework for LLM-powered autonomous agents.

## What Lives Here

- **Agents** (`agents/<name>/`): Personality, identity, preferences, memory — WHO does the work
- **Projects** (`projects/<name>/`): Knowledge, operating instructions, tasks, skills — WHAT gets done
- **Shared** (`agents/shared/`, `projects/shared/`): Workflow, code standards, skills, pipelines
- **Config** (`config/sigil.toml`): Agent definitions, project config, teams, budgets

## Build & Test

```bash
cargo build                    # Dev build
cargo build --release          # Release (7MB, LTO + strip)
cargo test                     # 145 tests across 8 crates
cargo clippy                   # Lint (zero warnings)
```

## Crate Map

| Crate | Path | Purpose |
|-------|------|---------|
| `sigil` | `sigil-cli/src/main.rs` | CLI binary, 22+ commands |
| `sigil-core` | `crates/sigil-core/` | Traits, config, agent loop, security, identity |
| `sigil-tasks` | `crates/sigil-tasks/` | Git-native task DAG (JSONL, hierarchical IDs) |
| `sigil-orchestrator` | `crates/sigil-orchestrator/` | Router, Supervisor, Worker, Daemon, Dispatch, Ledger, Metrics |
| `sigil-memory` | `crates/sigil-memory/` | SQLite+FTS5, vector search, hybrid, chunking |
| `sigil-providers` | `crates/sigil-providers/` | OpenRouter, Anthropic, Ollama + cost estimation |
| `sigil-gates` | `crates/sigil-gates/` | Telegram, Discord, Slack |
| `sigil-tools` | `crates/sigil-tools/` | Shell, file, git, tasks, delegate, skills |

## Key Patterns

- **Traits over concrete types**: Provider, Tool, Memory, Observer, Channel — all in `sigil-core/src/traits/`
- **Zero Framework Cognition**: Agent loop is a thin shell. LLM decides everything.
- **Workers ARE Orchestrators**: Claude Code mode gives workers Task tool access for sub-agent spawning.
- **Two-source identity**: `Identity::load(agent_dir, project_dir)` — agent personality + project context
- **Budget-Gated Execution**: `can_afford_project()` checked before every worker spawn
- **Config discovery**: `SigilConfig::discover()` walks up directory tree for `sigil.toml`

## Adding Things

- **New tool**: Implement `Tool` trait in `sigil-tools`, export from lib.rs, add to `build_project_tools()`
- **New provider**: Implement `Provider` trait in `sigil-providers`, export, add factory
- **New channel**: Implement `Channel` trait in `sigil-gates`, export, wire into daemon
- **New project**: Create `projects/<name>/` with AGENTS.md + KNOWLEDGE.md, add to `config/sigil.toml`

## Development Workflow

```bash
cargo test && cargo clippy
git commit -m "feat: description"
```

- Commit messages: `feat:`, `fix:`, `docs:`, `chore:`
- Run `cargo test` before committing
- Edition: Rust 2024

## Critical Rules

- Traits over concrete types — everything through Provider, Tool, Memory, Observer, Channel
- Zero Framework Cognition — agent loop is thin, LLM decides everything
- No hardcoded heuristics in the agent loop
- Shared templates in `projects/shared/` — never duplicate per-project what can be shared
