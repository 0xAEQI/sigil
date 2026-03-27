# Sigil UI

React web control plane for Sigil.

Canonical source lives in the Sigil monorepo at `apps/ui`.

## Quick Start

From the repository root:

```bash
npm run ui:install
npm run ui:dev
```

Or from inside `apps/ui`:

```bash
npm install
npm run dev
npm run build
```

The dev server runs on `http://127.0.0.1:5173`.

## API and Serving

- In development, the Vite server proxies `/api/*` to `sigil-web` on `http://127.0.0.1:8400`.
- In production, `sigil-web` can serve the compiled `dist/` directory directly when `[web].ui_dist_dir` is configured.
- `nginx` or `caddy` should sit in front only for TLS and host routing.

## Stack

- React 19
- Vite 6
- TypeScript 5
- Zustand
- React Router 7

Main project: <https://github.com/0xAEQI/sigil>
