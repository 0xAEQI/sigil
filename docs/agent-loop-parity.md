# Agent Loop Parity: AEQI vs Claude Code

## Goal

Make AEQI's agent loop (`aeqi-core/src/agent.rs`) at least as resilient, performant, and polished as Claude Code's (`refs/claude-code/src/query.ts`). The loop is the core product — everything else is orchestration on top.

## Status After This Session (2026-04-04)

### What Was Done Today (13 commits, 228 files)

**Agent loop:**
- True streaming tool execution (tools start during LLM stream via ToolUseComplete)
- All 13 ChatStreamEvent types emitted and forwarded to frontend
- Tool input_preview in ToolComplete (shows what was called with)
- Diminishing returns threshold relaxed (50 tok × 5 turns, was 500 × 3)
- Status events before compaction and memory recall
- DelegateStart/DelegateComplete event emission

**Data model:**
- Projects have UUIDs (auto-generated, persisted)
- Three-tier memory (agent → department → project with hierarchical_search)
- Sessions unified — one table, parent-child linked
- Everything creates a session (workers, delegates, triggers)
- SessionManager.spawn_session() as universal executor
- SpawnOptions builder (flat params, skills, auto_close)

**Architecture:**
- ChatEngine → MessageRouter
- UnifiedDelegateTool → DelegateTool (direct spawn, no dispatch bus)
- unified_delegate.rs → delegate.rs
- chat_ws.rs → session_ws.rs
- .company → .project on all DB structs
- Skill injection at spawn time
- architecture-audit skill
- Full web UI with interleaved segments, tool panels, session sidebar

### What AEQI's Agent Loop Does Well
- Streaming tool execution (ToolUseComplete → executor starts during stream)
- Context management: snip, microcompact, full compact, reactive compact
- Three-tier memory recall (hierarchical_search)
- Perpetual sessions (stays open, accepts follow-up messages)
- Token budget auto-continuation
- Output truncation recovery (MaxTokens → auto-continue)
- Fallback model switching on consecutive failures
- File change detection between turns
- Mid-loop memory recall
- Session memory extraction (fire-and-forget background task)
- Budget pressure injection into tool results (70% and 90% warnings)
- Diminishing returns detection

### Where Claude Code's Loop is Better

## Research Plan

### Phase 1: Deep Read Claude Code (query.ts ecosystem)

Read these files END TO END, not grep:

1. **`refs/claude-code/src/query.ts`** (~1729 lines)
   - The main agent loop. Every state transition, every recovery path.
   - Focus on: how `State` object carries context between iterations, the 7 continue sites, what `transition.reason` tracks
   - Map every error recovery strategy: context collapse drain, reactive compact, max output tokens retry, streaming fallback, model fallback

2. **`refs/claude-code/src/services/tools/StreamingToolExecutor.ts`**
   - How tools start during streaming (not after)
   - Concurrency model: read-safe vs exclusive
   - Bash error cascading vs non-bash independence
   - Progress message yielding
   - Discard pattern on streaming fallback

3. **`refs/claude-code/src/services/tools/toolOrchestration.ts`**
   - Batch execution (legacy path)
   - Partition logic: consecutive read-only → batch, non-read-only → sequential
   - Context modifier queuing during concurrent batches

4. **`refs/claude-code/src/services/compact/autoCompact.ts`**
   - Threshold calculation (effective window - 13K buffer)
   - Proactive vs reactive trigger
   - Circuit breaker (3 consecutive failures → stop)
   - Post-compact: file restoration, skill re-injection

5. **`refs/claude-code/src/services/compact/microCompact.ts`**
   - Time-based and token-based clearing
   - Which tool results are eligible
   - Prompt caching integration (cache_edits)

6. **`refs/claude-code/src/services/compact/contextCollapse.ts`** (if exists)
   - Persistent commit log across turns
   - Committed vs uncommitted blocks
   - How it survives across messages in long sessions

7. **`refs/claude-code/src/query/stopHooks.ts`**
   - Post-tool-batch validation
   - Can block continuation
   - Fire-and-forget vs blocking hooks
   - Hook types and when they run

8. **Error withholding pattern** (in query.ts)
   - Recoverable errors (prompt_too_long, max_output_tokens, media size)
   - Withheld from UI until recovery attempted
   - How withheld messages are managed if recovery fails

9. **Streaming fallback atomicity** (in query.ts)
   - When model fallback occurs mid-stream
   - Tombstoning orphaned partial messages
   - Buffer clearing (assistantMessages, toolResults, toolUseBlocks)
   - Executor discard + recreation

10. **Token budget system** (in query.ts)
    - Server-side task_budget parameter
    - Carryover across compactions
    - Nudge messages at threshold

### Phase 2: Deep Read AEQI (agent.rs ecosystem)

Read these files END TO END:

1. **`crates/aeqi-core/src/agent.rs`** (~3100 lines)
   - The full `run()` method and main loop
   - Every `LoopTransition` variant and what triggers each
   - Error handling: context-length, fallback model, observer on_error
   - `try_streaming_with_tools()` — the streaming call + tool execution
   - `call_streaming_with_tools()` — retry wrapper
   - Context management: snip_compact, microcompact, compact_messages
   - Result processing: observer hooks, persist/truncate, budget injection
   - Mid-loop memory recall
   - File change detection
   - Session memory extraction
   - Output truncation recovery
   - Token budget auto-continuation
   - Perpetual mode input waiting
   - format_tool_input() for input_preview

2. **`crates/aeqi-core/src/streaming_executor.rs`** (~500 lines)
   - StreamingToolExecutor queue management
   - Concurrent-safe checks
   - Sibling error signaling
   - Duration tracking
   - Discard + abort

3. **`crates/aeqi-core/src/chat_stream.rs`**
   - All 13 ChatStreamEvent variants
   - ChatStreamSender broadcast mechanics

4. **`crates/aeqi-core/src/traits/`**
   - Provider trait (chat + chat_stream)
   - Observer trait (before_model, after_model, before_tool, after_tool, on_error, etc.)
   - Memory trait (store, search, hierarchical_search)
   - Tool trait (execute, is_concurrent_safe)

### Phase 3: Side-by-Side Comparison

For each of these concerns, document CC's exact mechanism, AEQI's exact mechanism, the gap, and the concrete fix:

1. **Streaming tool execution during LLM response**
   - CC: Tools start as `content_block_stop` fires for each tool_use block
   - AEQI: Tools start on ToolUseComplete (same timing after today's work?)
   - Gap: ?

2. **Error recovery: prompt too long**
   - CC: Withhold error → context collapse drain → reactive compact → fail
   - AEQI: Reactive compact → retry. No withholding, no collapse drain.
   - Gap: No error withholding, no context collapse persistent log

3. **Error recovery: max output tokens**
   - CC: Withhold → reduce max_tokens by 50% → retry (circuit breaker at 3)
   - AEQI: Auto-continue with "Continue executing" prompt (max 3)
   - Gap: Different strategy. CC truncates, AEQI continues. Which is better?

4. **Error recovery: streaming fallback**
   - CC: Tombstone orphans, clear buffers atomically, discard executor, retry with fallback model
   - AEQI: Fallback model switch on 3 consecutive failures
   - Gap: No atomic retry with buffer clearing. No tombstoning.

5. **Context compaction: levels and triggers**
   - CC: Microcompact → snip → context collapse → auto-compact → reactive
   - AEQI: Snip → microcompact → full compact → reactive
   - Gap: No persistent context collapse log. Different ordering.

6. **Context compaction: post-compact restoration**
   - CC: Re-inject files + skills after compaction
   - AEQI: Re-inject recent files via recent_files tracking
   - Gap: Skills not re-injected after compaction?

7. **Stop hooks / post-turn validation**
   - CC: Shell-command hooks in settings.json, can block continuation
   - AEQI: Observer.after_turn() — Rust trait, not user-configurable
   - Gap: No user-configurable post-turn hooks

8. **Prefetching during streaming**
   - CC: Memory prefetch + skill discovery start during LLM streaming, consumed post-tools
   - AEQI: Memory recall happens after tools, not during
   - Gap: No prefetching during stream

9. **Tool result budget enforcement**
   - CC: Per-tool output size limits? (need to verify)
   - AEQI: max_tool_result_chars (50K), persist/truncate for oversized, aggregate budget per turn
   - Gap: ?

10. **Conversation repair**
    - CC: Tool_use/tool_result pairing invariant enforced
    - AEQI: repair_tool_pairing() after compaction
    - Gap: Same pattern? Need to compare.

11. **Worktree isolation for subagents**
    - CC: EnterWorktree/ExitWorktree tools, transparent CWD switch
    - AEQI: Not implemented
    - Gap: Missing entirely

12. **Permission model in the loop**
    - CC: before_tool hook checks permissions, can block/allow/ask
    - AEQI: observer.before_tool() can Halt, but no permission system
    - Gap: No permission system

### Phase 4: Implementation Priority

After the comparison, rank fixes by impact:

**P0 (Resilience):** Error recovery, streaming fallback, context collapse persistence
**P1 (Performance):** Prefetching during stream, worktree isolation
**P2 (UX):** Permission model, user-configurable hooks, stop hooks
**P3 (Polish):** Post-compact skill re-injection, token budget carryover

## Key Files Reference

### Claude Code
```
refs/claude-code/src/query.ts                          — Main agent loop (1729 lines)
refs/claude-code/src/services/tools/StreamingToolExecutor.ts  — Streaming tool scheduler
refs/claude-code/src/services/tools/toolOrchestration.ts      — Batch tool execution
refs/claude-code/src/services/tools/toolExecution.ts          — Tool execution engine
refs/claude-code/src/services/compact/autoCompact.ts          — Full compaction
refs/claude-code/src/services/compact/microCompact.ts         — Incremental compaction
refs/claude-code/src/query/stopHooks.ts                       — Post-turn hooks
refs/claude-code/src/utils/permissions/permissions.ts         — Permission system
refs/claude-code/src/tools/EnterWorktreeTool/                 — Worktree isolation
refs/claude-code/src/Tool.ts                                  — Tool interface
```

### AEQI
```
crates/aeqi-core/src/agent.rs              — Main agent loop (~3100 lines)
crates/aeqi-core/src/streaming_executor.rs — Tool executor (~500 lines)
crates/aeqi-core/src/chat_stream.rs        — Event types
crates/aeqi-core/src/traits/               — Provider, Observer, Memory, Tool traits
crates/aeqi-core/src/config.rs             — Agent + project config
crates/aeqi-core/src/identity.rs           — Agent identity (persona, knowledge, memory)
crates/aeqi-orchestrator/src/session_manager.rs — spawn_session (universal executor)
crates/aeqi-orchestrator/src/delegate.rs        — Delegation tool
crates/aeqi-orchestrator/src/middleware/        — Middleware chain (9 layers)
```

## How to Use This Document

In a fresh Claude Code session:

1. "Read `/home/claudedev/sigil/docs/agent-loop-parity.md` for context"
2. "Execute Phase 1: deep read Claude Code's query.ts and related files"
3. "Execute Phase 2: deep read AEQI's agent.rs"
4. "Execute Phase 3: produce the side-by-side comparison"
5. "Execute Phase 4: implement P0 fixes"

Each phase should be a focused research block — read entire files, don't grep. The goal is UNDERSTANDING, not pattern matching.
