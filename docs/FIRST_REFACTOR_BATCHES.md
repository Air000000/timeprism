# First Refactor Batches

These are the recommended first batches. They are intentionally small and behavior-preserving.

## Batch 0: Baseline And Safety

Primary domain: repo/process.

Goal:

- Create a stable rollback point before architecture changes.

Tasks:

1. Commit current documentation and small text fixes.
2. Tag the pre-refactor baseline.
3. Create `refactor/architecture`.
4. Optionally create a reference worktree from `origin/main`.

Commands:

```powershell
git status --short --branch
git add AGENTS.md docs src/components/GuardView.vue src/components/InsightsView.vue src/pet-panel.ts src/pet.ts
git commit -m "docs(refactor): add architecture refactor guide"
git tag pre-refactor-2026-06-08
git switch -c refactor/architecture
```

Validation:

- No code behavior change required.
- Run typecheck/build if including text fixes.

Rollback:

- `git switch main`
- `git reset --hard pre-refactor-2026-06-08` only if explicitly intended.

## Batch 1: Extract Backend DB Path And Connection

Primary domain: backend-db.

Goal:

- Move path resolution and connection opening out of `lib.rs` without behavior change.

Create:

```text
src-tauri/src/db/mod.rs
src-tauri/src/db/connection.rs
```

Move or wrap these responsibilities:

- `ensure_data_dir`
- `legacy_db_path`
- `db_path`
- `try_migrate_legacy_db`
- `open_connection`

Rules:

- Keep function behavior identical.
- Keep command functions unchanged except imports/calls.
- Do not change DB file names.
- Do not change migration behavior.

Validation:

```powershell
cd src-tauri
cargo check
```

Optional manual smoke:

- Launch app.
- Confirm Home loads.

## Batch 2: Extract Backend Migrations And Table Ensurers

Primary domain: backend-db.

Goal:

- Move table creation and schema-upgrade helpers out of `lib.rs`.

Create:

```text
src-tauri/src/db/migrations.rs
```

Move or wrap these responsibilities:

- `init_database`
- `ensure_app_rules_time_columns`
- `ensure_reminders_weekly_columns`
- `ensure_reminders_sort_order`
- `ensure_heatmap_snapshot_table`

Rules:

- No schema changes.
- SQL text should remain equivalent.
- Default seed data should remain equivalent.
- Keep command registration unchanged.

Validation:

```powershell
cd src-tauri
cargo check
```

Manual smoke:

- Fresh app launch on an empty test profile if practical.
- Existing DB launch if practical.

## Batch 3: Extract Domain Types

Primary domain: backend-domain.

Goal:

- Move serializable structs and input structs into focused modules.

Create:

```text
src-tauri/src/domain/mod.rs
src-tauri/src/domain/analytics.rs
src-tauri/src/domain/rules.rs
src-tauri/src/domain/reminders.rs
src-tauri/src/domain/privacy.rs
src-tauri/src/domain/idle.rs
src-tauri/src/domain/window.rs
```

Move structs such as:

- `Category`
- `TodaySummary`
- `TopApp`
- `ReminderEntry`
- `SaveReminderInput`
- `AppRuleEntry`
- `PendingRuleProcess`
- `PrivacySettings`
- `IdlePromptEntry`
- `PetWindowSettleResult`

Rules:

- Keep serde derives.
- Preserve field names.
- Preserve public visibility only where needed.
- Do not rename fields.

Validation:

```powershell
cd src-tauri
cargo check
```

## Batch 4: Extract Privacy Service

Primary domain: privacy/capture.

Goal:

- Move privacy and process/title normalization into a service module.

Create:

```text
src-tauri/src/services/mod.rs
src-tauri/src/services/privacy.rs
```

Move or wrap:

- `parse_bool_config`
- `normalize_process_key`
- `parse_browser_title_mode`
- `is_browser_process`
- `contains_incognito_keyword`
- `is_desktop_shell_window`
- `process_log_with_privacy`

Rules:

- Privacy behavior must remain identical.
- Add characterization tests if possible before changing logic.
- Do not alter title output strings yet.

Validation:

```powershell
cd src-tauri
cargo check
```

Manual smoke:

- Run S-600 from `SMOKE_TESTS.md`.

## Batch 5: Extract API Types On Frontend

Primary domain: frontend-api.

Goal:

- Split API types from command wrappers.

Create:

```text
src/api/types.ts
src/api/client.ts
src/api/commands/
```

First move:

- Type definitions from `src/api.ts` into `src/api/types.ts`.
- Thin `invoke` helper into `src/api/client.ts`.
- Keep `src/api.ts` as a compatibility re-export until all imports are migrated.

Rules:

- Do not change imported names used by existing components.
- Do not change command names.
- Do not split all command files in the same batch unless imports remain simple.

Validation:

```powershell
pnpm.cmd run typecheck
pnpm.cmd run build:check
```

Cleanup:

```powershell
Remove-Item -LiteralPath .\dist-codex-check -Recurse -Force
```

## Batch Selection Rule

Only start the next batch after:

- Current batch checks pass.
- `REFACTOR_LOG.md` is updated.
- Any contract/doc changes are committed.
