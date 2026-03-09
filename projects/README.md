# Projects Directory

Each subdirectory defines a project — WHAT gets done.

## Structure

```
projects/
  shared/            # Shared assets (all projects inherit)
    skills/          # Reusable skill definitions
    pipelines/       # Pipeline templates
  my-project/        # One directory per project
    AGENTS.md        # Operating instructions for workers
    KNOWLEDGE.md     # Domain knowledge and context
    HEARTBEAT.md     # Health check endpoints (optional)
    .tasks/          # Task storage (JSONL files)
    skills/          # Project-specific skills
```

## Creating a Project

1. Create `projects/my-project/` directory
2. Add `AGENTS.md` with operating instructions
3. Add `KNOWLEDGE.md` with domain context
4. Add the project to `config/sigil.toml`:

```toml
[[projects]]
name = "my-project"
prefix = "mp"
repo = "/path/to/repo"
model = "claude-sonnet-4-6"
max_workers = 2
execution_mode = "claude_code"
```

## Task Storage

Tasks are stored as JSONL files in `.tasks/`:
- `_tasks.jsonl` — task definitions
- `_missions.jsonl` — mission definitions (groups of tasks)

Use `sigil task create` to add tasks via CLI.
