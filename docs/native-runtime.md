# Native Runtime Design

This document defines the target shape of Sigil's native runtime.

It is written against the current codebase, not against a hypothetical rewrite from scratch.

## Why This Matters

Sigil's long-term product depends on owning the execution substrate.

If the runtime is weak, Sigil becomes a planner wrapped around external model products.
If the runtime is strong, Sigil can become:

- model-independent
- enterprise-deployable
- cost-efficient
- tightly integrated with its own memory, tasks, approvals, and metrics

That is why the runtime is not just an implementation detail. It is the product core.

## Current State

Sigil already has important building blocks:

- a native agent loop in [agent.rs](/home/claudedev/sigil/crates/sigil-core/src/agent.rs)
- worker execution through [agent_worker.rs](/home/claudedev/sigil/crates/sigil-orchestrator/src/agent_worker.rs)
- middleware, checkpoints, memory, verification, audit, and event broadcasting in `sigil-orchestrator`
- task outcome parsing in [executor.rs](/home/claudedev/sigil/crates/sigil-orchestrator/src/executor.rs)

The current strength is that Sigil already owns real infrastructure around execution.

The main weakness is that execution semantics are still too thin:

- the core loop is mostly "LLM response -> tool calls -> repeat"
- final state is still inferred from result text prefixes like `BLOCKED:` and `HANDOFF:`
- runtime state is not yet modeled as a first-class structured session
- verification and artifact capture exist, but they are not yet the canonical output contract

## Design Goal

The target is a native runtime that is deeply coupled to Sigil's orchestration layer without collapsing the two into one giant loop.

The runtime should be:

- native-first
- structured
- inspectable
- resumable
- verifiable
- model-agnostic

## Architectural Split

Sigil should keep three layers distinct:

### 1. Kernel

Deterministic control-plane logic:

- task graph
- scheduling
- approvals
- budgets
- audit
- readiness
- metrics
- policy

### 2. Native Runtime

Structured execution engine:

- context assembly
- step planning
- model interaction
- tool invocation
- artifact capture
- checkpointing
- recovery
- verification handoff

### 3. Interfaces

Operator-facing and system-facing surfaces:

- web UI
- CLI
- API
- channels

The kernel owns truth. The runtime owns execution. The interfaces expose control.

## Runtime Contract

Sigil should converge on a first-class runtime session model rather than text-only worker outputs.

### Core objects

`TaskEnvelope`
- task id
- project id
- mission id
- assignee
- operator intent
- success criteria
- budget and time limits
- approval policy

`RuntimeSession`
- session id
- task id
- worker id
- status
- started at / updated at
- active model and runtime settings
- checkpoint references

`StepRecord`
- step id
- phase
- summary
- rationale
- status
- timestamps

`ToolInvocation`
- tool name
- input
- output summary
- raw artifact references
- duration
- policy decision
- error state

`Artifact`
- type
- path or logical reference
- provenance
- verification relevance

`VerificationReport`
- checks run
- evidence captured
- confidence
- warnings
- approval requirement

`RuntimeOutcome`
- status: done | blocked | handoff | failed
- operator-facing summary
- machine-readable reason
- next action
- evidence bundle

## Execution Phases

The native runtime should operate in explicit phases.

### 1. Prime

Assemble execution context from:

- task description
- project context
- relevant memory
- recent audit and blackboard events
- previous checkpoint
- notes/directives
- policy and budget constraints

This should be a structured bootstrap step, not prompt stuffing.

### 2. Frame

Turn the task into a bounded execution frame:

- what is being changed
- what is unknown
- what constraints matter
- what "done" means
- whether approvals or clarifications are needed first

### 3. Act

Execute through a disciplined inspect-edit-verify loop:

- inspect repo and environment
- propose or select next step
- call tools
- capture artifacts
- update session state

### 4. Verify

Verification must be a first-class phase, not just an instruction in the prompt.

The runtime should produce:

- checks attempted
- checks passed or failed
- evidence references
- residual risk

### 5. Commit Outcome

The runtime returns a structured outcome to the orchestrator.

Text summaries still matter for the operator, but they should sit on top of structured runtime state rather than replacing it.

## Integration With Orchestration

The runtime should be designed for Sigil's current orchestration model:

- `ChatEngine` creates or continues work
- `WorkerPool` assigns and schedules
- `AgentWorker` executes the runtime session
- middleware shapes behavior during execution
- verification informs task closure or escalation
- audit, dispatch, metrics, and memory update around the session

The runtime should not bypass these systems. It should make them richer and more deterministic.

## Fusion Model

The runtime and orchestrator should be deeply integrated, but they should not collapse into one opaque mega-loop.

The right model is:

- hard-fused read access to Sigil context
- structured write access for execution artifacts and state
- mediated access to orchestration-side mutations

### Hard-fused read access

The native runtime should directly consume a structured context bundle containing:

- task envelope
- persistent agent identity
- project instructions and knowledge
- active skill prompt
- active notes and directives
- relevant memory recalls
- blackboard entries
- prior checkpoints
- workflow and policy constraints
- codebase context and future code-intelligence signals

This is where Sigil should be opinionated. The runtime should not think of itself as "a prompt plus tools." It should think of itself as executing from a rich, Sigil-owned operating context.

### Structured write access

The runtime should directly emit:

- step records
- tool invocations
- artifacts
- checkpoints
- verification evidence
- runtime events
- machine-readable outcomes

These are native execution outputs and should not require extra approval from the orchestrator to exist.

### Mediated orchestration access

The runtime should be able to request changes to the wider system, but it should not unilaterally mutate control-plane truth.

Examples:

- create subtask
- request approval
- propose reassignment
- request escalation
- suggest org change
- attach note to mission

The orchestrator or kernel should decide whether and how those requests are applied.

## Native Runtime Responsibilities

The runtime should own:

- structured step state
- tool-call lifecycle
- artifact capture
- checkpoint format
- execution event stream
- machine-readable outcome contract
- session-local recovery logic

The orchestrator should own:

- assignment
- routing
- retries across workers
- mission/task state transitions
- approvals
- background automation policy
- cost ceilings and system-level readiness

## Identity, Sessions, and Subagents

Sigil should keep persistent identities separate from ephemeral executions.

### Persistent identities

These are long-lived logical roles:

- `rei`
- `engineer`
- `reviewer`
- `researcher`
- domain-specific future agents

They own:

- role and mandate
- memory and preferences
- org placement
- audit history
- durable responsibility

### Ephemeral sessions

Each task execution should create a runtime session attached to a persistent identity.

The session owns:

- active execution state
- current step history
- tool-call log
- artifacts
- checkpoint chain
- verification state

### Subagents

Subagents should usually be ephemeral child sessions rather than fully independent persistent personas.

That means:

- parent session delegates a bounded subtask
- child session inherits a scoped context subset
- child returns artifacts, evidence, and structured outcome
- parent integrates the result into its own runtime session

This gives Sigil runtime-level orchestration inside a session without confusing it with system-level orchestration across the whole product.

## Workflow and Environment Policy

Sigil should increasingly move best practices out of prompt prose and into runtime policy.

Important examples already latent in the codebase:

- worktree-first repo editing
- external git-grounded checkpoints
- verification before completion
- middleware-enforced guardrails

The runtime should treat these as execution policy, not merely suggestions.

Examples of native policy:

- repo-changing work defaults to isolated worktrees
- risky tools require explicit approval states
- completion requires verification evidence
- checkpoints happen at meaningful execution boundaries
- handoffs carry structured resume state

## Code Intelligence Layer

Sigil should eventually expose a codebase intelligence layer to the runtime.

This is where GitNexus-style ideas fit best.

The runtime should be able to query for:

- symbol ownership
- likely impact radius
- dependency and call relationships
- nearby tests
- relevant modules
- historical change context

This should be a structured read model for the runtime, not a giant raw graph dump pasted into prompts.

## Model Strategy

The runtime must not assume one model class.

It should support:

- large frontier models for hard reasoning
- cheaper hosted models for easier steps
- local models where privacy or cost dominates

That means the runtime must compensate for weaker models through:

- tighter scopes
- better context shaping
- stronger deterministic guards
- explicit verification
- smaller step boundaries

## Immediate Gaps To Close

These are the most important gaps visible in the current code:

1. Replace text-prefix outcome parsing with a structured runtime outcome model.
2. Introduce a persistent `RuntimeSession` abstraction around worker execution.
3. Make artifacts and verification evidence first-class outputs.
4. Separate execution phases more explicitly inside the native loop.
5. Expose runtime session state cleanly to the web UI and event stream.

## Decision Rule

If a runtime feature increases:

- inspectability
- resumability
- verification quality
- model independence
- cost control

it is on the critical path.

If it only makes the loop look more magical while reducing structure, it is probably a regression.
