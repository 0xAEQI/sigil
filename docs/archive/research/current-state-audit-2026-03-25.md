# Sigil Current-State Audit

Date: 2026-03-25
Commit baseline: `29efa3e` (`Unify execution pipeline and harden orchestration`)

## Executive Summary

Sigil is now a real agent orchestrator harness with a meaningful control plane, not just an idea pile.
The daemon, worker pool loop, task DAG, budgets, audit log, blackboard, memory, checkpoints, dispatch
bus, readiness checks, and monitor surface all exist and are working together.

It is materially better after the latest hardening pass:

- one adaptive execution pipeline instead of tiered pipeline branching
- stable agent ownership instead of learning against ephemeral worker slot names
- more robust orchestration parsing via structured JSON-first control paths

That said, Sigil is still stronger as a backend orchestration system than as a finished operator
product. The core architecture is good. The proof, polish, resumability, and operator trust layers
are still incomplete.

## What Is Good Right Now

### 1. The orchestration substrate is real

Sigil has real long-running system pieces:

- daemon registry and patrol loop
- supervised project execution
- task DAG and mission substrate
- audit log
- blackboard
- memory
- expertise ledger
- dispatch bus
- readiness inspection
- operator monitor

This means Sigil is not dependent on one foreground chat session to be useful.

### 2. The execution model is cleaner than before

The system now uses one adaptive execution pipeline for all work:

- Discover
- Plan
- Implement
- Verify
- Finalize

Depth is supposed to scale with task complexity, instead of the runtime splitting tasks into named
pipeline species like `simple`, `moderate`, and `complex`. This is the correct direction. It is
easier to reason about and better aligned with how real work actually varies.

### 3. Ownership/routing is more logical

A major mismatch has been fixed: stable agent identity is now separated from ephemeral worker-run
identity.

That matters because:

- task assignees now point to durable agent identities
- expertise learning is tied to stable agents instead of worker slot names
- checkpoints and audit attribution are less misleading

This makes Sigil more believable as an actual multi-agent organization rather than a rotating pool
of anonymous worker shells.

### 4. The control loop is less brittle

Preflight, failure analysis, and proactive proposal parsing now prefer strict structured output
instead of relying entirely on fragile line-oriented model formatting.

That is a real orchestration improvement, not cosmetic cleanup.

### 5. The daemon/operator basics are usable

The daemon is live-restartable, readiness can be queried, and `sigil monitor` gives an actual
operator-facing summary. This is meaningful progress toward a control plane.

At the time of this snapshot:

- local daemon was rebuilt and restarted successfully
- readiness was green
- test suite passed

## What Is Still Weak

### 1. Sigil is still under-proven at the full claim level

The biggest issue is not missing nouns. It is missing proof.

Sigil claims a lot:

- route to the right agent
- execute well
- learn from results
- act proactively
- operate an AI company

The codebase has many component tests, but it still lacks a strong end-to-end evaluation harness that
proves the full orchestration loop produces consistently good outcomes.

### 2. Worker resumability is still not strong enough

Checkpoints exist, but the restart contract is still weaker than the ambition. The system does not
yet feel like it has a truly first-class "prime and resume work cleanly" model that reconstructs:

- task state
- recent audit trail
- unread dispatches
- relevant memory
- blackboard state
- previous checkpoint evidence

This remains one of the biggest gaps between Sigil's ambition and its practical resilience.

### 3. The proactive/autonomous loop is not trustworthy enough yet

Sigil can perform proactive scans and create ideas/tasks, but this part of the system still appears
more heuristic than deeply grounded. It is present, but not yet strong enough to justify broad trust
without tighter gating and better evaluation.

### 4. Runtime abstraction is incomplete

Sigil conceptually supports multiple runtimes/providers, but some control-plane logic still assumes
more than it should about the current provider/runtime mix. The shape is there, but it is not yet a
fully mature runtime capability model.

### 5. The operator product layer is still behind the backend

Sigil currently has stronger backend/control-plane primitives than operator-facing product quality.

The CLI/daemon story is more real than the dashboard story.
The system has an operator surface, but not yet a polished, trustworthy cockpit.

## Documentation Assessment

The docs are mixed. Some reflect reality well. Some are stale or should be reframed.

### Docs that are mostly trustworthy

- `docs/architecture.md`
- `README.md` with some caution
- `agents/shared/WORKFLOW.md`
- `projects/shared/WORKFLOW.md`
- `docs/claude-code-integration.md`

These are reasonably aligned with the current system shape.

### Docs that are stale or misleading

#### `docs/SIGIL.md`

This file is stale as a live reference.

At the time of this audit it still reports outdated numbers such as:

- crate count
- command count
- test count

It should either be rewritten from scratch or demoted from "live reference" status.

#### `README.md`

The README is directionally good, but it still overstates the operator product a bit.

Examples:

- "One dashboard, every project, full control"
- "Web dashboard" framing

Those statements are more aspirational than fully demonstrated today. The backend/control plane is
more mature than the operator UI layer.

#### `docs/competitive-analysis.md`

This is useful as a strategy/roadmap document, not as a current-state truth source. It contains
historical gap statements that are no longer fully accurate after later work.

#### `docs/formulas.md`

This appears stale and uses old command spellings. It should be updated or removed.

## Current Product Truth

The most honest current description of Sigil is:

Sigil is a backend-heavy AI organization/orchestration system with real durable state, real project
supervision, and a credible multi-agent control plane. It is already useful as an orchestration
harness. It is not yet a fully proven, fully polished operator product.

That is a good place to be, but it is not the final place.

## Recommended Next Priorities

### Priority 1: End-to-end orchestration evaluation

Build real evals for:

- assign -> route -> execute -> verify -> retry/escalate
- blocked / failed / handoff recovery
- proactive suggestion quality
- routing quality over repeated domain work

This is the single most important next step if the goal is to make Sigil genuinely excellent rather
than merely interesting.

### Priority 2: Strong resumability contract

Create a proper restart/prime bundle for workers that unifies:

- task
- checkpoint
- recent audit
- unread dispatches
- relevant memory
- blackboard context

### Priority 3: Runtime capability model

Make runtime capabilities explicit rather than implicit. The system should know, per runtime:

- tools support
- subagent support
- streaming progress support
- approval model
- cost fidelity
- resumability

### Priority 4: Operator trust layer

Keep improving:

- setup
- doctor
- monitor
- readiness
- service management

These are product features, not side chores.

### Priority 5: Documentation cleanup

Do this explicitly:

1. Make `docs/architecture.md` the canonical source of truth for "what exists today".
2. Rewrite or remove `docs/SIGIL.md`.
3. Tone the README claims to match the current operator surface.
4. Mark strategy/history docs clearly as non-authoritative.
5. Update or remove stale docs like `docs/formulas.md`.

## Final Assessment

Sigil is now:

- logically cleaner than before
- architecturally promising
- real enough to matter
- not yet proven enough to fully trust at its highest claim level

If developed with discipline from here, it can become a genuinely strong agent orchestrator. The
main risk is not lack of ideas. The main risk is letting ambition outrun validation and operator
trust.
