# Sigil

[![CI](https://github.com/0xAEQI/sigil/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/0xAEQI/sigil/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Backend: Rust](https://img.shields.io/badge/backend-Rust%202024-black)](Cargo.toml)
[![UI: React 19 + Vite 6](https://img.shields.io/badge/ui-React%2019%20%2B%20Vite%206-61dafb)](apps/ui)

Sigil is an agent runtime, multi-agent harness, orchestration engine, and web control plane for running persistent software work.

It is built for the full operating loop: run agents, coordinate specialist roles, persist state, verify work, and manage the system through a CLI, API, and web UI.

## What Sigil Includes

- A native agent runtime for direct task execution
- A multi-agent control plane for routing, supervision, retries, and verification
- Persistent state across tasks, memory, audit logs, dispatches, and project context
- A web UI and API for operating the system day to day

## Quick Start

Prerequisites:

- Rust stable
- Node.js 22+
- An LLM provider key such as `OPENROUTER_API_KEY` or `ANTHROPIC_API_KEY`

1. Create a local config:

```bash
cp config/sigil.example.toml config/sigil.toml
```

2. In `config/sigil.toml`, configure a provider and enable the web UI:

```toml
[providers.openrouter]
api_key = "${OPENROUTER_API_KEY}"
default_model = "xiaomi/mimo-v2-pro"

[web]
enabled = true
bind = "127.0.0.1:8400"
ui_dist_dir = "../apps/ui/dist"
auth_secret = "${SIGIL_WEB_SECRET}"
```

3. Build the backend and UI:

```bash
cargo build
npm run ui:install
npm run ui:build
```

4. Start the daemon and web server:

```bash
export SIGIL_WEB_SECRET=change-me
cargo run --bin sigil -- daemon start
cargo run --bin sigil -- web start
```

Then open `http://127.0.0.1:8400`.

## Local Development

Run the backend in one shell:

```bash
cargo run --bin sigil -- daemon start
```

Run the API server in another:

```bash
cargo run --bin sigil -- web start
```

Run the UI dev server in a third:

```bash
npm run ui:dev
```

The Vite app runs on `http://127.0.0.1:5173` and proxies `/api/*` to `sigil-web` on `:8400`.

## Production Model

The recommended deployment is:

- `sigil` daemon for orchestration
- `sigil-web` for the API and the compiled SPA
- `nginx` or `caddy` only as a thin TLS reverse proxy in front

That keeps the open-source product simple to self-host while still using a standard production edge.

## Repository Layout

```text
sigil/
  apps/ui/        # React + Vite web control plane
  crates/         # Workspace crates
  sigil-cli/      # sigil binary
  config/         # example config and local config path
  agents/         # agent definitions on disk
  projects/       # project definitions on disk
  docs/           # active docs and archives
```

## Core Components

| Component | Purpose |
|-----------|---------|
| `sigil-cli` | CLI entrypoint, daemon process, and operational commands |
| `sigil-orchestrator` | Routing, supervision, retries, middleware, verification, and chat execution |
| `sigil-web` | HTTP API, WebSocket transport, auth, and optional SPA serving |
| `sigil-memory` | SQLite-backed memory, retrieval, and knowledge persistence |
| `sigil-tasks` | Task DAGs, missions, and append-only task storage |
| `sigil-providers` | LLM provider integrations |
| `apps/ui` | Operator-facing web control plane |

## Docs

- [Getting started](docs/quickstart.md)
- [Deployment model](docs/deployment.md)
- [Architecture overview](docs/architecture.md)
- [Project setup](docs/project-setup.md)
- [Docs index](docs/README.md)
- [Contributing](CONTRIBUTING.md)

Historical research, synthesis notes, and design sketches live under [`docs/archive/`](docs/archive/README.md).

## License

MIT
