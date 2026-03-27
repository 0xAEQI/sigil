# Hermes Agent Codebase Baseline

Date: 2026-03-25
Hermes local checkout: `/home/claudedev/hermes-agent`
Revision analyzed: `e4033b2baf68`

## Purpose

This document is a code-level checkpoint for Hermes Agent so it can be compared
against Sigil from a grounded baseline instead of repo marketing or vague memory.

It focuses on:

- what Hermes actually does in code
- where major responsibilities live
- what design patterns are relevant to Sigil
- what Hermes is better at
- what Hermes is not trying to do
- how to compare Hermes and Sigil without collapsing the two systems into the same shape

This is based on the local Hermes codebase, not on the website docs as the primary source.

## Executive Summary

Hermes is not primarily a project/team orchestrator.
Hermes is primarily a persistent personal or messaging-first agent platform built around one large,
feature-dense agent runtime.

Its center of gravity is:

- durable user/session continuity
- broad tool access
- strong runtime/provider flexibility
- messaging gateway deployment
- cron-based unattended actions
- skill/memory accumulation around a single persistent agent identity

Sigil's center of gravity is different:

- project supervision
- task DAGs
- multi-project orchestration
- budgets
- audit
- blackboard
- durable organizational control-plane state

So Hermes should be treated as a source of implementation patterns and product lessons, not as a
system to copy wholesale.

## Core Architectural Shape

### 1. Hermes is built around a large single-agent runtime

The center of the system is:

- `run_agent.py`

`AIAgent` owns a very large amount of behavior:

- provider/runtime initialization
- tool loading
- session logging
- memory loading
- Honcho integration
- prompt caching
- context compression
- retries and fallback handling
- tool-call loop
- streaming behavior
- subagent delegation state
- checkpoints
- background memory/skill review

This is a powerful but monolithic architecture.

### 2. Hermes wraps that runtime with a persistent product shell

The agent runtime is surrounded by a product platform:

- `gateway/run.py`
- `gateway/session.py`
- `cron/scheduler.py`
- `hermes_state.py`
- `hermes_cli/runtime_provider.py`
- `toolsets.py`

This shell is where Hermes gets a lot of its practical power.

It is strong at:

- persistent messaging sessions
- routing outputs back to channels
- cron automation
- session reset policies
- runtime/provider management
- environment backends
- tool packaging

### 3. Hermes is not a daemon-supervisor-taskboard system

Hermes does have gateway and cron background execution, but it does not look like Sigil's:

- per-project supervisors
- task DAG orchestration
- agent team assignment
- expertise routing over task history
- audit-driven control plane

Hermes is better understood as:

"one persistent agent product with delegation and automations"

not:

"a multi-project AI organization kernel"

## Important Implementation Areas

### Agent Runtime

Primary file:

- `run_agent.py`

Key observations:

- The runtime is highly stateful.
- Session behavior and tool loop behavior are deeply intertwined.
- The provider/runtime path is more mature than Sigil's current abstraction layer.
- Retry and fallback handling are heavily built into the main runtime.
- Memory and skills are part of the same agent lifecycle rather than separate orchestration subsystems.

Relevant comparison for Sigil:

- Hermes is stronger at rich single-agent runtime engineering.
- Sigil is stronger conceptually at separating orchestration from worker execution.

### Session Persistence

Primary files:

- `hermes_state.py`
- `gateway/session.py`
- `tools/session_search_tool.py`

Key observations:

- Hermes has a real SQLite session store with FTS5.
- It stores session metadata, messages, costs, reasoning details, tool calls, titles, and parent session chains.
- Session search is not just raw retrieval; it uses search plus summarization to recall older sessions.
- Gateway session keys are thoughtfully constructed from platform/chat/thread/user dimensions.
- Session reset logic is explicit and policy-driven.

Relevant comparison for Sigil:

- Hermes is much stronger at conversation continuity and operator/user session persistence.
- Sigil has memory, audit, blackboard, and tasks, but not the same polished session substrate for a persistent personal agent.

### Delegation / Subagents

Primary file:

- `tools/delegate_tool.py`

Key observations:

- Delegation creates child `AIAgent` instances with isolated context.
- Children get fresh conversations, restricted toolsets, and their own iteration budgets.
- Delegation depth is capped.
- Parent only receives the summarized result, not the full child reasoning or intermediate tool noise.
- This is essentially bounded in-session delegation, not durable worker orchestration.

Relevant comparison for Sigil:

- Hermes’s delegation is compact and usable.
- Sigil's ambition is larger: durable worker ownership, task reassignment, retries, audit, patrol loops.
- Sigil should borrow the simplicity and boundedness of Hermes delegation, not its whole architectural frame.

### Memory and Skill Learning Loop

Primary files:

- `tools/memory_tool.py`
- `run_agent.py`
- `tools/skill_manager_tool.py`

Key observations:

- Hermes has file-backed curated memory in `MEMORY.md` and `USER.md`.
- That memory is bounded and snapshot-injected into the system prompt at session start.
- Mid-session memory writes update disk immediately but do not mutate the current system prompt, preserving cache stability.
- Hermes tracks turn and iteration counters for nudging memory/skill review.
- After enough turns or tool iterations, Hermes spawns a background review agent to decide whether to save memory or create/update skills.
- This is real and useful, but it is still "post-session or post-turn self-review" rather than a higher-level orchestrator learning system.

Relevant comparison for Sigil:

- Hermes is better at lightweight personal memory and skill self-curation.
- Sigil is stronger in organizational memory primitives like audit, blackboard, and task-linked project state.
- Sigil could benefit from borrowing Hermes’s "background reflection worker" pattern for narrow post-task learning actions.

### Tool Packaging

Primary files:

- `toolsets.py`
- `tools/__init__.py`

Key observations:

- Hermes has a mature toolset composition model.
- Tool exposure is scenario- and platform-aware.
- There is a clear difference between core tools, scenario toolsets, and platform toolsets.
- This gives Hermes a good capability-packaging layer.

Relevant comparison for Sigil:

- Sigil has skills and pipelines, but its capability packaging is less unified and less productized.
- Sigil should likely adopt a clearer capability/toolset contract closer to Hermes.

### Runtime / Provider Abstraction

Primary file:

- `hermes_cli/runtime_provider.py`

Key observations:

- Hermes has a stronger runtime/provider resolution layer than Sigil currently does.
- It explicitly resolves provider choice from config, env, explicit overrides, and custom providers.
- It handles multiple API modes.
- It treats runtime/provider choice as a shared concern across CLI, gateway, cron, and helpers.

Relevant comparison for Sigil:

- This is one of the strongest areas for Sigil to learn from directly.
- Sigil still has a split feeling between "internal agent loop" and "Claude Code worker path".
- Hermes shows how to centralize runtime resolution cleanly.

### Execution Environments

Primary files:

- `tools/environments/base.py`
- `tools/environments/local.py`
- `tools/environments/docker.py`
- `tools/environments/ssh.py`
- `tools/environments/modal.py`
- `tools/environments/daytona.py`
- `tools/environments/singularity.py`

Key observations:

- Hermes has a real execution backend abstraction.
- It supports local, Docker, SSH, Modal, Daytona, and Singularity.
- The Docker backend is relatively hardened.
- The local backend actively sanitizes subprocess env vars to avoid leaking Hermes-managed secrets.

Relevant comparison for Sigil:

- Hermes is significantly ahead in environment abstraction and execution backend engineering.
- Sigil needs a more explicit runtime capability/backend model if it wants to compete at this layer.

### Messaging Gateway and Cron

Primary files:

- `gateway/run.py`
- `gateway/delivery.py`
- `gateway/session.py`
- `cron/scheduler.py`
- `cron/jobs.py`

Key observations:

- Hermes integrates cron into the gateway process itself.
- Cron jobs are persisted, scheduled, executed through the same core agent runtime, and delivered back to configured channels.
- Gateway state is a major part of Hermes’s real-world usefulness.
- Messaging channels are a first-class product layer, not a thin add-on.

Relevant comparison for Sigil:

- Hermes is ahead in user-facing messaging/gateway product polish.
- Sigil has gateway-like pieces, but Hermes’s implementation is more unified as a persistent user product.
- Sigil is still stronger conceptually at organization-level control plane behavior.

### Checkpoints

Primary file:

- `tools/checkpoint_manager.py`

Key observations:

- Hermes uses shadow git repos for transparent filesystem checkpoints.
- Checkpoints happen automatically before file-mutating operations.
- This is local filesystem rollback infrastructure, not task- or supervisor-level resumption.

Relevant comparison for Sigil:

- Hermes has a clever local mutation safety pattern.
- Sigil’s checkpoint story is more tied to task/worker state, but still not as strong as it should be.
- These ideas are related but not identical.

### Safety / Approval / Secret Hygiene

Primary files:

- `tools/approval.py`
- `tools/environments/local.py`
- `tools/environments/docker.py`
- `tools/memory_tool.py`

Key observations:

- Hermes has a concrete dangerous-command approval system with per-session and permanent approval state.
- It persists allowlist entries.
- It strips Hermes-managed secrets from subprocess environments.
- It scans memory content for injection or exfiltration patterns before allowing those entries into persistent prompt-injected memory.

Relevant comparison for Sigil:

- This is one of Hermes’s clearest practical strengths.
- Sigil needs stronger execution policy and operator-facing trust rails.

### Programmatic Tool Calling

Primary file:

- `tools/code_execution_tool.py`

Key observations:

- Hermes lets the model write a Python script that can call a restricted set of Hermes tools via RPC.
- This reduces context churn by moving multi-step procedural work into one execution turn.
- The design is clever and pragmatic.

Relevant comparison for Sigil:

- This is a strong idea for Sigil to study.
- It could be useful anywhere Sigil currently burns too many turns on repetitive deterministic tool chains.

## What Hermes Is Better At Than Sigil

At the time of this checkpoint, Hermes appears better than Sigil at:

- persistent session UX
- messaging-first product design
- runtime/provider abstraction
- execution backend abstraction
- toolset packaging
- personal memory/product continuity
- command approval and secret hygiene
- practical polish for a user-facing persistent agent

These are serious strengths.

## What Hermes Is Not Better At Than Sigil

Hermes is not obviously better than Sigil at:

- project-level supervision
- explicit task DAG orchestration
- multi-project control plane design
- audit-centric orchestration architecture
- budget-aware org orchestration
- blackboard-style shared org state
- durable ownership of work across project teams

Those are areas where Sigil is aiming at a different and potentially more ambitious system.

## The Most Important Difference

Hermes is mostly:

"a powerful persistent agent product"

Sigil is trying to be:

"a persistent AI organization and project orchestration system"

That distinction matters because:

- Hermes optimizes for a single agent product shell with good persistence and lots of integrations.
- Sigil should optimize for control-plane coherence, durable project state, team routing, and trustworthy operator oversight.

If Sigil copies Hermes too literally, it risks collapsing its orchestration architecture into a feature-heavy monolith.

## What Sigil Should Learn From Hermes

### 1. Stronger runtime/provider abstraction

Hermes’s runtime resolution is cleaner and more centralized.
Sigil should improve:

- runtime registry
- provider/runtime capability metadata
- one clear resolver used everywhere

### 2. Better execution backend abstraction

Hermes has a real backend interface for local, Docker, SSH, and more.
Sigil should not remain implicitly split between only a few hardcoded execution modes.

### 3. Stronger operator trust and safety rails

Hermes does better at:

- dangerous command approvals
- secret isolation
- sandbox/backend hardening
- persistent operational defaults

Sigil needs a stronger policy model and operator trust layer.

### 4. Better capability packaging

Hermes’s toolsets are simple and legible.
Sigil should think more clearly about:

- capability sets
- runtime capabilities
- task/tool constraints
- operator-visible execution affordances

### 5. Better session continuity for human interaction surfaces

Sigil should study Hermes’s session store and session recall patterns for:

- chat surface continuity
- durable thread state
- search over prior interactions
- session resets and expirations

### 6. Narrow post-task reflection loops

Hermes’s background review agent is a useful pattern.
Sigil could apply this to:

- post-task memory extraction
- post-task skill updates
- targeted reflection after failures

without bloating the main orchestrator.

### 7. Programmatic tool execution for deterministic workflows

Hermes’s code-execution RPC model is a strong idea.
Sigil may benefit from a comparable mechanism for deterministic tool chains.

## What Sigil Should Not Copy Blindly

### 1. The monolith

Hermes gets a huge amount done in `run_agent.py`, but this concentration is also a cost.
Sigil should avoid merging orchestration, runtime, persistence, reflection, and execution control into one mega-loop.

### 2. Single-agent mental model

Hermes works because it is mostly about one persistent agent product.
Sigil should not lose its stronger separation between:

- orchestrator
- worker
- task substrate
- project state
- organization state

### 3. Session-centric over task-centric thinking

Hermes’s state model is session-first.
Sigil should remain task-/project-/operation-first where appropriate.

## How To Compare Hermes to Newer Sigil

When comparing the newer Sigil against Hermes, use these dimensions:

### 1. Runtime abstraction

Questions:

- How many execution backends are actually supported?
- Is capability metadata explicit?
- Is runtime resolution centralized or scattered?

### 2. Worker/orchestrator separation

Questions:

- Is the orchestrator cleanly separated from worker execution?
- Are retries, routing, checkpoints, and learning handled in the right layer?

### 3. Persistence model

Questions:

- What survives process/session restart?
- Is state session-based, task-based, project-based, or org-based?
- How complete is resumption?

### 4. Memory and learning model

Questions:

- What is stored?
- When is it stored?
- How is it recalled?
- Is learning explicit, background, or merely prompt advice?

### 5. Operator trust

Questions:

- What readiness/doctor/status surfaces exist?
- What policy and approval model exists?
- How are secrets and dangerous execution handled?

### 6. User/product surface

Questions:

- Is the system actually usable every day?
- Is it coherent from the operator’s point of view?
- Does the UI/CLI/gateway reflect the backend truth?

### 7. Autonomous and proactive behavior

Questions:

- Is autonomy grounded by real evidence and verification?
- Are autonomous actions conservative and inspectable?
- Is there strong evaluation for autonomous outcomes?

## Practical Comparison Hypothesis

Before looking at the newer Sigil, the likely comparison hypothesis is:

- Hermes will still be ahead on product polish, session continuity, execution backends, and safety hardening.
- Sigil can still be ahead on true orchestration architecture if its control plane remains coherent and its worker/task/org model continues improving.
- The best Sigil will not look like Hermes.
- The best Sigil will borrow Hermes’s strongest runtime/product/safety patterns while keeping a stronger orchestration core.

## Bottom Line

Hermes is a valuable reference implementation for:

- persistent agent product engineering
- runtime abstraction
- execution backend abstraction
- messaging/gateway continuity
- memory/skill reflection loops
- safety hardening

Hermes is not the target shape for Sigil.

The real opportunity is:

- keep Sigil’s stronger orchestration/control-plane architecture
- import Hermes’s stronger runtime, product, and safety patterns
- avoid inheriting Hermes’s monolithic single-agent architecture
