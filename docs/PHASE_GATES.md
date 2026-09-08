# Phase Gates

Every refactor phase must pass a gate before the next phase starts.

The gate is not only "build passes". It includes scope control, contract review, data/privacy review, validation evidence, and traceability.

## Gate States

Use these states in `docs/REFACTOR_LOG.md`:

- `Not Started`
- `In Progress`
- `Blocked`
- `Passed`
- `Passed With Known Risk`
- `Rolled Back`

`Passed With Known Risk` requires a follow-up entry in `docs/DECISIONS.md` or `docs/REFACTOR_LOG.md`.

## Universal Gate Checklist

Every phase must answer:

1. What changed?
2. What did not change?
3. Which files moved or changed?
4. Which Tauri commands changed?
5. Did command payloads or return shapes change?
6. Did database schema or data path behavior change?
7. Did privacy behavior change?
8. Did startup behavior change?
9. Which automated checks passed?
10. Which manual smoke tests passed?
11. What risks remain?
12. What is the rollback point?

## Phase 0 Gate: Baseline And Safety

Required evidence:

- Git status captured.
- Current docs committed or intentionally left uncommitted with reason.
- Pre-refactor tag created.
- Refactor branch created.
- Current source validation run if code changes are included.

Recommended checks:

```powershell
pnpm.cmd run typecheck
pnpm.cmd run build:check
cd src-tauri
cargo check
```

Manual smoke:

- S-000 Preflight.

Pass criteria:

- A stable rollback point exists.
- The refactor branch is ready.

## Phase 1 Gate: Backend DB And Migration Extraction

Required evidence:

- DB path functions moved or wrapped.
- Migration/table helpers moved or wrapped.
- No schema change unless explicitly recorded.
- Command names unchanged.

Required checks:

```powershell
cd src-tauri
cargo check
```

Recommended tests:

- Fresh DB initialization using a temporary profile or test DB.
- Existing DB open path if practical.

Manual smoke:

- S-100 Main Window.
- S-300 Home basic load.

Pass criteria:

- App opens existing data.
- Fresh database can initialize.
- No user data path behavior changed unexpectedly.

## Phase 2 Gate: Backend Domain And Service Extraction

Required evidence:

- Domain structs moved without field rename.
- Services extracted without behavior change.
- Command wrappers remain compatible.
- Characterization tests added for any risky logic that changed.

Required checks:

```powershell
cd src-tauri
cargo check
```

Recommended tests:

- Business day / interval tests.
- Reminder recurrence tests.
- Privacy title transformation tests.

Manual smoke:

- S-400 Reminders if reminder service changed.
- S-500 Guard if rules/idle/diagnostics changed.
- S-600 Privacy And Capture if privacy/capture changed.

Pass criteria:

- Existing frontend callers still work.
- Extracted services preserve current outputs.

## Phase 3 Gate: Frontend API And Shell Extraction

Required evidence:

- API types split without breaking existing imports.
- `src/api.ts` compatibility re-export exists until all callers migrate.
- `App.vue` shrinks or stops growing.
- No new `ctx: any` is introduced.

Required checks:

```powershell
pnpm.cmd run typecheck
pnpm.cmd run build:check
```

Manual smoke:

- S-100 Main Window.
- S-200 Navigation.

Pass criteria:

- Existing pages still render.
- API import paths remain stable or are migrated completely.

## Phase 4 Gate: Feature Extraction

Required evidence:

- One primary feature domain per batch.
- Feature state moved into typed composable/module.
- Feature component uses typed props/emits or typed composables.
- Shared helpers moved to `src/lib` only when reused by more than one feature.

Required checks:

```powershell
pnpm.cmd run typecheck
pnpm.cmd run build:check
```

Manual smoke:

- Settings/privacy: S-600.
- Reminders: S-400.
- Guard: S-500.
- Insights: S-700.
- Home: S-300.
- Pet/panel: S-800 and S-900.

Pass criteria:

- Feature parity is maintained.
- No unrelated feature behavior changed.

## Phase 5 Gate: Data Management And Hardening

Required evidence:

- Database path visible if implemented.
- Backup/export behavior documented.
- Restore/import behavior avoids accidental overwrite.
- Error/diagnostics behavior documented.

Required checks:

```powershell
pnpm.cmd run typecheck
cd src-tauri
cargo check
```

Manual smoke:

- Data export/backup flow, when implemented.
- S-600 Privacy And Capture.

Pass criteria:

- User data safety improved.
- No destructive behavior exists without confirmation.

## Phase 6 Gate: Cleanup

Required evidence:

- Dead code removed.
- Compatibility wrappers removed only after callers are migrated.
- Docs updated to final state.
- README reflects actual app behavior.

Required checks:

```powershell
pnpm.cmd run typecheck
pnpm.cmd run build:check
cd src-tauri
cargo check
```

Recommended:

- Full Tauri build.

Manual smoke:

- S-100 through S-900, scoped to release readiness.

Pass criteria:

- No major architecture red flags remain.
- The app is ready to merge back to `main`.

## Gate Record Template

Add this to `docs/REFACTOR_LOG.md` when closing a phase:

```text
## YYYY-MM-DD: Phase N Gate - <name>

State:

Scope completed:

Scope intentionally not completed:

Files changed:

Commands affected:

Schema/privacy/startup impact:

Automated validation:

Manual smoke tests:

Known risks:

Rollback point:

Next phase:
```
