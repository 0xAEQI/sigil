# Council of Agents

The Council is System's multi-agent advisory system. Instead of a single coordinator, System runs a lead agent (the lead agent) supported by specialist advisors who provide project-scoped expertise.

## Architecture

```
Incoming message (Telegram / CLI)
        |
        v
AgentRouter (Gemini Flash, ~$0.001/call)
  "Which advisors are relevant to this message?"
        |
        v
RouteDecision { advisors: ["advisor-a", "advisor-b"], confidence: 0.85 }
        |
        +-->  Advisor A (parallel task spawn, 60s timeout)
        +-->  Advisor B (parallel task spawn, 60s timeout)
        |
        v
Council input injected into the lead agent's context
        |
        v
the lead agent (Lead Agent) synthesizes and responds
```

## Agents

### the lead agent -- The Lead Agent

- **Role**: `orchestrator` -- primary coordinator, Telegram interface, task routing
- **Model**: Claude Opus (highest capability for orchestration decisions)
- **Mode**: Full Claude Code (can edit files, run commands, spawn sub-agents)
- **Scope**: All projects

### Advisor A -- The Financial Specialist

- **Role**: `advisor` -- financial and infrastructure specialist
- **Model**: Claude Opus
- **Scope**: `project-alpha` project
- **Personality**: Disciplined, data-driven, skeptical of unproven approaches
- **Specialties**: Financial systems, cost analysis, infrastructure hardening, risk assessment

### Advisor B -- The Product Specialist

- **Role**: `advisor` -- product and user experience specialist
- **Model**: Claude Sonnet
- **Scope**: `project-beta`, `project-gamma` projects
- **Personality**: Optimistic, user-focused, connects dots across projects
- **Specialties**: Product strategy, UX/UI, marketplace dynamics, pricing

### Advisor C -- The Systems Specialist

- **Role**: `advisor` -- systems architecture specialist
- **Model**: Claude Sonnet
- **Scope**: `sigil` project
- **Personality**: Minimal, precise -- speaks only when the signal is strong, architectural focus
- **Specialties**: Framework internals, performance optimization, system design, observability

## Configuration

```toml
# config/sigil.toml

# Agents
[[agents]]
name = "leader"
prefix = "fa"
model = "claude-opus-4-6"
role = "orchestrator"
telegram_token_secret = "TELEGRAM_BOT_TOKEN"

[[agents]]
name = "advisor-a"
prefix = "fk"
model = "claude-opus-4-6"
role = "advisor"
expertise = ["project-alpha"]
telegram_token_secret = "ADVISOR_A_TELEGRAM_TOKEN"

[[agents]]
name = "advisor-b"
prefix = "fm"
model = "claude-sonnet-4-6"
role = "advisor"
expertise = ["project-beta", "project-gamma"]
telegram_token_secret = "ADVISOR_B_TELEGRAM_TOKEN"

[[agents]]
name = "advisor-c"
prefix = "fv"
model = "claude-sonnet-4-6"
role = "advisor"
expertise = ["sigil"]
telegram_token_secret = "ADVISOR_C_TELEGRAM_TOKEN"

# Team cost controls
[team]
leader = "leader"
router_model = "gemini-flash"
router_cooldown_secs = 60
max_advisor_cost_usd = 0.50
```

## Routing

The `AgentRouter` uses a cheap classifier (Gemini Flash, ~$0.001/call) to determine which advisors are relevant:

```rust
RouteDecision {
    advisors: Vec<String>,    // ["advisor-a", "advisor-b"]
    confidence: f32,          // 0.85
    reasoning: String,        // "Message mentions trading and pricing"
}
```

Routing logic:
- Message mentions "finance", "costs", "budget" -> route to Advisor A
- Message mentions "product", "cards", "legal" -> route to Advisor B
- Message mentions "framework", "architecture", "performance" -> route to Advisor C
- Message mentions multiple projects -> route to all relevant advisors

## Council Mode

Force all advisors into debate with `/council`:

```
/council "Should we add WebSocket support to the trading pipeline?"
```

All advisors receive the question regardless of routing. Their responses are collected with attribution and injected into the lead agent's context for synthesis.

This creates visible multi-perspective debate:

```
[Advisor A]: WebSocket adds latency. At our tick rates (50us), every hop matters.
        If the data can be pushed via shared memory, that's strictly better.
        Cost: additional infra for WS servers, monitoring, reconnection logic.

[Advisor B]: Users expect real-time updates. The dashboard refresh lag is the #1
        complaint. WebSocket for the dashboard, keep shared memory for the
        execution path.

[Advisor C]: Hybrid approach. WS for observation plane (dashboards, monitoring),
        shared memory for execution plane. Clean separation. The existing
        Prometheus metrics path already handles observation -- extend it.
```

the lead agent synthesizes:
> Based on council input: hybrid architecture. WebSocket for dashboard/monitoring
> consumers, shared memory for execution path. Advisor C's observation/execution plane
> split aligns with our existing metrics infrastructure.

## Cost Control

| Control | Default | Purpose |
|---------|---------|---------|
| `max_advisor_cost_usd` | $0.50 | Max cost per individual advisor call |
| `router_cooldown_secs` | 60 | Minimum seconds between routing to same advisor |
| Router model | Gemini Flash | Cheapest possible classifier (~$0.001/call) |
| Advisor timeout | 60s | Max wait for advisor response before proceeding |

## Identity Files

Each agent has identity files in `agents/<name>/`:

```
agents/advisor-a/
  PERSONA.md     <- personality, expertise, communication style
  IDENTITY.md    <- name, role, project scope
  AGENTS.md      <- operating instructions for advisor role
  KNOWLEDGE.md   <- project-specific knowledge relevant to advisory
```

The PERSONA.md files define distinct personalities with trust layers and team dynamics:

- **Trust layers**: How the agent communicates at different trust levels (with Emperor vs with workers vs with other agents)
- **Team dynamics**: How agents interact with each other (Advisor A challenges Advisor B's optimism, Advisor C mediates)
- **Debate style**: How the agent argues (Advisor A: data-first, Advisor B: user-impact, Advisor C: architectural purity)

## Adding a New Advisor

1. Add to `config/sigil.toml`:
```toml
[[agents]]
name = "newadvisor"
prefix = "na"
model = "claude-sonnet-4-6"
role = "advisor"
expertise = ["target-project"]
```

2. Create identity files:
```bash
mkdir -p agents/newadvisor
# Create PERSONA.md, IDENTITY.md, AGENTS.md, KNOWLEDGE.md
```

3. The agent router will automatically include the new advisor in routing decisions based on its expertise scope.
