# Refactor Log

Use this file to record refactor work. Keep entries short and factual.

## 2026-06-08: Guidance Documents Created

Changed:

- Added repository-level `AGENTS.md`.
- Added `docs/REFACTOR_MASTER_PLAN.md`.
- Added `docs/CURRENT_SYSTEM_MAP.md`.
- Added `docs/API_CONTRACTS.md`.
- Added `docs/DATABASE_AND_PRIVACY.md`.
- Added `docs/ACCEPTANCE_CHECKLIST.md`.
- Added `docs/DECISIONS.md`.

Reason:

- Establish a small executable documentation set before architecture refactor begins.

Validation:

- Docs-only change. No build required for this entry.

Risks:

- Command contracts and database notes are based on current code inspection and should be refreshed when commands or schema change.

Next:

- Commit current cleanup changes.
- Tag the pre-refactor baseline.
- Create a refactor branch.
- Start with backend DB/path/migration extraction.

## 2026-06-08: Professional Refactor Guidance Expanded

Changed:

- Added `docs/README.md`.
- Added `docs/REFACTOR_PLAYBOOK.md`.
- Added `docs/TESTING_STRATEGY.md`.
- Added `docs/SMOKE_TESTS.md`.
- Added `docs/FIRST_REFACTOR_BATCHES.md`.
- Added `docs/ADR_TEMPLATE.md`.
- Linked these from `docs/REFACTOR_MASTER_PLAN.md`.
- Added decisions for active doc precedence, behavior-preserving batches, and characterization tests.

Reason:

- Make the guidance precise enough for long-running staged refactor work.

Validation:

- Docs-only change. No build required for this entry.

Risks:

- First batches still need to be verified against the latest local Git state before execution.

Next:

- Decide whether to commit current docs/text fixes before creating `refactor/architecture`.

## 2026-06-08: Phase Gates And Traceability Added

Status:

- In Progress until committed.

Changed:

- Added `docs/PHASE_GATES.md`.
- Added `docs/TRACEABILITY.md`.
- Linked both files from `docs/README.md`, `docs/REFACTOR_MASTER_PLAN.md`, and `docs/REFACTOR_PLAYBOOK.md`.

Reason:

- Ensure every phase has explicit evaluation/testing gates.
- Ensure refactor progress is traceable across files, commands, schema/privacy impact, validation, and rollback.

Validation:

- Docs-only change. No build required for this entry.

Risks:

- Gate evidence still needs to be filled during actual refactor batches.

Next:

- Use `PHASE_GATES.md` and `TRACEABILITY.md` when closing Batch 0.

## 2026-06-08: Automation And Contract Details Added

Status:

- In Progress until committed.

Changed:

- Added `.github/workflows/ci.yml`.
- Added `typecheck` and `build:check` npm scripts.
- Added `dist-codex-check` to `.gitignore`.
- Updated validation commands in refactor docs.
- Added backend command source line references and caller map to `docs/API_CONTRACTS.md`.
- Added current schema, indexes, and compatibility migrations to `docs/DATABASE_AND_PRIVACY.md`.
- Added proposed ADRs for `task_sessions` and idle prompt persistence.

Reason:

- Turn the refactor guidance into an executable engineering workflow with CI and more precise contracts.

Validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.
- `cargo check` from `src-tauri` passed.

Risks:

- GitHub CI has not run yet.
- ADRs are proposed, not accepted decisions.

Next:

- Commit the docs, CI, package scripts, and small text fixes.
- Tag the pre-refactor baseline.
- Create the refactor branch.

## 2026-06-08: R-000 Baseline And Safety Gate

Status:

- Passed.

Primary domain:

- repo/process

Intent:

- Preserve `main` as the original reference.
- Move active refactor work to `refactor/architecture`.
- Verify current docs, CI scripts, and small text fixes are build-safe.

Files changed:

- `AGENTS.md`
- `.github/workflows/ci.yml`
- `.gitignore`
- `package.json`
- `docs/**`
- `src/components/GuardView.vue`
- `src/components/InsightsView.vue`
- `src/pet-panel.ts`
- `src/pet.ts`

Moved/extracted:

- None.

Behavior expected to stay the same:

- All app behavior. This batch only adds docs, CI, validation scripts, and small text fixes.

Behavior intentionally changed:

- Adds `pnpm run typecheck`.
- Adds `pnpm run build:check`.
- Adds CI workflow.
- Fixes/standardizes a few visible text strings.

Tauri commands affected:

- None.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.
- `cargo check` from `src-tauri` passed.

Manual smoke tests:

- Not run for this docs/process batch.

Risks:

- GitHub CI has not run remotely yet.
- Proposed ADRs are not accepted decisions yet.

Rollback:

- `pre-refactor-2026-06-08` tag points to `fd2f3c5`.
- `main` remains at `fd2f3c5`.

Follow-up:

- Commit R-000.
- Start R-101 backend DB connection extraction on `refactor/architecture`.

## 2026-06-08: R-101 Backend DB Connection Extraction

Status:

- Passed.

Primary domain:

- backend-db

Intent:

- Move database path resolution, legacy database migration, and connection opening out of `src-tauri/src/lib.rs`.
- Preserve behavior and command compatibility.

Files changed:

- `src-tauri/src/lib.rs`
- `src-tauri/src/db/mod.rs`
- `src-tauri/src/db/connection.rs`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `src-tauri/src/lib.rs::ensure_data_dir` | `src-tauri/src/db/connection.rs::ensure_data_dir` | Behavior unchanged |
| `src-tauri/src/lib.rs::legacy_db_path` | `src-tauri/src/db/connection.rs::legacy_db_path` | Behavior unchanged |
| `src-tauri/src/lib.rs::db_path` | `src-tauri/src/db/connection.rs::db_path` | Behavior unchanged |
| `src-tauri/src/lib.rs::try_migrate_legacy_db` | `src-tauri/src/db/connection.rs::try_migrate_legacy_db` | Behavior unchanged |
| `src-tauri/src/lib.rs::open_connection` | `src-tauri/src/db/connection.rs::open_connection` | Behavior unchanged; now imported by `lib.rs` |

Behavior expected to stay the same:

- Preferred DB path remains `app_local_data_dir()/timeprism.db`.
- Legacy DB fallback/migration behavior remains the same.
- Existing commands still call `open_connection`.

Behavior intentionally changed:

- None.

Tauri commands affected:

- Implementation dependency only. Command names, inputs, and outputs unchanged.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `cargo check` from `src-tauri` passed.

Manual smoke tests:

- Not run for this extraction batch.

Risks:

- Fresh/legacy DB runtime smoke still pending.

Rollback:

- Revert this batch commit after it is committed, or reset to R-000 commit on `refactor/architecture`.

Follow-up:

- Commit R-101.
- Start R-102 backend migration/table helper extraction.

## 2026-06-08: R-102 Backend Migration Extraction

Status:

- Passed.

Primary domain:

- backend-db

Intent:

- Move SQLite schema initialization, schema-upgrade helpers, seed data, and heatmap snapshot table creation out of `src-tauri/src/lib.rs`.
- Preserve database behavior, SQL meaning, seed behavior, command registration, and runtime call sites.

Files changed:

- `src-tauri/src/lib.rs`
- `src-tauri/src/db/mod.rs`
- `src-tauri/src/db/migrations.rs`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `src-tauri/src/lib.rs::ensure_app_rules_time_columns` | `src-tauri/src/db/migrations.rs::ensure_app_rules_time_columns` | Behavior unchanged; private helper remains private inside migrations module |
| `src-tauri/src/lib.rs::ensure_reminders_weekly_columns` | `src-tauri/src/db/migrations.rs::ensure_reminders_weekly_columns` | Behavior unchanged; SQL copied without intentional schema change |
| `src-tauri/src/lib.rs::ensure_reminders_sort_order` | `src-tauri/src/db/migrations.rs::ensure_reminders_sort_order` | Behavior unchanged; sort backfill logic copied |
| `src-tauri/src/lib.rs::ensure_heatmap_snapshot_table` | `src-tauri/src/db/migrations.rs::ensure_heatmap_snapshot_table` | Behavior unchanged; public because `get_learn_heatmap` still calls it defensively |
| `src-tauri/src/lib.rs::init_database` | `src-tauri/src/db/migrations.rs::init_database` | Behavior unchanged; public startup entry imported by `lib.rs` |

Behavior expected to stay the same:

- Database schema creation and upgrade order stays the same.
- Default categories, app rules, app config, and whitelist seeding behavior stays the same.
- Startup still calls `init_database` during Tauri setup.
- Heatmap reads still call `ensure_heatmap_snapshot_table` before querying snapshots.

Behavior intentionally changed:

- None.

Tauri commands affected:

- Implementation dependency only. Command names, inputs, outputs, and registration unchanged.

Database/schema impact:

- No intended schema change.
- No migration versioning introduced in this batch.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `cargo check` from `src-tauri` passed after extraction.
- `cargo check` from `src-tauri` passed again after cleanup.

Manual smoke tests:

- Not run for this extraction batch.

Risks:

- Fresh/legacy DB launch smoke remains valuable before larger backend module splits.
- Future schema work still needs an ADR or explicit migration-versioning plan before changing SQL behavior.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-101 commit.

Follow-up:

- Commit R-102.
- Start R-103 backend domain type extraction.

## 2026-06-08: R-103 Backend Domain Type Extraction

Status:

- Passed.

Primary domain:

- backend-domain

Intent:

- Move serializable response structs and command input structs out of `src-tauri/src/lib.rs`.
- Keep command contracts, serde field names, and command registration unchanged.

Files changed:

- `src-tauri/src/lib.rs`
- `src-tauri/src/domain/mod.rs`
- `src-tauri/src/domain/analytics.rs`
- `src-tauri/src/domain/rules.rs`
- `src-tauri/src/domain/reminders.rs`
- `src-tauri/src/domain/privacy.rs`
- `src-tauri/src/domain/idle.rs`
- `src-tauri/src/domain/window.rs`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `Category`, `CreateCategoryInput`, `TodaySummary`, `TopApp`, `LearnHeatmapCell`, `UsageStackSegment`, `UsageStackDay`, `RecentLogEntry` | `src-tauri/src/domain/analytics.rs` | Behavior unchanged |
| `AppRuleEntry`, `PendingRuleProcess`, `SaveAppRuleInput`, `DeviationCheck` | `src-tauri/src/domain/rules.rs` | Behavior unchanged |
| `ReminderEntry`, `SaveReminderInput`, `SetReminderDoneInput`, `SetReminderOrderInput` | `src-tauri/src/domain/reminders.rs` | Behavior unchanged |
| `PrivacySettings`, `UpdatePrivacySettingsInput`, `SetWhitelistItemInput` | `src-tauri/src/domain/privacy.rs` | Behavior unchanged |
| `IdlePromptEntry`, `ResolveIdlePromptInput`, `IdleMemoryState` | `src-tauri/src/domain/idle.rs` | Behavior unchanged |
| `ForegroundCaptureDiagnostic`, `SettlePetWindowInput`, `PetWindowSettleResult` | `src-tauri/src/domain/window.rs` | Behavior unchanged |

Behavior expected to stay the same:

- Serialized field names stay the same.
- Deserialized command input field names stay the same.
- Tauri command names, inputs, outputs, and registration stay the same.
- Runtime state structs remain in `lib.rs`.

Behavior intentionally changed:

- None.

Tauri commands affected:

- Type import dependency only. Command signatures remain the same from the frontend/API perspective.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `cargo check` from `src-tauri` passed.

Manual smoke tests:

- Not run for this extraction batch.

Risks:

- Some DTO module placement may be revisited as services are extracted; no API behavior should depend on file location.
- Public visibility is intentionally limited to `pub(crate)` for structs and fields needed by current command implementations.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-102 commit.

Follow-up:

- Commit R-103.
- Start R-104 privacy service extraction.

## 2026-06-08: R-104 Privacy Service Extraction

Status:

- Passed.

Primary domain:

- privacy/capture

Intent:

- Move privacy config parsing, process-name normalization, browser-title handling, desktop-shell handling, and whitelist filtering out of `src-tauri/src/lib.rs`.
- Preserve all privacy behavior, title placeholder strings, block reasons, and command contracts.

Files changed:

- `src-tauri/src/lib.rs`
- `src-tauri/src/services/mod.rs`
- `src-tauri/src/services/privacy.rs`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `src-tauri/src/lib.rs::parse_bool_config` | `src-tauri/src/services/privacy.rs::parse_bool_config` | Behavior unchanged |
| `src-tauri/src/lib.rs::normalize_process_key` | `src-tauri/src/services/privacy.rs::normalize_process_key` | Behavior unchanged; now has characterization tests |
| `src-tauri/src/lib.rs::parse_browser_title_mode` | `src-tauri/src/services/privacy.rs::parse_browser_title_mode` | Behavior unchanged |
| `src-tauri/src/lib.rs::is_browser_process` | `src-tauri/src/services/privacy.rs::is_browser_process` | Behavior unchanged; private helper |
| `src-tauri/src/lib.rs::contains_incognito_keyword` | `src-tauri/src/services/privacy.rs::contains_incognito_keyword` | Behavior unchanged; private helper |
| `src-tauri/src/lib.rs::is_desktop_shell_window` | `src-tauri/src/services/privacy.rs::is_desktop_shell_window` | Behavior unchanged; private helper |
| `src-tauri/src/lib.rs::process_log_with_privacy` | `src-tauri/src/services/privacy.rs::process_log_with_privacy` | Behavior unchanged |

Behavior expected to stay the same:

- `curtain_enabled` still blocks storage with reason `curtain_enabled`.
- Browser title mode `BLUR` still stores `Web Browser`.
- Browser title mode `NONE` still stores `Not Collected`.
- Incognito/private browser titles still block storage with reason `incognito_window`.
- Desktop shell windows still normalize to `desktop.shell.exe` / `Desktop Shell`.
- Whitelist-only behavior still stores `uncategorized.exe` / `Hidden by Whitelist` with reason `whitelist_blocked` for unlisted processes.

Behavior intentionally changed:

- None.

Tauri commands affected:

- Implementation dependency only. Command names, inputs, outputs, and registration unchanged.

Database/schema impact:

- None.

Privacy impact:

- No intended behavior change.
- Added characterization tests for privacy-sensitive logic.

Startup/performance impact:

- None expected.

Automated validation:

- `cargo check` from `src-tauri` passed.
- `cargo test` from `src-tauri` passed: 8 privacy tests passed.

Manual smoke tests:

- S-600 interactive smoke was not run because it requires a real foreground browser/window session.
- S-600 core behavior is covered by new automated tests for browser `BLUR`, browser `NONE`, private-window blocking, and whitelist-only behavior.

Risks:

- Full end-to-end capture still needs a real Windows foreground-window smoke before release.
- Future capture extraction should keep raw-title handling behind privacy tests.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-103 commit.

Follow-up:

- Commit R-104.
- Start R-105 frontend API type extraction.

## 2026-06-08: R-105 Frontend API Type Extraction

Status:

- Passed.

Primary domain:

- frontend-api

Intent:

- Split frontend API type definitions from command wrappers.
- Introduce a thin client wrapper around Tauri `invoke`.
- Keep `src/api.ts` as the compatibility API surface so existing component imports continue to work.

Files changed:

- `src/api.ts`
- `src/api/types.ts`
- `src/api/client.ts`
- `src/api/commands/index.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| Exported API types in `src/api.ts` | `src/api/types.ts` | Names and field definitions preserved |
| Direct `@tauri-apps/api/core::invoke` use in `src/api.ts` | `src/api/client.ts::invokeCommand` | Command names and args preserved |
| Existing `./api` import surface | `src/api.ts` re-exports | Compatibility preserved |

Behavior expected to stay the same:

- Existing imports from `./api` keep working.
- Tauri command names stay the same.
- Command argument shapes stay the same.
- Return type names stay the same.

Behavior intentionally changed:

- None.

Tauri commands affected:

- Frontend wrapper dependency only. No backend command registration or command name changed.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.

Cleanup:

- Removed `dist-codex-check` after `build:check`.

Manual smoke tests:

- Not run for this frontend API extraction batch.

Risks:

- Future command wrapper splits should keep `src/api.ts` re-exporting until component imports are migrated deliberately.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-104 commit.

Follow-up:

- Commit R-105.
- Select the next refactor batch from the master plan; do not start broader semantic changes without a new batch scope.
