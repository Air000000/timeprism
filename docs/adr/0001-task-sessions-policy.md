# ADR-0001: Task Sessions Policy

## Status

Proposed

## Context

The current backend includes:

- `task_sessions` table.
- `start_session` command.
- `stop_active_session` command.
- `active_session_id` in `TodaySummary`.

The active product direction says automatic foreground sampling is the primary source of time facts and that traditional manual start/stop timers should not be a v1 core workflow.

Keeping `task_sessions` as a core source risks reintroducing double-counting or conflicting truth systems. Removing it immediately risks breaking compatibility with existing data and command callers.

## Decision

Proposed decision:

Keep `task_sessions` as legacy compatibility during architecture refactor, but do not expand the manual timer workflow.

During early refactor:

- Preserve existing table and commands.
- Do not add new UI around manual sessions.
- Do not make `task_sessions` the primary analytics source.
- Mark future semantics as compatibility-only until a product decision changes.

## Options Considered

### Option A: Keep As Compatibility Only

Pros:

- Lowest migration risk.
- Preserves current command compatibility.
- Avoids data loss.

Cons:

- Leaves legacy concepts in the codebase.
- Requires clear documentation to avoid accidental expansion.

### Option B: Remove Immediately

Pros:

- Aligns strictly with v1 product direction.
- Reduces schema/API complexity.

Cons:

- High compatibility risk.
- Requires migration and frontend/API cleanup.

### Option C: Reinterpret As Focus Intent

Pros:

- Keeps user intent without treating it as factual time.
- May support future focus-mode features.

Cons:

- Product semantics need design.
- Not safe as an early architecture cleanup task.

## Consequences

Positive:

- Refactor can begin without resolving all historical data semantics.
- Existing commands remain stable.

Negative:

- The codebase carries a compatibility burden.

Follow-up:

- Decide before any analytics rewrite whether `task_sessions` can affect totals.
- Add tests around analytics source priority before changing summary logic.

## Validation

- `start_session` and `stop_active_session` remain callable during compatibility period.
- Home/analytics behavior does not shift unexpectedly.
- No new manual-timer UI is added.

## Review Date

Before frontend Home extraction or analytics service rewrite.

