# AEQI Web Dashboard

Frontend for the AEQI agent runtime and control plane. Vite + React 19 + Zustand + TypeScript.

## Stack

- **Build:** Vite 6, React 19, TypeScript 5
- **State:** Zustand (auth store, daemon store, chat store, ui store)
- **Routing:** React Router v7
- **Styling:** CSS custom properties in `src/styles/tokens.css` (dark zinc palette, JetBrains Mono + Inter)
- **API:** `src/lib/api.ts` — fetch wrapper with JWT auth, auto-redirect on 401

## Layout

Three-layer navigation: CompanyRail (left icon bar) + AgentNav (left sidebar body) + floating nav bar (search via Cmd+K, page labels, notifications). Content renders in `<Outlet />` inside the content panel.

## Pages

| Page | Path | What it does |
|------|------|-------------|
| Dashboard | `/` | Stats, live activity feed, company/agent overview |
| Sessions | `/sessions` | Split pane: session list (perpetual + task-based) + transcript. WebSocket chat with agents |
| Tasks | `/issues` | Create/close tasks, filter by status, priority bars |
| Triggers | `/triggers` | Automation triggers (cron-based) |
| Skills | `/skills` | Agent skill registry |
| Memories | `/memories` | Memory entries (knowledge, recall) |
| Notes | `/notes` | Redirects to memories |
| Finance | `/finance` | Budget visualization, cost breakdown |
| Departments | `/departments/:id` | Department detail view |
| Login | `/login` | JWT authentication |

Legacy paths (`/blackboard`, `/agents`, `/settings`, `/cost`, `/chat`, `/audit`, `/knowledge`, `/tasks`) redirect to their current equivalents.

## Session Architecture

Sessions are the core interaction model. The Sessions page shows a split pane with a session list on the left and a message transcript on the right.

- **Perpetual session:** Always-on agent conversation (one per agent scope)
- **Task sessions:** Created from in-progress or completed tasks
- **Transport:** WebSocket at `/api/chat/stream?token=<jwt>` — sends `session_send` IPC to the daemon's SessionManager
- **Messages:** Streamed via WebSocket with `delta`, `tool_use`, `activity`, and `done` events. Tool activity and progress events render as an inline activity feed

The old `/api/chat`, `/api/chat/full`, and `/api/chat/poll` endpoints are deprecated.

## State Stores

| Store | File | Purpose |
|-------|------|---------|
| auth | `src/store/auth.ts` | JWT token, login/logout |
| daemon | `src/store/daemon.ts` | Daemon connection state |
| chat | `src/store/chat.ts` | Active channel/company scope, selected agent, per-channel thread state (persisted to localStorage) |
| ui | `src/store/ui.ts` | UI preferences |

## Deployment

```bash
cd apps/ui
npm run build
```

- Build outputs to `apps/ui/dist`
- Set `[web].ui_dist_dir` in `aeqi.toml`
- Run `aeqi web start`

## Dev

```bash
npm run dev  # Vite dev server on :5173, proxies /api to :8400
```
