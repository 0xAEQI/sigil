# Sigil Architecture

## Overview

Sigil is a lightweight Rust multi-agent orchestration framework. A single binary (`sg`, ~3.5MB) orchestrates AI agents across isolated Business Units (Rigs), using OpenRouter as the LLM provider.

```
            Emperor (Human via Claude Code)
                    |
                sg commands
                    |
            Sigil Daemon (sg daemon)
                    |
                Familiar (global coordinator)
                    |
            +-------+-------+
            |       |       |
         Witness  Witness  Witness    (per-rig supervisors)
            |       |       |
         Workers  Workers  Workers    (tokio task executors)
```

## Crate Structure

| Crate | Purpose |
|-------|---------|
| `sg` | CLI binary |
| `sigil-core` | Traits, config, agent loop, identity, security |
| `sigil-providers` | LLM providers (OpenRouter, Anthropic, Ollama) |
| `sigil-tools` | Tool implementations (shell, file, skills) |
| `sigil-beads` | Hierarchical task DAG with JSONL storage |
| `sigil-orchestrator` | Familiar, Witness, Worker, Rig, Molecules, Cron, Heartbeat |
| `sigil-memory` | SQLite + FTS5 hybrid memory with temporal decay |
| `sigil-channels` | Messaging channels (Telegram, Discord, Slack) |

## Core Concepts

### Rigs (Business Units)
Each Rig is an isolated container with its own:
- Identity files (SOUL.md, IDENTITY.md, AGENTS.md)
- Bead store (task DAG with hierarchical IDs)
- Memory database (SQLite + FTS5)
- Worker pool, skills, molecules
- Worktree root for git isolation

### Hierarchy
- **Familiar**: Global coordinator ("Mayor"). Routes work, handles cross-rig coordination.
- **Witness**: Per-rig supervisor. Patrols workers, assigns ready beads, respawns failures.
- **Worker**: Ephemeral task executor (tokio task). Picks up work via hooks, runs agent loop.

### Beads (Tasks)
Git-native JSONL task tracking with:
- Hierarchical IDs: `as-001`, `as-001.1`, `as-001.2`
- Dependency DAG: `depends_on` / `blocks`
- Priority: Low, Normal, High, Critical
- Status: Pending, InProgress, Done, Blocked, Cancelled

### Molecules (Workflows)
TOML-defined workflow templates with steps and dependencies:
```toml
[[steps]]
id = "implement"
title = "Implement the solution"
needs = ["plan"]
```
`sg mol pour feature-dev --rig algostaking` creates linked beads for each step.

### Skills
TOML-defined reusable capabilities with tool allowlists:
```toml
[skill]
name = "health-checker"
triggers = ["health check", "system status"]
[tools]
allow = ["shell", "read_file"]
[prompt]
system = "You are a health checker..."
```

### Memory
Hybrid search engine: BM25 keyword (FTS5) + temporal decay.
- 30-day half-life on non-evergreen memories
- Per-rig isolation with separate SQLite databases

### Heartbeat + Cron
- **Heartbeat**: Periodic health checks driven by HEARTBEAT.md, run by each Witness
- **Cron**: Persistent scheduled jobs (cron expressions or one-shot timestamps)

## Design Principles

1. **Zero Framework Cognition**: All decisions delegated to LLM. Rust is a thin, safe shell.
2. **Discovery Over Tracking**: Agents discover state from observables, not a master scheduler.
3. **GUPP**: "If there is work on your hook, you MUST run it."
4. **Trait-Driven Swappability**: Every subsystem is a trait. Swap providers, tools, memory.
5. **Bootstrap Files**: SOUL.md, IDENTITY.md, AGENTS.md - human-readable, git-versioned.

## Security

- ChaCha20-Poly1305 encrypted secret store
- Workspace-scoped file access (tools can't escape rig boundaries)
- Configurable autonomy levels: readonly, supervised, full
- Daily cost caps
