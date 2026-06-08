# Testing Strategy

The goal is not perfect coverage before refactor. The goal is enough feedback to make change safe.

## Test Categories

### 1. Static Checks

Use for every frontend or backend batch.

Frontend:

```powershell
pnpm.cmd run typecheck
pnpm.cmd run build:check
```

Backend:

```powershell
cargo check
```

### 2. Characterization Tests

Purpose: lock current behavior before moving risky code.

Use for:

- Business-day window calculation.
- Interval merging and overlap.
- Reminder recurrence.
- Privacy title transformation.
- App rule normalization.
- Heatmap level calculation.
- Usage-stack aggregation.

Important: characterization tests describe current behavior. They do not prove the behavior is ideal.

### 3. Contract Tests

Purpose: ensure command payloads and return shapes remain compatible.

Target:

- `src/api.ts` types.
- Tauri command names in `src-tauri/src/lib.rs`.
- Pet and pet-panel command usage.

Early practical approach:

- TypeScript typecheck.
- Manual command/caller table in `docs/API_CONTRACTS.md`.
- Add automated command smoke tests later when a stable test harness exists.

### 4. Repository / Database Tests

Purpose: protect schema, migration, and query semantics.

Target:

- Fresh DB initialization.
- Legacy DB path fallback.
- Default app rules/config insertion.
- Reminder table migrations.
- Heatmap snapshot table creation.

Use temporary SQLite files. Do not test against the user's real app data.

### 5. Manual Smoke Tests

Use `docs/SMOKE_TESTS.md`.

Manual smoke tests are required when changing:

- Window behavior.
- Pet behavior.
- Privacy settings.
- Capture loop.
- Reminder workflows.
- Navigation and page structure.

## Risk-Based Validation Matrix

| Change type | Required validation |
| --- | --- |
| Docs only | Readability check |
| Type-only frontend change | `vue-tsc` |
| Frontend component change | `vue-tsc`, Vite temp build, relevant smoke test |
| API wrapper change | `vue-tsc`, API contracts reviewed |
| Rust pure helper extraction | `cargo check`, unit/characterization test if available |
| Tauri command wrapper change | `cargo check`, frontend typecheck if caller touched |
| SQLite query change | `cargo check`, DB/repository test or manual DB smoke |
| Privacy/capture change | `cargo check`, privacy checklist, manual capture smoke |
| Pet/window change | Vite temp build, `cargo check`, pet smoke test |
| Full packaging change | full Tauri build |

## Initial Tests To Add First

Recommended first automated tests:

1. Business day starts at 04:00 local time.
2. `merge_intervals_total` handles overlapping and adjacent intervals.
3. Reminder daily/weekly next due calculation.
4. Browser/private title mode transformation.
5. Rule process-name normalization.

## Test Data Rules

- Use synthetic process names such as `code.exe`, `chrome.exe`, `game.exe`.
- Use synthetic titles that include private/incognito keywords.
- Use deterministic timestamps.
- Never rely on the user's real SQLite database.
- Never assert raw private titles should be stored.

## Temporary Build Output

The normal `pnpm.cmd build` may fail when `dist` is locked by a running app.

For source validation, use:

```powershell
pnpm.cmd run build:check
```

Then delete `dist-codex-check`.
