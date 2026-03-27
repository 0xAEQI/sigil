# Deployment

Sigil is best deployed as one product with a thin edge proxy.

## Recommended Topology

- `sigil` daemon handles orchestration, workers, background logic, and persistence
- `sigil-web` handles the HTTP API and can also serve the compiled UI
- `nginx` or `caddy` sits in front only for TLS termination, host routing, and standard reverse-proxy concerns

## Why This Model

- The open-source install stays simple: one backend application surface instead of split frontend hosting requirements
- The UI and API ship together, so version drift is reduced
- Standard edge tooling still handles TLS, compression, and host-based routing cleanly

## Local Development

Use separate processes:

- `sigil` daemon
- `sigil-web`
- `apps/ui` Vite server

That gives you fast frontend iteration without changing the production shape.

## Production Build

1. Build the UI with `npm run ui:build`
2. Point `[web].ui_dist_dir` at `../apps/ui/dist` or an absolute path
3. Run `sigil web start`
4. Put `nginx` or `caddy` in front of it

## systemd

The intended service model is:

- `sigil.service` for the daemon
- `sigil-web.service` for the API and UI surface

The reverse proxy should treat `sigil-web` as the single upstream for both `/api` and browser routes.
