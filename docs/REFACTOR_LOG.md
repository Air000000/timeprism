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

## 2026-06-08: R-106 Backend Rules Service Extraction

Status:

- Passed.

Primary domain:

- rules

Intent:

- Move app-rule lookup, app-rule upsert, save-rule validation, app-rule listing, and pending-rule process queries out of `src-tauri/src/lib.rs`.
- Keep Tauri command names and command signatures unchanged.

Files changed:

- `src-tauri/src/lib.rs`
- `src-tauri/src/services/mod.rs`
- `src-tauri/src/services/rules.rs`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `src-tauri/src/lib.rs::resolve_rule_mapping` | `src-tauri/src/services/rules.rs::resolve_rule_mapping` | Behavior unchanged; now covered by characterization tests |
| `src-tauri/src/lib.rs::upsert_app_rule_entry` | `src-tauri/src/services/rules.rs::upsert_app_rule_entry` | Behavior unchanged |
| `src-tauri/src/lib.rs::save_app_rule` validation body | `src-tauri/src/services/rules.rs::save_app_rule_entry` | Command shell remains in `lib.rs`; validation strings preserved |
| `src-tauri/src/lib.rs::list_app_rules` query body | `src-tauri/src/services/rules.rs::list_app_rule_entries` | Command shell remains in `lib.rs` |
| `src-tauri/src/lib.rs::list_pending_rule_processes` query body | `src-tauri/src/services/rules.rs::list_pending_rule_process_entries` | Command shell remains in `lib.rs` |

Behavior expected to stay the same:

- Unknown or invalid mapped types still resolve to `IGNORE`.
- `save_app_rule` keeps the same validation errors.
- App-rule listing order and limit handling stay the same.
- Pending-rule process threshold, ordering, and fallback title stay the same.
- Idle prompt persistence still uses the same rule upsert behavior.

Behavior intentionally changed:

- None.

Tauri commands affected:

- `list_app_rules`, `list_pending_rule_processes`, and `save_app_rule` now delegate to `services::rules`.
- Command names, inputs, outputs, and registration unchanged.

Database/schema impact:

- None.

Privacy impact:

- None intended. Rule privacy-level validation is unchanged.

Startup/performance impact:

- None expected.

Automated validation:

- `cargo test` from `src-tauri` passed: 11 tests passed.
- `cargo check` from `src-tauri` passed.

Manual smoke tests:

- Not run for this extraction batch.

Risks:

- `check_focus_deviation` still keeps stateful deviation logic in `lib.rs`; future extraction should treat it as a separate guard/focus batch.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-105 commit.

Follow-up:

- Commit R-106.
- Continue Phase 2 with the next backend service extraction, likely reminders.

## 2026-06-08: R-107 Backend Reminders Service Extraction

Status:

- Passed.

Primary domain:

- reminders

Intent:

- Move reminder recurrence, due-time calculation, CRUD, completion toggling, ordering, and snooze behavior out of `src-tauri/src/lib.rs`.
- Keep Tauri command names and payloads unchanged.

Files changed:

- `src-tauri/src/lib.rs`
- `src-tauri/src/services/mod.rs`
- `src-tauri/src/services/reminders.rs`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `normalize_repeat_rule`, `normalize_weekly_days`, `weekly_days_to_db`, `parse_weekly_days_db`, `next_weekly_due_timestamp` | `src-tauri/src/services/reminders.rs` | Behavior unchanged; core weekly helpers now covered by characterization tests |
| `collect_reminders` | `src-tauri/src/services/reminders.rs::collect_reminders` | Behavior unchanged; private service helper |
| `list_reminders` body | `src-tauri/src/services/reminders.rs::list_reminder_entries` | Command shell remains in `lib.rs` |
| `list_due_reminders` body | `src-tauri/src/services/reminders.rs::list_due_reminder_entries` | Command shell remains in `lib.rs` |
| `save_reminder` body | `src-tauri/src/services/reminders.rs::save_reminder_entry` | Command shell remains in `lib.rs`; validation strings preserved |
| `delete_reminder` body | `src-tauri/src/services/reminders.rs::delete_reminder_entry` | Command shell remains in `lib.rs` |
| `set_reminder_done` body | `src-tauri/src/services/reminders.rs::set_reminder_done_entry` | Command shell remains in `lib.rs` |
| `set_reminder_order` body | `src-tauri/src/services/reminders.rs::set_reminder_order_entries` | Command shell remains in `lib.rs` |
| `snooze_reminder` body | `src-tauri/src/services/reminders.rs::snooze_reminder_entry` | Command shell remains in `lib.rs` |

Behavior expected to stay the same:

- Reminder repeat rules and validation errors stay the same.
- Daily and weekly due-time calculations stay the same.
- Completed recurring reminders still use business-day keys.
- Sort ordering, snooze limits, query caps, and list ordering stay the same.

Behavior intentionally changed:

- None.

Tauri commands affected:

- `list_reminders`, `list_due_reminders`, `save_reminder`, `delete_reminder`, `set_reminder_done`, `set_reminder_order`, and `snooze_reminder` now delegate to `services::reminders`.
- Command names, inputs, outputs, and registration unchanged.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `cargo test` from `src-tauri` passed: 15 tests passed.
- `cargo check` from `src-tauri` passed.

Manual smoke tests:

- Not run for this extraction batch.

Risks:

- Business-day helpers are temporarily `pub(crate)` from `lib.rs`; a later analytics/time utility extraction should give them a proper home.
- Full reminder UI smoke remains valuable before release.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-106 commit.

Follow-up:

- Commit R-107.
- Continue Phase 2 with analytics service extraction.

## 2026-06-08: R-108 Backend Analytics Service Extraction

Status:

- Passed.

Primary domain:

- analytics

Intent:

- Move recent-log, today-summary, top-apps-today, and top-apps-all-time query bodies out of `src-tauri/src/lib.rs`.
- Keep heatmap and usage-stack extraction for a later, focused analytics batch.
- Keep Tauri command names and payloads unchanged.

Files changed:

- `src-tauri/src/lib.rs`
- `src-tauri/src/services/mod.rs`
- `src-tauri/src/services/analytics.rs`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `src-tauri/src/lib.rs::list_recent_logs` query body | `src-tauri/src/services/analytics.rs::list_recent_log_entries` | Command shell remains in `lib.rs` |
| `src-tauri/src/lib.rs::get_today_summary` query body | `src-tauri/src/services/analytics.rs::today_summary` | Command shell remains in `lib.rs` |
| `src-tauri/src/lib.rs::list_top_apps_today` query body | `src-tauri/src/services/analytics.rs::list_top_apps_today_entries` | Command shell remains in `lib.rs` |
| `src-tauri/src/lib.rs::list_top_apps_all_time` query body | `src-tauri/src/services/analytics.rs::list_top_apps_all_time_entries` | Command shell remains in `lib.rs` |

Behavior expected to stay the same:

- Business-day windows, query caps, filters, ordering, and error strings stay the same.
- `src-tauri/src/lib.rs` command signatures and registration stay the same.
- Heatmap and usage-stack behavior are untouched in this batch.

Behavior intentionally changed:

- None.

Tauri commands affected:

- `list_recent_logs`, `get_today_summary`, `list_top_apps_today`, and `list_top_apps_all_time` now delegate to `services::analytics`.
- Command names, inputs, outputs, and registration unchanged.

Database/schema impact:

- None.

Privacy impact:

- None intended. Queries continue to read already privacy-processed logs.

Startup/performance impact:

- None expected.

Automated validation:

- `cargo check` from `src-tauri` passed.
- `cargo test` from `src-tauri` passed: 15 tests passed.

Manual smoke tests:

- Not run for this extraction batch.

Risks:

- Heatmap and usage-stack remain in `lib.rs`; they should be moved in the next analytics batch with focused validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-107 commit.

Follow-up:

- Commit R-108.
- Continue analytics extraction with heatmap and usage stack.

## 2026-06-08: R-109 Backend Heatmap And Usage Stack Extraction

Status:

- Passed.

Primary domain:

- analytics

Intent:

- Move heatmap, heatmap-goal setting, and usage-stack command bodies into `services::analytics`.
- Preserve current heatmap snapshot behavior, usage-stack bucketing, query caps, filters, placeholders, and command contracts.

Files changed:

- `src-tauri/src/lib.rs`
- `src-tauri/src/services/analytics.rs`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `src-tauri/src/lib.rs::get_learn_heatmap` body | `src-tauri/src/services/analytics.rs::learn_heatmap` | Command shell remains in `lib.rs`; historical no-snapshot behavior preserved |
| `src-tauri/src/lib.rs::get_heatmap_goal_seconds_setting` body | `src-tauri/src/services/analytics.rs::heatmap_goal_seconds_setting` | Command shell remains in `lib.rs` |
| `src-tauri/src/lib.rs::set_heatmap_goal_seconds_setting` body | `src-tauri/src/services/analytics.rs::set_heatmap_goal_seconds_setting` | Command shell remains in `lib.rs`; error string preserved |
| `src-tauri/src/lib.rs::get_usage_stack` body | `src-tauri/src/services/analytics.rs::usage_stack` | Command shell remains in `lib.rs` |

Behavior expected to stay the same:

- Heatmap still seals historical snapshots before reading.
- Historical heatmap days without a snapshot still return gray/zero cells.
- Current-day heatmap still recomputes learn seconds.
- Heatmap goal setting clamp stays `0..=86400`.
- Usage stack still buckets by business day, reverses latest day first, and uses the same root filter validation.

Behavior intentionally changed:

- None.

Tauri commands affected:

- `get_learn_heatmap`, `get_heatmap_goal_seconds_setting`, `set_heatmap_goal_seconds_setting`, and `get_usage_stack` now delegate to `services::analytics`.
- Command names, inputs, outputs, and registration unchanged.

Database/schema impact:

- None.

Privacy impact:

- None intended. Queries continue to read already privacy-processed logs.

Startup/performance impact:

- None expected.

Automated validation:

- `cargo check` from `src-tauri` passed.
- `cargo test` from `src-tauri` passed: 15 tests passed.

Manual smoke tests:

- Not run for this extraction batch.

Risks:

- Shared time/interval helpers are still in `lib.rs` with `pub(crate)` visibility; they should move to a small utility module in a follow-up cleanup batch.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-108 commit.

Follow-up:

- Commit R-109.
- Continue Phase 2 with foreground/capture service extraction or shared time helper cleanup.

## 2026-06-09: R-110 Shared Time Helper Extraction

Status:

- Passed.

Primary domain:

- backend shared services
- analytics
- reminders

Intent:

- Move business-day and interval helper functions out of `src-tauri/src/lib.rs`.
- Keep analytics-specific heatmap helper functions private to `services::analytics`.
- Preserve command names, payloads, SQL behavior, reminder day-key behavior, and heatmap bucketing.

Files changed:

- `src-tauri/src/lib.rs`
- `src-tauri/src/services/mod.rs`
- `src-tauri/src/services/time.rs`
- `src-tauri/src/services/analytics.rs`
- `src-tauri/src/services/reminders.rs`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `src-tauri/src/lib.rs::business_day_start_from_local` | `src-tauri/src/services/time.rs::business_day_start_from_local` | Same 04:00 business-day boundary |
| `src-tauri/src/lib.rs::business_day_window_from_local` | `src-tauri/src/services/time.rs::business_day_window_from_local` | Same 86,400-second window |
| `src-tauri/src/lib.rs::business_day_start_for_timestamp` | `src-tauri/src/services/time.rs::business_day_start_for_timestamp` | Used by analytics bucketing |
| `src-tauri/src/lib.rs::business_day_key_from_start` | `src-tauri/src/services/time.rs::business_day_key_from_start` | Used by analytics and reminders |
| `src-tauri/src/lib.rs::overlap_seconds` | `src-tauri/src/services/time.rs::overlap_seconds` | Shared interval clipping helper |
| `src-tauri/src/lib.rs::merge_intervals_total` | `src-tauri/src/services/time.rs::merge_intervals_total` | Shared interval total helper |
| `src-tauri/src/lib.rs::parse_i64_config` | `src-tauri/src/services/analytics.rs::parse_i64_config` | Currently only needed by heatmap settings |
| `src-tauri/src/lib.rs::compute_learn_seconds_for_window` | `src-tauri/src/services/analytics.rs::compute_learn_seconds_for_window` | Kept private to analytics |
| `src-tauri/src/lib.rs::seal_historical_heatmap_snapshot` | `src-tauri/src/services/analytics.rs::seal_historical_heatmap_snapshot` | Kept private to analytics |

Behavior expected to stay the same:

- Business days still start at local 04:00.
- Heatmap and usage-stack windows still use the same local business-day timestamps.
- Reminder completion day keys still use the same business-day key helper.
- Interval merging and overlap calculations are unchanged.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None directly. Existing command shells continue to call service functions with unchanged names and payloads.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `cargo check` from `src-tauri` passed.
- `cargo test` from `src-tauri` passed: 17 tests passed.

Manual smoke tests:

- Not run for this extraction batch.

Risks:

- Time helpers still depend on `chrono::Local`, so business-day behavior is tied to the host local timezone as before.
- Day length remains hard-coded to 86,400 seconds; this preserves current behavior but does not model daylight-saving-time day length changes.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-109 commit.

Follow-up:

- Commit R-110.
- Continue Phase 2 with foreground/capture extraction or command-shell thinning.

## 2026-06-09: R-111 Foreground Platform Primitive Extraction

Status:

- Passed.

Primary domain:

- foreground capture
- backend platform services

Intent:

- Move low-level foreground-window and idle-time platform wrappers out of `src-tauri/src/lib.rs`.
- Keep `capture_foreground_once` control flow, diagnostics, idle prompts, privacy processing, and command contract unchanged.

Files changed:

- `src-tauri/src/lib.rs`
- `src-tauri/src/services/mod.rs`
- `src-tauri/src/services/foreground.rs`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| Windows API imports in `src-tauri/src/lib.rs` | `src-tauri/src/services/foreground.rs` | `windows_sys` usage is now localized |
| `src-tauri/src/lib.rs::capture_foreground_window` | `src-tauri/src/services/foreground.rs::capture_foreground_window` | Same Windows behavior and same non-Windows error |
| `src-tauri/src/lib.rs::current_idle_millis` | `src-tauri/src/services/foreground.rs::current_idle_millis` | Same Windows idle calculation and same non-Windows fallback |

Behavior expected to stay the same:

- Foreground capture still returns the same process/title tuple, including `pid-{pid}.exe` fallback.
- Empty titles still become `Untitled Window`.
- Non-Windows foreground capture still returns the same unsupported error.
- Non-Windows idle time still returns `0`.
- `capture_foreground_once` still owns sampling state, idle prompt creation, privacy processing, and diagnostics.

Behavior intentionally changed:

- None.

Tauri commands affected:

- `capture_foreground_once` now calls `services::foreground` for platform primitives.
- Command name, input, return shape, diagnostics, and persistence behavior unchanged.

Database/schema impact:

- None.

Privacy impact:

- None intended. Privacy filtering still happens after capture in the existing flow.

Startup/performance impact:

- None expected.

Automated validation:

- `cargo check` from `src-tauri` passed.
- `cargo test` from `src-tauri` passed: 17 tests passed.

Manual smoke tests:

- Not run for this extraction batch.

Risks:

- This batch does not yet separate foreground sampling state or idle prompt coordination; those remain in `lib.rs`.
- Platform behavior is validated by compile/tests, not by a manual Windows foreground smoke test in this batch.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-110 commit.

Follow-up:

- Commit R-111.
- Continue foreground extraction by moving sampling-state helpers or idle prompt coordination in a small follow-up batch.

## 2026-06-09: R-112 Startup Service Extraction

Status:

- Passed.

Primary domain:

- startup integration
- backend platform services

Intent:

- Move Windows startup registry query/update logic out of `src-tauri/src/lib.rs`.
- Keep Tauri command names, return shapes, Windows registry path/value, error strings, and non-Windows fallbacks unchanged.

Files changed:

- `src-tauri/src/lib.rs`
- `src-tauri/src/services/mod.rs`
- `src-tauri/src/services/startup.rs`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `src-tauri/src/lib.rs::WINDOWS_RUN_REGISTRY_PATH` | `src-tauri/src/services/startup.rs::WINDOWS_RUN_REGISTRY_PATH` | Private Windows-only constant |
| `src-tauri/src/lib.rs::WINDOWS_RUN_VALUE_NAME` | `src-tauri/src/services/startup.rs::WINDOWS_RUN_VALUE_NAME` | Private Windows-only constant |
| `src-tauri/src/lib.rs::query_auto_start_enabled_internal` | `src-tauri/src/services/startup.rs::get_auto_start_enabled_state` | Same `reg query` behavior |
| `src-tauri/src/lib.rs::set_auto_start_enabled_internal` | `src-tauri/src/services/startup.rs::set_auto_start_enabled_state` | Same `reg add/delete` behavior |

Behavior expected to stay the same:

- Windows startup value remains `TimePrism`.
- Windows registry path remains `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
- Enabling startup still writes the current executable path as a quoted `REG_SZ`.
- Disabling startup still returns `false` if the delete command reports a non-success status.
- Non-Windows commands still return `false`.

Behavior intentionally changed:

- None.

Tauri commands affected:

- `get_auto_start_enabled` now delegates to `services::startup`.
- `set_auto_start_enabled` now delegates to `services::startup`.
- Command names, inputs, outputs, and registration unchanged.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `cargo check` from `src-tauri` passed.
- `cargo test` from `src-tauri` passed: 17 tests passed.

Manual smoke tests:

- Not run. This batch did not execute registry-writing commands.

Risks:

- Registry behavior is compile-validated only in this batch; manual Windows startup toggling should be smoke-tested before release packaging.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-111 commit.

Follow-up:

- Commit R-112.
- Continue with foreground sampling-state extraction or window service extraction.

## 2026-06-09: R-113 Window Service Extraction

Status:

- Passed.

Primary domain:

- window management
- pet window
- pet panel

Intent:

- Move main-window, pet-window, and pet-panel behavior out of `src-tauri/src/lib.rs`.
- Keep Tauri command names and payloads unchanged while making command bodies thin delegates.
- Add focused pure tests around pet settle mode normalization and monitor-bound clamping.

Files changed:

- `src-tauri/src/lib.rs`
- `src-tauri/src/services/mod.rs`
- `src-tauri/src/services/window.rs`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `set_main_close_behavior` | `services::window::set_main_close_behavior` | Setup and recreated main windows use the same close-to-hide behavior |
| `reveal_main_window` | `services::window::reveal_main_window` | Same unminimize/center/show/focus sequence |
| `ensure_pet_window_position` | `services::window::ensure_pet_window_position` | Same corner fallback and always-on-top behavior |
| Pet monitor/clamp/settle helpers | `services::window` private helpers | Same mode names, size, and clamping rules |
| Pet panel positioning helper | `services::window` private helper | Same side-flip and bounds clamping |
| Main/pet/panel command bodies | `services::window` service functions | Command shells remain in `lib.rs` |

Behavior expected to stay the same:

- Main window is still hidden instead of closed.
- Startup setup still reveals the main window and positions the pet window.
- Global shortcut still summons the pet window.
- Pet and panel labels, URLs, titles, sizes, transparency, taskbar behavior, and events are unchanged.
- Pet settle modes remain `free`, `dock_left`, and `dock_right`.
- Pet panel resize clamps remain width `150..=300` and height `96..=260`.

Behavior intentionally changed:

- None.

Tauri commands affected:

- `show_main_window`, `show_main_window_section`, `summon_pet_window`, `sync_pet_window_layout`, `settle_pet_window`, `show_pet_panel`, `hide_pet_panel`, `sync_pet_panel_position`, `resize_pet_panel`, `move_pet_window`, `hide_pet_window`, `close_pet_window`, and `begin_pet_drag` now delegate to `services::window`.
- Command names, inputs, outputs, emitted event names, and registration unchanged.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `cargo check` from `src-tauri` passed.
- `cargo test` from `src-tauri` passed: 19 tests passed.

Manual smoke tests:

- Not run. This batch did not launch the Tauri app or manually move windows.

Risks:

- Tauri window behavior is compile-tested but not manually smoke-tested in this batch.
- The service still depends directly on Tauri APIs, which is appropriate for the current extraction phase but limits pure unit coverage.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-112 commit.

Follow-up:

- Commit R-113.
- Continue foreground sampling-state extraction or begin command module thinning.

## 2026-06-09: R-114 Category Service Extraction

Status:

- Passed.

Primary domain:

- categories
- backend services

Intent:

- Move category list/create SQL out of `src-tauri/src/lib.rs`.
- Keep category command names, inputs, outputs, ordering, default colors, and error strings unchanged.

Files changed:

- `src-tauri/src/lib.rs`
- `src-tauri/src/services/mod.rs`
- `src-tauri/src/services/categories.rs`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `list_categories` SQL body | `services::categories::list_category_entries` | Command shell remains in `lib.rs` |
| `create_category` SQL body | `services::categories::create_category_entry` | Command shell remains in `lib.rs` |

Behavior expected to stay the same:

- Categories are still ordered by `id ASC`.
- Created child categories still inherit the parent `root_type`.
- Default category colors remain `#4ade80` for `LEARN` and `#fb923c` otherwise.
- Missing parents still return `parent category not found`.

Behavior intentionally changed:

- None.

Tauri commands affected:

- `list_categories` and `create_category` now delegate to `services::categories`.
- Command names, inputs, outputs, and registration unchanged.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `cargo check` from `src-tauri` passed.
- `cargo test` from `src-tauri` passed: 19 tests passed.

Manual smoke tests:

- Not run for this extraction batch.

Risks:

- No category-specific database characterization test was added in this batch; behavior is preserved by direct extraction and compile/test validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-113 commit.

Follow-up:

- Commit R-114.
- Continue extracting task-session and app-usage-log service logic.

## 2026-06-09: R-115 Task Session Service Extraction

Status:

- Passed.

Primary domain:

- task sessions
- backend services

Intent:

- Move legacy/manual task-session SQL out of `src-tauri/src/lib.rs`.
- Keep start/stop session commands and focus-deviation active-root lookup behavior unchanged.

Files changed:

- `src-tauri/src/lib.rs`
- `src-tauri/src/services/mod.rs`
- `src-tauri/src/services/sessions.rs`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `active_root_type` | `services::sessions::active_root_type` | Still used by `check_focus_deviation` |
| `start_session` SQL body | `services::sessions::start_session_entry` | Command shell remains in `lib.rs` |
| `stop_active_session` SQL body | `services::sessions::stop_active_session_entry` | Command shell remains in `lib.rs` |

Behavior expected to stay the same:

- Starting a session still closes all currently active sessions first.
- New sessions still use `is_flow_target = 0`.
- Stopping a session still returns whether any active row was changed.
- Focus-deviation checks still read the latest active session root type.

Behavior intentionally changed:

- None.

Tauri commands affected:

- `start_session` and `stop_active_session` now delegate to `services::sessions`.
- `check_focus_deviation` now reads active root type through `services::sessions`.
- Command names, inputs, outputs, and registration unchanged.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `cargo check` from `src-tauri` passed.
- `cargo test` from `src-tauri` passed: 19 tests passed.

Manual smoke tests:

- Not run for this extraction batch.

Risks:

- No task-session-specific database characterization test was added in this batch; behavior is preserved by direct extraction and compile/test validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-114 commit.

Follow-up:

- Commit R-115.
- Continue extracting app-usage-log append/privacy integration or foreground sampling state.

## 2026-06-09: R-116 Usage Log Service Extraction

Status:

- Passed.

Primary domain:

- app usage logs
- idle prompt persistence
- privacy integration

Intent:

- Move app usage log append/merge helpers out of `src-tauri/src/lib.rs`.
- Keep privacy processing, idle decision persistence, and command behavior unchanged.
- Preserve existing idle segment titles exactly, including `Idle Segment · Learn`, `Idle Segment · Rest`, and `Idle Segment · Unclassified`.

Files changed:

- `src-tauri/src/lib.rs`
- `src-tauri/src/services/mod.rs`
- `src-tauri/src/services/usage.rs`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `append_usage_log_direct` | `services::usage` private helper | Same contiguous segment merge behavior |
| `append_usage_log_record` | `services::usage::append_usage_log_record` | Still applies `services::privacy::process_log_with_privacy` before persistence |
| `persist_idle_prompt_decision` | `services::usage::persist_idle_prompt_decision` | Still upserts idle pseudo-rules before writing a usage log |

Behavior expected to stay the same:

- Negative durations are still clamped to zero before persistence.
- Contiguous logs with the same process/title still extend the latest row.
- Privacy-filtered logs still return `(false, block_reason)` without writing.
- Idle decisions still map `LEARN`, `REST`, and `IDLE` to the same pseudo-processes and titles.

Behavior intentionally changed:

- None.

Tauri commands affected:

- `append_app_usage_log`, `capture_foreground_once`, and `resolve_idle_prompt` now delegate usage persistence to `services::usage`.
- Command names, inputs, outputs, and registration unchanged.

Database/schema impact:

- None.

Privacy impact:

- None intended. Existing privacy processing is still called before app usage log persistence.

Startup/performance impact:

- None expected.

Automated validation:

- `cargo check` from `src-tauri` passed.
- `cargo test` from `src-tauri` passed: 19 tests passed.

Manual smoke tests:

- Not run for this extraction batch.

Risks:

- No usage-log-specific database characterization test was added in this batch; behavior is preserved by direct extraction and compile/test validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-115 commit.

Follow-up:

- Commit R-116.
- Continue foreground sampling-state extraction or move whitelist/privacy commands fully behind service functions.

## 2026-06-09: R-117 Privacy Settings And Whitelist Extraction

Status:

- Passed.

Primary domain:

- privacy
- whitelist
- backend services

Intent:

- Move privacy settings read/write and whitelist SQL out of `src-tauri/src/lib.rs`.
- Keep privacy command contracts, config keys, validation, and error strings unchanged.

Files changed:

- `src-tauri/src/lib.rs`
- `src-tauri/src/services/privacy.rs`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `get_privacy_settings` body | `services::privacy::get_privacy_settings_entry` | Command shell remains in `lib.rs` |
| `update_privacy_settings` body | `services::privacy::update_privacy_settings_entry` | Same transaction and config key writes |
| `list_whitelist` body | `services::privacy::list_whitelist_entries` | Same ordering and error strings |
| `set_whitelist_item` body | `services::privacy::set_whitelist_item_entry` | Same normalization and add/remove behavior |

Behavior expected to stay the same:

- Browser title mode validation remains `FULL`, `BLUR`, or `NONE`.
- `browser_blur_enabled` compatibility key is still written from browser title mode.
- Whitelist entries are still normalized through `normalize_process_key`.
- Whitelist listing is still ordered by process name.

Behavior intentionally changed:

- None.

Tauri commands affected:

- `get_privacy_settings`, `update_privacy_settings`, `list_whitelist`, and `set_whitelist_item` now delegate to `services::privacy`.
- Command names, inputs, outputs, and registration unchanged.

Database/schema impact:

- None.

Privacy impact:

- No behavior change intended; this batch centralizes privacy persistence code.

Startup/performance impact:

- None expected.

Automated validation:

- `cargo check` from `src-tauri` passed.
- `cargo test` from `src-tauri` passed: 19 tests passed.

Manual smoke tests:

- Not run for this extraction batch.

Risks:

- Settings write behavior is covered by compile/test validation but not by a dedicated settings round-trip test in this batch.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-116 commit.

Follow-up:

- Commit R-117.
- Continue foreground sampling-state extraction or focus-deviation service extraction.

## 2026-06-09: R-118 Focus Deviation Service Extraction

Status:

- Passed.

Primary domain:

- focus guard
- deviation detection
- backend services

Intent:

- Move focus deviation state and debounce/cooldown logic out of `src-tauri/src/lib.rs`.
- Keep focus guard command contracts, reasons, debounce defaults, cooldown defaults, and suggested category behavior unchanged.

Files changed:

- `src-tauri/src/lib.rs`
- `src-tauri/src/services/mod.rs`
- `src-tauri/src/services/focus.rs`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `DeviationState` and `DEVIATION_STATE` | `services::focus` | Same in-memory lifetime |
| `check_focus_deviation` body | `services::focus::check_focus_deviation_state` | Command shell remains in `lib.rs` |
| `snooze_focus_guard` body | `services::focus::snooze_focus_guard_state` | Command shell remains in `lib.rs` |

Behavior expected to stay the same:

- Debounce default remains `60` seconds and clamps to `5..=600`.
- Snooze default remains `900` seconds and clamps to `60..=7200`.
- No active session still clears pending/mismatch state and returns `no_active_session`.
- `IGNORE` or matching mapped type still returns `matched_or_ignored`.
- Suggested category IDs remain `1` for `LEARN` and `2` otherwise.

Behavior intentionally changed:

- None.

Tauri commands affected:

- `check_focus_deviation` and `snooze_focus_guard` now delegate to `services::focus`.
- Command names, inputs, outputs, reasons, and registration unchanged.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `cargo check` from `src-tauri` passed.
- `cargo test` from `src-tauri` passed: 19 tests passed.

Manual smoke tests:

- Not run for this extraction batch.

Risks:

- Focus deviation state remains in memory as before; this batch does not change restart behavior.
- No dedicated focus-guard state-machine characterization test was added in this batch.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-117 commit.

Follow-up:

- Commit R-118.
- Continue foreground sampling-state extraction or command module thinning.

## 2026-06-09: R-119 Foreground Sampling And Idle Service Extraction

Status:

- Passed.

Primary domain:

- foreground capture
- idle prompts
- diagnostics

Intent:

- Move foreground sampling state, diagnostics, idle prompt queue, idle memory, and foreground capture command bodies out of `src-tauri/src/lib.rs`.
- Keep existing platform capture wrappers in `services::foreground`.
- Keep Tauri command names, payloads, diagnostics, and idle prompt behavior unchanged.

Files changed:

- `src-tauri/src/lib.rs`
- `src-tauri/src/services/foreground.rs`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `ForegroundSampleState`, `ForegroundSnapshot`, and `FOREGROUND_SAMPLE_STATE` | `services::foreground` | Same in-memory lifetime |
| `push_foreground_diagnostic` | `services::foreground` private helper | Same 60-entry cap |
| `capture_foreground_once` body | `services::foreground::capture_foreground_once` | Command shell remains in `lib.rs` |
| `list_pending_idle_prompts` body | `services::foreground::list_pending_idle_prompts` | Same cap and sorting |
| `resolve_idle_prompt` body | `services::foreground::resolve_idle_prompt` | Same decisions, skip defer, and remember-this-session behavior |
| `get_idle_memory_state` body | `services::foreground::get_idle_memory_state` | Same returned memory shape |
| `clear_idle_memory_state` body | `services::foreground::clear_idle_memory_state` | Same clear behavior |
| `list_foreground_capture_diagnostics` body | `services::foreground::list_foreground_capture_diagnostics` | Same dedupe and priority sort |

Behavior expected to stay the same:

- Idle threshold remains `300_000` ms.
- Idle prompts are still capped at 20 pending entries.
- Diagnostics are still capped at 60 entries.
- First foreground sample still establishes a baseline only.
- Foreground elapsed span still clamps to `500..=60000` ms.
- Diagnostic sorting still prioritizes unsaved rules, then unstored logs, then recency.

Behavior intentionally changed:

- None.

Tauri commands affected:

- `capture_foreground_once`, `list_pending_idle_prompts`, `resolve_idle_prompt`, `get_idle_memory_state`, `clear_idle_memory_state`, and `list_foreground_capture_diagnostics` now delegate to `services::foreground`.
- Command names, inputs, outputs, and registration unchanged.

Database/schema impact:

- None.

Privacy impact:

- None intended. Foreground persistence still calls `services::usage`, which still applies privacy processing.

Startup/performance impact:

- None expected.

Automated validation:

- `cargo check` from `src-tauri` passed.
- `cargo test` from `src-tauri` passed: 19 tests passed.

Manual smoke tests:

- Not run. This batch did not manually sample foreground windows or resolve idle prompts in the running app.

Risks:

- Foreground and idle prompt state remain in memory as before; this batch does not change restart behavior.
- Foreground behavior is compile/test validated but still needs a manual runtime smoke test before release.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-118 commit.

Follow-up:

- Commit R-119.
- Continue with command module thinning or a phase gate review for backend extraction.

## 2026-06-09: Phase 2 Gate - Backend Domain And Service Extraction

State:

- Passed with manual smoke still pending.

Scope completed:

- Backend database connection and migrations are split from `src-tauri/src/lib.rs`.
- Backend DTO/domain structs are grouped under `src-tauri/src/domain/`.
- Backend service modules now own privacy, rules, reminders, analytics, foreground sampling, focus deviation, startup, window management, categories, task sessions, usage logs, and shared time helpers.
- `src-tauri/src/lib.rs` is now primarily Tauri command wrappers, app setup, command registration, and global shortcut wiring.
- Existing Tauri command names, registrations, input shapes, and return shapes were preserved.

Scope intentionally not completed:

- Command wrappers have not yet moved into feature command modules.
- Frontend feature/component extraction is not part of this gate.
- Manual runtime smoke tests were not run during this automated pass.
- In-memory foreground/idle/focus state remains in memory as before; restart persistence remains a product decision.

Files changed:

- No code files changed in this gate record.
- `docs/REFACTOR_LOG.md`

Commands affected:

- No new command contract changes in this gate record.
- Phase 2 extraction affected command internals only; command names and registration remained unchanged.

Schema/privacy/startup impact:

- No schema changes in this gate record.
- Privacy behavior expected unchanged after service extraction.
- Startup registry behavior expected unchanged after service extraction.

Automated validation:

- `cargo check` from `src-tauri` passed.
- `cargo test` from `src-tauri` passed: 19 tests passed.
- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.

Manual smoke tests:

- Not run.
- Recommended before release/merge: S-400 Reminders, S-500 Guard, S-600 Privacy And Capture, plus a pet/main-window smoke after window service extraction.

Known risks:

- Runtime foreground sampling, idle prompt resolution, pet window movement, privacy settings writes, and startup toggling still need manual smoke coverage.
- `dist-codex-check/` was generated by `pnpm.cmd run build:check`; it is ignored by git. Cleanup command was blocked by safety review, so the ignored directory may remain locally.

Rollback point:

- Revert to R-119 or reset `refactor/architecture` to commit `463f7ce`.

Next phase:

- Phase 3/command-shell thinning: move command wrappers out of `src-tauri/src/lib.rs` into feature command modules, then continue frontend shell/API cleanup.

## 2026-06-09: R-120 Frontend API Command Wrapper Split

Status:

- Passed.

Primary domain:

- frontend API

Intent:

- Split frontend Tauri command wrapper functions out of the broad `src/api.ts` compatibility file.
- Keep all existing frontend import paths working through `src/api.ts`.
- Preserve command names, argument casing, defaults, and TypeScript return types.

Files changed:

- `src/api.ts`
- `src/api/commands/index.ts`
- `src/api/commands/analytics.ts`
- `src/api/commands/categories.ts`
- `src/api/commands/foreground.ts`
- `src/api/commands/privacy.ts`
- `src/api/commands/reminders.ts`
- `src/api/commands/rules.ts`
- `src/api/commands/sessions.ts`
- `src/api/commands/startup.ts`
- `src/api/commands/usage.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| Analytics API wrappers | `src/api/commands/analytics.ts` | Summary, top apps, heatmap, usage stack, recent logs |
| Category API wrappers | `src/api/commands/categories.ts` | List/create categories |
| Foreground/idle API wrappers | `src/api/commands/foreground.ts` | Capture diagnostics and idle prompts |
| Privacy API wrappers | `src/api/commands/privacy.ts` | Settings and whitelist |
| Reminder API wrappers | `src/api/commands/reminders.ts` | Reminder CRUD, done, snooze, ordering |
| Rule/focus API wrappers | `src/api/commands/rules.ts` | App rules, pending processes, focus guard |
| Session API wrappers | `src/api/commands/sessions.ts` | Start/stop task session |
| Startup API wrappers | `src/api/commands/startup.ts` | Auto-start get/set |
| Usage API wrappers | `src/api/commands/usage.ts` | Manual app usage append |

Behavior expected to stay the same:

- Existing imports from `src/api.ts` continue to work.
- Type exports from `src/api.ts` continue to work.
- All wrapper function names and defaults are unchanged.
- Tauri command names and payload casing are unchanged.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None on the backend. This is a frontend wrapper organization change only.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- Build output now has smaller command-wrapper chunks, but runtime behavior is expected to stay the same.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.

Manual smoke tests:

- Not run for this extraction batch.

Risks:

- `dist-codex-check/` is generated and ignored by git; cleanup remains blocked by safety review in this session.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the Phase 2 gate commit.

Follow-up:

- Commit R-120.
- Continue frontend shell extraction by reducing `src/App.vue` responsibilities and replacing broad `ctx: any` component contracts.

## 2026-06-09: R-121 Locale Composable Extraction

Status:

- Passed.

Primary domain:

- frontend shell
- locale

Intent:

- Move main-window locale state, translation helper, document language sync, and localStorage persistence out of `src/App.vue`.
- Preserve existing locale behavior and existing child-component context shape.

Files changed:

- `src/App.vue`
- `src/composables/useLocale.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `LocaleCode` | `src/composables/useLocale.ts` | Exported for future reuse |
| `LOCALE_STORAGE_KEY` | `src/composables/useLocale.ts` | Same key: `timeprism-locale` |
| `locale` ref | `src/composables/useLocale.ts` | Same default: `zh-CN` |
| `tx`, `applyLocale`, `initLocale` | `src/composables/useLocale.ts` | Same behavior and error swallowing |

Behavior expected to stay the same:

- Stored `zh-CN` or `en-US` locale still wins.
- Browser language still initializes Chinese when it starts with `zh`.
- `document.documentElement.lang` is still updated on locale changes.
- LocalStorage failures are still ignored.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.

Manual smoke tests:

- Not run for this extraction batch.

Risks:

- Main window locale behavior was not manually toggled in a running app during this batch.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-120 commit.

Follow-up:

- Commit R-121.
- Continue extracting App shell state into focused composables.

## 2026-06-09: R-122 Theme Mode Composable Extraction

Status:

- Passed.

Primary domain:

- frontend shell
- theme

Intent:

- Move main-window theme state, body theme attribute sync, system preference fallback, and localStorage persistence out of `src/App.vue`.
- Preserve existing settings UI behavior and template bindings.

Files changed:

- `src/App.vue`
- `src/composables/useThemeMode.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `ThemeMode` | `src/composables/useThemeMode.ts` | Exported for future reuse |
| `themeMode` ref | `src/composables/useThemeMode.ts` | Same default: `light` |
| `applyTheme`, `toggleThemeMode`, `initThemeMode` | `src/composables/useThemeMode.ts` | Same localStorage key and system fallback |

Behavior expected to stay the same:

- Stored `light` or `dark` theme still wins.
- System `prefers-color-scheme: dark` still selects dark when no stored setting exists.
- `document.body[data-theme]` is still updated when applying a theme.
- LocalStorage failures are still ignored.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.

Manual smoke tests:

- Not run for this extraction batch.

Risks:

- Theme toggling was not manually smoke-tested in a running app during this batch.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-121 commit.

Follow-up:

- Commit R-122.
- Continue extracting App shell state into focused composables.

## 2026-06-09: R-123 Frontend Time Helper Extraction

Status:

- Passed.

Primary domain:

- frontend shell
- shared frontend helpers

Intent:

- Move pure frontend time/date formatting and parsing helpers out of `src/App.vue`.
- Start the `src/lib` shared helper area described in Phase 3 without changing UI behavior.

Files changed:

- `src/App.vue`
- `src/lib/time.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `formatSeconds` | `src/lib/time.ts` | Same `HH:mm:ss` output and non-negative floor behavior |
| `timeMinutesLabel` | `src/lib/time.ts` | Same `HH:mm` output |
| `parseDateTimeLocalToUnix` | `src/lib/time.ts` | Same invalid/empty `null` behavior |
| `toDateTimeLocalValue` | `src/lib/time.ts` | Same local datetime input value format |
| `parseClockToMinutes` | `src/lib/time.ts` | Same `HH:mm` validation and minute conversion |
| `localDayKeyFromDate`, `currentLocalDayKey`, `currentLocalWeekday`, `dayKeyFromUnixSeconds` | `src/lib/time.ts` | Same local date semantics |

Behavior expected to stay the same:

- Reminder date/time inputs still parse and render the same way.
- Home calendar and today visibility checks still use local dates.
- Duration labels still render as `HH:mm:ss`.
- No locale-dependent formatter was moved in this batch.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.

Manual smoke tests:

- Not run for this helper extraction batch.

Risks:

- Calendar/reminder behavior was validated by typecheck/build only, not by interactive smoke testing.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-122 commit.

Follow-up:

- Commit R-123.
- Continue Phase 3 with navigation shell extraction or typed feature context preparation.

## 2026-06-09: R-124 App Navigation Composable Extraction

Status:

- Passed.

Primary domain:

- frontend shell
- navigation

Intent:

- Move main-window navigation refs, tab label metadata, and insights history subview state out of `src/App.vue`.
- Keep data refresh and settings warm-up side effects in `src/App.vue` so the new composable remains state-only.

Files changed:

- `src/App.vue`
- `src/composables/useAppNavigation.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `MainViewKey`, `InsightsPrimaryViewKey`, `HistorySubViewKey` | `src/composables/useAppNavigation.ts` | Exported for `App.vue` handlers |
| `currentMainView`, `mainViews` | `src/composables/useAppNavigation.ts` | Same default main view: `home` |
| `insightsPrimaryView`, `historySubView`, `historySubViews` | `src/composables/useAppNavigation.ts` | Same default history subview: `topApps` |
| navigation state mutations | `src/composables/useAppNavigation.ts` | Refresh side effects remain in `App.vue` |

Behavior expected to stay the same:

- Main navigation still starts on Home.
- Entering Insights from the top nav still resets the history subview to Today's Duration when already in the history section.
- Insights subnav and history subnav still trigger the same data refreshes.
- Guard shortcut still opens the Guard view and refreshes guard data.
- Settings view still mounts and refreshes from `App.vue`.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.

Manual smoke tests:

- Not run for this shell extraction batch.

Risks:

- Navigation behavior was not interactively smoke-tested in a running app during this batch.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-123 commit.

Follow-up:

- Commit R-124.
- Continue Phase 3 by preparing typed view context contracts or extracting another state-only shell composable.

## 2026-06-09: R-125 Guard View Context Typing

Status:

- Passed.

Primary domain:

- frontend shell
- typed component contracts
- guard

Intent:

- Replace the existing `ctx: any` contract in `src/components/GuardView.vue` with a typed Guard view context.
- Let `src/App.vue` construct `guardCtx` against the same contract so missing or mismatched fields fail at typecheck time.

Files changed:

- `src/App.vue`
- `src/components/GuardView.vue`
- `src/components/viewContexts.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| implicit Guard `ctx` shape | `src/components/viewContexts.ts` | New `GuardViewContext` type |
| broad `ctx: any` in `GuardView.vue` | `ctx: GuardViewContext` | Template behavior unchanged |
| untyped `guardCtx` computed return | `computed<GuardViewContext>` | Compile-time field coverage for App-to-Guard boundary |

Behavior expected to stay the same:

- Guard stepper, pending app actions, idle resolution, rule editing, and diagnostics use the same props and handlers.
- No template text, command call, or view flow was intentionally changed.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected. Type-only runtime impact.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.

Manual smoke tests:

- Not run for this type-contract batch.

Risks:

- Guard interactions were validated by typecheck/build only, not by interactive smoke testing.
- `HomeView.vue` and `InsightsView.vue` still use existing `ctx: any` and need separate batches.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-124 commit.

Follow-up:

- Commit R-125.
- Add typed contexts for `InsightsView.vue` and `HomeView.vue` in separate small batches.

## 2026-06-09: R-126 Insights View Context Typing

Status:

- Passed.

Primary domain:

- frontend shell
- typed component contracts
- insights

Intent:

- Replace the existing `ctx: any` contract in `src/components/InsightsView.vue` with a typed Insights view context.
- Type only the fields the current Insights component actually consumes, while allowing `insightsCtx` to keep extra legacy fields until later cleanup.

Files changed:

- `src/App.vue`
- `src/components/InsightsView.vue`
- `src/components/viewContexts.ts`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| implicit Insights `ctx` shape | `src/components/viewContexts.ts` | New `InsightsViewContext` type |
| broad `ctx: any` in `InsightsView.vue` | `ctx: InsightsViewContext` | Template behavior unchanged |
| unchecked consumed Insights fields | `satisfies InsightsViewContext & Record<string, unknown>` | Verifies required fields while permitting temporary extra context fields |

Behavior expected to stay the same:

- History subnav, top-app list, all-time filters, and recent timeline use the same fields and handlers.
- Existing extra `insightsCtx` fields remain available for later cleanup; no visible UI was removed.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected. Type-only runtime impact.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.

Manual smoke tests:

- Not run for this type-contract batch.

Risks:

- Insights interactions were validated by typecheck/build only, not by interactive smoke testing.
- `HomeView.vue` still uses existing `ctx: any` and needs its own batch.
- `insightsCtx` still contains legacy fields not consumed by the current component.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-125 commit.

Follow-up:

- Commit R-126.
- Type `HomeView.vue` context in a separate batch or remove unused Insights context fields.

## 2026-06-09: R-127 Home View Context Typing

Status:

- Passed.

Primary domain:

- frontend shell
- typed component contracts
- home

Intent:

- Replace the existing `ctx: any` contract in `src/components/HomeView.vue` with a typed Home view context.
- Remove local `any` usage in Home reminder drag/reorder and rhythm summary helpers.
- Share reminder upsert input typing between `HomeView.vue` and `src/App.vue`.

Files changed:

- `src/App.vue`
- `src/components/HomeView.vue`
- `src/components/viewContexts.ts`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| implicit Home `ctx` shape | `src/components/viewContexts.ts` | New `HomeViewContext` type |
| inline reminder upsert input type in `App.vue` | `ReminderUpsertInput` in `src/components/viewContexts.ts` | Same fields and optionality |
| local Home rhythm bar assumptions | `HomeRhythmBar` in `src/components/viewContexts.ts` | Used by Home tooltip/summary typing |
| broad `ctx: any` in `HomeView.vue` | `ctx: HomeViewContext` | Template behavior unchanged |

Behavior expected to stay the same:

- Home status, goal progress, schedule/reminder actions, calendar heatmap, and rhythm tooltip use the same fields and handlers.
- Reminder create/edit/delete/done/snooze/reorder paths call the same App handlers.
- No template text, command call, or view flow was intentionally changed.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected. Type-only runtime impact.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.
- `rg -n "any" src/components/HomeView.vue src/components/GuardView.vue src/components/InsightsView.vue src/components/viewContexts.ts` returned no matches.

Manual smoke tests:

- Not run for this type-contract batch.

Risks:

- Home reminder/rhythm interactions were validated by typecheck/build only, not by interactive smoke testing.
- `homeCtx` still contains a few extra legacy fields not consumed by the current component; removing them should be a separate cleanup batch.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the R-126 commit.

Follow-up:

- Commit R-127.
- Consider a dedicated cleanup batch for unused context fields and dead Insights usage-stack helpers.

## 2026-06-09: Phase 3 Gate - Frontend API And Shell Extraction

State:

- Passed With Known Risk.

Scope completed:

- Split broad frontend Tauri command wrappers from `src/api.ts` into `src/api/commands/*`.
- Kept `src/api.ts` as a compatibility export surface for existing callers.
- Extracted main-window locale, theme, navigation, and pure time helpers out of `src/App.vue`.
- Added typed view context contracts for `HomeView.vue`, `InsightsView.vue`, and `GuardView.vue`.
- Removed existing `ctx: any` usage from current feature view components.

Scope intentionally not completed:

- Did not remove all unused legacy `homeCtx` / `insightsCtx` fields because pruning them exposed dead usage-stack helpers that need a separate cleanup audit.
- Did not extract feature state into full feature composables; that belongs to Phase 4.
- Did not remove `src/api.ts` compatibility exports; they should stay until callers are fully stable or intentionally migrated.

Files changed:

- `src/api.ts`
- `src/api/client.ts`
- `src/api/types.ts`
- `src/api/commands/*`
- `src/App.vue`
- `src/composables/useLocale.ts`
- `src/composables/useThemeMode.ts`
- `src/composables/useAppNavigation.ts`
- `src/lib/time.ts`
- `src/components/HomeView.vue`
- `src/components/InsightsView.vue`
- `src/components/GuardView.vue`
- `src/components/viewContexts.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Commands affected:

- None on the Tauri backend.
- Frontend command wrapper organization changed, but command names and payload shapes were preserved.

Schema/privacy/startup impact:

- Database schema: none.
- Privacy behavior: none expected.
- Startup behavior: none expected; locale/theme init behavior was moved but intended to remain the same.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.
- `rg -n "ctx:\s*any|defineProps<\{ ctx: any \}>|any" src/components src/composables src/lib src/api` returned no matches.
- `git diff --stat fd1f132..HEAD -- src/App.vue src/api.ts src/api src/composables src/components src/lib docs/CURRENT_SYSTEM_MAP.md docs/REFACTOR_LOG.md` reviewed for Phase 3 scope.

Manual smoke tests:

- Not run in an interactive Tauri window during this gate.

Known risks:

- S-100 Main Window and S-200 Navigation still need manual smoke testing before considering this phase release-ready.
- `dist-codex-check/` remains ignored locally after build validation.
- Some unused context fields and dead Insights usage-stack helpers remain for a later cleanup batch.

Rollback point:

- Reset `refactor/architecture` to `fd1f132` to return to the Phase 2 gate.
- Or revert individual Phase 3 commits from R-120 through R-127.

Next phase:

- Phase 4: feature extraction, starting with settings/privacy or reminders.

## 2026-06-09: R-128 Settings View Component Extraction

Status:

- Passed.

Primary domain:

- frontend feature extraction
- settings/privacy

Intent:

- Move the Settings/Privacy page template out of `src/App.vue` into a feature view component.
- Keep settings/privacy state and API handlers in `src/App.vue` for this batch, so the extraction is focused on the view boundary.
- Add a typed settings view context to preserve App-to-settings field coverage.

Files changed:

- `src/App.vue`
- `src/components/SettingsView.vue`
- `src/components/viewContexts.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| Settings/Privacy template block in `App.vue` | `src/components/SettingsView.vue` | Same sections: display, startup, sampling policy, whitelist |
| implicit settings field usage | `SettingsViewContext` in `src/components/viewContexts.ts` | Typed context for settings component |
| `v-model` for primitive App refs | explicit event handlers in `App.vue` | Language, auto-start, and whitelist input now update through handlers |

Behavior expected to stay the same:

- Settings page still mounts lazily through `privacyViewMounted`.
- Language select still updates `locale`, and existing locale watcher still persists/applies it.
- Theme toggle still calls `toggleThemeMode`.
- Auto-start checkbox still updates `autoStartEnabled`; save still persists it.
- Privacy setting fields still mutate the same `privacy` object before save.
- Whitelist input/add/remove still calls the same App handlers.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None. Existing frontend handlers still call the same API wrappers.

Database/schema impact:

- None.

Privacy impact:

- No intended privacy behavior change.

Startup/performance impact:

- No intended startup behavior change. The settings view is a component instead of an inline App template.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.

Manual smoke tests:

- Not run for this component extraction batch.

Risks:

- Settings/privacy interactions were validated by typecheck/build only, not by interactive S-600 smoke testing.
- Settings/privacy state still lives in `src/App.vue`; moving it to a feature composable remains future work.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to the Phase 3 gate commit `db6dc27`.

Follow-up:

- Commit R-128.
- Extract settings/privacy state and handlers into a focused composable or feature module.

## 2026-06-09: R-129 Settings Privacy Composable Extraction

Status:

- Passed.

Primary domain:

- frontend feature extraction
- settings/privacy

Intent:

- Move settings/privacy state, refresh caching, auto-start loading/saving, privacy saving, whitelist add/remove, and related feedback out of `src/App.vue`.
- Keep `SettingsView.vue` unchanged and preserve the typed settings context boundary.

Files changed:

- `src/App.vue`
- `src/composables/useSettingsPrivacy.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `privacy`, `autoStartEnabled`, `whitelist`, `whitelistInput`, privacy feedback refs | `useSettingsPrivacy` | Same defaults |
| `refreshSettingsData`, `refreshPrivacy`, `refreshAutoStartSetting` | `useSettingsPrivacy` | Same 60s refresh cache and loading guard |
| `handleSavePrivacySettings` | `useSettingsPrivacy` | Same API calls and feedback text |
| `handleAddWhitelist`, `handleRemoveWhitelist` | `useSettingsPrivacy` | Same lowercasing, clearing, refresh, and feedback behavior |
| `browserModeText`, auto-start/whitelist input handlers | `useSettingsPrivacy` | Same display text and input updates |

Behavior expected to stay the same:

- Settings data still refreshes lazily and is cached for 60 seconds.
- Saving settings still persists auto-start first, then privacy settings, then refreshes privacy and current data.
- Whitelist add/remove still calls the same command wrappers and refreshes whitelist state.
- Settings feedback text and tones remain the same.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None. Command wrappers called by settings/privacy are unchanged.

Database/schema impact:

- None.

Privacy impact:

- No intended privacy behavior change. Privacy settings and whitelist calls were moved only.

Startup/performance impact:

- No intended startup behavior change. The existing settings refresh cache moved with the feature state.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.
- `rg -n "getPrivacySettings|getAutoStartEnabled|listWhitelist|setAutoStartEnabled|setWhitelistItem|updatePrivacySettings|loadingSettings|settingsLoadedAt|refreshPrivacy|refreshAutoStartSetting|browserModeText" src/App.vue src/composables/useSettingsPrivacy.ts` confirmed the settings/privacy API dependency lives in `useSettingsPrivacy.ts`.

Manual smoke tests:

- Not run for this composable extraction batch.

Risks:

- Settings/privacy interactions were validated by typecheck/build only, not by interactive S-600 smoke testing.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to R-128.

Follow-up:

- Commit R-129.
- Continue Phase 4 with reminders feature extraction.

## 2026-06-09: R-130 Reminder Composable Extraction

Status:

- Passed.

Primary domain:

- frontend feature extraction
- reminders

Intent:

- Move reminder list state, action loading state, sorting, and reminder mutations out of `src/App.vue`.
- Keep Home UI and App refresh orchestration behavior stable while moving reminder feature logic into a focused composable.

Files changed:

- `src/App.vue`
- `src/composables/useReminders.ts`
- `src/components/viewContexts.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `reminders` ref and `reminderActionLoading` | `useReminders` | Same defaults |
| reminder sort helpers | `useReminders` | Preserved original order: done group, sort order, repeat rank, due time, updated time desc, id |
| `reminderListForPanel` | `useReminders` | Same sorted list output |
| `handleUpsertReminder`, `handleDeleteReminder`, `handleReminderDone`, `handleReminderReorder`, `handleReminderSnooze` | `useReminders` | Same API wrappers, optimistic reorder, error propagation |
| `ReminderUpsertInput` | `useReminders` | Shared back into view context typing |

Behavior expected to stay the same:

- Reminder create/edit validation and API payloads remain the same.
- Daily/weekly/no-repeat reminder handling remains the same.
- Reminder reorder keeps the same optimistic update and rollback behavior.
- Home still receives the same reminder fields and handlers through `HomeViewContext`.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None. Existing frontend command wrappers are unchanged.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.
- `rg -n "saveReminder|setReminderOrder|deleteReminder|setReminderDone|snoozeReminder|parseClockToMinutes|parseDateTimeLocalToUnix|reminderGroupRank|reminderRepeatRank|compareReminders|async function handleUpsertReminder" src/App.vue src/composables/useReminders.ts src/components/viewContexts.ts` confirmed reminder mutation logic now lives in `useReminders.ts`.

Manual smoke tests:

- Not run for this composable extraction batch.

Risks:

- Reminder interactions were validated by typecheck/build only, not by interactive S-400 smoke testing.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to R-129.

Follow-up:

- Commit R-130.
- Continue Phase 4 with Guard or Insights feature extraction.

## 2026-06-09: R-131 Guard Workflow Composable Extraction

Status:

- Passed.

Primary domain:

- frontend feature extraction
- guard

Intent:

- Move Guard workflow step completion/unlock state, labels, current step text/index, review completion action, and reset watcher out of `src/App.vue`.
- Keep Guard data fetching and command mutations in `src/App.vue` for a later, larger Guard feature extraction batch.

Files changed:

- `src/App.vue`
- `src/composables/useGuardWorkflow.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `guardStep3Done` | `useGuardWorkflow` | Same default: `false` |
| Guard step completion/unlock computed values | `useGuardWorkflow` | Based on the same pending rule and idle prompt refs |
| `guardCurrentStepText`, `guardCurrentStepIndex`, `guardStepLabels` | `useGuardWorkflow` | Same localized text |
| `markGuardStep3Done` | `useGuardWorkflow` | Same feedback type/text mutation |
| `watch(guardStep3Unlocked)` reset logic | `useGuardWorkflow` | Same reset when rules/idle prerequisites become incomplete |

Behavior expected to stay the same:

- Guard workflow still unlocks steps in the same order.
- Rule review completion still sets the same feedback and unlocks diagnostics.
- If earlier Guard prerequisites become incomplete, rule review completion resets.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.
- `rg -n "guardStep1Complete|guardStep2Complete|function markGuardStep3Done|watch\\(guardStep3Unlocked" src/App.vue src/composables/useGuardWorkflow.ts` confirmed the workflow logic now lives in `useGuardWorkflow.ts`.

Manual smoke tests:

- Not run for this composable extraction batch.

Risks:

- Guard workflow interactions were validated by typecheck/build only, not by interactive S-500 smoke testing.
- Guard data loading and command mutations still live in `src/App.vue`.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to R-130.

Follow-up:

- Commit R-131.
- Continue Guard extraction by moving data loading and mutation handlers into a focused composable.

## 2026-06-09: R-132 Guard Data Composable Extraction

Status:

- Passed.

Primary domain:

- frontend feature extraction
- guard

Intent:

- Move Guard data loading, pending-rule/idle/diagnostic state, rule filtering/sorting, idle resolution, rule mutations, diagnostic text, and Guard input handlers out of `src/App.vue`.
- Keep App-level lifecycle, polling, and capture timer orchestration in `src/App.vue`.

Files changed:

- `src/App.vue`
- `src/composables/useGuardData.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| Guard refs: auto-capture, feedback, diagnostics, app rules, pending rules, idle prompts, rule filters | `useGuardData` | Same defaults |
| `filteredSortedRules`, `currentIdlePrompt` | `useGuardData` | Same sort/filter behavior |
| `refreshGuardData` | `useGuardData` | Same loading guard and same 5-way API fetch |
| `handleResolveIdle` | `useGuardData` | Same decisions, remember-choice behavior, feedback, and refresh call |
| diagnostic/rule helpers and save handlers | `useGuardData` | Same API payloads and feedback |
| Guard input handlers | `useGuardData` | Same checkbox/search/sort/mapped-type behavior |

Behavior expected to stay the same:

- Guard refresh still loads diagnostics, rules, pending apps, pending idle prompts, and idle memory.
- Idle prompt decisions still resolve the same prompt and refresh data afterward.
- Pending rule and diagnostic save actions still save the same rule payloads.
- Rule search/sort and mapped-type edits behave the same.
- Auto-capture checkbox still toggles the same `autoCaptureEnabled` state used by the capture timer.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None. Existing frontend command wrappers are unchanged.

Database/schema impact:

- None.

Privacy impact:

- No intended privacy behavior change. Capture diagnostics and privacy block text were moved only.

Startup/performance impact:

- None expected.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.
- `rg -n "listAppRules|listForegroundCaptureDiagnostics|getIdleMemoryState|resolveIdlePrompt|saveAppRule|type AppRule|type ForegroundCaptureDiagnostic|type IdleMemoryState|PendingRuleProcess|loadingGuard|appRules|function onAutoCaptureToggle|function handleResolveIdle|const filteredSortedRules" src/App.vue src/composables/useGuardData.ts` confirmed Guard data/action dependencies now live in `useGuardData.ts` except Home overview's `listPendingRuleProcesses`.

Manual smoke tests:

- Not run for this composable extraction batch.

Risks:

- Guard interactions were validated by typecheck/build only, not by interactive S-500 smoke testing.
- App still owns the foreground capture timer and top-level idle prompt inline banner.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to R-131.

Follow-up:

- Commit R-132.
- Consider extracting the inline idle prompt banner or continuing with Insights/Home feature-state cleanup.

## 2026-06-09: R-133 Idle Prompt Banner Component Extraction

Status:

- Passed.

Primary domain:

- frontend feature extraction
- guard
- idle prompt

Intent:

- Move the top-level idle prompt confirmation banner out of `src/App.vue`.
- Keep idle prompt state and actions in `useGuardData`, while making App template a thinner shell.

Files changed:

- `src/App.vue`
- `src/components/IdlePromptBanner.vue`
- `src/components/viewContexts.ts`
- `src/composables/useGuardData.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| Inline idle prompt banner template in `App.vue` | `src/components/IdlePromptBanner.vue` | Same visible copy and buttons |
| implicit idle banner fields | `IdlePromptBannerContext` in `src/components/viewContexts.ts` | Typed prompt, checkbox, loading, and action contract |
| direct checkbox `v-model` in App template | `onIdleRememberChoiceChange` in `useGuardData` | Same boolean update behavior |

Behavior expected to stay the same:

- The banner still appears only when there is a current idle prompt.
- The remember-choice checkbox still updates the same `idleRememberChoice` state.
- Learn/Rest/Away/Skip buttons still call `handleResolveIdle` with the same decisions.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.

Manual smoke tests:

- Not run for this component extraction batch.

Risks:

- Idle prompt banner interactions were validated by typecheck/build only, not by interactive S-500 smoke testing.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to R-132.

Follow-up:

- Commit R-133.
- Continue Phase 4 with Insights/Home feature-state cleanup or close a scoped Phase 4 checkpoint for settings/reminders/guard.

## 2026-06-09: R-134 Insights Hidden Usage Stack Context Cleanup

Status:

- Passed.

Primary domain:

- frontend cleanup
- insights

Intent:

- Remove hidden Insights usage-stack state and handlers that were no longer rendered by `InsightsView.vue`.
- Keep the current visible Insights history tabs behavior unchanged.
- Reduce `src/App.vue` surface before the next feature-state extraction batch.

Files changed:

- `src/App.vue`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Removed:

| From | Notes |
| --- | --- |
| Hidden Insights usage-stack refs and filter state in `App.vue` | Current `InsightsViewContext` no longer consumes them |
| Segment rendering, tooltip, day/process drilldown helpers | No current template caller after `InsightsView.vue` extraction |
| Extra usage-stack fields passed through `insightsCtx` | `insightsCtx` now matches the visible typed `InsightsViewContext` |
| Three hidden `getUsageStack(14, ...)` calls in `refreshInsightsData` | Home's `getUsageStack(7, "ALL")` rhythm data remains intact |

Behavior expected to stay the same:

- Insights history tabs still show today's top apps, all-time app usage, and recent logs.
- All-time filters and include-unclassified toggle still refresh via the same API wrappers.
- Home rhythm bars still load from `getUsageStack(7, "ALL")`.

Behavior intentionally changed:

- None for currently visible UI.

Tauri commands affected:

- None. Existing frontend command wrappers are unchanged.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- Insights refresh now avoids three unused usage-stack queries, reducing hidden work when opening or refreshing Insights.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.
- `rg -n "usageStack|usageRootFilter|usageShowIgnore|selectedUsage|tooltipLinesForSegment|handleSegment|processDrill|linkedRecentLogs|onGoalSliderInput|onUsageIgnoreToggle|stackTooltip|RenderSegment|TooltipLine|dayRender|usageSegment|topSegmentSummary|switchUsageFilter|backToUsageOverview|insightsPrimaryView|selectInsightsView" src/App.vue` returned no matches.

Manual smoke tests:

- Not run for this hidden context cleanup batch.

Risks:

- This removes code for an already-hidden older Insights stack/drilldown experience. Reintroducing that UI later should use a fresh typed component/composable rather than restoring hidden App-level state.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to R-133.

Follow-up:

- Commit R-134.
- Continue Phase 4 with remaining `src/App.vue` Home/Insights state extraction.

## 2026-06-09: R-135 Insights Data Composable Extraction

Status:

- Passed.

Primary domain:

- frontend feature extraction
- insights

Intent:

- Move visible Insights data loading and derived view helpers out of `src/App.vue`.
- Keep shared Home/Insights refs for recent logs and heatmap in `App.vue` for this batch.
- Preserve current Insights history tab behavior while reducing App-level feature ownership.

Files changed:

- `src/App.vue`
- `src/composables/useInsightsData.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `topApps`, `allTimeTopApps`, all-time filter state | `useInsightsData` | Same defaults and same API inputs |
| `refreshInsightsData` | `useInsightsData` | Same four visible data requests after R-134 cleanup |
| `allTimeBarWidth`, `topAppsBarWidth`, `recentDurationWidth` | `useInsightsData` | Same percentage clamping behavior |
| recent-log day grouping helper | `useInsightsData` | Same 04:00 business-day boundary |
| all-time filter and include-ignore event handlers | `useInsightsData` | Still refresh through App-level `refreshData` |

Behavior expected to stay the same:

- Opening Insights still refreshes visible top-app, all-time, recent-log, and heatmap data.
- History subtab switching still refreshes Insights data.
- All-time filter buttons and include-unclassified checkbox still trigger the same full refresh path.
- Home still owns and refreshes recent logs, heatmap, reminders, pending rules, idle prompts, and rhythm stack.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None. Existing frontend command wrappers are unchanged.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected beyond R-134's already-recorded removal of unused hidden usage-stack queries.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.

Manual smoke tests:

- Not run for this composable extraction batch.

Risks:

- Insights interactions were validated by typecheck/build only, not by interactive S-500 smoke testing.
- `recentLogs` and `learnHeatmap` are still shared between Home and Insights; this is intentional for this batch and should be handled deliberately if Home data extraction moves them later.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to R-134.

Follow-up:

- Commit R-135.
- Continue Phase 4 by extracting Home overview/rhythm/heatmap state or by separating shared recent-log/heatmap ownership.

## 2026-06-09: R-136 Heatmap Calendar Composable Extraction

Status:

- Passed.

Primary domain:

- frontend feature extraction
- home
- heatmap

Intent:

- Move heatmap calendar/month derived state out of `src/App.vue`.
- Keep heatmap data loading, saved goal loading, and goal persistence timers unchanged for this batch.
- Preserve Home heatmap rendering and month summary behavior while reducing App-level computed state.

Files changed:

- `src/App.vue`
- `src/composables/useHeatmapCalendar.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `viewMonthDate`, month title, weekday headers | `useHeatmapCalendar` | Same defaults and locale-dependent labels |
| current/full month heatmap and padded calendar cells | `useHeatmapCalendar` | Same 42-cell layout |
| future/today/cell-class/day-label helpers | `useHeatmapCalendar` | Same class thresholds and labels |
| `shiftHeatmapMonth`, `getHeatmapFetchDays` | `useHeatmapCalendar` | Same month navigation and fetch-window calculation |
| month goal progress and active streak summaries | `useHeatmapCalendar` | Same source heatmap and date rules |

Behavior expected to stay the same:

- Home heatmap calendar still shows the same month, weekday labels, day labels, today/future styling, and intensity classes.
- Month navigation still changes the view month in the same way.
- Home and Insights refresh still use the same `getHeatmapFetchDays()` result.
- Goal persistence/loading remains in `App.vue` unchanged.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.

Manual smoke tests:

- Not run for this composable extraction batch.

Risks:

- Home heatmap interactions were validated by typecheck/build only, not by interactive S-500 smoke testing.
- `learnHeatmap` remains shared App-level data; only derived view state moved.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to R-135.

Follow-up:

- Commit R-136.
- Continue Phase 4 by extracting Home rhythm/overview data or by creating a focused Home data composable.

## 2026-06-09: R-137 Home Rhythm Composable Extraction

Status:

- Passed.

Primary domain:

- frontend feature extraction
- home

Intent:

- Move Home seven-day rhythm bar derivation out of `src/App.vue`.
- Keep Home data loading and `homeUsageStack` ownership unchanged for this batch.

Files changed:

- `src/App.vue`
- `src/composables/useHomeRhythm.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `HomeRhythmBar` local shape, rhythm height constants, and height helper | `useHomeRhythm` | Same values and math |
| `homeMonthRhythmBars` computed | `useHomeRhythm` | Same seven-day window, labels, today marker, and stacked height calculation |

Behavior expected to stay the same:

- Home rhythm bars still render the same seven local days.
- Learn/rest heights, total height, labels, today marker, and computer-total data are derived the same way.
- `refreshHomeData` still writes `getUsageStack(7, "ALL")` into `homeUsageStack`.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.

Manual smoke tests:

- Not run for this composable extraction batch.

Risks:

- Home rhythm rendering was validated by typecheck/build only, not by interactive S-500 smoke testing.
- `homeUsageStack` remains App-level state because data loading has not moved yet.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to R-136.

Follow-up:

- Commit R-137.
- Continue Phase 4 by extracting remaining Home overview/schedule summary data, or pause for a phase-gate review.

## 2026-06-09: R-138 Home Schedule Presentation Extraction

Status:

- Passed.

Primary domain:

- frontend feature extraction
- home
- reminders

Intent:

- Move Home reminder visibility filtering and due-text formatting out of `src/App.vue`.
- Keep reminder data ownership, sorting, and CRUD actions in `useReminders`.

Files changed:

- `src/App.vue`
- `src/composables/useHomeSchedule.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| reminder date/time formatting and weekday labels | `useHomeSchedule` | Same locale source and label arrays |
| weekly-day text formatting | `useHomeSchedule` | Same validation/dedup/sort behavior |
| `reminderDueText` | `useHomeSchedule` | Same snooze/daily/weekly/one-time text behavior |
| `reminderIsVisibleToday` and `homeScheduleItems` | `useHomeSchedule` | Same daily/weekly/one-time visibility rules |

Behavior expected to stay the same:

- Home schedule still lists the same visible reminders for today.
- Reminder due text still uses the same locale, snooze, repeat, done-state, and one-time display logic.
- Reminder CRUD/reorder/snooze/done behavior remains in `useReminders` unchanged.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.

Manual smoke tests:

- Not run for this composable extraction batch.

Risks:

- Home reminder display was validated by typecheck/build only, not by interactive S-500 smoke testing.
- Reminder presentation is now separate from reminder mutation logic; keep this boundary unless product behavior changes.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to R-137.

Follow-up:

- Commit R-138.
- Continue Phase 4 with remaining Home overview state or perform a frontend phase-gate audit.

## 2026-06-09: R-139 Home Overview State Extraction

Status:

- Passed.

Primary domain:

- frontend feature extraction
- home

Intent:

- Move Home summary and status derived state out of `src/App.vue`.
- Keep Home data loading, polling, and lifecycle orchestration in `src/App.vue` for this batch.

Files changed:

- `src/App.vue`
- `src/composables/useHomeOverview.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `todaySummary` state and today learn/rest derived values | `useHomeOverview` | `refreshHomeData` still writes the same summary ref |
| goal progress percentage/fill/overflow derived state | `useHomeOverview` | Same goal-minute input and clamping behavior |
| recent activity summary | `useHomeOverview` | Same latest-log formatting, process cleanup, and clock helper |
| due reminder count, status label/tone, pending summary | `useHomeOverview` | Same priority order and copy |

Behavior expected to stay the same:

- Home top status, goal progress, recent activity summary, due-reminder count, and pending summary should render the same.
- `refreshHomeData` still fetches today summary, recent logs, heatmap, pending rules, idle prompts, reminders, and rhythm stack.
- Polling and navigation refresh behavior are unchanged.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.

Manual smoke tests:

- Not run for this composable extraction batch.

Risks:

- Home overview display was validated by typecheck/build only, not by interactive S-500 smoke testing.
- `useHomeOverview` depends on shared refs owned by App/composables; keep this boundary explicit until Home data loading is extracted.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to R-138.

Follow-up:

- Commit R-139.
- Consider a frontend phase-gate audit before further App-level orchestration changes.

## 2026-06-09: R-140 Shared Display Formatters Extraction

Status:

- Passed.

Primary domain:

- frontend cleanup
- shared display helpers

Intent:

- Move cross-view display formatters out of `src/App.vue`.
- Keep all existing formatting rules and translated labels unchanged.

Files changed:

- `src/App.vue`
- `src/composables/useDisplayFormatters.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `formatClock` | `useDisplayFormatters` | Same locale and time options |
| `mappedTypeText` | `useDisplayFormatters` | Same Learn/Rest/Unclassified labels |
| `cleanProcessName` | `useDisplayFormatters` | Same idle/system/process-name normalization |
| `formatIdlePromptSpan` | `useDisplayFormatters` | Same span and duration formatting |

Behavior expected to stay the same:

- Home, Insights, Guard, and idle prompt banner display the same process names, clocks, mapped labels, and idle prompt spans.
- `useGuardData` still receives the same `mappedTypeText` callback.
- `useHomeOverview` still receives the same `cleanProcessName` and `formatClock` callbacks.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.

Manual smoke tests:

- Not run for this helper extraction batch.

Risks:

- Display helper output was validated by typecheck/build only, not by interactive S-500 smoke testing.
- The formatter composable is shared by multiple feature contexts, so future label changes should still be reviewed across Home/Insights/Guard.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to R-139.

Follow-up:

- Commit R-140.
- Run a frontend phase-gate audit before larger App lifecycle or refresh orchestration changes.

## 2026-06-09: R-141 Phase 4 Checkpoint - Main Window Feature Extraction

Status:

- Passed With Known Risk.

Primary domain:

- frontend feature extraction
- phase checkpoint

Intent:

- Evaluate the main-window Phase 4 extraction work from R-128 through R-140 before making larger lifecycle or refresh-orchestration changes.
- Confirm typed feature contexts, extracted composables, and removed hidden Insights usage-stack state remain build-stable.
- Record remaining risk explicitly instead of treating Phase 4 as fully complete.

Files changed:

- `docs/REFACTOR_LOG.md`

Checkpoint scope:

- Main window feature/component extraction only.
- Pet window and pet panel extraction are not included in this checkpoint.
- No business behavior changes were made in this checkpoint record.

Evidence reviewed:

| Evidence | Result |
| --- | --- |
| Git branch/status | `refactor/architecture`, clean before this docs record |
| `src/App.vue` line count | 3330 lines after R-140 |
| View component line counts | Home 679, Guard 148, Insights 97, Settings 82 |
| `rg -n "ctx:\s*any|defineProps<\{ ctx: any \}>|usageStack|usageRootFilter|usageShowIgnore|selectedUsage|stackTooltip|RenderSegment|TooltipLine|any" src/App.vue src/components src/composables src/lib src/api` | No matches |
| `src/App.vue` remaining explicit state | shared recent logs, heatmap, home usage stack, goal slider, loading/error/privacy-mounted shell state |
| `src/App.vue` remaining functions | shell navigation, home refresh, global refresh, goal persistence timer, locale change, section flash/scroll, context assembly, lifecycle timers |

Behavior expected to stay the same:

- Main Home, Insights, Guard, Settings, and idle prompt surfaces use the same user-visible behavior as before the Phase 4 extraction batches.
- Tauri command names, payload shapes, and return shapes remain unchanged.
- Database schema and privacy behavior remain unchanged.
- Hidden old Insights usage-stack context remains removed; the current visible Insights history tabs remain supported.

Behavior intentionally changed:

- None in this checkpoint record.

Tauri commands affected:

- None.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- No new startup behavior change in this checkpoint record.
- Earlier R-134 removed unused hidden Insights usage-stack queries; that remains the only recorded performance-oriented change in this Phase 4 slice.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.
- `cargo check` from `src-tauri` passed.

Manual smoke tests:

- Not run in this checkpoint.

Risks:

- This is not the final Phase 4 gate for the whole application; pet/panel extraction and smoke coverage are still outside this checkpoint.
- Manual S-300/S-400/S-500/S-600/S-700 smoke tests were not run interactively.
- `src/App.vue` is much thinner but still owns Home refresh orchestration, global timers, context assembly, and a large CSS block.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Reset `refactor/architecture` to R-140 commit `c4a7c2a`, or revert this docs-only checkpoint commit after it is committed.

Follow-up:

- Commit R-141.
- Next safe options: run manual smoke checks, extract Home refresh orchestration, or start pet/panel feature extraction as a separate Phase 4 slice.

## 2026-06-09: R-142 Home Data Refresh Extraction

Status:

- Passed.

Primary domain:

- frontend feature extraction
- home

Intent:

- Move Home-specific data refresh orchestration out of `src/App.vue`.
- Keep global refresh routing, polling timers, and lifecycle ownership in `src/App.vue`.

Files changed:

- `src/App.vue`
- `src/composables/useHomeData.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| Home `loadingHome` guard | `useHomeData` | Same early-return behavior |
| Home seven-way refresh request | `useHomeData` | Same APIs, limits, and parameters |
| Assignments to Home/shared refs after refresh | `useHomeData` | Same target refs and values |
| Home refresh error handling | `useHomeData` | Same `setErrorMessage` callback |

Behavior expected to stay the same:

- Home refresh still loads today summary, recent logs, heatmap, pending rules, pending idle prompts, reminders, and rhythm stack.
- Initial load and polling still call `refreshHomeData` from `App.vue`.
- Global `refreshData` still refreshes Home first, then the active secondary view.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None. Existing frontend command wrappers are unchanged.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.

Manual smoke tests:

- Not run for this composable extraction batch.

Risks:

- Home refresh behavior was validated by typecheck/build only, not by interactive S-300 smoke testing.
- `refreshData` still has global sequencing responsibilities in `App.vue`.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to R-141.

Follow-up:

- Commit R-142.
- Consider extracting global refresh/lifecycle coordination only after manual smoke checks or a smaller orchestration design note.

## 2026-06-09: R-143 Desktop Release Build Check

Status:

- Passed With Known Risk.

Primary domain:

- validation
- desktop build

Intent:

- Verify that the refactored frontend and Rust Tauri shell still compile together as a release desktop app.
- Record the file-lock issue encountered during the first attempts so the validation remains auditable.

Files changed:

- `docs/REFACTOR_LOG.md`

Behavior expected to stay the same:

- No source behavior changed in this docs-only validation record.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None.

Automated validation:

- `tauri build --no-bundle` first failed because `src-tauri/target/release/timeprism.exe` was locked by a running `timeprism` process at PID 25912.
- `taskkill /PID 25912` sent a termination signal, but the process remained alive.
- `taskkill /PID 25912 /F` terminated the repository release process.
- Re-running `tauri build --no-bundle` passed.
- Release artifact produced: `src-tauri/target/release/timeprism.exe`, size 16165888 bytes, last write time 2026-06-09 16:23:35.
- `git status --short` was clean after the build.

Manual smoke tests:

- Not run. The release app was not relaunched after build, to avoid leaving a running process that would lock future release builds.

Risks:

- This validates compilation and frontend build integration, not interactive UI behavior.
- A running `timeprism.exe` process can lock future release rebuilds until stopped.
- `dist/`, `dist-codex-check/`, and `src-tauri/target/` build artifacts remain local build outputs.

Rollback:

- No source rollback needed for the build itself.
- Revert this docs-only commit after it is committed if the checkpoint record is not wanted.

Follow-up:

- Commit R-143.
- Continue with either manual smoke tests or the next small refactor batch.

## 2026-06-09: R-144 App Global CSS Extraction

Status:

- Passed.

Primary domain:

- frontend cleanup
- styling

Intent:

- Move the global main-window CSS out of `src/App.vue`.
- Reduce `App.vue` noise without changing selectors or style rules.

Files changed:

- `src/App.vue`
- `src/main.ts`
- `src/styles/app.css`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| Final `<style>` block in `src/App.vue` | `src/styles/app.css` | Mechanical move, style content preserved |
| CSS loading responsibility | `src/main.ts` import | App entry now imports `./styles/app.css` |

Behavior expected to stay the same:

- Main-window global styles should be applied in the same way through Vite's CSS import pipeline.
- Component templates and script logic are unchanged.
- Pet and pet-panel entry styles are unchanged.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.
- `src/App.vue` line count dropped from 3305 to 590 after moving 2712 CSS lines.

Manual smoke tests:

- Not run for this CSS extraction batch.

Risks:

- Visual parity was validated by build only, not by screenshot or interactive smoke testing.
- Because the CSS is global, future style edits should still be treated as application-wide changes.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to R-143.

Follow-up:

- Commit R-144.
- Consider screenshot/manual smoke checks before larger visual or layout refactors.

## 2026-06-09: R-145 App Top Navigation Extraction

Status:

- Passed.

Primary domain:

- frontend shell
- navigation UI

Intent:

- Move the main-window brand block and primary navigation tabs out of `src/App.vue`.
- Keep navigation state and refresh side effects in `App.vue` for now.

Files changed:

- `src/App.vue`
- `src/components/AppTopNav.vue`
- `src/composables/useAppNavigation.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| Top navigation template in `src/App.vue` | `src/components/AppTopNav.vue` | Template-only extraction with the same CSS classes |
| Inline main-view option shape | `MainViewOption` in `useAppNavigation.ts` | Exported for typed component props |

Behavior expected to stay the same:

- Main navigation labels, active state, brand display, and tab selection behavior remain unchanged.
- `App.vue` still owns `setMainView`, view refresh side effects, and lazy settings mount behavior.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.
- `git diff --check` passed.
- `git diff --cached --check` passed.

Manual smoke tests:

- Not run for this template-only extraction batch.

Risks:

- Navigation behavior is validated by typecheck/build only, not by interactive S-200 smoke testing.
- `App.vue` still owns broad refresh orchestration and context assembly.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to R-144.

Follow-up:

- Commit R-145.
- Continue with smaller shell/context extractions before changing refresh orchestration.

## 2026-06-09: R-146 Heatmap Goal Setting Extraction

Status:

- Passed.

Primary domain:

- frontend composable
- Home/Insights shared heatmap preference

Intent:

- Move heatmap goal slider state, saved-goal loading, 15-minute normalization, debounced persistence, and cleanup out of `src/App.vue`.
- Keep `App.vue` focused on wiring the heatmap goal into Home/Insights data refresh and Home overview context.

Files changed:

- `src/App.vue`
- `src/composables/useHeatmapGoalSetting.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `learnGoalSliderMinutes` state | `useHeatmapGoalSetting` | Still returned to `App.vue` for Home overview |
| `getHeatmapGoalSeconds` / slider normalization | `useHeatmapGoalSetting` | Existing read-time sync behavior preserved |
| heatmap goal API load/save timer | `useHeatmapGoalSetting` | Same backend commands, same debounce value |
| on-unmount timer cleanup | `cleanupHeatmapGoalSetting` | Called from `App.vue` unmount hook |

Behavior expected to stay the same:

- Saved heatmap goal seconds are loaded on main-window mount and rounded to 15-minute slider steps.
- Home and Insights heatmap fetches still call the same `getHeatmapGoalSeconds` function shape.
- Debounced save still uses `setHeatmapGoalSecondsSetting` with the same 260ms delay.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None. Existing frontend calls still target `get_heatmap_goal_seconds_setting` and `set_heatmap_goal_seconds_setting` through the API layer.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- No intended startup or performance change; the saved-goal load still runs during main-window mount.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.
- `git diff --check` passed.
- `git diff --cached --check` passed.

Manual smoke tests:

- Not run for this composable extraction batch.

Risks:

- Existing heatmap-goal persistence semantics were preserved, including the current read-time sync/save scheduling behavior.
- Home/Insights heatmap behavior was validated by typecheck/build only, not by interactive S-300 or S-700 smoke testing.
- `dist-codex-check/` remains ignored locally after build validation.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to R-145.

Follow-up:

- Commit R-146.
- Consider a separate, explicit hardening batch for heatmap-goal save scheduling if runtime smoke testing shows repeated saves.

## 2026-06-09: R-147 Heatmap Goal Save Scheduling Hardening

Status:

- Passed.

Primary domain:

- frontend composable
- persistence hardening

Intent:

- Prevent the heatmap-goal debounced save timer from rescheduling itself while persisting.
- Keep the public `getHeatmapGoalSeconds` function shape and stored goal value semantics unchanged.

Files changed:

- `src/composables/useHeatmapGoalSetting.ts`
- `docs/REFACTOR_LOG.md`

Behavior expected to stay the same:

- Heatmap goal minutes are still rounded to 15-minute steps and clamped to 0-1440 minutes.
- Heatmap callers still receive goal seconds through `getHeatmapGoalSeconds`.
- Saved goal loading still uses `get_heatmap_goal_seconds_setting`; saving still uses `set_heatmap_goal_seconds_setting`.

Behavior intentionally changed:

- The debounced save callback now normalizes the current slider value directly instead of calling `getHeatmapGoalSeconds`.
- This prevents a save callback from scheduling another save callback solely because it read the value it is persisting.

Tauri commands affected:

- None. Command names and payload shapes are unchanged.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- Startup behavior is unchanged.
- Background persistence should avoid an unintended repeated-save loop after a heatmap-goal read queues a save.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.
- `git diff --check` passed.
- `git diff --cached --check` passed.

Manual smoke tests:

- Not run for this narrow scheduling hardening batch.

Risks:

- No frontend unit-test harness exists, so the timer behavior was reviewed by code inspection and validated by typecheck/build only.
- Heatmap goal persistence still queues a save when `getHeatmapGoalSeconds` is called; only the self-rescheduling save callback path was changed.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to R-146.

Follow-up:

- Commit R-147.
- Consider adding a small frontend test harness before deeper timer or lifecycle changes.

## 2026-06-09: R-148 Insights Section Navigation Extraction

Status:

- Passed.

Primary domain:

- frontend composable
- Insights navigation polish

Intent:

- Move Insights section target switching, DOM flash highlighting, and flash timer cleanup out of `src/App.vue`.
- Keep the Tauri `navigate-insights-section` event listener in `App.vue` so external event wiring remains visible at the shell boundary.

Files changed:

- `src/App.vue`
- `src/composables/useInsightsSectionNavigation.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `flashInsightsSection` | `useInsightsSectionNavigation` | DOM class toggle behavior preserved |
| `scrollToInsightsSection` | `useInsightsSectionNavigation` | Still switches to Insights/top-apps before flashing |
| `sectionFlashTimer` cleanup | `cleanupInsightsSectionNavigation` | Called from `App.vue` unmount hook |

Behavior expected to stay the same:

- `navigate-insights-section` still switches the main view to Insights and selects the top-apps subview.
- The `insights-stack-anchor` element still receives the same `section-flash` class for 1000ms.
- Tauri event registration and unlisten cleanup remain in `App.vue`.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.
- `git diff --check` passed.
- `git diff --cached --check` passed.

Manual smoke tests:

- Not run for this DOM/timer extraction batch.

Risks:

- The Insights section event path was validated by typecheck/build only, not by an interactive pet-to-Insights navigation smoke test.
- The `section` payload is still intentionally ignored, matching previous behavior.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to R-147.

Follow-up:

- Commit R-148.
- Consider an interactive S-700/S-900 smoke check before changing cross-window navigation further.

## 2026-06-09: R-149 Auto Capture Sampler Extraction

Status:

- Passed.

Primary domain:

- frontend composable
- Guard/capture polling

Intent:

- Move the auto-capture foreground sampling interval and feedback text out of `src/App.vue`.
- Keep `App.vue` responsible for starting the sampler on mount and stopping it on unmount.

Files changed:

- `src/App.vue`
- `src/composables/useAutoCaptureSampler.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `captureTimer` interval state | `useAutoCaptureSampler` | Timer start/stop returned to `App.vue` |
| `captureForegroundOnce(5000)` polling body | `sampleAutoCapture` | Same default 5000ms interval |
| auto-capture feedback messages | `useAutoCaptureSampler` | Same Chinese/English strings |

Behavior expected to stay the same:

- Auto capture still starts during main-window mount and stops during unmount.
- Disabled auto capture still skips foreground sampling.
- Feedback text still distinguishes stored samples, privacy/baseline skipped samples, and capture failures.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None. The same frontend API wrapper still invokes the same foreground capture command.

Database/schema impact:

- None.

Privacy impact:

- None intended; privacy filtering remains in the backend foreground/usage services.

Startup/performance impact:

- None expected; the sampler still uses a 5000ms interval and does not fire immediately on mount.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.
- `git diff --check` passed.
- `git diff --cached --check` passed.

Manual smoke tests:

- Not run for this timer extraction batch.

Risks:

- Auto-capture runtime behavior was validated by typecheck/build only, not by S-500/S-600 interactive capture smoke testing.
- The sampler still depends on `autoCaptureEnabled` and `autoCaptureFeedback` refs owned by Guard data state.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to R-148.

Follow-up:

- Commit R-149.
- Consider interactive capture smoke testing before further capture lifecycle changes.

## 2026-06-09: R-150 Lazy Settings Mount Extraction

Status:

- Passed.

Primary domain:

- frontend composable
- Settings lifecycle

Intent:

- Move settings view lazy mount state and warmup timer cleanup out of `src/App.vue`.
- Keep `App.vue` responsible for deciding when the Settings tab is selected and when mount/unmount lifecycle starts.

Files changed:

- `src/App.vue`
- `src/composables/useLazySettingsMount.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `privacyViewMounted` state | `useLazySettingsMount` | Still consumed by the Settings view `v-if` |
| zero-delay refresh when Settings is opened | `showSettingsViewAndRefresh` | Same `window.setTimeout(..., 0)` behavior |
| 1200ms settings warmup timer | `startSettingsWarmup` | Same warmup delay |
| warmup timer cleanup | `cleanupSettingsWarmup` | Called from `App.vue` unmount hook |

Behavior expected to stay the same:

- Settings view is still lazily mounted.
- Opening Settings still mounts it and queues an immediate settings refresh.
- Main-window mount still warms Settings data after 1200ms.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None.

Database/schema impact:

- None.

Privacy impact:

- None intended; privacy settings loading/saving remains in `useSettingsPrivacy`.

Startup/performance impact:

- None expected; same warmup delay and refresh call timing.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.
- `git diff --check` passed.
- `git diff --cached --check` passed.

Manual smoke tests:

- Not run for this settings lifecycle extraction batch.

Risks:

- Settings open/warmup behavior was validated by typecheck/build only, not by S-600 interactive smoke testing.
- The zero-delay refresh timer remains intentionally untracked, matching previous behavior.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to R-149.

Follow-up:

- Commit R-150.
- Consider Settings/privacy interactive smoke testing before changing settings persistence behavior.

## 2026-06-09: R-151 Main Refresh Polling Extraction

Status:

- Passed.

Primary domain:

- frontend composable
- main-window refresh lifecycle

Intent:

- Move the 5-second Home/Guard refresh polling timer out of `src/App.vue`.
- Keep the initial Home refresh and the refresh function implementations unchanged.

Files changed:

- `src/App.vue`
- `src/composables/useMainRefreshPolling.ts`
- `docs/CURRENT_SYSTEM_MAP.md`
- `docs/REFACTOR_LOG.md`

Moved/extracted:

| From | To | Notes |
| --- | --- | --- |
| `pollTimer` interval state | `useMainRefreshPolling` | Timer start/stop returned to `App.vue` |
| 5-second refresh callback | `runMainRefreshPoll` | Still refreshes Home every tick and Guard only while active |

Behavior expected to stay the same:

- Main-window mount still performs an immediate Home refresh before starting the interval.
- The polling interval still runs every 5000ms.
- Guard data still refreshes during polling only when the current main view is Guard.

Behavior intentionally changed:

- None.

Tauri commands affected:

- None.

Database/schema impact:

- None.

Privacy impact:

- None.

Startup/performance impact:

- None expected; interval timing and initial refresh timing are unchanged.

Automated validation:

- `pnpm.cmd run typecheck` passed.
- `pnpm.cmd run build:check` passed.
- `git diff --check` passed.
- `git diff --cached --check` passed.

Manual smoke tests:

- Not run for this timer extraction batch.

Risks:

- Home/Guard periodic refresh behavior was validated by typecheck/build only, not by S-300/S-500 interactive smoke testing.
- The poller still starts only after the initial mount sequence reaches the previous start point.

Rollback:

- Revert this batch commit after it is committed, or reset `refactor/architecture` to R-150.

Follow-up:

- Commit R-151.
- Consider a main-window smoke pass before deeper lifecycle consolidation.
