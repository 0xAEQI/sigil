# Quick Start

This guide gets Sigil running locally with the daemon, API, and web UI.

## Prerequisites

- Rust stable
- Node.js 22+
- At least one model provider key

## 1. Create Local Config

```bash
cp config/sigil.example.toml config/sigil.toml
```

`config/sigil.toml` is local-only and should stay uncommitted.

Configure a provider and enable the web server:

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

## 2. Build

```bash
cargo build
npm run ui:install
npm run ui:build
```

## 3. Start Sigil

In one shell:

```bash
cargo run --bin sigil -- daemon start
```

In a second shell:

```bash
export SIGIL_WEB_SECRET=change-me
cargo run --bin sigil -- web start
```

Open `http://127.0.0.1:8400`.

## 4. UI Development Mode

If you want Vite hot reload instead of the compiled UI:

```bash
npm run ui:dev
```

That serves the frontend on `http://127.0.0.1:5173` and proxies `/api/*` to `sigil-web` on `:8400`.
