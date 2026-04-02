# Sigil v4 — The Architecture

One system. One loop. Everything connects.

---

## Status

### Implementation Status by Phase

| Phase | Description | Status |
|-------|-------------|--------|
| Phase 1 | Middleware Foundation | IMPLEMENTED |
| Phase 2 | Verification & Safety | IMPLEMENTED |
| Phase 3 | Memory Graph | IMPLEMENTED |
| Phase 4 | Intelligent Retrieval | IMPLEMENTED |
| Phase 5 | Notes & Directives | IMPLEMENTED |
| Phase 6 | Proactive Engine | IMPLEMENTED |
| Phase 7 | Progressive Intelligence | IMPLEMENTED |
| Phase 8 | Scale & Polish | PLANNED |

### Middleware Status

8 middleware implementations exist and are wired into worker execution:

| Middleware | Status | File |
|-----------|--------|------|
| LoopDetection | IMPLEMENTED | `middleware/loop_detection.rs` |
| Guardrails | IMPLEMENTED | `middleware/guardrails.rs` |
| CostTracking | IMPLEMENTED | `middleware/cost_tracking.rs` |
| ContextCompression | IMPLEMENTED | `middleware/context_compression.rs` |
| ContextBudget | IMPLEMENTED | `middleware/context_budget.rs` |
| MemoryRefresh | IMPLEMENTED | `middleware/memory_refresh.rs` |
| Clarification | IMPLEMENTED | `middleware/clarification.rs` |
| SafetyNet | IMPLEMENTED | `middleware/safety_net.rs` |

### Not Yet Implemented

These items are described in the architecture below but do not exist in code:

- **PlanningGate** — worker must outline approach before executing
- **ToolFilter** — progressive tool loading, role-based tool permissions
- **MessageQueue** — injected messages between tool calls for mid-run course correction
- **DanglingToolPatch** — detect interrupted tool calls, inject synthetic error responses
- **Checkpoint-as-middleware** — checkpoint logic exists in workers but is not a standalone middleware implementation

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
Terminal (sigil)    ──┐
Chat (web)          ──┤
Telegram/WhatsApp   ──┤
Notes panel         ──┼──→ ChatEngine ──→ IntentStream
API                 ──┤
Cron/Watchdog       ──┤
Proactive engine    ──┘
```

Every input — human message, automated trigger, proactive suggestion — becomes
an `Intent`:

```rust
pub struct Intent {
    pub id: Ulid,
    pub source: IntentSource,         // Terminal, Chat, Telegram, Note, Cron, Proactive, Api
    pub channel: Channel,             // Project, Department, Global
    pub content: String,              // raw text
    pub urgency: Urgency,             // Immediate, Normal, Background
    pub author: Author,               // Human(name), Agent(name), System
    pub attachments: Vec<Attachment>,  // files, images, URLs
    pub parent_id: Option<Ulid>,      // threaded: reply to another intent
    pub created_at: DateTime<Utc>,
}
```

**Key insight from DeerFlow:** The clarification tool pattern. When the system
can't resolve intent, it doesn't guess — it asks. The `ask_clarification` intent
flows back through the same surface it came in on.

**Key insight from Open SWE:** Deterministic intent IDs from source. `SHA256("telegram:{chat_id}:{msg_id}")`
prevents duplicate task creation from webhook retries. Same event = same intent = idempotent.

### Layer 1: Understand

Turn raw intent into structured work.

```
IntentStream
    ↓
┌──────────────────────────────────┐
│ Intent Classifier                 │
│                                   │
│ Quick:                            │  Status, note, close, list → immediate response
│ Complex:                          │  Build, fix, research, deploy → decomposition
│ Ambiguous:                        │  Unclear scope → clarification request
│ Multi-intent:                     │  "fix the bot AND deploy pricing" → split
│                                   │
│ Classification uses:              │
│   - keyword patterns (fast path)  │
│   - LLM classification (fallback) │
│   - channel context (scoping)     │
└──────────────────────────────────┘
    ↓ (complex)
┌──────────────────────────────────┐
│ Decomposer                        │
│                                   │
│ Contracts:                        │
│   - Each task: ≤10 min scope      │
│   - Each task: verifiable DONE condition (assertion, not description)
│   - Each task: self-contained context (no shared state assumptions)
│   - Each task: explicit file paths, commands, expected outputs
│   - Each task: estimated cost (from similar past tasks)
│                                   │
│ Output: Task DAG with:            │
│   - dependency edges              │
│   - parallelism annotations       │
│   - rollback instructions per task│
│   - total estimated cost          │
│                                   │
│ Validation:                       │
│   - reject DAGs that exceed budget│
│   - reject single tasks > 10 min  │
│   - require at least one task with no dependencies (entry point)
└──────────────────────────────────┘
    ↓
┌──────────────────────────────────┐
│ Context Assembler                 │
│                                   │
│ Per task, generate typed queries  │ (from OpenViking's intent-driven planning)
│ with priorities:                  │
│                                   │
│   Priority 5: domain knowledge    │  "How does our pricing system work?"
│   Priority 4: recent decisions    │  "What did we decide about the API?"
│   Priority 3: system patterns     │  "How do we deploy to production?"
│   Priority 2: similar past tasks  │  "How did we handle this before?"
│   Priority 1: general context     │  "What is this project about?"
│                                   │
│ Merge results by priority into    │
│ a context bundle capped at ~200   │
│ lines (from Claude Code practices)│
│                                   │
│ Tag critical context with         │
│ <important> markers so it         │
│ survives attention decay in long  │
│ sessions.                         │
└──────────────────────────────────┘
```

**Key insight from Superpowers:** Task sizing matters. Tasks scoped at 2-10 min
have dramatically higher success rates than larger tasks. The decomposer enforces
this as a hard constraint, not a suggestion.

**Key insight from OpenViking:** Don't do one flat memory search. Generate multiple
typed queries with priorities. A trading task needs domain knowledge (high priority),
recent decisions (medium), and system patterns (low). Merge by relevance.

**Key insight from Claude Code practices:** Cap injected context at ~200 lines.
Prioritize: task description > relevant memory > blackboard > project context.
Beyond that, workers lose focus.

### Layer 2: Orchestrate

Route tasks to the right agent at the right time.

```
Task DAG
    ↓
┌──────────────────────────────────┐
│ WorkerPool                        │
│                                   │
│ 1. Expertise routing              │  Match task domain → agent expertise scores
│    - weighted by recency          │  Recent success on similar tasks scores higher
│    - penalized by recent failure  │  Failing agents deprioritized automatically
│                                   │
│ 2. Preflight assessment           │  LLM evaluates: can this agent handle this?
│    - structured JSON output       │  { can_handle: bool, confidence: f32, reason: str }
│    - runs OUTSIDE task lock       │  No blocking the patrol loop
│                                   │
│ 3. Resource check                 │  Budget available? Worker slot open? Rate limit ok?
│    - per-project budget gates     │  Project can't overspend even if global has room
│    - rate limit awareness         │  Back off if provider is throttled
│                                   │
│ 4. Dependency resolution          │  Predecessor tasks complete?
│    - failed dependency → cascade  │  Dependent tasks marked BLOCKED, not silently queued
│    - partial DAG execution        │  Independent branches start immediately
│                                   │
│ 5. Isolation setup                │  Git worktree for parallel execution on same repo
│    - auto-detect project type     │  Install deps, run baseline tests before work starts
│    - merge strategy: rebase       │  Clean commit history after parallel work
│                                   │
│ Safety policies:                  │
│   Three-strikes:   3 failures → escalate model or human
│   Recursive ban:   Workers cannot create tasks that create tasks
│   Priority queue:  Urgent work preempts background tasks
│   Cooldown:        Failed agent can't retry same domain for 30 min
│   Budget kill:     Task killed if it exceeds 3× estimated cost
└──────────────────────────────────┘
```

**Key insight from DeerFlow:** Hard limit on delegation depth. Workers cannot
recursively spawn sub-agents. The WorkerPool is the only entity that creates tasks.
This prevents cascade failures. DeerFlow also limits parallel sub-agents to 3-4
with a middleware clamping layer — Sigil should enforce similar limits.

**Key insight from Superpowers:** Git worktrees for parallel execution. When two
workers touch the same project, each gets an isolated worktree. Auto-setup
(detect project type, install deps, baseline tests) before work begins.

**Key insight from Open SWE:** Sandbox-per-task isolation. Even if not full Docker,
git worktrees provide filesystem isolation. The key is that parallel workers
never share mutable state.

### Layer 3: Execute

The middleware chain. This is where Sigil becomes composable.

```
Task assigned to Worker
    ↓
┌──────────────────────────────────────────────┐
│              Middleware Chain                  │
│                                               │
│  ┌─ Context Budget ────────────────────────┐ │
│  │  Cap enrichment at ~200 lines           │ │
│  │  Priority: task > memory > blackboard   │ │
│  │  Use <important> tags for critical rules│ │
│  │  Inject project AGENTS.md if exists     │ │
│  │  Inject matched skill SKILL.md content  │ │
│  └─────────────────────────────────────────┘ │
│  ┌─ Planning Gate ─────────────────────────┐ │
│  │  Worker must outline approach first     │ │
│  │  DONE condition stated as assertion     │ │
│  │  Verify plan is within task scope       │ │
│  │  Reject plans that exceed budget        │ │
│  └─────────────────────────────────────────┘ │
│  ┌─ Tool Filter ───────────────────────────┐ │
│  │  Progressive loading: metadata only     │ │  (from DeerFlow)
│  │  Full schemas loaded via tool_search    │ │
│  │  Role-based permissions:                │ │
│  │    Engineer: full access                │ │
│  │    Researcher: read + search only       │ │
│  │    Reviewer: read-only + comment        │ │
│  │  Project-specific .claude/settings.json │ │
│  └─────────────────────────────────────────┘ │
│  ┌─ Guardrails ────────────────────────────┐ │
│  │  Pre-execution filter on tool calls     │ │  (from DeerFlow)
│  │  Deny: rm -rf, force push, drop table   │ │
│  │  Deny: production deploys without flag  │ │
│  │  Structured denial message for recovery │ │
│  │  Fail-closed: unknown ops blocked       │ │
│  │  Configurable per project/agent role    │ │
│  └─────────────────────────────────────────┘ │
│  ┌─ Loop Detection ────────────────────────┐ │
│  │  MD5 hash tool calls in sliding window  │ │  (from DeerFlow)
│  │  Window size: 10 calls                  │ │
│  │  Warn at 3 repeats (inject warning msg) │ │
│  │  Hard kill at 5 repeats → FAILED        │ │
│  │  Thread-safe with LRU eviction          │ │
│  └─────────────────────────────────────────┘ │
│  ┌─ Message Queue ─────────────────────────┐ │
│  │  Check for injected messages between    │ │  (from Open SWE)
│  │  each tool call                         │ │
│  │  Enables mid-run course correction      │ │
│  │  Queue persisted in LangGraph store     │ │
│  │  FIFO, deduplicated per thread          │ │
│  └─────────────────────────────────────────┘ │
│  ┌─ Cost Tracking ─────────────────────────┐ │
│  │  Token/cost accumulation per task       │ │
│  │  Budget ceiling: kill if exceeded 3×    │ │
│  │  Real-time cost visible in status bar   │ │
│  │  Per-project and global budget gates    │ │
│  └─────────────────────────────────────────┘ │
│  ┌─ Checkpoint ────────────────────────────┐ │
│  │  Git commit at milestones               │ │
│  │  State snapshot: task progress + context│ │
│  │  Resume from last checkpoint on failure │ │
│  │  Checkpoint interval: every 3 tool calls│ │
│  │  or on significant file changes         │ │
│  └─────────────────────────────────────────┘ │
│  ┌─ Dangling Tool Patch ───────────────────┐ │
│  │  Detect interrupted tool calls          │ │  (from DeerFlow)
│  │  Inject synthetic error responses       │ │
│  │  Maintain conversation integrity        │ │
│  └─────────────────────────────────────────┘ │
│                                               │
│  ┌─ Core Agent Loop ───────────────────────┐ │
│  │  Claude Code subprocess                 │ │
│  │  --permission-mode auto                 │ │
│  │  Identity + memory + skill prompt       │ │
│  │  ReAct: think → tool → observe          │ │
│  │  Planning-first: outline before execute │ │
│  └─────────────────────────────────────────┘ │
│                                               │
│  ┌─ Safety Net ────────────────────────────┐ │
│  │  On failure: scan for artifacts         │ │  (from Open SWE)
│  │  Check: git diff, new files, partial    │ │
│  │  output, API responses collected        │ │
│  │  Preserve partial work before discard   │ │
│  │  Force-create PR if commits exist       │ │
│  └─────────────────────────────────────────┘ │
└──────────────────────────────────────────────┘
    ↓
Outcome: DONE | DONE_WITH_CONCERNS | BLOCKED | NEEDS_CONTEXT | HANDOFF | FAILED
         + confidence: f32 (0.0 - 1.0)
         + artifacts: Vec<Artifact>
         + cost_usd: f64
         + turns: u32
         + duration_ms: u64
```

Every middleware is a Rust trait:

```rust
/// Result of a middleware step.
pub enum MiddlewareAction {
    Continue,                    // proceed to next middleware
    Skip,                       // skip remaining middleware, proceed to core
    Halt(String),                // stop execution with reason
    Inject(Vec<Message>),        // add messages to context
    Transform(WorkerContext),     // replace context entirely
}

/// A composable behavior layer around the agent execution core.
#[async_trait]
pub trait Middleware: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn order(&self) -> u32;       // execution priority (lower = earlier)

    async fn on_start(&self, _ctx: &mut WorkerContext) -> MiddlewareAction {
        MiddlewareAction::Continue
    }
    async fn before_model(&self, _ctx: &mut WorkerContext) -> MiddlewareAction {
        MiddlewareAction::Continue
    }
    async fn after_model(&self, _ctx: &mut WorkerContext) -> MiddlewareAction {
        MiddlewareAction::Continue
    }
    async fn before_tool(&self, _ctx: &mut WorkerContext, _call: &ToolCall) -> MiddlewareAction {
        MiddlewareAction::Continue
    }
    async fn after_tool(&self, _ctx: &mut WorkerContext, _call: &ToolCall, _result: &ToolResult) -> MiddlewareAction {
        MiddlewareAction::Continue
    }
    async fn on_complete(&self, _ctx: &mut WorkerContext, _outcome: &Outcome) -> MiddlewareAction {
        MiddlewareAction::Continue
    }
    async fn on_error(&self, _ctx: &mut WorkerContext, _error: &anyhow::Error) -> MiddlewareAction {
        MiddlewareAction::Continue
    }
}

/// Ordered chain of middleware.
pub struct MiddlewareChain {
    layers: Vec<Box<dyn Middleware>>,
}

impl MiddlewareChain {
    pub fn new(mut layers: Vec<Box<dyn Middleware>>) -> Self {
        layers.sort_by_key(|m| m.order());
        Self { layers }
    }
}
```

Default chain (8 implemented middleware):

| Middleware | Status | Purpose |
|-----------|--------|---------|
| LoopDetection | IMPLEMENTED | MD5 hash sliding window (10 calls), warn at 3, kill at 5 |
| Guardrails | IMPLEMENTED | Block dangerous ops (rm -rf, force push, drop table) |
| CostTracking | IMPLEMENTED | Token/cost accumulation, budget ceiling (3x estimate) |
| ContextCompression | IMPLEMENTED | Compress at 50% window, protect first/last messages |
| ContextBudget | IMPLEMENTED | Cap enrichment at ~200 lines |
| MemoryRefresh | IMPLEMENTED | Re-search memory every N tool calls |
| Clarification | IMPLEMENTED | Structured questions to user, halts execution |
| SafetyNet | IMPLEMENTED | On failure: scan artifacts, preserve partial work |

Not yet implemented (described in architecture above but no code):
PlanningGate, ToolFilter, MessageQueue, DanglingToolPatch, Checkpoint-as-middleware

**Key insight from DeerFlow:** This is the architectural unlock. Every feature
becomes a plugin, not a patch. Loop detection, guardrails, memory, progressive
tool loading — all composable, all testable independently. DeerFlow proves this
with 13 middleware components in production.

### Layer 4: Verify

Don't trust self-reported outcomes.

```
Worker reports outcome
    ↓
┌──────────────────────────────────┐
│ Verification Pipeline             │
│                                   │
│ Stage 1: Artifact Check           │
│   Did the worker produce:         │
│   - git commits / diffs?          │
│   - new or modified files?        │
│   - API responses or output?      │
│   No artifacts + DONE = suspicious│
│                                   │
│ Stage 2: Automated Testing        │
│   If tests exist:                 │
│   - Run test suite                │
│   - Compare before/after          │
│   - Regression = reject           │
│   If no tests:                    │
│   - Check compilation/lint        │
│   - Type checking if applicable   │
│                                   │
│ Stage 3: Spec Compliance          │  (from Superpowers)
│   Separate reviewer agent checks: │
│   - Does output match DONE cond?  │
│   - Are all requirements met?     │
│   - Any scope creep?              │
│                                   │
│ Stage 4: Quality Review           │  (from Superpowers)
│   Second reviewer agent checks:   │
│   - Code quality                  │
│   - Security issues               │
│   - Performance concerns          │
│   - Documentation if needed       │
│                                   │
│ Stage 5: Confidence Scoring       │  (from GitNexus)
│   Aggregate all signals:          │
│     artifacts present     +0.2    │
│     tests pass            +0.3    │
│     spec compliant        +0.3    │
│     quality approved      +0.1    │
│     worker self-confidence+0.1    │
│                                   │
│   Total → confidence: f32         │
│                                   │
│ Routing:                          │
│   confidence ≥ 0.8 → auto-close   │
│   confidence 0.5-0.8 → close + flag for human review
│   confidence < 0.5 → reject, re-queue or escalate
│                                   │
│ Escalation policy:                │
│   1st rejection: same agent retry │
│   2nd rejection: different agent  │  (from Superpowers: three-strikes)
│   3rd rejection: escalate model   │
│   4th rejection: human required   │
└──────────────────────────────────┘
```

**Key insight from Superpowers:** Two-stage review (spec compliance + quality)
as separate automated checks. The reviewer runs automatically after every task,
not on manual request. Evidence-based: "tests pass" requires actual test output
showing 0 failures — agent claims alone are insufficient.

**Key insight from Superpowers:** Three fix attempts, then stop and question
whether the architecture is wrong rather than continuing to patch symptoms.

**Key insight from GitNexus:** Confidence metadata on outcomes. The worker pool
uses confidence to decide auto-close vs. human review. Confidence propagates
into the memory system — verified memories are more trustworthy.

### Layer 5: Learn

Every task makes the system smarter. This is the memory architecture.

```
Completed task + verification result
    ↓
┌──────────────────────────────────────────────────────────┐
│                    Learning Pipeline                      │
│                                                           │
│  ═══ PHASE 1: EXTRACTION (async, two-phase commit) ═══  │
│                                                           │
│  ┌─ Fast Acknowledgment ──────────────────────────────┐  │
│  │  Task outcome written to audit log immediately     │  │  (from OpenViking)
│  │  Worker freed for next task                        │  │
│  │  Memory extraction queued for background           │  │
│  └────────────────────────────────────────────────────┘  │
│  ┌─ Reflection Agent ─────────────────────────────────┐  │
│  │  Analyze: what worked, what failed, why            │  │
│  │  Extract candidates with structured categories:    │  │
│  │                                                    │  │
│  │  CASES:       specific problem + solution pair     │  │  (from OpenViking)
│  │  PATTERNS:    reusable process / method            │  │  (from OpenViking)
│  │  FACTS:       domain knowledge                     │  │
│  │  DECISIONS:   choice made + rationale              │  │
│  │  PREFERENCES: style, convention, approach          │  │
│  │  INSIGHTS:    non-obvious observation              │  │
│  │                                                    │  │
│  │  Each candidate includes:                          │  │
│  │    content, category, scope                        │  │
│  │    provenance: { agent, task_id, verified: bool }  │  │
│  │    confidence: f32 (from verification pipeline)    │  │
│  │    relationships: [ { target_key, relation_type } ]│  │
│  └────────────────────────────────────────────────────┘  │
│                                                           │
│  ┌─ Debounce Queue ───────────────────────────────────┐  │
│  │  Batch reflection writes                           │  │  (from DeerFlow)
│  │  Deduplicate per-project (newer replaces older)    │  │
│  │  Process after configurable debounce window (30s)  │  │
│  │  Prevents redundant LLM calls on rapid task chains │  │
│  └────────────────────────────────────────────────────┘  │
│                                                           │
│  ═══ PHASE 2: DEDUPLICATION ═══                          │
│                                                           │
│  ┌─ Vector Pre-Filter ────────────────────────────────┐  │
│  │  Generate embeddings for each candidate            │  │  (from OpenViking)
│  │  Find top-5 similar existing memories              │  │
│  │  Similarity threshold: 0.85                        │  │
│  └────────────────────────────────────────────────────┘  │
│  ┌─ LLM Judgment ─────────────────────────────────────┐  │
│  │  For each candidate vs. similar existing:          │  │
│  │                                                    │  │
│  │  SKIP:   candidate is duplicate, discard           │  │
│  │  CREATE: candidate is novel, store as new          │  │
│  │  MERGE:  candidate enhances existing, consolidate  │  │
│  │  SUPERSEDE: candidate contradicts existing,        │  │
│  │            replace old with new + decay old hotness│  │
│  │                                                    │  │
│  │  Batch-internal dedup prevents creating duplicates │  │
│  │  within the same extraction batch                  │  │
│  └────────────────────────────────────────────────────┘  │
│                                                           │
│  ═══ PHASE 3: STORAGE ═══                                │
│                                                           │
│  ┌─ Memory Graph ─────────────────────────────────────┐  │
│  │                                                    │  │
│  │  Nodes (memories):                                 │  │
│  │    id: Ulid                                        │  │
│  │    key: String                                     │  │
│  │    content: String                                 │  │
│  │    category: Case|Pattern|Fact|Decision|Pref|Insight│ │
│  │    scope: Entity|Domain|System                     │  │
│  │    confidence: f32 (0.0-1.0)                       │  │
│  │    provenance: Provenance { agent, task_id, verified }│
│  │    hotness: f32 (computed)                         │  │
│  │    access_count: u32                               │  │
│  │    created_at: DateTime                            │  │
│  │    last_accessed_at: DateTime                      │  │
│  │    embedding: Vec<f32>                             │  │
│  │                                                    │  │
│  │  Edges (relationships):                            │  │
│  │    source_id → target_id                           │  │
│  │    relation: CausedBy | Contradicts | Supports |   │  │
│  │              DerivedFrom | Supersedes | RelatedTo  │  │
│  │    strength: f32 (0.0-1.0)                         │  │
│  │    created_at: DateTime                            │  │
│  │                                                    │  │
│  │  SQLite schema:                                    │  │
│  │    memories (id, key, content, category, scope,    │  │
│  │             confidence, agent, task_id, verified,  │  │
│  │             access_count, created_at,              │  │
│  │             last_accessed_at, session_id)          │  │
│  │    memory_fts (FTS5 on key + content)              │  │
│  │    memory_embeddings (id, embedding BLOB)          │  │
│  │    memory_edges (source_id, target_id, relation,   │  │
│  │                  strength, created_at)             │  │
│  └────────────────────────────────────────────────────┘  │
│                                                           │
│  ┌─ Hierarchical Summaries ───────────────────────────┐  │
│  │                                                    │  │  (from OpenViking)
│  │  Memories organized into logical directories:      │  │
│  │    {project}/domain/                               │  │
│  │    {project}/system/                               │  │
│  │    {project}/decisions/                            │  │
│  │    {project}/cases/                                │  │
│  │    {project}/patterns/                             │  │
│  │                                                    │  │
│  │  Each directory maintains:                         │  │
│  │    L0: one-sentence abstract (auto-generated)      │  │
│  │    L1: paragraph overview (auto-generated)         │  │
│  │    L2: full memory content                         │  │
│  │                                                    │  │
│  │  L0/L1 regenerated when directory changes          │  │
│  │  Search navigates L0 → L1 → L2 (tree, not flat)   │  │
│  └────────────────────────────────────────────────────┘  │
│                                                           │
│  ┌─ Hotness Scoring ──────────────────────────────────┐  │
│  │                                                    │  │  (from OpenViking)
│  │  frequency_score = sigmoid(log1p(access_count))    │  │
│  │  recency_score = exp(-λ × days_since_access)       │  │
│  │    λ = ln(2) / 7  (7-day half-life)                │  │
│  │                                                    │  │
│  │  hotness = 0.6 × frequency + 0.4 × recency        │  │
│  │                                                    │  │
│  │  On contradiction:                                 │  │
│  │    contradicted memory hotness *= 0.3              │  │
│  │    (rapid decay when superseded)                   │  │
│  │                                                    │  │
│  │  On access:                                        │  │
│  │    access_count += 1                               │  │
│  │    last_accessed_at = now()                         │  │
│  │    (naturally promotes useful memories)             │  │
│  └────────────────────────────────────────────────────┘  │
│                                                           │
│  ┌─ Contradiction Detection ──────────────────────────┐  │
│  │                                                    │  │
│  │  On every MERGE or CREATE:                         │  │
│  │    1. Find memories with high similarity but       │  │
│  │       opposing sentiment/content                   │  │
│  │    2. LLM judgment: do these contradict?           │  │
│  │    3. If yes:                                      │  │
│  │       - newer memory gets Supersedes edge          │  │
│  │       - older memory hotness decayed               │  │
│  │       - flag for human review if both high-conf    │  │
│  │    4. Temporal context preserved:                  │  │
│  │       "Before migration: MySQL. After: PostgreSQL" │  │
│  │       (both are true at different times)           │  │
│  └────────────────────────────────────────────────────┘  │
│                                                           │
│  ┌─ Cross-Project Federation ─────────────────────────┐  │
│  │                                                    │  │
│  │  System-scope memories queryable across projects   │  │
│  │  Query: search local project first, then federate  │  │
│  │  to other projects for system-scope matches        │  │
│  │                                                    │  │
│  │  Use case: "How did we solve auth in project A?"   │  │
│  │  → finds auth decision from project A's memory     │  │
│  │  → injects into project B's worker context         │  │
│  │                                                    │  │
│  │  Federation is read-only (no cross-project writes) │  │
│  │  Results carry source_project metadata             │  │
│  └────────────────────────────────────────────────────┘  │
│                                                           │
│  ═══ PHASE 4: EVOLUTION ═══                              │
│                                                           │
│  ┌─ Skill Promotion ─────────────────────────────────┐  │
│  │                                                    │  │  (from Claude Code practices)
│  │  Monitor pattern category memories                 │  │
│  │  When 3+ similar patterns cluster:                 │  │
│  │    1. Propose formal skill definition              │  │
│  │    2. Generate SKILL.md with:                      │  │
│  │       - when to use (trigger conditions)           │  │
│  │       - step-by-step workflow                      │  │
│  │       - verification criteria                      │  │
│  │       - estimated cost and duration                │  │
│  │    3. Human approval to promote                    │  │
│  │    4. Skill injected into future worker context    │  │
│  │       when task matches trigger conditions         │  │
│  │                                                    │  │
│  │  Skills are living: updated when new patterns      │  │
│  │  improve on existing skill steps                   │  │
│  └────────────────────────────────────────────────────┘  │
│                                                           │
│  ┌─ Expertise Calibration ────────────────────────────┐  │
│  │                                                    │  │
│  │  Per agent, per domain:                            │  │
│  │    score = f(successes, failures, recency)         │  │
│  │    confidence = f(sample_size, consistency)         │  │
│  │                                                    │  │
│  │  On verified DONE:                                 │  │
│  │    agent expertise score ↑ for task domain         │  │
│  │    confidence ↑ (more data)                        │  │
│  │                                                    │  │
│  │  On FAILED after verification:                     │  │
│  │    agent expertise score ↓ for task domain         │  │
│  │    cooldown period on that domain                  │  │
│  │                                                    │  │
│  │  Informs Layer 2 routing decisions                 │  │
│  └────────────────────────────────────────────────────┘  │
│                                                           │
│  ┌─ Memory Lifecycle ─────────────────────────────────┐  │
│  │                                                    │  │
│  │  Pruning (monthly):                                │  │
│  │    hotness < 0.05 AND age > 90 days → archive      │  │
│  │    archived memories queryable but deprioritized   │  │
│  │                                                    │  │
│  │  Compaction (weekly):                              │  │
│  │    Cluster similar low-hotness memories            │  │
│  │    Merge into consolidated entries                 │  │
│  │    Preserve provenance chain                       │  │
│  │                                                    │  │
│  │  Audit (always):                                   │  │
│  │    Every write logged with timestamp + trigger     │  │
│  │    Every delete preserves tombstone for 30 days    │  │
│  │    Full memory history reconstructible             │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

### Memory Retrieval (the read path)

```
Query arrives (from Context Assembler or direct search)
    ↓
┌──────────────────────────────────┐
│ Retrieval Pipeline                │
│                                   │
│ 1. Intent Analysis                │  (from OpenViking)
│    Generate typed queries:        │
│    [domain:5, decisions:4, ...]   │
│                                   │
│ 2. Hierarchical Navigation        │  (from OpenViking)
│    Search L0 abstracts globally   │
│    → priority queue of dirs       │
│    → descend into promising dirs  │
│    → L1 overview narrows further  │
│    → L2 full content as results   │
│                                   │
│ 3. Hybrid Scoring                 │
│    BM25 keyword score (FTS5)      │  (existing)
│    + vector cosine similarity     │  (existing)
│    + hotness blending (0.2 wt)    │  (from OpenViking)
│    + confidence weighting         │  (new)
│    + graph boost (connected to    │
│      other high-relevance nodes)  │  (new)
│                                   │
│    final = 0.35 × BM25            │
│          + 0.35 × vector          │
│          + 0.10 × hotness         │
│          + 0.10 × confidence      │
│          + 0.10 × graph_boost     │
│                                   │
│ 4. Contradiction Filtering        │
│    If result A supersedes B:      │
│    drop B unless temporal query   │
│                                   │
│ 5. Cross-Project Federation       │
│    If local results insufficient: │
│    query system-scope memories    │
│    across other project DBs       │
│                                   │
│ 6. Temporal Scoping               │
│    "What did we know before X?"   │
│    Filter by created_at < X       │
│    Useful for debugging decisions │
│                                   │
│ 7. Result Assembly                │
│    Top-K results with metadata:   │
│    { content, source, confidence, │
│      hotness, provenance, related }│
└──────────────────────────────────┘
```

### Layer 6: Proact

The layer that makes Sigil a CEO, not a tool.

```
┌──────────────────────────────────────────────────────────┐
│                    Proactive Engine                        │
│                                                           │
│  ═══ CONTINUOUS (every patrol cycle, ~5 min) ═══         │
│                                                           │
│  ┌─ Health Monitor ───────────────────────────────────┐  │
│  │  Stale tasks: no progress in > 1 hour → nudge     │  │
│  │  Stuck workers: loop detection triggered → kill    │  │
│  │  Budget drift: spending > 2× daily average → alert │  │
│  │  Missed crons: expected run didn't happen → report │  │
│  │  Dead dispatches: undelivered messages → retry     │  │
│  │  Memory staleness: project changed, memory stale   │  │  (from GitNexus)
│  └────────────────────────────────────────────────────┘  │
│                                                           │
│  ┌─ Anomaly Detection ────────────────────────────────┐  │
│  │  Statistical baselines per project:                │  │
│  │    - avg tasks/day, avg cost/task, avg duration    │  │
│  │    - failure rate, escalation rate                 │  │
│  │                                                    │  │
│  │  Detect:                                           │  │
│  │    Cost spike: > 3σ from baseline → alert          │  │
│  │    Error surge: failure rate > 2× baseline → pause │  │
│  │    Performance drop: duration > 2× baseline → flag │  │
│  │    Agent degradation: expertise score dropping     │  │
│  │                                                    │  │
│  │  Response:                                         │  │
│  │    Info: log + dashboard indicator                  │  │
│  │    Warning: push notification to human              │  │
│  │    Critical: pause project execution, await human   │  │
│  └────────────────────────────────────────────────────┘  │
│                                                           │
│  ═══ SCHEDULED (configurable) ═══                        │
│                                                           │
│  ┌─ Morning Brief ────────────────────────────────────┐  │
│  │  Daily at configured time (default 08:00 local)    │  │
│  │                                                    │  │
│  │  Content:                                          │  │
│  │    "Good morning. Here's what happened overnight." │  │
│  │    - Tasks completed (count, names, projects)      │  │
│  │    - Tasks blocked (with reasons)                  │  │
│  │    - Decisions needed (with context)               │  │
│  │    - Cost summary (spent, remaining, trend)        │  │
│  │    - Note status changes (what manifested)         │  │
│  │    - Anomalies detected                            │  │
│  │                                                    │  │
│  │  Delivery:                                         │  │
│  │    1. Web dashboard (always)                       │  │
│  │    2. Telegram message (if configured)             │  │
│  │    3. Email digest (if configured)                 │  │
│  │    4. WhatsApp (future)                            │  │
│  │                                                    │  │
│  │  Format: structured, scannable, actionable         │  │
│  │  Each item has: [Acknowledge] [Act] [Dismiss]      │  │
│  └────────────────────────────────────────────────────┘  │
│                                                           │
│  ┌─ Note Watcher ─────────────────────────────────────┐  │
│  │  Scan notes for new/changed directives             │  │
│  │  Match against existing tasks (fuzzy embedding)    │  │
│  │  Suggest task creation for unmatched directives    │  │
│  │  Update note status when tasks complete            │  │
│  │  Push: "Your note 'X' is now complete"             │  │
│  └────────────────────────────────────────────────────┘  │
│                                                           │
│  ═══ REACTIVE (event-driven) ═══                         │
│                                                           │
│  ┌─ Completion Notifier ──────────────────────────────┐  │
│  │  Task DONE → push to originating surface           │  │
│  │  "Task as-3021 complete: fixed trading bot param"  │  │
│  │  Include: summary, artifacts, cost, duration       │  │
│  └────────────────────────────────────────────────────┘  │
│                                                           │
│  ┌─ Suggestion Engine ────────────────────────────────┐  │
│  │  Triggered by: new note, completed task, new       │  │
│  │  knowledge, pattern detection                      │  │
│  │                                                    │  │
│  │  "You wrote 'optimize API latency' — want me to   │  │
│  │   start? I found 3 similar tasks we did before."   │  │
│  │                                                    │  │
│  │  "Task X completed. Based on the pattern, you      │  │
│  │   usually follow this with Y. Should I queue it?" │  │
│  │                                                    │  │
│  │  "I noticed a recurring deployment failure at 3am. │  │
│  │   Want me to create a watchdog for this?"          │  │
│  │                                                    │  │
│  │  Suggestions are non-blocking:                     │  │
│  │    [Accept] [Modify] [Dismiss]                     │  │
│  │  Dismissed patterns deprioritized for future       │  │
│  └────────────────────────────────────────────────────┘  │
│                                                           │
│  Output → new Intents (back to Layer 0)                  │
└──────────────────────────────────────────────────────────┘
```

---

## The Notes System

Notes are the second surface of Layer 0. Chat is ephemeral conversation.
Notes are persistent will.

```
Chat:  "fix the trading bot"          → executes now, fades into history
Notes: "Q2: pricing, bot, 100 users"  → persists, tracks, updates, manifests
```

Both feed into the same IntentStream.

### Storage

Per-project SQLite in `.sigil/notes.db`:

```sql
CREATE TABLE notes (
    id TEXT PRIMARY KEY,             -- ulid
    channel TEXT NOT NULL,           -- project or department scope
    content TEXT NOT NULL,           -- markdown
    version INTEGER DEFAULT 1,       -- optimistic concurrency
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE note_directives (
    id TEXT PRIMARY KEY,             -- ulid
    note_id TEXT NOT NULL REFERENCES notes(id),
    line_number INTEGER NOT NULL,    -- which line in the note
    content TEXT NOT NULL,           -- the directive text
    status TEXT DEFAULT 'pending',   -- pending, active, done, failed
    task_id TEXT,                    -- linked task (explicit activation)
    matched_task_id TEXT,            -- fuzzy-matched task (suggestion)
    confidence REAL DEFAULT 0.0,     -- match confidence
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

### Directive Detection

Pattern matching on note lines:

```
Imperative verbs:    build, fix, deploy, create, launch, redesign, research, migrate, ...
Goal patterns:       "X by <date>", "achieve X", "reach X", "<number> users"
Task-like:           "[ ] something" (checkbox syntax)
```

Each detected directive gets a status indicator in the UI:

```
Q2 Goals:
  ○  launch pricing page              (no matching task)
  ⟳  fix trading bot overshoot        (matched to as-3021, in progress)
  ✓  migrate auth to JWT              (matched to sg-1844, completed)
  ✗  100 users by April               (matched to mission, behind schedule)
```

### UI

Context panel evolves to three tabs:

```
┌─────────────────────────────────┐
│  Notes │ Context │ Brief        │
├─────────────────────────────────┤
│                                 │
│  Q2 Goals:                      │
│  ○ launch pricing page    [▸]   │  ← click [▸] to activate as task
│  ⟳ fix trading bot              │  ← auto-matched, shows progress
│  ✓ migrate auth to JWT          │  ← completed, green
│                                 │
│  Infrastructure:                │
│  ○ set up monitoring            │
│  ○ CDN for static assets        │
│                                 │
│  ─────────────────────────────  │
│  [just type here to add notes]  │
│                                 │
└─────────────────────────────────┘
```

### Cross-Surface Sync

```
Web notes panel  ←→  Telegram "note: ..."  ←→  Terminal "sigil note ..."  ←→  API
```

Write in any surface, visible everywhere. Agents see notes during execution
(injected into worker context when relevant to the task's project/channel).

---

## The Terminal Experience

`sigil` with no arguments = interactive chat connected to the full orchestration plane.

```
$ sigil

  sigil v4.0 · 6 projects · 4 agents · daemon online
  $4.23 / $100 today · 2 tasks active

  sigil > fix the trading bot parameter overshoot

  Decomposing... 2 tasks created:
    as-3025  analyze recent PnL for parameter sensitivity
    as-3026  adjust overshoot threshold based on analysis

  as-3025 assigned to trader (expertise: 0.87)
  as-3025 executing...

  sigil > status

  Active:
    as-3025  analyze PnL sensitivity   trader   ⟳ 2m elapsed
    sg-1901  deploy pricing page       engineer ⟳ 8m elapsed

  Blocked:
    as-3026  adjust threshold          (waiting on as-3025)

  sigil >
```

Implementation: `sigil-cli/src/cmd/chat.rs` connects to daemon IPC,
sends intents through the same ChatEngine as web/Telegram, renders
results with terminal colors/formatting.

---

## Build Phases

### Phase 1: Middleware Foundation
1. `Middleware` trait + `MiddlewareChain` in `sigil-orchestrator`
2. Extract existing worker logic into middleware: ContextBudget, CostTracking, Checkpoint
3. New middleware: LoopDetection, Guardrails, DanglingToolPatch
4. Worker outcomes: add DONE_WITH_CONCERNS, NEEDS_CONTEXT + confidence field
5. `sigil` interactive terminal chat (IPC to daemon ChatEngine)

### Phase 2: Verification & Safety
6. Verification pipeline: artifact check, test runner, spec compliance
7. Confidence scoring: aggregate signals into f32
8. Three-strikes escalation in worker pool
9. Safety-net artifact preservation on failure
10. Git worktree isolation for parallel workers on same repo

### Phase 3: Memory Graph
11. Schema migration: add confidence, provenance, access_count, last_accessed_at
12. Memory edges table: relationships between memories
13. Deduplication pipeline: vector pre-filter + LLM judgment (SKIP/CREATE/MERGE/SUPERSEDE)
14. Contradiction detection: flag or supersede conflicting memories
15. Hotness scoring: frequency × sigmoid + recency × exp decay
16. Hierarchical summaries: L0/L1/L2 per memory directory

### Phase 4: Intelligent Retrieval
17. Intent-driven query planning: typed queries with priorities
18. Hierarchical navigation: L0 → L1 → L2 tree search
19. Confidence-weighted ranking: verified memories score higher
20. Graph boost: connected memories get relevance lift
21. Cross-project federation: system-scope search across projects
22. Temporal scoping: "what did we know before X?"

### Phase 5: Notes & Directives
23. Notes storage: per-project SQLite with note_directives table
24. Notes API: CRUD endpoints via daemon IPC + web API
25. Notes UI: editable panel with directive status indicators
26. Directive detection: imperative pattern matching on note lines
27. Note → task activation: explicit (click) + fuzzy suggestion
28. Cross-surface sync: web ↔ Telegram ↔ terminal ↔ API

### Phase 6: Proactive Engine
29. Morning brief generation: structured summary of overnight work
30. Push delivery: Telegram, email, web notification
31. Completion notifier: push to originating surface on task DONE
32. Note watcher: scan for new directives, update status on completion
33. Suggestion engine: "you wrote X, should I start?"
34. Anomaly detection: cost spikes, error surges, performance drops

### Phase 7: Progressive Intelligence
35. Skill promotion: 3+ similar patterns → candidate SKILL.md
36. Progressive tool loading: metadata listing + on-demand schema
37. Memory lifecycle: pruning (90d cold), compaction (weekly), audit trail
38. Debounced reflection: batch + dedup before memory writes
39. Two-phase session commit: fast ack, background extraction
40. Memory compaction: cluster and consolidate low-hotness entries

### Phase 8: Scale & Polish
41. Hosted version: isolated daemon per user, zero-setup onboarding
42. Landing page: sigil.ceo with brand, signup, pricing
43. WhatsApp integration
44. Mobile PWA
45. API access for developers
46. Rate limiting and fair scheduling for multi-tenant

---

## The Test

Sigil v4 is done when:

1. You write "launch the new pricing page" in your notes on your phone
2. It appears in Sigil instantly across all surfaces
3. Sigil decomposes it into tasks, routes to the right agents
4. Workers execute in isolated worktrees with middleware protection
5. Verification confirms the work — tests pass, spec met, quality checked
6. Memory extracts insights, deduplicates, links to related knowledge
7. You wake up to a Telegram message:
   "Morning. Pricing page deployed. 3 tasks completed. $2.14 spent.
    One thing needs your call: the hero copy has two options. [A] [B]"
8. You tap B
9. Your note shows: ✓ launch the new pricing page
10. The system is measurably smarter for the next task

That is a sigil manifesting into reality.
That is proactive AI.
That is the product.
