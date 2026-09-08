# Traceability

This file defines how refactor work stays auditable.

The goal is to make it possible to answer:

- What changed?
- Why did it change?
- Which behavior was expected to stay the same?
- Which tests or smoke checks proved it?
- How can we roll back?
- Which docs/contracts need updating next?

## Traceability IDs

Use stable IDs for batches:

```text
R-000 baseline and safety
R-101 backend db connection extraction
R-102 backend migrations extraction
R-201 domain type extraction
R-301 frontend api split
R-401 settings/privacy extraction
R-402 reminders extraction
R-403 guard extraction
R-404 insights extraction
R-405 home extraction
R-501 pet extraction
```

Add new IDs as work becomes concrete.

## Required Records Per Batch

Every batch must have:

1. A log entry in `docs/REFACTOR_LOG.md`.
2. A Git commit or a clear note that the batch is still uncommitted.
3. A validation result.
4. A list of touched files.
5. A statement of API/schema/privacy impact.
6. A rollback point.

## Refactor Log Entry Template

Use this template for each batch:

```text
## YYYY-MM-DD: R-XXX <short title>

Status:

Primary domain:

Intent:

Files changed:

Moved/extracted:

Behavior expected to stay the same:

Behavior intentionally changed:

Tauri commands affected:

Database/schema impact:

Privacy impact:

Startup/performance impact:

Automated validation:

Manual smoke tests:

Risks:

Rollback:

Follow-up:
```

## File Movement Ledger

When moving code, record source and destination:

```text
| Batch | From | To | Notes |
| --- | --- | --- | --- |
| R-101 | src-tauri/src/lib.rs::open_connection | src-tauri/src/db/connection.rs | Behavior unchanged |
```

Keep this ledger either in the batch log entry or in a dedicated section below.

## Command Contract Trace

When a command changes implementation:

- Update `docs/API_CONTRACTS.md`.
- Record whether command name changed.
- Record whether input shape changed.
- Record whether return shape changed.
- Record frontend callers checked.

If only implementation moved and shape stayed the same, say so explicitly.

## Database Trace

When database behavior changes:

- Update `docs/DATABASE_AND_PRIVACY.md`.
- Record whether schema changed.
- Record migration name/version if any.
- Record whether old DB opens.
- Record whether backup/export is affected.

## Privacy Trace

When capture/title/privacy code changes:

- Record raw data sources involved.
- Record where privacy is applied.
- Record what is stored.
- Run or document S-600 from `docs/SMOKE_TESTS.md`.

## Decision Trace

Use `docs/DECISIONS.md` for lightweight decisions.

Use ADR files for heavier decisions:

```text
docs/adr/0001-task-sessions-policy.md
docs/adr/0002-idle-prompt-persistence.md
```

Heavy decisions include:

- User data semantics.
- Privacy defaults.
- Command breaking changes.
- Schema migrations.
- Startup architecture.
- Removing compatibility code.

## Traceability Review

Before merging a refactor branch back to `main`, check:

- Every batch has a log entry.
- Every contract change has a doc update.
- Every schema/privacy change has a doc update.
- Every open risk has a follow-up or decision.
- The latest commit can be mapped to one or more refactor IDs.

