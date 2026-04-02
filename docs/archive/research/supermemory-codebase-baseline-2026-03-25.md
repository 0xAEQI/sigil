# Supermemory Codebase Baseline

Date: 2026-03-25
Local snapshot: `fe5d16509a9f`
Repo: `/home/claudedev/supermemory`

## Scope

This baseline is based on the local codebase, not repo branding or docs as the primary source.

The goal is to answer:

- what Supermemory actually is in code
- where the important logic lives
- what is in this repo versus what clearly lives behind hosted APIs
- how to compare it fairly against Sigil
- what Sigil should learn from it
- what Sigil should not copy blindly

## Executive Read

Supermemory is not a peer to Sigil in the same category.

From the code, Supermemory is primarily:

- a hosted memory platform
- a set of SDK and middleware adapters that inject that memory into agent runtimes
- an MCP surface and UI layer around that hosted memory product

It is not, from this repo, a deep local agent orchestrator with worker pools, task DAGs, project patrol loops, durable worker ownership, or org-level control logic.

That matters because the right comparison is not "which orchestrator is better?" The fair comparison is:

- Supermemory as memory substrate and agent-memory product
- Sigil as orchestration substrate and operator control plane

If Sigil wants to improve, the Supermemory lessons are mostly about packaging, integration ergonomics, and productized memory surfaces, not about replacing Sigil's orchestration architecture.

## What Supermemory Is In Code

The clearest pattern in the repo is that most important runtime behavior calls an external Supermemory API rather than implementing the memory backend locally.

Examples:

- `apps/mcp/src/index.ts` authenticates requests, validates API keys or OAuth tokens, and forwards into the MCP layer with `api.supermemory.ai` as the default backend.
- `apps/mcp/src/server.ts` defines the MCP-facing tools and resources, but they are adapters over a client, not the core memory engine.
- `apps/mcp/src/client.ts` uses the `supermemory` SDK plus direct `fetch` calls to hosted endpoints like `/v3/projects`, `/v3/graph/bounds`, `/v3/graph/viewport`, and `/v4/profile`.
- `packages/tools/src/shared/memory-client.ts` builds memory text for prompt injection by calling `/v4/profile`.
- `packages/tools/src/openai/middleware.ts` and `packages/tools/src/vercel/middleware.ts` retrieve memory from the hosted API and inject it into prompts or save conversations back to the service.
- `packages/agent-framework-python` and `packages/openai-sdk-python` repeat the same pattern for Python agent stacks.

So the repo is best understood as a memory client and integration monorepo around a remote service.

## Repo Shape

The highest-signal directories are:

- `apps/mcp`: Cloudflare Worker / Durable Object MCP server and auth surface
- `apps/web`: web product shell
- `packages/tools`: TypeScript tools and middleware for memory injection and persistence
- `packages/ai-sdk`: thin export layer over tool integrations
- `packages/openai-sdk-python`: Python OpenAI wrapper and middleware
- `packages/agent-framework-python`: Python middleware for Microsoft Agent Framework
- `packages/memory-graph`: graph visualization and client-side graph shaping

The repo looks like a product ecosystem around one service, not one monolithic backend.

## Core Implementation Patterns

### 1. Hosted Memory API As The Real Backend

This is the most important architectural fact.

The code repeatedly calls hosted endpoints for the real work:

- profile retrieval
- search
- conversation ingestion
- project listing
- graph viewport and graph bounds

That means the difficult parts of the memory system are mostly off-repo or hidden behind the SDK/API boundary:

- extraction
- indexing
- ranking
- graph construction
- memory/profile synthesis
- document processing
- "smart diffing and append detection" for conversations

This repo exposes the contract around those capabilities more than it exposes the underlying engine.

### 2. Memory As Prompt Injection Middleware

Across TypeScript and Python, the core product loop is similar:

1. take the last user message
2. call the Supermemory profile/search endpoint
3. deduplicate static, dynamic, and search-result memories
4. format them into a system-prompt block
5. optionally persist the conversation back into Supermemory

This pattern is visible in:

- `packages/tools/src/shared/memory-client.ts`
- `packages/tools/src/openai/middleware.ts`
- `packages/tools/src/vercel/middleware.ts`
- `packages/agent-framework-python/src/supermemory_agent_framework/middleware.py`
- `packages/openai-sdk-python/src/supermemory_openai/middleware.py`

This is a strong product pattern because it is simple, portable, and easy to integrate. It is also narrower than "agent orchestration." It is memory augmentation around an existing agent runtime.

### 3. Strong Adapter Surface Across Frameworks

One of Supermemory's clearest strengths is packaging.

The repo provides parallel surfaces for:

- MCP
- TypeScript AI SDK tooling
- OpenAI middleware
- Vercel AI SDK middleware
- Claude-oriented memory tooling
- Python OpenAI middleware
- Python Microsoft Agent Framework middleware

This is exactly the sort of thing that makes a platform sticky. Even when the backend is hosted elsewhere, the integration layer is productized well.

### 4. MCP Is Productized, Not Experimental

The MCP layer is not a toy wrapper.

`apps/mcp/src/server.ts` exposes:

- `memory`
- `recall`
- `listProjects`
- `whoAmI`
- profile resources
- project resources
- a `context` prompt
- a memory-graph app resource and graph fetch tooling

This is important. Supermemory treats MCP as a first-class way to consume the product.

### 5. Conversation Ingestion Is A First-Class Concept

`packages/tools/src/conversations-client.ts` posts structured conversations to `/v4/conversations`, and `packages/tools/src/vercel/middleware.ts` prefers that path when a `conversationId` exists.

That is a useful product idea: do not force every integration to flatten everything into one text blob if the platform can ingest richer conversational structure.

### 6. Memory Graph Is Partly A UI/Product Feature

The graph story is interesting.

`apps/mcp/src/server.ts` exposes graph bounds and viewport tools through the hosted API, but `packages/memory-graph/src/hooks/use-graph-data.ts` also computes client-side document similarity edges with a bounded comparison strategy and uses embeddings already attached to document data.

So the graph product is a mix of:

- hosted graph data retrieval
- client-side graph shaping and visualization logic

That is a useful distinction. The repo does not prove a full local graph engine. It proves a real graph product surface.

### 7. Claude Memory Is File-Like UX On Top Of The Memory Service

`packages/tools/src/claude-memory.ts` is a clever pattern.

It gives Claude a file-like memory interface:

- `view`
- `create`
- `str_replace`
- `insert`
- `delete`
- `rename`

But underneath, it maps those commands onto Supermemory document operations and uses normalized paths as custom IDs. That is a nice UX bridge. It is not a real filesystem memory layer.

## What Supermemory Is Good At

Based on code, Supermemory is stronger than Sigil in these areas:

- Memory productization. It offers a cleaner developer-facing memory product than Sigil currently does.
- Integration breadth. The same memory capability is packaged across multiple ecosystems instead of living only inside one runtime.
- MCP polish. The MCP surface is clearly intentional and consumer-ready.
- Hosted-service ergonomics. The repo assumes a clean API contract and builds around it consistently.
- Conversation ingestion as a product concept. Treating conversations as structured memory input is useful.
- User-facing memory affordances. Profile, recall, projects, graph, and file-like memory operations are all easier to consume than Sigil's current memory surfaces.

## What Supermemory Is Not Trying To Do

This is where comparison discipline matters.

Supermemory, from the code we can see, is not trying to provide:

- a daemon-worker_pool control plane
- project patrol loops
- task DAG execution
- multi-agent routing and stable worker ownership
- retry/escalation policy around autonomous execution
- org-level audit and budget control
- long-lived operator intervention workflows

Sigil is trying to do those things.

So if Supermemory looks "cleaner," part of that is because it is solving a narrower problem.

## Fair Comparison To Sigil

The right comparison boundary is:

### Supermemory

- memory backend contract
- profile/search/recall abstractions
- conversation ingestion
- graph inspection surface
- SDK and middleware ergonomics
- MCP packaging

### Sigil

- orchestration control plane
- task routing and execution
- project and org coordination
- audit, budgets, checkpoints, dispatch
- operator oversight

If we compare Supermemory's clean memory middleware to Sigil's whole orchestrator, the result will be misleading.

If we compare Supermemory's memory product to Sigil's memory subsystem and operator-facing memory affordances, the comparison becomes useful.

## What Sigil Should Learn From Supermemory

### 1. Separate Memory Product Surface From Internal Orchestration Plumbing

Supermemory exposes memory as a clean product:

- profile
- recall
- projects or containers
- graph
- conversation ingestion

Sigil should likely do the same. Right now Sigil has memory and blackboard primitives, but they do not yet present as a crisp, reusable product surface.

### 2. Provide Stable Integration Contracts

Supermemory has a clear contract style:

- one hosted memory API
- thin adapters for each ecosystem
- similar semantics everywhere

Sigil should consider doing something analogous for its own durable context and memory layers:

- one canonical runtime API for memory, task context, checkpoints, and audit lookup
- adapters for CLI, web UI, external agents, and future runtimes

### 3. Make Conversation Ingestion First-Class

The `/v4/conversations` pattern is strong.

Sigil should probably support a canonical "ingest this conversation or run transcript into memory/audit/learning" path rather than relying on ad hoc note capture or tool-specific storage.

### 4. Improve Namespacing And Memory Scope

Supermemory's `containerTag` pattern is simple and effective.

Sigil should likely sharpen its own scoping story for:

- project memory
- agent memory
- user/operator memory
- shared org memory
- ephemeral run-local context

### 5. Build Better Memory Inspection UX

Supermemory treats graph and profile inspection as product features, not backend leftovers.

Sigil should do the same for:

- agent memory
- task-relevant recall
- why a worker used a fact
- cross-project knowledge links
- what was learned versus what was inferred

### 6. Package Features For External Consumption

Supermemory's adapters are good because they assume the core capability should be consumed from many runtimes.

Sigil should think more this way for:

- daemon APIs
- memory APIs
- orchestration events
- intervention hooks
- external agent attachment

## What Sigil Should Not Copy Blindly

### 1. Do Not Collapse Into "Memory Injection Equals Intelligence"

Supermemory's memory loop is useful, but it is still mostly:

- retrieve memory
- inject into prompt
- save new interaction

Sigil's ambition is larger. Memory injection alone does not solve orchestration, delegation, verification, retries, or autonomous work quality.

### 2. Do Not Outsource Core Differentiation Into An Opaque Backend

Supermemory can hide complex behavior behind a hosted API because that is its product model.

Sigil should be careful here. If Sigil wants to be a trustworthy orchestrator, too much of its core reasoning or coordination logic cannot disappear into opaque off-repo services.

### 3. Do Not Duplicate Near-Identical Logic Across Too Many Adapters

One tradeoff in Supermemory's design is repeated middleware logic across TypeScript and Python surfaces.

That may be acceptable for a memory platform. For Sigil, it would create drift if copied too aggressively. Better to centralize semantics and expose thinner adapters.

### 4. Do Not Confuse UI Product With Backend Depth

Supermemory's graph product is attractive, but the code suggests a lot of the hard backend graph logic is abstracted behind API endpoints, with some client-side shaping layered on top.

Sigil should copy the inspectability, not the illusion that visualization itself equals robust backend intelligence.

## Current Comparison Judgment

If the question is "which repo has the better orchestrator?" the answer is still Sigil, because Supermemory is not really that kind of system from the visible code.

If the question is "which repo has the better developer-facing memory product?" Supermemory is ahead.

If the question is "which repo is better packaged as a consumable platform capability?" Supermemory is ahead in memory-related surfaces.

If the question is "which repo has the stronger control-plane substrate for autonomous work?" Sigil still has the more ambitious and more relevant architecture.

## Practical Relevance For Improving Sigil

The most valuable takeaways for Sigil are:

- create a first-class memory API instead of memory being mostly an internal primitive
- create a first-class conversation ingestion path
- create a first-class memory inspection surface
- standardize context scoping and namespaces
- package Sigil capabilities behind stable APIs so multiple runtimes and surfaces can consume them
- improve MCP and external-tool ergonomics

The least valuable takeaway would be trying to turn Sigil into "another hosted memory wrapper." That would miss Sigil's actual opportunity.

## Open Limits Of This Read

Some important Supermemory behavior is not directly visible in this repo because it appears to live behind the hosted service boundary.

That means this baseline cannot fully judge:

- ranking quality
- extraction quality
- profile synthesis quality
- graph construction quality
- document processing quality
- conversation diffing quality

What we can judge confidently is the architecture contract exposed by the codebase, and that contract is clear enough to compare against Sigil productively.

## Bottom Line

Supermemory is best understood as a strong memory platform and integration product, not as a full autonomous orchestration system.

Sigil should learn from Supermemory's:

- API clarity
- framework adapters
- MCP polish
- memory UX
- conversation ingestion model

Sigil should not abandon its own differentiator:

- durable orchestration
- operator control
- multi-agent execution
- project and org coordination

The right move is to make Sigil's memory and context layers as clean and consumable as Supermemory's product surfaces, while keeping Sigil's stronger orchestration architecture intact.
