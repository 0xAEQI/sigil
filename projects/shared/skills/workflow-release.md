```toml
[skill]
name = "workflow-release"
description = "Use when shipping a release, deploying to production, or cutting a release branch. Triggers: deploy, ship, release, merge to main, go live."
phase = "workflow"
```

# Release Workflow

Preflight → Build → Deploy → Verify → Announce. Every step has a gate. No shortcuts.

```
Preflight → Build → Deploy → Verify → Announce
```

---

## Phase 1: Preflight

1. **Tests pass** — full suite, not just the module you changed. `cargo test --workspace` / `npm test`
2. **Lints clean** — `cargo clippy -- -D warnings` / `eslint .`
3. **No uncommitted changes** — `git status` is clean
4. **Branch is up to date** — rebased on latest main
5. **Changelog updated** — what changed and why (user-facing, not commit-level)
6. **Dependencies reviewed** — any new deps? Known vulnerabilities?
7. **Config validated** — environment variables, feature flags, secrets present

<HARD-GATE>
Every preflight check must pass. Skipping one means deploying without knowing if it works.
</HARD-GATE>

---

## Phase 2: Build

1. **Clean build** — from scratch, not incremental. `cargo build --release`
2. **Artifacts produced** — binaries, containers, packages. Verify they exist and are the right size.
3. **Version stamped** — binary reports correct version when asked

---

## Phase 3: Deploy

1. **Deploy incrementally** — canary first (1 instance), watch metrics, then roll forward
2. **Monitor during rollout** — error rate, latency, CPU/memory
3. **Rollback ready** — know the exact command to rollback before you start
4. **No deploys on Friday** (unless emergency)

---

## Phase 4: Verify

1. **Health check** — service responds to health endpoint
2. **Smoke test** — core user journey works end-to-end
3. **Metrics comparison** — error rate, latency, throughput within 10% of pre-deploy baseline
4. **Watch period** — 15 minutes minimum after full rollout

<HARD-GATE>
Claiming "deployed" without verifying is the same as not deploying. Verify.
</HARD-GATE>

---

## Phase 5: Announce

1. **Changelog** — post to team channel
2. **Tag** — `git tag vX.Y.Z` and push
3. **Close task** — `sigil_close_task`
4. **Store** — `sigil_remember` any deployment learnings

---

## Anti-Rationalization Table

| Excuse | Reality |
|--------|---------|
| "Tests pass locally" | CI is the source of truth. Run full suite. |
| "It's a small change" | Small changes cause big outages. Preflight anyway. |
| "I'll fix it in the next deploy" | The next deploy inherits this bug. Fix now. |
| "Canary is overkill" | Canary is the cheapest insurance. Use it. |
