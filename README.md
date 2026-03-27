# sigil-ui

Web dashboard for the Sigil AI agent orchestrator.

**Tech stack:** Vite 6 + React 19 + TypeScript 5 + Zustand + React Router v7

## Quick Start

```bash
npm install
npm run dev      # Dev server on http://localhost:5173
npm run build    # Production build to dist/
```

## API

The dev server proxies `/api/*` requests to the sigil-web backend on `localhost:8400`.
In production, configure your reverse proxy (e.g. Nginx) to forward `/api/*` to the backend.

## Backend

Sigil orchestrator and API server: <https://github.com/0xAEQI/sigil>
