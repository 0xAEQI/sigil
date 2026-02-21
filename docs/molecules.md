# Molecules — Workflow Templates

Molecules are TOML-defined workflow templates that create linked task chains (beads) in a single command.

## Format

```toml
[molecule]
name = "feature-dev"
description = "Full feature development lifecycle"

[vars]
issue_id = { type = "string", required = true }

[[steps]]
id = "understand"
title = "Understand the requirement"
instructions = "Read the issue {{issue_id}}, explore relevant code."
needs = []

[[steps]]
id = "plan"
title = "Plan the implementation"
instructions = "Design the solution approach."
needs = ["understand"]

[[steps]]
id = "implement"
title = "Implement the solution"
instructions = "Write the code. Commit with descriptive message."
needs = ["plan"]
```

## Usage

### Pour a molecule
```bash
sg mol pour feature-dev --rig algostaking --var issue_id=as-123
```

Creates a parent bead with child step beads linked by dependencies:
```
as-005 [Pending] feature-dev
  as-005.1 [Pending] Understand the requirement (ready)
  as-005.2 [Pending] Plan the implementation (needs: as-005.1)
  as-005.3 [Pending] Implement the solution (needs: as-005.2)
```

### List available molecules
```bash
sg mol list                    # All rigs
sg mol list --rig algostaking  # Specific rig
```

### Check molecule progress
```bash
sg mol status as-005
```
Output:
```
as-005 [Pending] feature-dev
Progress: 1/3

  [x] as-005.1 Understand the requirement
  [~] as-005.2 Plan the implementation
  [ ] as-005.3 Implement the solution
```

## Variable Interpolation

Variables declared in `[vars]` can be used in step instructions with `{{var_name}}`:
```toml
[vars]
issue_id = { type = "string", required = true }

[[steps]]
id = "understand"
instructions = "Read issue {{issue_id}} and understand the problem."
```

Pass variables with `--var`:
```bash
sg mol pour template --var issue_id=as-123 --var priority=high
```

## Step Dependencies

Steps declare dependencies with the `needs` array:
```toml
[[steps]]
id = "test"
needs = ["implement"]  # Can't start until "implement" is done
```

When a step's bead is closed, downstream steps become unblocked automatically.

## Built-in Molecules

### feature-dev.toml
5-step feature development: understand -> plan -> implement -> test -> review

### incident.toml
5-step incident response: detect -> diagnose -> fix -> verify -> postmortem

## Location

Molecule templates live in `rigs/<rig-name>/molecules/*.toml`.
