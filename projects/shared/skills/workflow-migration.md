```toml
[skill]
name = "workflow-migration"
description = "Use when upgrading frameworks, migrating databases, changing APIs, or moving between systems. Triggers: migration, upgrade, version bump, breaking change, deprecation."
phase = "workflow"
```

# Migration Workflow

Analyze → Plan → Execute incrementally → Verify at each step → Clean up. Never big-bang.

```
Analyze → Plan → Migrate Incrementally → Verify → Clean Up
```

---

## Phase 1: Analyze

1. **What's changing** — exact version/API/schema differences between old and new
2. **Blast radius** — what code touches the thing being migrated? `sigil_graph` impact analysis
3. **Breaking changes** — enumerate every breaking change in the upgrade path
4. **Data migration** — does data need transforming? Is it reversible?
5. **Rollback plan** — can we undo this? At what cost?

Post: `sigil_blackboard` post with key `task:{id}:migration-analysis`

---

## Phase 2: Plan

Use the **expand-contract pattern** (parallel run):
1. **Expand** — add new alongside old (both work)
2. **Migrate** — switch consumers to new
3. **Contract** — remove old once all consumers migrated

For each step:
- Exact files to change
- Test command to verify
- Rollback command if it fails
- Must compile and pass tests independently

<HARD-GATE>
Never do a big-bang migration. Every step must leave the system working. If step 3 of 10 fails, steps 1-2 are still live and correct.
</HARD-GATE>

---

## Phase 3: Migrate Incrementally

1. **One step at a time** — make the change, test, commit
2. **Run full suite after each step** — not just the changed module
3. **If tests fail** — the migration step is wrong. Revert and fix the approach, don't fix forward.
4. **Data migration** — backup first, migrate in batches, verify row counts

---

## Phase 4: Verify

1. **All tests pass** — including integration tests
2. **No deprecation warnings** — from the new version
3. **Performance baseline** — compare with pre-migration metrics
4. **Data integrity** — row counts, checksums, spot-check samples

---

## Phase 5: Clean Up

1. **Remove old code** — dead code from the expand phase
2. **Remove compatibility shims** — any adapters between old and new
3. **Update documentation** — reflect the new state
4. **Store** — `sigil_remember` migration learnings

---

## Anti-Rationalization Table

| Excuse | Reality |
|--------|---------|
| "Let's just upgrade everything at once" | Big-bang migrations fail unpredictably. Incrementally. |
| "The old version still works" | Until it doesn't. Migrate before it's urgent. |
| "We can skip the expand phase" | Expand-contract is how you rollback safely. Don't skip it. |
| "Data migration is straightforward" | Backup first. Always. |
