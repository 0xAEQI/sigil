# Sigil vs Hermes, Supermemory, and Deer Flow

Date: 2026-03-25
Current Sigil revision: `a517e2a26cb1`

Comparison inputs:

- `docs/hermes-agent-codebase-baseline-2026-03-25.md`
- `docs/supermemory-codebase-baseline-2026-03-25.md`
- `docs/deer-flow-codebase-baseline-2026-03-25.md`

Validation note:

- Current local Sigil passed `cargo test --workspace -q`
- Total locally passing tests from the run: 433

## Purpose

This document is the first direct comparison pass against the current Sigil v4 codebase.

The goal is not to decide which repo is "coolest."
The goal is to define how Sigil becomes the strongest system across all of the important angles:

- orchestration
- runtime shell
- memory
- UX
- trust
- product completeness

## Executive Judgment

Current Sigil is now the strongest orchestration architecture in this set.

That was not fully true before v4. It is much more defensible now.

The reason is that current Sigil has a real layered system:

- Intent
- Understand
- Orchestrate
- Execute
- Verify
- Learn
- Proact

And those layers are no longer just docs language. They now show up in code through:

- orchestration middleware
- verification pipeline
- memory graph and intelligent retrieval
- notes and directives
- proactive engine
- skill promotion

So the current comparison is:

- Sigil leads on orchestration control plane design
- Hermes still leads on persistent agent product shell and runtime maturity
- Supermemory still leads on memory productization and external memory consumption
- Deer Flow still leads on live super-agent UX and harness/app product polish

That means Sigil does not need to copy any of them wholesale.
It needs to beat each one on their own angle while preserving its stronger core.

## What Sigil Already Wins

### 1. Orchestration as a first-class system

Sigil is now the clearest "real orchestrator" of the group.

It has:

- daemon and supervisor logic
- task and mission substrate
- durable project and org state
- routing and stable agent identity
- verification
- expertise accumulation
- notes that become directives and tasks
- proactive analysis and action surfaces

Hermes does not really do this.
Supermemory is not even in this category.
Deer Flow gets closer, but is still thread/run-centric rather than org/control-plane-centric.

### 2. Verification as part of the core architecture

Current Sigil has an explicit verification pipeline in `crates/sigil-orchestrator/src/verification.rs`.

That is important because "most powerful orchestrator" does not mean "most active agent."
It means:

- execution is checked
- confidence is computed
- bad outcomes are rejected or escalated

This is more aligned with real orchestration than the others.

### 3. Memory is becoming structural, not just assistive

With v4, Sigil now has:

- memory graph
- hotness scoring
- contradiction/supersession handling
- intelligent retrieval with multiple signals
- provenance and verification-aware memory

That is a deeper memory architecture than Hermes and much more internal sophistication than what Supermemory exposes in-repo.

The gap is no longer "Sigil lacks memory depth."
The gap is now mostly "Sigil lacks memory product surfaces and integrations."

### 4. Notes/directives are a real differentiator

The notes system in `crates/sigil-orchestrator/src/notes.rs` is one of the clearest Sigil-native moats.

It turns persistent intent into:

- directives
- tracked status
- linked tasks

That is a better foundation for long-lived AI execution than simple chat history or memory injection alone.

### 5. Proactive architecture is more explicit than the others

Current Sigil's proactive layer is not just "cron plus agent."
It is designed as a layer with:

- morning briefs
- anomaly detection
- suggestions
- notifications

That is closer to what a real operator system should become.

## Where Hermes Still Beats Sigil

Hermes is still stronger in the following areas:

- persistent personal/session UX
- messaging-first continuity
- runtime/provider resolution
- execution environment abstraction
- dangerous-command approvals
- secret hygiene
- toolset packaging
- practical day-to-day "one agent can just do things for me" product polish

In plain terms:

Hermes is still more complete as a persistent agent product.

Sigil needs to beat Hermes by importing these strengths:

### What Sigil must take from Hermes

- one centralized runtime registry
- explicit runtime capability metadata
- real execution backend abstraction
- session store and recall for chat surfaces
- approval and trust rails
- better gateway and messaging continuity
- tighter tool/capability packaging

### What Sigil must not copy from Hermes

- the monolithic runtime center of gravity
- the single-agent worldview
- session-first thinking replacing task/project/org thinking

## Where Supermemory Still Beats Sigil

Supermemory is still stronger in the following areas:

- memory as a product
- memory API clarity
- memory consumption from many runtimes
- conversation ingestion as a first-class memory path
- MCP memory surface
- memory profile and recall ergonomics
- graph and memory inspection as user-facing features

In plain terms:

Supermemory is still better at making memory easy to use.

Sigil now has more interesting memory internals than before, but Supermemory is still better at turning memory into a clean external product.

### What Sigil must take from Supermemory

- first-class memory API
- first-class recall/profile/graph endpoints
- structured conversation ingestion
- clear memory namespaces
- adapters across CLI, web, MCP, and future SDKs
- better memory inspection UX

### What Sigil must not copy from Supermemory

- memory-as-intelligence reductionism
- dependence on opaque externalized core behavior for Sigil's main truth
- over-optimizing for integration wrappers while neglecting orchestration depth

## Where Deer Flow Still Beats Sigil

Deer Flow is still stronger in the following areas:

- live frontend integration
- thread streaming UX
- subtask streaming and subagent progress UI
- harness/app separation
- embedded client experience
- practical artifact/upload/thread ergonomics
- runtime modularity inside one harness

In plain terms:

Deer Flow feels more immediately productized in the live super-agent shell.

Sigil is deeper as an orchestrator, but Deer Flow is ahead in the experience of interacting with agent execution in real time.

### What Sigil must take from Deer Flow

- a stronger embedded client/API surface
- live task and subtask streaming into the UI
- clearer harness/product-surface separation
- better artifact and workspace UX
- better frontend coupling to real backend state
- tighter event protocol for live execution progress

### What Sigil must not copy from Deer Flow

- thread/run as the dominant system abstraction
- middleware composition as a replacement for orchestration control logic
- built-in subagent types as if that equals durable organizational structure

## The Current Scorecard

This is the most honest current scoreboard.

### Orchestration control plane

Leader: Sigil

Reason:

- project and org state
- routing
- verification
- notes/directives
- proactive layer

### Persistent personal-agent product shell

Leader: Hermes

Reason:

- session continuity
- gateway polish
- runtime and environment maturity
- operator trust rails

### Memory product and external memory ergonomics

Leader: Supermemory

Reason:

- recall/profile/graph as product
- framework adapters
- conversation ingestion
- MCP memory consumption

### Live super-agent UX

Leader: Deer Flow

Reason:

- live LangGraph streaming UI
- subtask cards
- embedded client
- real frontend wiring

### Deep memory architecture

Most interesting current system: Sigil

Reason:

- graph structure
- contradiction handling
- provenance
- retrieval scoring
- hotness

But this is only a win if it becomes easy to consume and inspect.

### Verification-aware execution

Leader: Sigil

Reason:

- explicit verification pipeline
- confidence scoring
- reject/flag/approve logic

### Runtime/provider/env maturity

Leader: Hermes

Runner-up: Deer Flow

Sigil is still behind here.

### UI/backend integration maturity

Leader: Deer Flow

Sigil is still behind here.

## What "Best Possible Sigil" Actually Means

The best possible Sigil is not:

- Hermes plus tasks
- Supermemory plus routing
- Deer Flow plus a daemon

The best possible Sigil is:

- Sigil's orchestration core
- Hermes's runtime maturity
- Supermemory's memory product clarity
- Deer Flow's live execution UX

That is the real synthesis target.

## The Roadmap To Beat All Three

### Track 1. Defend and deepen Sigil's native moat

This is the part no other repo is matching well.

Build harder on:

- task/project/org control plane
- notes and directives
- verification and escalation
- proactive operations
- stable worker identity
- expertise and skill promotion

This is the core that must stay uniquely Sigil.

### Track 2. Beat Hermes on runtime and trust

This is the highest-priority deficit area now.

Build:

- centralized runtime registry
- provider and environment capability model
- session substrate for human-facing surfaces
- approvals and policy enforcement
- secret hygiene and environment isolation
- gateway continuity across chat surfaces

Win condition:

Sigil becomes as reliable to operate daily as Hermes, while remaining much stronger in orchestration.

### Track 3. Beat Supermemory on memory product

Current Sigil now has enough internal memory depth that this is realistic.

Build:

- memory APIs
- recall/profile/graph endpoints
- conversation ingestion
- scoped namespaces
- memory provenance UI
- MCP and external adapter surfaces

Win condition:

Sigil keeps deeper orchestration-linked memory than Supermemory, but becomes just as easy to consume.

### Track 4. Beat Deer Flow on live operator UX

Build:

- real live web cockpit
- streamed task/subtask progress
- artifact-first task views
- embedded client
- clearer harness/UI contract
- backend events shaped for operator observability

Win condition:

Sigil's live experience becomes as legible and satisfying as Deer Flow's, but grounded in a stronger control plane.

### Track 5. Prove the claim

This is still mandatory.

Even with v4, Sigil still needs stronger proof than nouns.

Build:

- end-to-end eval harnesses
- routing benchmarks
- verification quality evals
- proactive suggestion quality evals
- recovery/resume tests
- operator trust metrics

Win condition:

Sigil is not only more ambitious than the others. It is measurably better.

## The Current Risk

The biggest risk is now different from before.

It is no longer:

"Sigil has ideas but not enough system."

It is now:

"Sigil has a strong system, but it may still lose in product reality to narrower tools that feel easier, safer, and more complete."

That is the real competitive danger.

Hermes can beat Sigil in daily usability.
Supermemory can beat Sigil in memory usability.
Deer Flow can beat Sigil in live execution UX.

So Sigil must not only be deeper.
It must become equally usable.

## Final Assessment

Current Sigil is now credible as the strongest orchestration architecture in this comparison set.

It already has the best answer to:

- how work is structured
- how it is routed
- how it is verified
- how intent persists
- how memory can become part of an organization instead of just a chat aid

But it is not yet the strongest total product.

To become the best possible system, Sigil must now do three things:

1. keep deepening its native orchestration moat
2. absorb Hermes's runtime and trust strengths
3. absorb Supermemory's memory product strengths and Deer Flow's live-shell strengths

If that happens, Sigil will not just be better at "orchestration."
It will become the only system here that is both:

- deeper than the others
- and more complete than the others

That is the winning path.
