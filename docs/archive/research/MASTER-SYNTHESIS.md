# Sigil Master Synthesis

This document is archived research material. The source clones referenced here are kept outside the `sigil` repository.

The definitive plan for making Sigil the best agent orchestrator in existence.
Derived from deep code-level analysis of 11 repositories + Sigil itself.

---

## The Synthesis Process (Repeatable)

```
1. IDENTIFY    — What capability area to improve
2. CLONE       — Clone competitor repos locally
3. READ CODE   — Not READMEs. Actual source. Trace the full lifecycle.
4. COMPARE     — What does Sigil do? What do they do? Where's the gap?
5. SYNTHESIZE  — Design something SUPERIOR to both (not copy, not ignore)
6. IMPLEMENT   — Build it in Sigil
7. VERIFY      — Tests pass, clippy clean, deployed
8. DOCUMENT    — Update docs to match reality
9. REPEAT      — Next capability area
```

This process is itself a skill. It can be applied to any capability area.
The goal: excellence through informed synthesis, not imitation.

---

## Sources (11 Repos, All Cloned Locally)

| Repo | Path | Primary Strength |
|------|------|-----------------|
| bytedance/deer-flow | /home/claudedev/deer-flow | Middleware chain, live streaming, subagent execution |
| NousResearch/hermes-agent | /home/claudedev/hermes-agent | Runtime maturity, approval rails, context compression, toolsets |
| volcengine/OpenViking | /home/claudedev/synthesis-sources/openviking | Hierarchical retrieval, hotness scoring, LLM-based dedup, two-phase commit |
| supermemory | /home/claudedev/supermemory | Memory product (Profile API, MCP, graph viz, conversation ingestion) |
| obra/superpowers | /home/claudedev/synthesis-sources/superpowers | Phased skills, verification-before-completion, red flags, skill composition |
| affaan-m/everything-claude-code | /home/claudedev/synthesis-sources/everything-claude-code | Instinct evolution, compliance testing, continuous learning |
| alirezarezvani/claude-skills | /home/claudedev/synthesis-sources/claude-skills | 205 production skills, domain hierarchy, agent-as-orchestrator |
| shareAI-lab/learn-claude-code | /home/claudedev/synthesis-sources/learn-claude-code | Agent design philosophy, progressive complexity |
| langchain-ai/open-swe | /home/claudedev/synthesis-sources/open-swe | Sandbox isolation, mid-run injection, safety nets, AGENTS.md |
| abhigyanpatwari/GitNexus | /home/claudedev/synthesis-sources/gitnexus | Code knowledge graph, impact analysis, blast radius, MCP tools |
| lightpanda-io/browser | /home/claudedev/synthesis-sources/lightpanda | Headless browser for agents, semantic DOM, MCP interface |

---

## Sigil Current State (What We Have)

**Backend:** 9 crates, 466 tests, clippy clean
- 8 active middleware (loop detection, guardrails, cost tracking, context compression,
  context budget, memory refresh, clarification, safety net)
- Verification pipeline with confidence scoring (wired to supervisor)
- Escalation tracker with three-strikes (wired to supervisor)
- Memory graph with dedup, hotness, hierarchy, query planner (wired to workers)
- Notes with directive detection (wired to daemon patrol + web API)
- Proactive engine with anomaly detection (wired to patrol)
- Skill promotion from patterns (built, needs integration)
- Event broadcasting (daemon → supervisor → WebSocket)

**Frontend:** Chat-first UI, 3-tab context panel (Notes/Context/Brief),
layout switcher (Focus/Split/Stack), draggable divider, command palette

**What works end-to-end today:**
Chat → intent classification → task creation → expertise routing → worker execution
with 8 middleware → outcome parsing → verification → escalation → memory dedup →
event broadcasting → brief generation

---

## The 10 Capability Areas — Ranked by Impact

### TIER 1: CRITICAL (Must fix — system trust depends on it)

#### 1. RESILIENCE
**Gap:** Weak resumability, no context tier stepping, no partial completion
**From:** Hermes (context probing), Open SWE (sandbox isolation)
**Build:**
- Context tier stepping in ContextCompressionMiddleware on_error hook
  (200K → 128K → 64K → 32K, retry with compression)
- Full WorkerContext serialization at checkpoint (not just git commits)
- Partial completion status (distinct from FAILED, preservable)
- Provider fallback chain (Anthropic → OpenRouter → Ollama)
**Files:** middleware/context_compression.rs, agent_worker.rs, checkpoint.rs

#### 2. VERIFICATION
**Gap:** Heuristic signals, not real test execution; no compliance testing
**From:** Superpowers (evidence-required), ECC (skill-comply)
**Build:**
- Real test execution in check_tests() (shell out to cargo test / npm test)
- Evidence storage in audit log (test output, diffs, exit codes)
- Red flag patterns in skill definitions (halt on rationalization)
- Regression baseline tracking (test count before vs after)
**Files:** verification.rs, skill struct in sigil-tools/src/skill.rs

#### 3. CHAT/UX
**Gap:** No live subtask streaming, no approval UI, no interactive briefs
**From:** Deer Flow (streaming), Hermes (progressive editing)
**Build:**
- Live subtask cards in chat (task DAG as cards with status, agent, cost)
- Approval dialog: [Allow Once] [Allow Session] [Allow Always] [Deny]
- Clarification inline: question with typed options, tap to resume
- Progressive message editing for Telegram (edit same message)
**Files:** sigil-ui ChatPage.tsx, ContextPanel.tsx, new ApprovalDialog component

#### 4. MEMORY PRODUCT
**Gap:** Deep internals, invisible externally
**From:** Supermemory (Profile API, graph viz), OpenViking (hierarchical retrieval)
**Build:**
- GET /api/memory/profile?project=X → { static: [...], dynamic: [...] }
- Graph visualization API (nodes + edges + positions for D3)
- Conversation ingestion endpoint (accept threads → extract facts)
- Injection safety scanning before storage (Hermes pattern)
- Workers access memory via existing IPC + CLI (no MCP needed — Sigil is self-contained)
**Files:** new routes in sigil-web, memory profile logic in sigil-memory

### TIER 2: HIGH (Significant improvement, builds on Tier 1)

#### 5. TOOLS
**Gap:** is_tool_allowed() never called, no progressive loading
**From:** Hermes (toolset composition), Deer Flow (deferred loading)
**Build:**
- Wire is_tool_allowed() in supervisor before worker execution
- Tool registry with capability metadata (reads_fs, writes_fs, network, dangerous)
- Workers access Sigil via CLI (`sigil task create`, `sigil daemon query`)
- Progressive tool loading (ToolFilter middleware — metadata only, schemas on demand)
- NOTE on tool gating by execution mode:
  - Internal agent mode: `is_tool_allowed()` directly filters Rust Tool vec — real enforcement
  - Claude Code mode: tool restrictions injected as prompt instructions — advisory only
    (Claude Code manages its own tools; external filters are prompt-level, not enforced)
  - Both modes benefit from skill-level tool policy, but enforcement differs
- NOTE: MCP server is DEFERRED — only needed when external consumers exist.
  Sigil is self-contained: daemon ↔ IPC ↔ workers ↔ Claude Code.
**Files:** supervisor.rs, middleware/tool_filter.rs

#### 6. EXECUTION
**Gap:** 5 planned middleware not built
**From:** Deer Flow (13 middleware), Hermes (context compression error recovery)
**Build:**
- PlanningGate middleware (worker outlines approach before executing)
- MessageQueue middleware (inject messages between tool calls)
- DanglingToolPatch middleware (synthetic error for interrupted calls)
- Checkpoint-as-middleware (periodic state snapshots as middleware hook)
- NOTE: RuntimeBackend trait (Docker, SSH) is DEFERRED — all workers execute locally
  via Claude Code subprocess or internal agent loop. Environment abstraction adds
  complexity without current benefit. Revisit when hosted multi-tenant version ships.
**Files:** 4 new middleware files in middleware/

#### 7. IDENTITY & SKILLS
**Gap:** Skill schema incomplete, conditional activation not wired, tool gating unused
**From:** Superpowers (phases, verification), Hermes (conditional), ECC (instincts)
**Build:**
- Extended skill TOML schema (conditions, verification, phases, red_flags, composition)
- Conditional activation: check requires_tools + requires_expertise before injection
- Phased execution with gates (prevent code before design)
- Skill chaining (next_skill on completion, fallback_skill on failure)
- Project-scoped promotion (2+ projects to become global)
**Files:** sigil-tools/src/skill.rs, supervisor.rs, skill_promotion.rs

#### 8. LEARNING
**Gap:** No compliance measurement, no cross-project threshold, no instinct formation
**From:** ECC (skill-comply, continuous learning), Hermes (background review)
**Build:**
- Compliance testing: auto-generate behavioral sequences from skills, measure adherence
- Cross-project promotion threshold (pattern in 2+ projects → global skill)
- Background learning worker (holistic review after N task completions)
- Instinct formation (5+ times, >90% success → always-on rule)
**Files:** skill_promotion.rs, new compliance.rs

### TIER 3: MEDIUM (Polish on existing strength)

#### 9. PROACTIVE
**Gap:** Exists but no interactive surfaces
**Build:**
- Proposals with [Accept] [Modify] [Dismiss] in chat
- Interactive morning brief (each item has action buttons)
- Operator presence awareness (batch digest if absent > 8 hours)
- Pattern-driven suggestions (3+ similar task sequences → suggest pipeline)
**Files:** proactive.rs, sigil-ui ContextPanel

#### 10. ORCHESTRATION
**Gap:** Already leads; needs polish
**Build:**
- Parallel worker clamping (max concurrent per project)
- DAG visualization in dashboard (live task graph with status per node)
- Cost-based scheduling (use history to estimate and batch)
- Delegation summarization (structured summary, not full reasoning)
**Files:** supervisor.rs, sigil-ui new DAGView component

---

## Implementation Phases

### Phase A: Trust Foundation (Weeks 1-2)
Items: Resilience (#1) + Verification (#2)
- Context tier stepping in middleware
- Real test execution in verification
- Evidence storage in audit
- Checkpoint serialization
- Provider fallback chain

### Phase B: Visible Alive (Weeks 3-4)
Items: Chat/UX (#3) + Memory Product (#4)
- Live subtask cards in frontend
- Approval dialog component
- Profile API + MCP server
- Graph visualization endpoint

### Phase C: Tool & Execution Depth (Weeks 5-6)
Items: Tools (#5) + Execution (#6)
- Wire is_tool_allowed()
- MCP server for Sigil capabilities
- RuntimeBackend trait
- 4 missing middleware

### Phase D: Intelligence Evolution (Weeks 7-8)
Items: Identity (#7) + Learning (#8)
- Extended skill schema
- Conditional activation
- Compliance testing
- Project-scoped promotion

### Phase E: Polish (Weeks 9-10)
Items: Proactive (#9) + Orchestration (#10)
- Interactive briefs
- DAG visualization
- Operator presence awareness

---

## The Ambition

Sigil aims to be excellent in ways that are KNOWN (best practices from 11 repos)
AND in ways that are UNKNOWN (synthesis that creates something new).

The known: middleware chains, verification pipelines, memory graphs, skill systems.
Every competitor has some of these. Sigil has all of them, wired together.

The unknown: notes that reshape the system's identity. Proactive action proposals.
Skill promotion from execution patterns. Memory that reasons about what it knows.
No competitor has these. Sigil does.

The synthesis process ensures we always learn from the frontier while pushing
beyond it. Every external repo is an input, never a ceiling.

This document is the reference. Update it as we build.
