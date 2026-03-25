# Sigil v4 — The Architecture

One system. One loop. Everything connects.

---

## The Core Abstraction

A sigil is intent that manifests into reality. The entire system is one loop:

```
Intent → Understand → Orchestrate → Execute → Verify → Learn → Proact
   ↑                                                              │
   └──────────────────────────────────────────────────────────────┘
```

Every component in Sigil exists to serve one step of this loop.
Nothing exists outside it.

---

## The Seven Layers

### Layer 0: Intent

Where intent enters the system. Multiple surfaces, one pipeline.

```
Chat (web)          ──┐
Telegram/WhatsApp   ──┤
Notes panel         ──┼──→ ChatEngine ──→ IntentStream
API                 ──┤
Cron/Watchdog       ──┤
Proactive engine    ──┘
```

Every input — human message, automated trigger, proactive suggestion — becomes
an `Intent` with:
- source (chat, note, cron, proactive, api)
- channel (project, department, global)
- content (raw text)
- urgency (immediate, normal, background)
- author (human, agent, system)

**Key insight from DeerFlow:** The clarification tool pattern. When the system
can't resolve intent, it doesn't guess — it asks. The `ask_clarification` intent
flows back through the same surfaces it came in on.

### Layer 1: Understand

Turn raw intent into structured work.

```
IntentStream
    ↓
┌─────────────────────────┐
│ Intent Classifier        │  Quick (status, note, close) → immediate response
│                          │  Complex (build, fix, research) → decomposition
│                          │  Ambiguous → clarification request
└─────────────────────────┘
    ↓ (complex)
┌─────────────────────────┐
│ Decomposer              │  One intent → task DAG
│                          │  Each task: ≤10 min scope
│                          │  Each task: verifiable DONE condition
│                          │  Each task: self-contained context (no shared state assumptions)
│                          │  Explicit file paths, exact commands, expected outputs
└─────────────────────────┘
    ↓
┌─────────────────────────┐
│ Query Planner            │  Per task, generate typed queries:
│                          │    - domain knowledge (priority 5)
│                          │    - recent decisions (priority 4)
│                          │    - system patterns (priority 3)
│                          │    - similar past tasks (priority 2)
│                          │  Merge results by priority into context bundle
└─────────────────────────┘
```

**Key insight from Superpowers:** Task sizing matters. Tasks should be 2-10 min
of agent work. Larger tasks get subdivided. Each task must specify the DONE condition
as a verifiable assertion, not a vague description.

**Key insight from OpenViking:** Don't do one flat memory search. Generate multiple
typed queries with priorities. A trading task needs domain knowledge (high priority),
recent decisions (medium), and system patterns (low). Merge by relevance.

### Layer 2: Orchestrate

Route tasks to the right agent at the right time.

```
Task DAG
    ↓
┌─────────────────────────┐
│ Supervisor                │
│                          │
│ 1. Expertise routing     │  Match task domain → agent expertise scores
│ 2. Preflight assessment  │  LLM evaluates: can this agent handle this?
│ 3. Resource check        │  Budget available? Worker slot open? Rate limit ok?
│ 4. Dependency resolution │  Predecessor tasks complete?
│ 5. Isolation setup       │  Git worktree for parallel execution on same repo
│                          │
│ Three-strikes rule:      │  3 failures on same task → escalate model or human
│ Recursive ban:           │  Workers cannot create tasks that create tasks
│ Scheduling:              │  Background tasks yield to urgent work
└─────────────────────────┘
```

**Key insight from DeerFlow:** Hard limit on delegation depth. Workers cannot
recursively spawn sub-agents. The Supervisor is the only entity that creates tasks.
This prevents cascade failures.

**Key insight from Superpowers:** Git worktrees for parallel execution. When two
workers touch the same project, each gets an isolated worktree. Merge after completion.

### Layer 3: Execute

The middleware chain. This is where Sigil becomes composable.

```
Task assigned to Worker
    ↓
┌─────────────────────────────────────────┐
│           Middleware Chain                │
│                                          │
│  ┌─ Context Budget ──────────────────┐  │  Cap enrichment at ~200 lines
│  │  Priority: task > memory > board  │  │  Use <important> tags for critical rules
│  └───────────────────────────────────┘  │
│  ┌─ Planning Gate ───────────────────┐  │  Worker must outline approach first
│  │  Verify plan before execution     │  │  DONE condition stated upfront
│  └───────────────────────────────────┘  │
│  ┌─ Tool Filter ─────────────────────┐  │  Progressive loading: metadata only
│  │  Role-based permissions           │  │  Full schemas loaded on demand
│  └───────────────────────────────────┘  │
│  ┌─ Guardrails ──────────────────────┐  │  Pre-execution filter on tool calls
│  │  Deny dangerous ops, structured   │  │  Fail-closed by default
│  │  explanations for recovery        │  │
│  └───────────────────────────────────┘  │
│  ┌─ Loop Detection ──────────────────┐  │  MD5 hash tool calls in window
│  │  Warn at 3 repeats, kill at 5     │  │  Prevents stuck agents
│  └───────────────────────────────────┘  │
│  ┌─ Message Queue ───────────────────┐  │  Check for injected messages
│  │  Between each tool call           │  │  Enables mid-run course correction
│  └───────────────────────────────────┘  │
│  ┌─ Cost Tracking ───────────────────┐  │  Token/cost accumulation per task
│  │  Budget ceiling enforcement       │  │  Kill if budget exceeded
│  └───────────────────────────────────┘  │
│  ┌─ Checkpoint ──────────────────────┐  │  Periodic state snapshots
│  │  Git commit at milestones         │  │  Resume from last checkpoint on failure
│  └───────────────────────────────────┘  │
│                                          │
│  ┌─ Core Agent Loop ─────────────────┐  │
│  │  Claude Code subprocess           │  │
│  │  Identity + memory + skill prompt │  │
│  │  ReAct: think → tool → observe    │  │
│  └───────────────────────────────────┘  │
│                                          │
│  ┌─ Safety Net ──────────────────────┐  │  On failure: check for artifacts
│  │  Salvage partial work (commits,   │  │  Preserve before discarding
│  │  files, findings)                 │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
    ↓
Outcome: DONE | DONE_WITH_CONCERNS | BLOCKED | NEEDS_CONTEXT | HANDOFF | FAILED
```

Every middleware is a Rust trait:
```rust
#[async_trait]
trait Middleware: Send + Sync {
    async fn before_model(&self, ctx: &mut WorkerContext) -> MiddlewareResult;
    async fn after_model(&self, ctx: &mut WorkerContext) -> MiddlewareResult;
    async fn before_tool(&self, ctx: &mut WorkerContext, call: &ToolCall) -> MiddlewareResult;
    async fn after_tool(&self, ctx: &mut WorkerContext, call: &ToolCall, result: &ToolResult) -> MiddlewareResult;
}
```

Middleware is configured per agent role. Engineer gets all middleware. Researcher
skips guardrails on read-only operations. Reviewer gets extra verification middleware.

**Key insight from DeerFlow:** This is the architectural unlock. Every feature
becomes a plugin, not a patch. Loop detection, guardrails, memory, progressive
tool loading — all composable, all testable independently.

### Layer 4: Verify

Don't trust self-reported outcomes.

```
Worker reports DONE
    ↓
┌─────────────────────────┐
│ Verification Pipeline     │
│                          │
│ 1. Artifact check        │  Did the worker produce commits/files/output?
│ 2. Test runner           │  If tests exist, do they pass?
│ 3. Spec compliance       │  Does output match the task's DONE condition?
│ 4. Quality review        │  Automated code quality check (optional)
│ 5. Confidence scoring    │  Rate outcome confidence: high/medium/low
│                          │
│ Result:                  │
│   high confidence  → auto-close task
│   medium           → close with flag for human review
│   low              → reject, re-queue or escalate
└─────────────────────────┘
```

**Key insight from Superpowers:** Two-stage review (spec compliance + quality)
as separate automated checks after every task completion. The reviewer agent
runs automatically, not on request.

**Key insight from GitNexus:** Confidence metadata on outcomes. The supervisor
uses confidence to decide auto-close vs. human review.

### Layer 5: Learn

Every task makes the system smarter.

```
Completed task + verification result
    ↓
┌─────────────────────────────────────────┐
│ Learning Pipeline                        │
│                                          │
│ ┌─ Reflection ───────────────────────┐  │  Extract insights from execution
│ │  What worked? What failed? Why?    │  │  Categorize: case, pattern, fact, preference
│ └────────────────────────────────────┘  │
│ ┌─ Deduplication ────────────────────┐  │  Vector similarity vs existing memories
│ │  SKIP (duplicate)                  │  │  LLM judgment: CREATE, MERGE, or DELETE
│ │  CREATE (new insight)              │  │
│ │  MERGE (consolidate with existing) │  │
│ └────────────────────────────────────┘  │
│ ┌─ Hierarchical Storage ─────────────┐  │  Organize into directory-like structure
│ │  L0: one-line abstract             │  │  L0/L1 guide search, L2 is full content
│ │  L1: paragraph overview            │  │
│ │  L2: full content                  │  │
│ └────────────────────────────────────┘  │
│ ┌─ Hotness Scoring ──────────────────┐  │  frequency × sigmoid + recency × exp decay
│ │  7-day half-life                   │  │  Blend with semantic score at 0.2 weight
│ │  Accessed memories get hotter      │  │  Old unused memories naturally fade
│ └────────────────────────────────────┘  │
│ ┌─ Skill Promotion ─────────────────┐  │  Recurring patterns → formal skill files
│ │  3+ similar insights = candidate   │  │  SKILL.md: when, steps, verification
│ │  Human approval to promote         │  │
│ └────────────────────────────────────┘  │
│ ┌─ Expertise Update ────────────────┐  │  Agent scores adjust based on outcomes
│ │  Success → score up               │  │  Failure → score down
│ │  Informs future routing            │  │
│ └────────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

**Key insight from OpenViking:** Memory deduplication before storage, hierarchical
organization with summaries at each level, and hotness scoring that naturally surfaces
recent knowledge. This is dramatically better than flat append-only storage.

**Key insight from Claude Code practices:** Promote recurring reflection insights
into formal skill definitions. Three similar insights about "how to deploy" →
a `deploy.md` skill file that workers load on demand.

### Layer 6: Proact

The layer that makes Sigil a CEO, not a tool.

```
┌─────────────────────────────────────────┐
│ Proactive Engine                         │
│                                          │
│ ┌─ Patrol Loop ─────────────────────┐   │  Every 5 min: scan all projects
│ │  Stale tasks? → nudge or escalate │   │
│ │  Budget drift? → alert            │   │
│ │  Worker stuck? → loop detection   │   │
│ │  Missed cron? → report            │   │
│ └────────────────────────────────────┘  │
│ ┌─ Morning Brief ────────────────────┐  │  Daily at configured time
│ │  What completed overnight          │  │  Delivered: web + Telegram + email
│ │  What's blocked                    │  │  Includes note status updates
│ │  What needs your decision          │  │
│ │  Cost summary                      │  │
│ └────────────────────────────────────┘  │
│ ┌─ Anomaly Detection ───────────────┐  │  Pattern-based alerts
│ │  Cost spike → notify              │  │
│ │  Error rate increase → investigate │  │
│ │  Agent performance drop → flag     │  │
│ └────────────────────────────────────┘  │
│ ┌─ Suggestion Engine ───────────────┐  │  Based on notes + project state
│ │  "You wrote X, should I start?"   │  │
│ │  "Task Y is similar to past Z"    │  │
│ │  "This could be automated as..."  │  │
│ └────────────────────────────────────┘  │
│                                          │
│ Output → new Intents (back to Layer 0)  │
└─────────────────────────────────────────┘
```

This is the moat. Every other AI tool waits. Sigil moves.

---

## The Notes System

Notes are not a feature. They are the second surface of the intent layer.

```
Chat:  ephemeral, conversational    "fix the trading bot"
Notes: persistent, declarative      "Q2: pricing page, bot profitable, 100 users"
```

Both feed into the same IntentStream. The difference is persistence and framing.

### Storage
Per-project SQLite table in `.sigil/notes.db`:
```sql
CREATE TABLE notes (
    id TEXT PRIMARY KEY,
    channel TEXT NOT NULL,      -- project or department scope
    content TEXT NOT NULL,      -- markdown
    updated_at TEXT NOT NULL,
    version INTEGER DEFAULT 1
);
```

### Directive Detection
Lines that match imperative patterns get tracked:
```
"launch pricing page"     → ○ pending (no matching task)
"fix trading bot"         → ⟳ active  (matched to task as-3021)
"redesign landing page"   → ✓ done    (matched to completed task)
```

Matching: explicit link (user activates) or fuzzy embedding similarity as suggestion.

### UI
The context panel evolves:
- Tab 1: **Notes** — editable textarea per channel, directive status indicators
- Tab 2: **Context** — tasks, knowledge, team (current behavior)
- Tab 3: **Brief** — latest morning brief (global channel only)

### Cross-Surface
```
Web notes panel  ←→  Telegram "note: ..."  ←→  Memory API
```
Write in one place, visible everywhere. Agents see notes during execution
(injected into worker context when relevant).

---

## The Middleware Trait (Implementation)

The core abstraction that makes the execution layer composable:

```rust
/// Result of a middleware step.
pub enum MiddlewareAction {
    Continue,                    // proceed to next middleware
    Skip,                       // skip remaining middleware, proceed to core
    Halt(String),                // stop execution with reason
    Inject(Vec<Message>),        // add messages to context
}

/// A composable behavior layer around the agent execution core.
#[async_trait]
pub trait Middleware: Send + Sync + 'static {
    fn name(&self) -> &str;

    /// Called before sending messages to the model.
    async fn before_model(&self, _ctx: &mut WorkerContext) -> MiddlewareAction {
        MiddlewareAction::Continue
    }

    /// Called after receiving model response, before tool execution.
    async fn after_model(&self, _ctx: &mut WorkerContext) -> MiddlewareAction {
        MiddlewareAction::Continue
    }

    /// Called before each tool invocation.
    async fn before_tool(
        &self,
        _ctx: &mut WorkerContext,
        _call: &ToolCall,
    ) -> MiddlewareAction {
        MiddlewareAction::Continue
    }

    /// Called after each tool invocation.
    async fn after_tool(
        &self,
        _ctx: &mut WorkerContext,
        _call: &ToolCall,
        _result: &ToolResult,
    ) -> MiddlewareAction {
        MiddlewareAction::Continue
    }
}

/// Chain of middleware executed in order.
pub struct MiddlewareChain {
    layers: Vec<Box<dyn Middleware>>,
}
```

Default chain per role:

| Middleware | Engineer | Researcher | Reviewer |
|-----------|----------|-----------|----------|
| ContextBudget | yes | yes | yes |
| PlanningGate | yes | no | no |
| ToolFilter | yes (full) | yes (read+search) | yes (read-only) |
| Guardrails | yes | relaxed | strict |
| LoopDetection | yes | yes | yes |
| MessageQueue | yes | no | no |
| CostTracking | yes | yes | yes |
| Checkpoint | yes | no | no |
| SafetyNet | yes | no | no |

---

## What Exists vs. What To Build

### Already Built (v3.2)
- [x] Daemon + supervisor patrol loop
- [x] Task DAGs + missions + dependency inference
- [x] Expertise routing + preflight assessment
- [x] Memory (SQLite + FTS5 + embeddings)
- [x] Blackboard (ephemeral inter-agent knowledge)
- [x] Cost tracking + per-project budgets
- [x] Audit log + dispatch bus
- [x] ChatEngine (quick + full paths)
- [x] Web API (Axum + JWT + WebSocket)
- [x] Chat-first UI with context panel
- [x] Collapsible sidebar, command palette, breadcrumbs
- [x] Telegram gate
- [x] Claude Code worker execution
- [x] Cron jobs + watchdogs

### v4 Build Order

**Phase 1: Foundation (the middleware unlock)**
1. Middleware trait + chain in `sigil-orchestrator`
2. Extract existing AgentWorker logic into middleware:
   - ContextBudget (from current enrichment)
   - CostTracking (from current cost ledger)
   - Checkpoint (from current checkpoint logic)
3. Add new middleware: LoopDetection, Guardrails
4. Worker outcome: add DONE_WITH_CONCERNS, NEEDS_CONTEXT

**Phase 2: Verification & Safety**
5. Verification pipeline after worker completion
6. Three-strikes escalation in supervisor
7. Safety-net artifact preservation on failure
8. Git worktree isolation for parallel workers

**Phase 3: Memory Evolution**
9. Memory deduplication (vector similarity + LLM judgment)
10. Hotness scoring (frequency + recency decay)
11. Intent-driven query planning (typed queries with priorities)
12. Two-phase session commit (fast ack, background extraction)

**Phase 4: Notes + Proactive**
13. Notes table + API endpoints
14. Notes panel in context panel UI
15. Directive detection (imperative pattern matching)
16. Morning brief via Telegram push
17. Note → task activation (explicit + fuzzy suggestion)

**Phase 5: Progressive Intelligence**
18. Skill promotion from recurring insights
19. Progressive tool loading (metadata → on-demand)
20. Suggestion engine ("you wrote X, should I start?")
21. Anomaly detection (cost spikes, error rate, performance drop)

---

## The Test

Sigil v4 is done when:

1. You write "launch the new pricing page" in your notes
2. Sigil decomposes it into tasks, routes to the right agents
3. Workers execute in isolated worktrees with middleware protection
4. Verification confirms the work is correct
5. Memory extracts and deduplicates what was learned
6. You wake up to a Telegram message: "Pricing page deployed. 3 tasks completed overnight."
7. Your note shows: ✓ launch the new pricing page

That is a sigil manifesting into reality.
That is the product.
