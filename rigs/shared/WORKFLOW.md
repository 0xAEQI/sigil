# Shared Workflow

These rules apply to ALL domains. Domain-specific AGENTS.md may add to but never contradict these.

## Git Workflow

1. **Always work in worktrees** — never edit `dev` or `master` directly
2. Create worktree: `git worktree add ~/worktrees/feat/<name> -b feat/<name>`
3. Work, test, commit in the worktree
4. Merge to `dev` for auto-deploy to dev environment
5. Test on dev, then merge `dev` → `master` for production
6. Cleanup: `git worktree remove ~/worktrees/feat/<name> && git branch -d feat/<name>`

## Code Standards

| Rule | Rationale |
|------|-----------|
| NO COMMENTS | Code is self-documenting. `//!` and `///` on public APIs only. |
| NO BACKWARD COMPATIBILITY HACKS | No `_unused`, no `#[deprecated]`, no shims. Change everywhere or don't. |
| CONSISTENT NAMING | Same concept = same name across entire codebase. |
| DRY → SHARED CODE | See a pattern twice? Extract it. Three places = refactor. |
| BEST IMPLEMENTATION ONLY | Find the optimal approach. No "good enough". |
| SCHEMA = SOURCE OF TRUTH | DB changes must update schema files. Fresh setup must work. |

## Autonomous Execution Mandate

**Task assignment = full authorization to execute, for hours, without check-ins.**

When a quest is assigned, the spirit owns it end-to-end:
- Spawn sub-agents freely
- Make architectural decisions within the domain
- Commit, deploy to dev, iterate — without pausing for approval
- Ambiguity in the spec? Make the best call. Document the decision. Keep moving.

Mid-task check-ins are a failure mode, not a safety mechanism. The Architect's silence is not confusion — it's trust.

## Spirit Protocol

When executing a quest (task), spirits must signal their outcome:

- **Completed**: Provide a clear summary of what changed (files, commits, deployments)
- **BLOCKED:** prefix: Need a decision or information. State the specific question. Be precise — it gets passed to another agent or human.
- **FAILED:** prefix: Technical error (build failure, test failure). Include error output and what was tried.

### What DOES qualify as BLOCKED

- External credential missing from config and not discoverable anywhere in the codebase
- Build failure that cannot be resolved after genuine attempts
- Architectural conflict where two valid paths require a human choice between funded alternatives (e.g., competing database schemas both of which require migration)

### What does NOT qualify as BLOCKED

- "I need design direction" — pick the best option and build it
- "I'm not sure which approach to take" — research, decide, execute
- "The spec doesn't mention X" — infer from context, make a decision
- "Should I proceed with Y?" — yes. Always yes unless it's destructive and irreversible
- Landing page copy, visual design choices, component structure — just build it

## Batch Dispatch

When multiple quests are ready for a domain, dispatch ALL of them — not sequentially with permission gates. The Architect delegates at the domain level. A domain with 4 ready quests should have 4 spirits running, not 1 spirit running and 3 waiting for human approval that was never required.

The familiar (or any orchestrator) must never:
- List ready quests and ask "which one?"
- Dispatch one and wait for feedback before dispatching the next
- Treat quest selection as requiring human input

Quest triage is the orchestrator's job. If quests have dependencies, sequence them. If they're independent, parallelize them.

## Sub-Agent Orchestration

Spirits have full access to Claude Code's Task tool. Each spirit IS an orchestrator.

For complex tasks, follow the **R→D→R pipeline** (Research → Develop → Review):

1. **Research**: Spawn an Explore agent to map relevant code, find patterns, identify constraints
2. **Develop**: Implement based on research findings. Work in worktree, commit.
3. **Review**: Spawn a review agent to check for anti-patterns, security issues, correctness

Simple tasks (single-file fix, config change) don't need the full pipeline — just do the work.

## Escalation

If you genuinely cannot determine something from the codebase:
1. First try harder — check docs, configs, related code, git history
2. If truly stuck, respond with `BLOCKED:` and a specific question
3. The Scout will attempt domain-level resolution (spawn another spirit with your question)
4. If still stuck, escalates to Shadow (cross-domain knowledge)
5. If Shadow can't resolve, escalates to human via Telegram

## Safety

- Never commit secrets or API keys to git
- Never edit files in `/var/www/` (auto-deployed, read-only)
- Never deploy to production without testing on dev first
- Never trust client-side values for server-side operations
