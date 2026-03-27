# Architecture Overview

Sigil has three main layers:

1. Agent runtime
2. Orchestration and persistence
3. Operator interfaces

## System Map

```text
Operator
  |-> CLI (`sigil`)
  |-> Web UI (`apps/ui`)
  |-> API / WebSocket (`sigil-web`)
                |
                v
        Orchestrator (`sigil-orchestrator`)
                |
                +-> agent runtime and tools
                +-> task boards and missions
                +-> memory and retrieval
                +-> audit, dispatches, watchdogs, lifecycle
                |
                v
        Providers (`sigil-providers`)
```

## Runtime Layer

The runtime is responsible for direct agent execution: prompt assembly, tools, middleware, retries, and outcome parsing.

## Orchestration Layer

The orchestration layer routes work, supervises workers, tracks task state, persists memory, and decides when to retry, escalate, or learn from outcomes.

## Interface Layer

- `sigil` is the CLI and daemon entrypoint
- `sigil-web` exposes the API and browser-facing surface
- `apps/ui` is the operator control plane

## Repository Shape

```text
apps/ui/                 frontend
crates/sigil-core/       shared config and traits
crates/sigil-memory/     retrieval and persistence
crates/sigil-orchestrator/
crates/sigil-providers/
crates/sigil-tasks/
crates/sigil-tools/
crates/sigil-web/
sigil-cli/
```
