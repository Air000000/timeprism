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
