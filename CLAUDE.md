# Realm Development

## Build & Test

```bash
cargo build                    # Dev build
cargo build --release          # Release (7MB, LTO + strip)
cargo test                     # All 20 tests
cargo clippy                   # Lint
```

## Crate Map

| Crate | Path | Purpose |
|-------|------|---------|
| `rm` | `rm/src/main.rs` | CLI binary, all 20 commands |
| `realm-core` | `crates/realm-core/` | Traits, config, agent loop, security, identity |
| `realm-quests` | `crates/realm-quests/` | Git-native task DAG (JSONL, hierarchical IDs) |
| `realm-orchestrator` | `crates/realm-orchestrator/` | Shadow, Scout, Spirit, Summoner, Whisper, Rituals, Raids, Fate, Pulse |
| `realm-memory` | `crates/realm-memory/` | SQLite+FTS5, vector search, hybrid, chunking |
| `realm-providers` | `crates/realm-providers/` | OpenRouter, Anthropic, Ollama |
| `realm-gates` | `crates/realm-gates/` | Telegram, Discord, Slack |
| `realm-tools` | `crates/realm-tools/` | Shell, file, git, quests, delegate, magic |

## Key Patterns

- **Traits over concrete types**: Provider, Tool, Memory, Observer, Channel — all traits in `realm-core/src/traits/`
- **Zero Framework Cognition**: Agent loop is a thin shell. No hardcoded heuristics. LLM decides everything.
- **Config**: TOML at `config/realm.toml`, loaded via `RealmConfig::discover()` (walks up directory tree)
- **Identity files**: SOUL.md, IDENTITY.md, AGENTS.md, HEARTBEAT.md in `rigs/<name>/`
- **Quests**: Each domain has `.quests/` dir with `<prefix>.jsonl` files
- **Memory**: Per-domain SQLite at `rigs/<name>/.sigil/memory.db`

## Adding a New Tool

1. Create struct implementing `Tool` trait in `crates/realm-tools/src/`
2. Implement `execute()`, `spec()`, `name()`
3. Export from `crates/realm-tools/src/lib.rs`
4. Add to `build_domain_tools()` in `rm/src/main.rs`

## Adding a New Provider

1. Create struct implementing `Provider` trait in `crates/realm-providers/src/`
2. Implement `chat()`, `health_check()`, `name()`
3. Export from `crates/realm-providers/src/lib.rs`
4. Add config section + factory in `rm/src/main.rs`

## Adding a New Channel

1. Create struct implementing `Channel` trait in `crates/realm-gates/src/`
2. Implement `start()` (returns mpsc::Receiver), `send()`, `stop()`, `name()`
3. Export from `crates/realm-gates/src/lib.rs`
4. Wire into summoner channel loop

## Working in This Repo

- Use standard worktree workflow (same as all domains): `git worktree add ~/worktrees/feat/<name> -b feat/<name>`
- Merge to `dev` for testing, then `dev` → `master` for production
- Commit messages: `feat:`, `fix:`, `docs:`, `chore:`
- Run `cargo test` before committing
- Edition: Rust 2024
- Default model: MiniMax M2.5, fallback: DeepSeek v3.2

## Config Location

- Dev config: `config/realm.toml`
- Domain definitions: `rigs/<name>/`
- Skills: `rigs/<name>/skills/*.toml`
- Rituals: `rigs/<name>/molecules/*.toml`
- Data dir: `~/.sigil/` (PID file, socket, fate, raids, secrets)
