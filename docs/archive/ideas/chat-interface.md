# Sigil Chat Design

This document defines the first serious `sigil chat` interface.

It is a design target, not a claim that the full surface exists today. The goal is to turn Sigil from a command set plus daemon into a daily-driver terminal shell for operating an AI organization.

## Product Goal

`sigil chat` should not be "Sigil, but with a prettier prompt box."

It should be the operator cockpit for:

- direct chat with roles, teams, councils, and projects
- live monitoring of readiness, dispatches, budgets, and drift
- intervention actions such as approve, pause, reroute, resume, and escalate
- persistent thread history across restarts
- proactive summaries that tell the operator what needs attention now

The target is better operator leverage than a generic coding chat:

- Claude Code and Codex are strong at a single foreground session
- Sigil should be strong at persistent, multi-role, inspectable operations

## Design Principles

- Keyboard-first. No mouse required.
- Persistent. Threads, inboxes, and state survive restarts.
- Role-aware. The operator can talk to a specific role, not only one generic assistant.
- Proactive. The UI should surface risk and drift before the operator asks.
- Auditable. Every action should be tied back to tasks, dispatches, memory, or audit events.
- Graceful degradation. If the daemon is offline, the UI should still open in local-only mode and make that obvious.

## Primary User Jobs

Operators need to do five things quickly:

1. See what matters now.
2. Jump into the right thread or role.
3. Understand current work, blockage, and cost.
4. Intervene without dropping to raw subcommands.
5. Resume context after time away.

## Default Screen Layout

The default screen should use four regions.

### Top Bar

Persistent status strip with:

- workspace name
- daemon state: `online`, `degraded`, `offline`
- readiness
- active profile or project filter
- cost today and remaining budget
- pending alerts count

Example:

```text
Sigil | online | ready | workspace: sigil | cost: $4.18 / $25.00 | alerts: 3
```

### Left Pane: Inbox

This is the navigation spine. It is not just a list of chats.

Sections:

- `Alerts`
- `Threads`
- `Roles`
- `Projects`
- `Councils`
- `Incidents`

Thread types:

- direct role thread: `@leader`, `@cto`, `@researcher`
- project room: `#sigil`, `#gacha-agency`
- council room: `&product`, `&incident-review`
- mission or incident thread: `!sg-m004`, `!incident-2026-03-16`

Each row should show:

- name
- unread marker
- last activity time
- short status hint

Examples:

```text
@leader            waiting on approval
@reviewer          delivered diff summary
#sigil             2 critical tasks ready
&product           converged
!sg-m004           blocked on repo access
```

### Center Pane: Active Thread

The center pane is the main conversation and execution timeline.

It should render:

- user messages
- agent replies
- tool activity summaries
- task changes
- approvals and interventions
- handoffs between roles

This pane should support:

- streaming token output
- expandable tool and task events
- paging older history
- retry or resend of failed messages

Sigil should treat messages and control-plane events as part of the same thread timeline rather than hiding orchestration in a separate log.

### Right Pane: Context

The right pane changes with focus.

For a role thread, show:

- role name and mandate
- org unit
- manager, peers, direct reports
- current assignments
- recent blackboard items
- recent memory hits

For a project room, show:

- readiness
- ready and blocked task counts
- repo state
- active workers
- dispatch health
- budget status

For a mission or incident, show:

- mission summary
- owner
- status
- critical path tasks
- blockers
- latest audit trail

### Bottom Composer

The bottom area is both a chat composer and a command surface.

- plain text sends a message to the current thread
- `/` starts a slash command
- `@` mentions or retargets a role
- `#` jumps to a project room
- `!` jumps to a mission or incident

The composer should always show the current target:

```text
to @leader > review the current monitor alerts and tell me what needs operator action
```

## Primary Views

`sigil chat` should support a small set of explicit views instead of many disconnected modes.

### Inbox View

Default landing screen.

Purpose:

- show alerts
- show recent threads
- show where operator attention is needed

### Thread View

Focused conversation plus contextual sidebar.

Purpose:

- talk to one role, project room, council, or mission
- inspect live execution and related state

### Monitor View

A richer version of `sigil monitor`.

Purpose:

- show control-plane health
- highlight interventions
- jump into the responsible thread directly

### Tasks View

Task and mission browser tied to the active project or org unit.

Purpose:

- inspect work queues
- reassign, close, hook, or decompose work

### Dispatch View

Transport and routing health.

Purpose:

- inspect unread, overdue, retrying, or dead-letter dispatches
- repair routing problems before work silently stalls

## Command Model

`sigil chat` should support both slash commands and keyboard shortcuts.

### Core Slash Commands

- `/help`
- `/goto <target>`
- `/threads`
- `/roles`
- `/projects`
- `/monitor`
- `/tasks`
- `/missions`
- `/dispatches`
- `/budget`
- `/status`

### Conversation Commands

- `/new @role`
- `/reply`
- `/clear-unread`
- `/summarize`
- `/rename`
- `/pin`
- `/close-thread`

### Operator Actions

- `/approve <id>`
- `/reject <id> [reason]`
- `/pause <project|mission|worker>`
- `/resume <project|mission|worker>`
- `/reroute <task> @role`
- `/handoff <task> @role`
- `/escalate <task> @role`

### Knowledge and Audit Commands

- `/blackboard [tags]`
- `/memory <query>`
- `/audit [task|project|role]`
- `/cost`
- `/readiness`

### Execution Commands

- `/run <prompt>`
- `/skill <name> [prompt]`
- `/council <topic>`

`/run` is the escape hatch.
It should not be the primary model.

## Keyboard Model

The TUI should be fast enough for all-day use.

Recommended bindings:

- `Ctrl-p`: command palette
- `Ctrl-k`: quick thread switcher
- `Tab` / `Shift-Tab`: cycle panes
- `j` / `k`: move within lists
- `Enter`: open selected thread
- `Esc`: back to inbox
- `g i`: inbox
- `g m`: monitor
- `g t`: tasks
- `g d`: dispatches
- `g r`: roles
- `[` / `]`: previous or next unread thread

## Thread Model

The interface should expose four first-class thread categories.

### Role Threads

Examples:

- `@leader`
- `@cto`
- `@research`
- `@reviewer`

Use for:

- directed conversation
- approvals
- strategic or departmental questions

### Project Rooms

Examples:

- `#sigil`
- `#gacha-agency`

Use for:

- project-wide state
- task coordination
- project monitor and memory view

### Council Threads

Examples:

- `&product`
- `&launch-review`

Use for:

- multi-agent synthesis
- explicit debate and convergence

### Mission and Incident Threads

Examples:

- `!sg-m004`
- `!incident-db-latency`

Use for:

- time-bounded work
- investigation and resolution
- operator intervention on a single critical effort

## Message Semantics

Not every line in the thread is a chat bubble.

The center timeline should support these event kinds:

- `message`
- `tool`
- `task_change`
- `dispatch`
- `approval`
- `handoff`
- `watchdog`
- `system_alert`

That distinction matters because Sigil's advantage is not chat fluency. It is operational context.

## Proactive Feed

The inbox should always have an alert feed derived from daemon and local state.

Priority order:

1. readiness blockers
2. dead-letter or overdue dispatches
3. critical ready backlog
4. budget exhaustion or pressure
5. missing repo or task-store failures
6. stale missions or stalled workers

Each alert should include a recommended action and a jump target.

Examples:

- `Dispatch backlog: 3 dead letters -> open Dispatch View`
- `Budget pressure in #sigil -> open project room`
- `@leader awaiting approval for sg-014 -> open thread`

## What Already Exists and Should Be Reused

Sigil already has backend pieces that `sigil chat` should reuse instead of replacing:

- daemon IPC with `status`, `readiness`, `projects`, `dispatches`, `metrics`, and `cost`
- `sigil monitor` aggregation logic
- SQLite-backed `ConversationStore`
- channel metadata via `ConversationStore::list_channels`
- persistent audit log
- blackboard
- cost ledger
- organization kernel in config
- council orchestration

This is enough to build a useful first version without inventing a separate backend.

## Backend Gaps To Close

`sigil chat` still needs dedicated daemon surfaces.

### Required for Phase 1

- `conversation.list`
  - list known threads and channels
- `conversation.read`
  - paginated thread history
- `conversation.send`
  - submit a local message into the same routing path used by daemon chat ingress
- `conversation.mark_read`
- `roles.list`
  - resolved roles, org unit, and thread target metadata

### Required for Phase 2

- `events.tail`
  - streaming or poll-friendly event feed for thread updates
- `approvals.list`
  - pending operator approvals
- `workers.list`
  - active workers by project and role
- `missions.list`
  - mission summaries with state and blockers

### Required for Phase 3

- `council.open`
- `council.reply`
- `intervention.apply`
  - pause, resume, reroute, escalate, approve, reject

## Rollout Plan

### Phase 1: Usable Daily Driver

Ship:

- inbox
- thread view
- monitor pane
- local polling
- role and project switching
- slash commands for monitor, tasks, dispatches, blackboard, audit, cost

This version should already replace `sigil monitor --watch` plus a pile of ad hoc daemon queries.

### Phase 2: True Operator Console

Ship:

- streaming updates
- approval queue
- dispatch repair workflow
- richer task and mission controls
- better thread event rendering

### Phase 3: Sigil-Native Advantage

Ship:

- council threads
- relationship-aware role threads
- operator interventions inline
- proactive summaries tied to org roles and budgets

This is the point where Sigil stops competing as "another coding chat" and starts competing as an AI control plane.

## Non-Goals

The first version should not try to be:

- a web app
- a full IDE
- a generic shell multiplexer
- a visual org-chart editor
- a clone of Claude Code or Codex

The TUI should be opinionated and operational.

## Success Criteria

`sigil chat` is successful when an operator can do the following without leaving the interface:

- understand whether the system is healthy
- see what needs attention now
- talk to a specific role or project
- inspect task, dispatch, audit, and budget context
- approve or intervene on live work
- come back after hours and recover context in under a minute
