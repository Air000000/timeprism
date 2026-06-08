# Decisions

Record product and architecture decisions here when they affect refactor direction.

## Accepted

### D-000: Active Refactor Docs Live In This Repository

Decision: `timeprism/AGENTS.md` and `timeprism/docs/*` are the active refactor guidance set.

Reason: the workspace contains older docs in sibling directories. Keeping an explicit active set prevents conflicting instructions during long-running refactor work.

### D-001: Keep Current App Runnable During Refactor

Decision: refactor in stages and keep the current app runnable.

Reason: the app already has working capture, reminders, analytics, and pet behavior. A big-bang rewrite would make parity hard to judge.

### D-002: Preserve Existing Tauri Commands Early

Decision: early refactor work preserves current command names and payload shapes.

Reason: this lets backend and frontend be split independently.

### D-003: Privacy Before Persistence

Decision: foreground capture must apply privacy settings before inserting usage logs.

Reason: local-first does not remove the need for safe handling of browser titles and private windows.

### D-004: Pet Stays Lightweight

Decision: pet and pet-panel may consume shared APIs, but they should not own full settings, analytics, or correction workflows.

Reason: pet is a companion and prompt surface, not a second full application.

### D-005: Use Behavior-Preserving Batches First

Decision: initial refactor batches must be move-only or extract-only unless a decision record says otherwise.

Reason: the current app already has valuable working behavior. Architecture cleanup should first reduce file/module risk without changing product semantics.

### D-006: Use Characterization Tests For Risky Domain Logic

Decision: before changing analytics, privacy, reminder recurrence, or business-day logic, add characterization tests where practical.

Reason: these areas contain user-visible semantics that are easy to accidentally change while extracting modules.

## Open Questions

### Q-001: What Should Happen To `task_sessions`?

Current state:

- The database and API still include `task_sessions`, `start_session`, and `stop_active_session`.
- Product direction says traditional manual timers are not a v1 core source of truth.

Needed decision:

- Keep as legacy compatibility only.
- Remove from UI but keep database read compatibility.
- Reinterpret as optional focus intent without adding time totals.
- Fully remove after migration.

Proposed ADR:

- `docs/adr/0001-task-sessions-policy.md`

### Q-002: Should Idle Prompts Be Persisted?

Current state:

- Pending idle prompts live in process memory.

Needed decision:

- Accept loss on restart for now.
- Persist pending prompts in SQLite.
- Persist only prompts above a larger threshold.

Proposed ADR:

- `docs/adr/0002-idle-prompt-persistence.md`

### Q-003: What Is The First Refactor Branch Scope?

Candidate:

- Batch 0 baseline safety, then backend split of DB/path/migrations first.

Reason:

- It lowers risk for every later data and command change.

## Decision Template

```text
### D-XXX: Title

Decision:

Context:

Reason:

Consequences:
```
