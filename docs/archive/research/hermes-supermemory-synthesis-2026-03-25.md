# Hermes + Supermemory Synthesis For Sigil

Date: 2026-03-25

Related baselines:

- `docs/hermes-agent-codebase-baseline-2026-03-25.md`
- `docs/supermemory-codebase-baseline-2026-03-25.md`

## Purpose

This document answers a narrower and more useful question than "which project is best?"

The real question is:

- what Hermes contributes
- what Supermemory contributes
- what a smart combination of the two would look like
- what Sigil should borrow from that combination
- what Sigil should avoid copying

## Executive Thesis

Hermes and Supermemory fit together cleanly, but they do not by themselves replace Sigil's reason to exist.

The clean mental model is:

- Hermes = persistent agent runtime and product shell
- Supermemory = memory platform and memory integration layer
- Sigil = orchestration control plane

If you combine Hermes and Supermemory, you get a strong persistent personal or messaging-first agent product with better memory.

If you combine Sigil with the best ideas from Hermes and Supermemory, you get something more ambitious and more valuable:

- a real orchestration system
- with a better runtime shell
- with a better memory product
- with better operator trust and usability

That is the right direction.

## What Hermes Contributes

Hermes is strongest at the runtime and product-shell layer.

Its most useful contributions are:

- persistent session continuity
- messaging gateway design
- cron integrated into the same agent product
- runtime/provider abstraction
- execution backend abstraction
- toolset packaging
- safety rails for dangerous commands and secret handling
- compact bounded delegation
- post-task reflection and skill/memory review
- pragmatic multi-step tool execution patterns

Hermes solves "how do I make one persistent agent actually usable every day?"

That is important because Sigil is still weaker at this layer than it should be.

## What Supermemory Contributes

Supermemory is strongest at the memory product layer.

Its most useful contributions are:

- a clean memory API contract
- profile and recall as first-class concepts
- conversation ingestion as a first-class concept
- simple scoping via containers or projects
- MCP-ready memory surfaces
- memory graph and profile inspection surfaces
- thin adapters for multiple frameworks
- memory packaged as a consumable product instead of a buried subsystem

Supermemory solves "how do I make memory easy to consume, inspect, and integrate?"

That is important because Sigil has memory primitives, but they are not yet productized that cleanly.

## What The Combination Actually Produces

If you combine Hermes and Supermemory directly, the result is roughly:

- one persistent agent
- good session continuity
- good gateway and chat surfaces
- strong framework adapters
- better memory recall and profile context
- better conversation persistence

That is a strong product.

But it is still mostly:

- session-centric
- agent-centric
- product-shell-centric

It is not naturally:

- project-supervisor-centric
- task-DAG-centric
- org-control-plane-centric

So Hermes + Supermemory is not "Sigil but better." It is a different class of system.

## Recommended Role Split

If Sigil is the main system, the correct split is:

### Sigil owns

- task substrate
- project supervision
- routing
- retries and escalation
- budgets
- audit
- blackboard and org state
- operator control plane

### Hermes-inspired layer owns

- runtime/provider resolution
- execution backend abstraction
- session continuity for human-facing channels
- gateway delivery
- cron or scheduled user-facing actions
- approvals and secret hygiene
- bounded delegation ergonomics

### Supermemory-inspired layer owns

- memory API contract
- profile and recall
- conversation ingestion
- memory namespaces or containers
- graph and profile inspection
- adapter surfaces for external runtimes and MCP

That is the cleanest synthesis.

## The Best Combined Architecture

The best version of Sigil after learning from both systems would likely have four clear layers.

### 1. Orchestration Layer

This stays Sigil-native.

Responsibilities:

- missions, tasks, checkpoints, retries
- worker assignment
- stable ownership
- budgets and policies
- audit and intervention
- cross-project coordination

Hermes should not replace this.
Supermemory should not replace this.

### 2. Runtime Layer

This should become much more Hermes-like.

Responsibilities:

- one centralized runtime registry
- provider selection
- capability metadata
- execution environments
- channel/session bridging
- per-runtime approvals and policy

Sigil currently needs this badly.

### 3. Memory Layer

This should become much more Supermemory-like.

Responsibilities:

- explicit memory APIs
- scoped recall
- user, project, agent, and org memory namespaces
- conversation ingestion
- memory inspection and provenance
- external adapters and MCP access

Sigil currently has fragments of this, but not a clear product surface.

### 4. Product Surface Layer

This should combine ideas from both.

From Hermes:

- messaging-first continuity
- durable sessions
- scheduled actions

From Supermemory:

- profile views
- graph views
- easy recall tools
- external integrations

From Sigil:

- operator oversight
- intervention
- project and org dashboards

## What Sigil Should Copy From Hermes

### 1. Runtime Registry And Capability Metadata

Sigil should have one explicit runtime registry with capability declarations such as:

- tool use
- streaming
- approvals
- resumability
- environment types
- cost fidelity
- checkpoint support

Hermes is ahead here.

### 2. Execution Environment Abstraction

Sigil should stop feeling narrowly bound to a few execution modes.

It should have a clean environment contract for:

- local
- containerized
- remote
- specialized runtimes

### 3. Session And Gateway Product Thinking

Sigil needs a better persistent human-facing shell:

- thread continuity
- stable session identity
- search over prior interactions
- delivery back to channels
- scheduled follow-ups

### 4. Trust Rails

Sigil should adopt stronger:

- command approval flows
- secret scrubbing
- execution policy enforcement
- operator-visible safety state

### 5. Narrow Reflection Workers

Hermes's background reflection idea is worth borrowing for:

- post-task memory extraction
- post-task skill updates
- targeted failure reflection

Not as a monolith, but as small support loops.

### 6. Deterministic Tool Programs

Hermes's code-execution RPC pattern is strong for repetitive deterministic workflows.

Sigil should consider a safe equivalent for:

- long tool chains
- transformations
- repetitive repo analysis
- structured maintenance tasks

## What Sigil Should Copy From Supermemory

### 1. Memory As A Product, Not Just Storage

Sigil should expose first-class memory concepts:

- profile
- recall
- scoped search
- conversation history ingestion
- graph or relationship views

### 2. Stable Memory Scoping

Supermemory's container-tag style is simple and effective.

Sigil should have equally crisp scopes for:

- operator memory
- agent memory
- task memory
- project memory
- org memory

### 3. Structured Conversation Ingestion

Sigil should support a canonical path for ingesting:

- conversations
- channel history
- task transcripts
- intervention notes

without flattening everything into one generic note format.

### 4. Thin Adapters Across Surfaces

Sigil should expose its memory layer through:

- internal runtime APIs
- MCP
- CLI
- web UI
- future external SDKs

### 5. Better Memory Inspection UX

Sigil needs better answers to:

- what is remembered
- why it was remembered
- where it came from
- when it was last used
- what influenced this task

Supermemory treats this as product surface. Sigil should too.

## What Sigil Should Not Copy

### 1. Hermes's Monolith

Sigil should not fold orchestrator, runtime, gateway, memory, reflection, and execution into one giant loop.

That would destroy one of Sigil's main architectural advantages.

### 2. Hermes's Single-Agent Center Of Gravity

Sigil should not degrade into "one agent with helpers."

It should remain:

- task-centric
- project-centric
- org-centric

where that matters.

### 3. Supermemory's Opaque Core Dependency Model

Supermemory can hide hard logic behind hosted APIs because that is its product model.

Sigil should not make its core orchestration truth depend on opaque external behavior.

### 4. Memory-As-Intelligence Thinking

Good memory does not equal good orchestration.

Memory should support:

- routing
- context
- continuity
- explanation

It should not be mistaken for verification or planning quality by itself.

### 5. Repeated Adapter Logic Everywhere

Both Hermes and Supermemory tolerate some duplication because they optimize for product breadth.

Sigil should be more disciplined:

- central semantics
- thin adapters
- minimal drift

## Best Practical Recommendation

If the goal is to make Sigil much better, the right move is not:

- rewrite Sigil into Hermes
- bolt Supermemory on top blindly
- merge everything into one mega-agent

The right move is:

1. Keep Sigil as the orchestration kernel.
2. Rebuild Sigil's runtime layer using Hermes-style discipline.
3. Rebuild Sigil's memory layer using Supermemory-style productization.
4. Expose both through a stronger operator and messaging surface.

That gives Sigil a much stronger shape without losing what makes it distinct.

## Recommended Build Order

### Phase 1. Runtime And Policy

Borrow mostly from Hermes.

Build:

- centralized runtime registry
- capability metadata
- environment abstraction
- policy and approval layer
- secret hygiene

### Phase 2. Memory Product Surface

Borrow mostly from Supermemory.

Build:

- memory API
- recall and profile APIs
- conversation ingestion
- scoped namespaces
- memory inspection views

### Phase 3. Human-Facing Continuity

Borrow mostly from Hermes.

Build:

- session store
- channel and thread continuity
- scheduled tasks for human-facing surfaces
- search over prior interaction history

### Phase 4. Operator Integration

Keep this Sigil-native, but enriched by the new layers.

Build:

- task-aware memory traces
- why-this-worker and why-this-context explanations
- intervention and resume flows
- dashboards that connect memory, runtime, and orchestration state

## The Ideal End State

The ideal end state is not "Hermes plus Supermemory."

The ideal end state is:

- Sigil's control plane
- Hermes's runtime maturity
- Supermemory's memory product clarity

In plain terms:

- Sigil decides and coordinates work
- Hermes-like infrastructure makes workers and human-facing surfaces reliable
- Supermemory-like infrastructure makes context and memory usable, inspectable, and portable

That combination is much stronger than copying either project wholesale.

## Bottom Line

Hermes and Supermemory are complementary.

Hermes teaches Sigil how to become a better persistent agent runtime and product shell.
Supermemory teaches Sigil how to become a better memory platform and memory product.

Neither should replace Sigil's orchestration core.

The right synthesis is:

- keep Sigil's control-plane architecture
- import Hermes's runtime and safety discipline
- import Supermemory's memory contracts and product surfaces

That is the most logical combination and the best path to a much stronger Sigil.
