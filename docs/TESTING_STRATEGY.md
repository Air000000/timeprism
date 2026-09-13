# Testing Strategy

The goal is not nominal coverage. The goal is enough deterministic feedback to make refactors and desktop-runtime changes safe.

## Test Categories

### 1. Static Checks

Run for every frontend or backend batch.

Frontend:

```powershell
pnpm.cmd run typecheck
pnpm.cmd run build:check
```

Backend:

```powershell
cargo check --all-targets
cargo test
```

CI runs these checks on Windows so the platform-specific backend and the Rust test target are compiled in the same environment used by the application.

### 2. Characterization Tests

Purpose: lock important existing behavior before moving or rewriting risky code.

Covered areas include:

- business-day window calculation;
- interval merging and overlap;
- reminder recurrence and wall-clock semantics;
- privacy title transformation and diagnostic redaction;
- app-rule normalization;
- analytics aggregation;
- idle-interval replacement.

Characterization tests describe current behavior. They are regression evidence, not proof that every existing behavior is ideal.

### 3. Contract Checks

Purpose: ensure frontend callers and Tauri command boundaries remain compatible.

Current safeguards:

- TypeScript typecheck for command payload/return usage;
- Tauri command inventory in `docs/API_CONTRACTS.md`;
- Rust compilation of all targets;
- manual smoke coverage for multi-window behavior that is not practical to prove with pure unit tests.

Automated end-to-end Tauri command tests remain a later hardening option if a stable desktop test harness is introduced.

### 4. Repository / Database Tests

Purpose: protect schema, migration, seed, and file-lifecycle semantics.

`src-tauri/src/db/tests.rs` uses synthetic temporary SQLite files and covers:

- fresh database initialization;
- required schema/table creation;
- default app rules and config insertion;
- close/reopen persistence;
- repeated initialization without overwriting user-modified values;
- legacy `app_rules` / `reminders` schema migration without losing rows;
- one-time migration from the legacy database location to the preferred location.

Service-level tests also use in-memory SQLite connections for rules, privacy, reminders, usage, and analytics behavior.

Never run automated tests against the user's real TimePrism database.

### 5. Manual Smoke Tests

Use `docs/SMOKE_TESTS.md` when changing:

- window behavior;
- pet behavior;
- privacy settings;
- capture loop;
- reminder workflows;
- navigation and page structure;
- packaged Windows installers.

## Risk-Based Validation Matrix

| Change type | Required validation |
| --- | --- |
| Docs only | Readability check |
| Type-only frontend change | `vue-tsc` |
| Frontend component change | `vue-tsc`, Vite temp build, relevant smoke test |
| API wrapper change | `vue-tsc`, API contracts reviewed |
| Rust pure helper extraction | `cargo check --all-targets`, relevant unit/characterization test |
| Tauri command wrapper change | `cargo check --all-targets`, frontend typecheck if caller touched |
| SQLite query/schema change | `cargo check --all-targets`, `cargo test`, DB lifecycle/repository test |
| Privacy/capture change | Rust checks/tests, privacy checklist, manual capture smoke |
| Pet/window change | Vite temp build, Rust checks/tests, pet smoke test |
| Full packaging change | release-workflow dry run, installer smoke test |

## Test Data Rules

- Use synthetic process names such as `code.exe`, `chrome.exe`, and `game.exe`.
- Use synthetic titles that include private/incognito keywords where privacy behavior is under test.
- Use deterministic timestamps unless a test explicitly targets current-time behavior.
- Use in-memory or temporary-file SQLite databases.
- Never rely on the user's real SQLite database.
- Never assert that raw private titles should be persisted.

## Temporary Build Output

The normal `pnpm.cmd build` may fail when `dist` is locked by a running app.

For source validation, use:

```powershell
pnpm.cmd run build:check
```

Then delete `dist-codex-check` if it is not needed.
