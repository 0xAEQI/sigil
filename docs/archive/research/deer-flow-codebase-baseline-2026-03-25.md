# Deer Flow Codebase Baseline

Date: 2026-03-25
Local checkout: `/home/claudedev/deer-flow`
Revision analyzed: `ac97dc6d426c`

## Purpose

This document is a code-first baseline for Deer Flow so it can be compared to Sigil from actual implementation shape rather than repo branding.

The main questions are:

- what Deer Flow actually is in code
- where the important logic lives
- what it is genuinely strong at
- what it is not trying to be
- how to compare it fairly with Sigil
- what Sigil should learn from it
- what Sigil should not copy blindly

## Executive Summary

Deer Flow is much closer to Sigil's category than Supermemory is, but it is still not the same kind of system.

From the code, Deer Flow is best understood as:

- a publishable super-agent harness
- built around LangChain and LangGraph agent runtime composition
- with subagents, memory, sandboxes, skills, MCP, gateway APIs, channels, uploads, artifacts, and a live frontend

Its center of gravity is:

- thread-scoped agent execution
- middleware-driven runtime behavior
- live chat and task streaming
- tool and skill composition
- sandboxed work and artifact production

Sigil's center of gravity is different:

- project and org-level orchestration
- durable control-plane state
- task ownership and routing
- audit and intervention
- supervisor logic
- budgets and patrol loops

So Deer Flow is not "Sigil but better." It is a strong agent harness and product shell with a more mature live UX than Sigil, but with a shallower orchestration model.

## Core Architectural Shape

### 1. Deer Flow has a deliberate harness/app split

One of the best architectural signals in the repo is that the harness is intentionally separated from the app layer.

The clearest proof is `backend/tests/test_harness_boundary.py`, which explicitly prevents the harness package from importing `app.*`.

That means Deer Flow is trying to maintain:

- a reusable harness package in `backend/packages/harness/deerflow`
- a separate app layer in `backend/app`

This is a good pattern and more disciplined than many agent repos.

### 2. The agent runtime is built around LangChain/LangGraph `create_agent`

The lead agent is not a handwritten mega-loop like Hermes.

Instead, `backend/packages/harness/deerflow/agents/lead_agent/agent.py` builds a LangChain agent with:

- model resolution
- tool loading
- prompt templating
- a middleware chain
- a thread state schema

The important thing here is that Deer Flow's intelligence and behavior are heavily expressed through middleware composition rather than one monolithic runtime file.

### 3. The main persistence unit is the thread

`backend/packages/harness/deerflow/agents/thread_state.py` defines a thread-shaped state model:

- sandbox state
- thread-local paths
- title
- artifacts
- todos
- uploaded files
- viewed images

The embedded client in `backend/packages/harness/deerflow/client.py` is explicit that without a checkpointer, calls are stateless and `thread_id` is mainly for file isolation.

That is a big distinction from Sigil. Deer Flow is thread-centric. Sigil is trying to be task/project/org-centric.

### 4. The runtime is middleware-heavy

The lead agent middleware chain in `backend/packages/harness/deerflow/agents/lead_agent/agent.py` includes:

- summarization
- todo/plan mode
- token usage
- title generation
- memory queueing
- view-image support
- deferred tool filtering
- subagent limits
- loop detection
- clarification
- shared tool-error handling middleware

This is a strong pattern for incremental runtime behavior, but it also means the system is more like a layered super-agent runtime than a full orchestration kernel.

## Important Implementation Areas

### Lead Agent Runtime

Primary file:

- `backend/packages/harness/deerflow/agents/lead_agent/agent.py`

Key observations:

- Model resolution is centralized enough to be meaningful.
- Thinking mode, reasoning effort, plan mode, and subagent mode are configured through runtime config.
- Agent creation is unified and fairly legible.
- Custom agent configuration exists, but still within the same overall harness shape.

Relevant comparison for Sigil:

- Deer Flow is cleaner than Sigil at packaging one configurable agent runtime.
- Sigil is still aiming higher at orchestration and separation between control plane and worker fleet.

### Embedded Client

Primary file:

- `backend/packages/harness/deerflow/client.py`

Key observations:

- Deer Flow can be consumed directly as an embedded Python client without always requiring gateway or server processes.
- The client exposes chat and stream flows and reuses the same harness internals.
- It can manage uploads, threads, skill installs, and stateful runs.
- It makes the harness feel like a product capability, not just a server app.

Relevant comparison for Sigil:

- This is one of Deer Flow's strongest ideas.
- Sigil would benefit from a clearer embedded/runtime API for consuming its orchestration and worker capabilities.

### Subagents

Primary files:

- `backend/packages/harness/deerflow/tools/builtins/task_tool.py`
- `backend/packages/harness/deerflow/subagents/executor.py`
- `backend/packages/harness/deerflow/subagents/registry.py`
- `backend/packages/harness/deerflow/subagents/builtins/general_purpose.py`
- `backend/packages/harness/deerflow/subagents/builtins/bash_agent.py`

Key observations:

- Subagents are a first-class feature.
- They run through a dedicated executor with background-task tracking, streaming updates, and timeout handling.
- The parent gets streaming custom events like `task_started`, `task_running`, `task_completed`, and `task_failed`.
- There are built-in subagent types rather than a durable evolving org of specialists.
- Tools are filtered per subagent and recursion is prevented.

This is strong practical delegation, but it is still bounded delegation within one harness, not a persistent organizational worker system.

Relevant comparison for Sigil:

- Deer Flow is better than Sigil at live subtask UX and streaming delegated-task progress into the UI.
- Sigil still has the more relevant ambition for durable worker ownership, routing, and project-level coordination.

### Memory

Primary files:

- `backend/packages/harness/deerflow/agents/middlewares/memory_middleware.py`
- `backend/packages/harness/deerflow/agents/memory/updater.py`
- `backend/packages/harness/deerflow/agents/memory/prompt.py`

Key observations:

- Memory is file-backed JSON, with global and per-agent memory paths.
- Memory updates are queued asynchronously after agent execution.
- The update step is model-driven: a prompt asks the model to rewrite structured memory sections and facts.
- Deer Flow carefully filters out ephemeral upload bookkeeping from long-term memory.
- Memory is prompt-injected back into the system later.

This is a useful memory system, but it is still centered on personalized conversation memory and thread/agent continuity, not org-scale knowledge and control-plane learning.

Relevant comparison for Sigil:

- Deer Flow is ahead of Sigil in having a cleaner user-facing memory loop inside the harness.
- Sigil is stronger conceptually in org/project state primitives, but weaker in memory productization.

### Checkpointing And Persistence

Primary files:

- `backend/packages/harness/deerflow/agents/checkpointer/provider.py`
- `backend/packages/harness/deerflow/config/checkpointer_config.py`
- `backend/app/gateway/routers/threads.py`

Key observations:

- Deer Flow can use LangGraph checkpointers for persistence.
- It supports multiple backends, but falls back to `InMemorySaver` when not configured.
- Thread-local filesystem data is managed separately from LangGraph thread state.
- The gateway exposes deletion of Deer Flow-managed thread filesystem data, while LangGraph thread state remains elsewhere.

This is practical, but it is still thread persistence, not a durable orchestration substrate.

Relevant comparison for Sigil:

- Better than a purely stateless agent shell.
- Still narrower than Sigil's desired task/project/org persistence model.

### Models And Runtime Resolution

Primary file:

- `backend/packages/harness/deerflow/models/factory.py`

Key observations:

- Model construction is centralized.
- Support for thinking and reasoning-effort is explicit.
- Provider class resolution is reflection-based from config.
- Codex/OpenAI-compatible special handling exists.
- Tracing is attached centrally.

This is a solid runtime/model factory design.

Relevant comparison for Sigil:

- Deer Flow is ahead of Sigil in centralized model/runtime resolution.
- This is an area Sigil should learn from directly.

### Sandbox

Primary files:

- `backend/packages/harness/deerflow/sandbox/sandbox.py`
- `backend/packages/harness/deerflow/sandbox/local/local_sandbox.py`
- `backend/tests/test_sandbox_tools_security.py`

Key observations:

- Deer Flow has a real sandbox abstraction.
- The local implementation is fairly thin: shell execution, read/write/update/list.
- The hardening is not mainly in `LocalSandbox` itself but in surrounding tool/path validation and virtual-path handling.
- There is real test attention around path traversal, virtual path mapping, and skills/user-data separation.

Relevant comparison for Sigil:

- Deer Flow has a more explicit sandbox abstraction than Sigil.
- But the local execution layer is not obviously as mature or hardened as Hermes's broader execution-environment story.

### Guardrails

Primary files:

- `backend/packages/harness/deerflow/guardrails/provider.py`
- `backend/packages/harness/deerflow/guardrails/middleware.py`

Key observations:

- Guardrails are a pluggable pre-tool-call authorization layer.
- Denied tool calls return tool-error messages so the agent can adapt.
- Fail-open vs fail-closed behavior is configurable.
- The model is structured and clean.

This is a good policy hook, though not yet a full operator trust framework.

Relevant comparison for Sigil:

- Deer Flow is ahead of Sigil in explicit tool-call guardrail hooks.
- Sigil should adopt something similarly structured.

### Gateway And Channels

Primary files:

- `backend/app/gateway/app.py`
- `backend/app/channels/*`

Key observations:

- Deer Flow has a real FastAPI gateway with routes for models, memory, skills, artifacts, uploads, threads, agents, suggestions, channels, and MCP.
- IM channels are a first-class backend concern.
- The gateway is part of the product shell, not an afterthought.

Relevant comparison for Sigil:

- Deer Flow has a more concretely packaged live product shell than Sigil right now.
- Sigil's backend ideas are still more orchestrator-centric.

### Frontend

Primary files:

- `frontend/src/core/api/api-client.ts`
- `frontend/src/core/threads/hooks.ts`
- `frontend/src/core/*`

Key observations:

- The frontend is genuinely wired to live backend and LangGraph streams.
- It uses the LangGraph SDK directly for thread streaming.
- It consumes backend APIs for models, skills, memory, uploads, agents, artifacts, MCP, and thread cleanup.
- It handles live subtask custom events and updates UI state from them.

This is a materially more mature live UI integration than Sigil currently has.

## What Deer Flow Is Better At Than Sigil

Based on code, Deer Flow appears stronger than current Sigil in these areas:

- live frontend and streaming UX
- harness/app layering discipline
- embedded client usability
- middleware-based runtime composition
- thread-centric live agent product shell
- subtask streaming and delegated-task UX
- centralized model factory
- explicit guardrail hook design
- practical uploads/artifacts/thread filesystem integration

It also has a real test surface. I counted 59 backend test files, and the tests cover memory, checkpointer behavior, guardrails, sandbox path security, client behavior, tools, uploads, suggestions, skills, and more. I did not run the suite, but the repo is not light on validation work.

## What Deer Flow Is Not Better At Than Sigil

Deer Flow is not obviously stronger than Sigil at:

- project supervision
- task DAG orchestration
- org-level control plane design
- stable worker ownership across projects
- budgets and patrol loops
- audit-centric orchestration
- durable shared blackboard-style org state
- explicit operator intervention over a persistent organization

Those are still closer to Sigil's intended differentiator.

## The Most Important Difference

Deer Flow is mainly:

"a live super-agent harness and product shell"

Sigil is trying to be:

"a persistent AI organization and orchestration control plane"

That distinction matters because Deer Flow optimizes for:

- threads
- runs
- subagents
- tools
- live streaming UI

Sigil should optimize for:

- tasks
- projects
- org state
- routing
- supervision
- trustable operator control

## What Sigil Should Learn From Deer Flow

### 1. Harness/App Separation

Sigil should keep its orchestration kernel cleanly separated from product-shell and UI-serving layers.

Deer Flow's harness boundary test is a strong pattern worth copying.

### 2. Better Embedded Runtime Surface

Sigil would benefit from a first-class embedded client or SDK surface, not just daemon and CLI entrypoints.

### 3. Better Live Streaming UX

Deer Flow is ahead in:

- live thread streaming
- task status events
- delegated subtask visibility
- artifact-aware chat surfaces

Sigil should borrow those UX and protocol ideas.

### 4. Better Runtime Composition

Deer Flow's middleware stack is a good lesson in keeping runtime features modular:

- memory
- title generation
- loop detection
- tool error shaping
- clarification
- token tracking
- plan mode

Sigil should keep orchestration separate, but runtime concerns could be packaged more cleanly like this.

### 5. Better Model Factory

Sigil should centralize:

- model creation
- provider resolution
- thinking/reasoning capability negotiation
- tracing hooks

### 6. Better Thread/File/Artifact Product Ergonomics

Deer Flow makes uploads, thread-local data, and artifacts feel native to the product.

Sigil should likely improve its equivalent operator-facing artifact and workspace handling.

### 7. Better Test Discipline Around Edges

Deer Flow tests many risky seams directly:

- path traversal
- upload filtering in memory
- checkpointer fallback behavior
- harness/app boundary
- guardrail behavior

Sigil should do more of that.

## What Sigil Should Not Copy Blindly

### 1. Thread-Centric Mental Model

Sigil should not let thread/run state become the main organizing abstraction.

Its stronger path is still task/project/org state.

### 2. Middleware As A Substitute For Orchestration

Deer Flow's middleware design is good for runtime behavior, but it does not replace a real orchestration control plane.

Sigil should not flatten supervision, routing, retries, and audit into one agent middleware stack.

### 3. Built-In Subagent Registry As "Organization"

Deer Flow's subagents are useful, but they are still built-in role types inside one harness.

Sigil should preserve the distinction between:

- orchestrator
- workers
- specialist identity
- durable assignment history

### 4. Optional Persistence As Default

Deer Flow's fallback to in-memory persistence is practical for developer ergonomics, but not enough for a trustworthy orchestration product.

Sigil should be stricter about durability expectations.

### 5. Thin Local Sandbox As A Finished Safety Story

Deer Flow has some good path-safety work, but the local execution layer is still relatively thin.

Sigil should aim higher on execution policy and environment hardening.

## Fair Comparison To Sigil

The most useful comparison boundary is:

### Deer Flow

- super-agent harness
- thread-based runtime
- subagents and tools
- live streaming UI
- embedded client
- memory and artifacts inside a thread/run product shell

### Sigil

- orchestration substrate
- task/project/org state
- supervision
- routing
- audit and intervention
- durable control plane

If we compare Deer Flow's highly polished live harness against Sigil's full orchestration ambition, the comparison will be misleading.

If we compare Deer Flow's runtime shell and UX against Sigil's worker/runtime/product surfaces, the comparison becomes very useful.

## Current Comparison Judgment

If the question is "which one has the better live super-agent product shell?" Deer Flow is ahead.

If the question is "which one has the better orchestration substrate for a persistent AI organization?" Sigil is still the more relevant architecture.

If the question is "which one would users experience as more complete today in chat/task UI terms?" Deer Flow is likely ahead.

If the question is "which one has the stronger long-term control-plane idea?" Sigil still does.

## Bottom Line

Deer Flow is a serious and useful reference repo for Sigil.

It shows how to build:

- a clean harness/app split
- a live agent product shell
- a strong embedded client
- good streaming task UX
- modular runtime behavior
- real frontend integration

But it does not replace Sigil's reason to exist.

The right move is:

- keep Sigil's orchestration core
- borrow Deer Flow's runtime-shell and UX discipline
- borrow Deer Flow's harness layering and testing habits
- do not collapse Sigil into a thread-centric super-agent runtime

That is the productive lesson.
