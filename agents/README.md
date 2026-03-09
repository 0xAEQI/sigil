# Agents Directory

Each subdirectory defines an agent — WHO does the work.

## Structure

```
agents/
  shared/            # Shared workflow (all agents inherit)
    WORKFLOW.md      # Base workflow instructions
  my-agent/          # One directory per agent
    agent.toml       # Execution config (model, role, budget)
    PERSONA.md       # Personality and character
    IDENTITY.md      # Identity and background
    PREFERENCES.md   # Working preferences
    MEMORY.md        # Accumulated knowledge
    KNOWLEDGE.md     # Domain expertise
```

## agent.toml

```toml
name = "my-agent"
prefix = "ma"
role = "advisor"       # orchestrator | advisor
voice = "vocal"        # vocal | silent
model = "claude-sonnet-4-6"
expertise = ["my-project"]
max_budget_usd = 1.0
```

## Creating an Agent

1. Create `agents/my-agent/` directory
2. Add `agent.toml` with execution config
3. Add `PERSONA.md` with personality description
4. Reference the agent in your team config in `sigil.toml`

Agents are auto-discovered from disk — no need to add `[[agents]]` to the config file.
