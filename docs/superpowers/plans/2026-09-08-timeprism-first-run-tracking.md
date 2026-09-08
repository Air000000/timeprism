# TimePrism First-Run Tracking Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make a fresh TimePrism install initialize SQLite before any product WebView can query it, show a one-time tracking disclosure before foreground capture starts, and persist tracking pause/resume state across restarts.

**Architecture:** Keep the existing `commands -> services -> domain/db` structure. Add a small SQLite-backed tracking-state service, manually create configured Tauri windows only after database initialization, and gate the existing Vue runtime behind a persisted `TrackingState`. The foreground sampler keeps its current approximately 5-second cadence; policy/persistence stays outside the sampler.

**Tech Stack:** Tauri 2, Rust, rusqlite/SQLite, Vue 3, TypeScript, Vite, GitHub Actions on Windows.

**Spec:** `docs/superpowers/specs/2026-09-08-timeprism-first-run-tracking-design.md`

## Global Constraints

- Fresh databases start with `onboarding_completed=false` and `auto_capture_enabled=false`.
- Existing pre-change TimePrism databases migrate to `onboarding_completed=true` and `auto_capture_enabled=true` unless either key already exists.
- No foreground capture may occur before onboarding activation succeeds.
- A user-initiated pause persists across restart; restart must never silently turn capture back on.
- Keep the foreground sampling interval at approximately 5 seconds.
- Keep Home and pet refresh cadence at approximately 5 seconds.
- The first foreground sample remains baseline-only to avoid synthetic overcounting.
- No new frontend test framework is added solely for this feature.
- Database initialization failure must prevent normal product runtime startup.
- Do not perform another broad backend or frontend directory refactor.
- Product copy must state that TimePrism reads foreground application/window metadata, stores core data locally, privacy-processes browser titles, does not capture screenshots/keystroke contents, and that Insights activity differs from classified `LEARN` / `REST` totals.

---

## File Structure

### Create

- `src-tauri/src/domain/tracking.rs` — serialized backend `TrackingState` contract.
- `src-tauri/src/services/tracking.rs` — SQLite-backed tracking-state invariants and transitions.
- `src/api/commands/tracking.ts` — typed frontend wrappers for tracking-state Tauri commands.
- `src/api/commands/window.ts` — typed main-window wrapper for showing the pet after first activation.
- `src/composables/useTrackingState.ts` — frontend persistence-facing tracking state only; no timer ownership.
- `src/components/OnboardingView.vue` — one-screen first-run disclosure and activation UI.

### Modify

- `src-tauri/src/db/migrations.rs` — classify fresh vs recognizable existing DB before schema creation; seed tracking keys with migration-safe defaults.
- `src-tauri/src/db/tests.rs` — file-backed regression coverage for fresh/existing tracking defaults and persisted pause preservation.
- `src-tauri/src/domain/mod.rs` — expose tracking domain model.
- `src-tauri/src/services/mod.rs` — expose tracking service.
- `src-tauri/src/lib.rs` — thin tracking commands; fail-closed DB startup; create windows only after DB init; gate startup pet creation/shortcut by onboarding state.
- `src-tauri/src/services/window.rs` — create main/pet windows from `tauri.conf.json` config after DB readiness instead of duplicating startup config.
- `src-tauri/tauri.conf.json` — set configured WebViews to `create: false` so Tauri cannot auto-create them before `.setup()` completes.
- `src/api/types.ts` and `src/api.ts` — expose `TrackingState` to TypeScript callers.
- `src/api/commands/index.ts` — export tracking/window command modules.
- `src/composables/useAutoCaptureSampler.ts` — timer only; remove policy ownership from the sampler.
- `src/composables/useGuardData.ts` — consume persisted auto-capture state rather than creating `ref(true)`.
- `src/composables/useMainWindowLifecycle.ts` — split shell initialization from idempotent product-runtime start.
- `src/composables/useErrorMessage.ts` — add explicit clearing for a recovered transient read error.
- `src/composables/useHomeData.ts` — clear its own transient global read error after a successful core refresh.
- `src/App.vue` — composition-root wiring for tracking state, onboarding, activation, pause/resume, runtime start, and pet reveal.
- `src/pet.ts` — render active vs paused tracking state while retaining 5-second summary polling.
- `src/styles/app.css` — onboarding layout/styles using existing visual tokens.
- `docs/SMOKE_TESTS.md` and `docs/RELEASE_CHECKLIST.md` — record fresh-install and upgrade acceptance gates.

---

### Task 1: Add migration-safe tracking defaults

**Files:**
- Modify: `src-tauri/src/db/migrations.rs` around `initialize_connection`
- Modify/Test: `src-tauri/src/db/tests.rs` around the fresh/idempotent/legacy database tests

**Interfaces:**
- Consumes: existing `app_config(key TEXT PRIMARY KEY, value TEXT NOT NULL)` table.
- Produces: guaranteed `onboarding_completed` and `auto_capture_enabled` config keys after every successful `initialize_connection(&Connection)` call.

- [ ] **Step 1: Add failing database tests for fresh tracking defaults**

Extend `fresh_file_database_initialization_seeds_schema_and_defaults_and_reopens` with exact assertions:

```rust
let onboarding: String = conn
    .query_row(
        "SELECT value FROM app_config WHERE key = 'onboarding_completed'",
        [],
        |row| row.get(0),
    )
    .expect("read onboarding default");
let auto_capture: String = conn
    .query_row(
        "SELECT value FROM app_config WHERE key = 'auto_capture_enabled'",
        [],
        |row| row.get(0),
    )
    .expect("read auto capture default");
assert_eq!(onboarding, "false");
assert_eq!(auto_capture, "false");
```

- [ ] **Step 2: Add failing migration tests for an existing recognizable database**

In `legacy_schema_is_upgraded_without_losing_existing_rows`, after creating the legacy `app_rules` / `reminders` schema and calling `initialize_connection`, assert:

```rust
let onboarding: String = conn.query_row(
    "SELECT value FROM app_config WHERE key = 'onboarding_completed'",
    [],
    |row| row.get(0),
).expect("read migrated onboarding state");
let auto_capture: String = conn.query_row(
    "SELECT value FROM app_config WHERE key = 'auto_capture_enabled'",
    [],
    |row| row.get(0),
).expect("read migrated capture state");
assert_eq!(onboarding, "true");
assert_eq!(auto_capture, "true");
```

- [ ] **Step 3: Add a failing regression test proving initialization preserves a user pause**

Add:

```rust
#[test]
fn database_initialization_preserves_persisted_tracking_pause() {
    let path = temp_db_path("tracking-pause");
    remove_sqlite_files(&path);
    let conn = Connection::open(&path).expect("open sqlite file");

    super::migrations::initialize_connection(&conn).expect("initialize fresh database");
    conn.execute(
        "UPDATE app_config SET value = 'true' WHERE key = 'onboarding_completed'",
        [],
    ).expect("mark onboarding complete");
    conn.execute(
        "UPDATE app_config SET value = 'false' WHERE key = 'auto_capture_enabled'",
        [],
    ).expect("persist capture pause");

    super::migrations::initialize_connection(&conn).expect("reinitialize database");

    let state: (String, String) = conn.query_row(
        "SELECT
            (SELECT value FROM app_config WHERE key = 'onboarding_completed'),
            (SELECT value FROM app_config WHERE key = 'auto_capture_enabled')",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).expect("read persisted tracking state");
    assert_eq!(state, ("true".to_string(), "false".to_string()));

    drop(conn);
    remove_sqlite_files(&path);
}
```

- [ ] **Step 4: Run the focused tests and confirm RED**

Run:

```bash
cd src-tauri
cargo test db::tests::fresh_file_database_initialization_seeds_schema_and_defaults_and_reopens
cargo test db::tests::legacy_schema_is_upgraded_without_losing_existing_rows
cargo test db::tests::database_initialization_preserves_persisted_tracking_pause
```

Expected: the first two fail because the new keys do not exist; the third cannot observe the requested keys yet.

- [ ] **Step 5: Detect whether the DB was recognizable before schema creation**

Add a helper before `initialize_connection`:

```rust
fn has_recognizable_timeprism_schema(conn: &Connection) -> Result<bool, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*)
             FROM sqlite_master
             WHERE type = 'table'
               AND name IN ('categories', 'task_sessions', 'app_usage_logs', 'app_rules', 'app_config', 'reminders')",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("failed to inspect existing TimePrism schema: {e}"))?;
    Ok(count > 0)
}
```

At the start of `initialize_connection`, before the existing `CREATE TABLE IF NOT EXISTS ...` batch:

```rust
let was_existing_timeprism_db = has_recognizable_timeprism_schema(conn)?;
```

- [ ] **Step 6: Seed tracking keys with fresh-vs-upgrade defaults using `INSERT OR IGNORE`**

After the existing default config loop, add:

```rust
let tracking_defaults = if was_existing_timeprism_db {
    [("onboarding_completed", "true"), ("auto_capture_enabled", "true")]
} else {
    [("onboarding_completed", "false"), ("auto_capture_enabled", "false")]
};

for (key, value) in tracking_defaults {
    conn.execute(
        "INSERT OR IGNORE INTO app_config (key, value) VALUES (?1, ?2)",
        params![key, value],
    )
    .map_err(|e| format!("failed to seed tracking config {key}: {e}"))?;
}
```

Do not update existing rows; `INSERT OR IGNORE` is the preservation mechanism.

- [ ] **Step 7: Run focused tests and the full Rust suite**

Run:

```bash
cd src-tauri
cargo test db::tests::fresh_file_database_initialization_seeds_schema_and_defaults_and_reopens
cargo test db::tests::legacy_schema_is_upgraded_without_losing_existing_rows
cargo test db::tests::database_initialization_preserves_persisted_tracking_pause
cargo test
```

Expected: all pass; existing DB lifecycle and mojibake tests stay green.

- [ ] **Step 8: Commit Task 1**

```bash
git add src-tauri/src/db/migrations.rs src-tauri/src/db/tests.rs
git commit -m "feat(db): persist first-run tracking defaults"
```

---

### Task 2: Add the backend tracking-state invariant

**Files:**
- Create: `src-tauri/src/domain/tracking.rs`
- Create: `src-tauri/src/services/tracking.rs`
- Modify: `src-tauri/src/domain/mod.rs`
- Modify: `src-tauri/src/services/mod.rs`
- Modify: `src-tauri/src/lib.rs` command imports/handlers

**Interfaces:**
- Produces: `TrackingState { onboarding_completed: bool, auto_capture_enabled: bool }`.
- Produces: `get_tracking_state_entry(&Connection) -> Result<TrackingState, String>`.
- Produces: `complete_onboarding_entry(&Connection) -> Result<TrackingState, String>`.
- Produces: `set_auto_capture_enabled_entry(&Connection, bool) -> Result<TrackingState, String>`.
- Produces Tauri commands: `get_tracking_state`, `complete_tracking_onboarding`, `set_auto_capture_enabled`.

- [ ] **Step 1: Create the domain contract**

`src-tauri/src/domain/tracking.rs`:

```rust
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TrackingState {
    pub onboarding_completed: bool,
    pub auto_capture_enabled: bool,
}
```

Expose it from `domain/mod.rs`:

```rust
pub mod tracking;
```

- [ ] **Step 2: Write failing service tests before the service implementation**

Create `src-tauri/src/services/tracking.rs` with a `#[cfg(test)]` module first. Use a minimal in-memory table:

```rust
#[cfg(test)]
mod tests {
    use rusqlite::{params, Connection};

    use super::{
        complete_onboarding_entry, get_tracking_state_entry, set_auto_capture_enabled_entry,
    };

    fn tracking_conn(onboarding: bool, enabled: bool) -> Connection {
        let conn = Connection::open_in_memory().expect("open tracking test db");
        conn.execute_batch(
            "CREATE TABLE app_config (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        ).expect("create app_config");
        for (key, value) in [
            ("onboarding_completed", onboarding.to_string()),
            ("auto_capture_enabled", enabled.to_string()),
        ] {
            conn.execute(
                "INSERT INTO app_config (key, value) VALUES (?1, ?2)",
                params![key, value],
            ).expect("seed tracking state");
        }
        conn
    }

    #[test]
    fn completing_onboarding_atomically_enables_capture() {
        let conn = tracking_conn(false, false);
        let state = complete_onboarding_entry(&conn).expect("complete onboarding");
        assert!(state.onboarding_completed);
        assert!(state.auto_capture_enabled);
        assert_eq!(get_tracking_state_entry(&conn).unwrap(), state);
    }

    #[test]
    fn pause_and_resume_persist_after_onboarding() {
        let conn = tracking_conn(true, true);
        assert!(!set_auto_capture_enabled_entry(&conn, false).unwrap().auto_capture_enabled);
        assert!(set_auto_capture_enabled_entry(&conn, true).unwrap().auto_capture_enabled);
    }

    #[test]
    fn enabling_capture_before_onboarding_is_rejected() {
        let conn = tracking_conn(false, false);
        let err = set_auto_capture_enabled_entry(&conn, true).unwrap_err();
        assert!(err.contains("onboarding"));
        assert!(!get_tracking_state_entry(&conn).unwrap().auto_capture_enabled);
    }
}
```

- [ ] **Step 3: Run the tracking tests and confirm RED**

Run:

```bash
cd src-tauri
cargo test services::tracking::tests
```

Expected: compile/test failure because the service functions do not exist yet.

- [ ] **Step 4: Implement strict config reads and the three state transitions**

Use exact boolean parsing rather than a default that could hide a missing/corrupt key:

```rust
use rusqlite::{params, Connection};
use crate::domain::tracking::TrackingState;

fn read_bool_config(conn: &Connection, key: &str) -> Result<bool, String> {
    let value: String = conn
        .query_row("SELECT value FROM app_config WHERE key = ?1", [key], |row| row.get(0))
        .map_err(|e| format!("failed to read tracking config {key}: {e}"))?;
    match value.trim().to_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(format!("invalid boolean tracking config {key}: {other}")),
    }
}

pub(crate) fn get_tracking_state_entry(conn: &Connection) -> Result<TrackingState, String> {
    Ok(TrackingState {
        onboarding_completed: read_bool_config(conn, "onboarding_completed")?,
        auto_capture_enabled: read_bool_config(conn, "auto_capture_enabled")?,
    })
}
```

`complete_onboarding_entry` must use one SQLite transaction and update both rows before commit:

```rust
pub(crate) fn complete_onboarding_entry(conn: &Connection) -> Result<TrackingState, String> {
    let tx = conn.unchecked_transaction()
        .map_err(|e| format!("failed to begin onboarding transaction: {e}"))?;
    tx.execute(
        "UPDATE app_config SET value = 'true' WHERE key IN ('onboarding_completed', 'auto_capture_enabled')",
        [],
    ).map_err(|e| format!("failed to complete onboarding: {e}"))?;
    tx.commit().map_err(|e| format!("failed to commit onboarding: {e}"))?;
    get_tracking_state_entry(conn)
}
```

`set_auto_capture_enabled_entry` must read current state first, reject `true` before onboarding, write one key, then re-read and return persisted state.

- [ ] **Step 5: Expose the service module and run tests GREEN**

Add to `services/mod.rs`:

```rust
pub mod tracking;
```

Run:

```bash
cd src-tauri
cargo test services::tracking::tests
cargo test
```

Expected: all pass.

- [ ] **Step 6: Add thin Tauri command wrappers**

In `lib.rs`, import the model/service and add:

```rust
#[tauri::command(async)]
fn get_tracking_state(app: AppHandle) -> Result<TrackingState, String> {
    let conn = open_connection(&app)?;
    get_tracking_state_entry(&conn)
}

#[tauri::command]
fn complete_tracking_onboarding(app: AppHandle) -> Result<TrackingState, String> {
    let conn = open_connection(&app)?;
    complete_onboarding_entry(&conn)
}

#[tauri::command]
fn set_auto_capture_enabled(app: AppHandle, enabled: bool) -> Result<TrackingState, String> {
    let conn = open_connection(&app)?;
    set_auto_capture_enabled_entry(&conn, enabled)
}
```

Register all three in `tauri::generate_handler!`.

- [ ] **Step 7: Compile all Rust targets**

Run:

```bash
cd src-tauri
cargo check --all-targets
cargo test
```

Expected: pass.

- [ ] **Step 8: Commit Task 2**

```bash
git add src-tauri/src/domain/tracking.rs src-tauri/src/domain/mod.rs src-tauri/src/services/tracking.rs src-tauri/src/services/mod.rs src-tauri/src/lib.rs
git commit -m "feat(tracking): add persisted tracking state invariant"
```

---

### Task 3: Eliminate the DB/WebView startup race

**Files:**
- Modify: `src-tauri/tauri.conf.json` window definitions
- Modify: `src-tauri/src/services/window.rs` creation/reveal helpers and tests
- Modify: `src-tauri/src/lib.rs` `.setup()` and global shortcut handler

**Interfaces:**
- Consumes: `get_tracking_state_entry(&Connection)` from Task 2.
- Produces: `create_configured_window(app: &AppHandle, label: &str) -> Result<tauri::WebviewWindow, String>`.
- Invariant: no configured product WebView exists before `init_database` succeeds.

- [ ] **Step 1: Add a failing config characterization test**

In `services/window.rs` tests, add:

```rust
#[test]
fn configured_product_windows_require_manual_creation() {
    let config: serde_json::Value = serde_json::from_str(include_str!("../../tauri.conf.json"))
        .expect("parse tauri config");
    let windows = config["app"]["windows"].as_array().expect("window config array");
    for label in ["main", "pet", "pet-panel"] {
        let item = windows.iter()
            .find(|item| item["label"] == label)
            .unwrap_or_else(|| panic!("missing window config {label}"));
        assert_eq!(item["create"], false, "{label} must not auto-create before DB init");
    }
}
```

- [ ] **Step 2: Run the test and confirm RED**

Run:

```bash
cd src-tauri
cargo test services::window::tests::configured_product_windows_require_manual_creation
```

Expected: failure because `create` is absent/default-true.

- [ ] **Step 3: Set every configured WebView to manual creation**

In each of the `main`, `pet`, and `pet-panel` objects in `tauri.conf.json`, add:

```json
"create": false
```

Keep all existing URL, size, decoration, transparency, visibility, and icon settings unchanged.

- [ ] **Step 4: Add one config-driven window creation helper**

In `services/window.rs`:

```rust
pub(crate) fn create_configured_window(
    app: &AppHandle,
    label: &str,
) -> Result<tauri::WebviewWindow, String> {
    if let Some(existing) = app.get_webview_window(label) {
        return Ok(existing);
    }
    let config = app.config().app.windows.iter()
        .find(|item| item.label == label)
        .ok_or_else(|| format!("window config not found: {label}"))?;
    tauri::WebviewWindowBuilder::from_config(app, config)
        .map_err(|e| format!("failed to build window config {label}: {e}"))?
        .build()
        .map_err(|e| format!("failed to create configured window {label}: {e}"))
}
```

Use this helper for the startup creation path. Do not rewrite unrelated panel/docking behavior in this task.

- [ ] **Step 5: Make `.setup()` fail closed and order initialization before window creation**

Replace the warning-only DB startup with this order:

```rust
.setup(|app| {
    init_database(app.handle()).map_err(std::io::Error::other)?;

    let conn = open_connection(app.handle()).map_err(std::io::Error::other)?;
    let tracking = get_tracking_state_entry(&conn).map_err(std::io::Error::other)?;
    drop(conn);

    let main_window = window_service::create_configured_window(app.handle(), "main")
        .map_err(std::io::Error::other)?;
    window_service::set_main_close_behavior(&main_window);

    if tracking.onboarding_completed {
        window_service::create_configured_window(app.handle(), "pet")
            .map_err(std::io::Error::other)?;
        window_service::ensure_pet_window_position(app.handle(), true)
            .map_err(std::io::Error::other)?;
    }

    // existing global shortcut registration remains after the DB/window readiness boundary
    Ok(())
})
```

Do not create `pet-panel` at startup; existing lazy panel creation remains.

- [ ] **Step 6: Prevent the global shortcut from revealing a pre-onboarding pet**

In the shortcut handler, read tracking state first. Only call `window_service::summon_pet_window` when `onboarding_completed` is true. If DB/state read fails, log the error and do not create the pet.

The policy belongs in the composition root (`lib.rs`); keep `window_service` free of DB reads.

- [ ] **Step 7: Run Rust verification**

Run:

```bash
cd src-tauri
cargo test services::window::tests::configured_product_windows_require_manual_creation
cargo check --all-targets
cargo test
```

Expected: pass.

- [ ] **Step 8: Commit Task 3**

```bash
git add src-tauri/tauri.conf.json src-tauri/src/services/window.rs src-tauri/src/lib.rs
git commit -m "fix(startup): initialize database before webviews"
```

---

### Task 4: Add typed frontend tracking state and separate sampler mechanism from policy

**Files:**
- Modify: `src/api/types.ts`
- Modify: `src/api.ts`
- Create: `src/api/commands/tracking.ts`
- Create: `src/api/commands/window.ts`
- Modify: `src/api/commands/index.ts`
- Create: `src/composables/useTrackingState.ts`
- Modify: `src/composables/useAutoCaptureSampler.ts`

**Interfaces:**
- Produces TS type `TrackingState`.
- Produces `getTrackingState()`, `completeTrackingOnboarding()`, `setAutoCaptureEnabled(enabled)`, `summonPetWindow()`.
- Produces composable refs `trackingState`, `trackingStateLoading`, `trackingActionLoading`, `onboardingCompleted`, `autoCaptureEnabled` plus persistence methods.
- Sampler `startAutoCaptureSampler()` / `stopAutoCaptureSampler()` remains idempotent and is started only by the runtime gate.

- [ ] **Step 1: Add the shared TypeScript contract and command wrappers**

In `src/api/types.ts`:

```ts
export type TrackingState = {
  onboarding_completed: boolean;
  auto_capture_enabled: boolean;
};
```

Export it from `src/api.ts`.

Create `src/api/commands/tracking.ts`:

```ts
import { invokeCommand } from "../client";
import type { TrackingState } from "../types";

export async function getTrackingState(): Promise<TrackingState> {
  return invokeCommand("get_tracking_state");
}

export async function completeTrackingOnboarding(): Promise<TrackingState> {
  return invokeCommand("complete_tracking_onboarding");
}

export async function setAutoCaptureEnabled(enabled: boolean): Promise<TrackingState> {
  return invokeCommand("set_auto_capture_enabled", { enabled });
}
```

Create `src/api/commands/window.ts`:

```ts
import { invokeCommand } from "../client";

export async function summonPetWindow(): Promise<void> {
  await invokeCommand("summon_pet_window");
}
```

Export both modules from `src/api/commands/index.ts`.

- [ ] **Step 2: Create `useTrackingState` with persistence-only responsibility**

Implement:

```ts
import { computed, ref } from "vue";
import {
  completeTrackingOnboarding,
  getTrackingState,
  setAutoCaptureEnabled,
  type TrackingState,
} from "../api";

export function useTrackingState(setErrorMessage: (error: unknown) => void) {
  const trackingState = ref<TrackingState | null>(null);
  const trackingStateLoading = ref(true);
  const trackingActionLoading = ref(false);
  const onboardingCompleted = computed(() => trackingState.value?.onboarding_completed === true);
  const autoCaptureEnabled = computed(() => trackingState.value?.auto_capture_enabled === true);

  async function loadTrackingState() {
    trackingStateLoading.value = true;
    try {
      trackingState.value = await getTrackingState();
      return trackingState.value;
    } catch (error) {
      setErrorMessage(error);
      throw error;
    } finally {
      trackingStateLoading.value = false;
    }
  }

  async function activateTracking() {
    trackingActionLoading.value = true;
    try {
      trackingState.value = await completeTrackingOnboarding();
      return trackingState.value;
    } finally {
      trackingActionLoading.value = false;
    }
  }

  async function persistAutoCaptureEnabled(enabled: boolean) {
    trackingActionLoading.value = true;
    try {
      trackingState.value = await setAutoCaptureEnabled(enabled);
      return trackingState.value;
    } finally {
      trackingActionLoading.value = false;
    }
  }

  return {
    trackingState,
    trackingStateLoading,
    trackingActionLoading,
    onboardingCompleted,
    autoCaptureEnabled,
    loadTrackingState,
    activateTracking,
    persistAutoCaptureEnabled,
  };
}
```

Route action errors through `setErrorMessage` in the final implementation; never mutate the ref optimistically before the backend returns.

- [ ] **Step 3: Remove policy from `useAutoCaptureSampler`**

Change `AutoCaptureSamplerOptions` so it no longer accepts `autoCaptureEnabled`. `sampleAutoCapture()` should always perform one `captureForegroundOnce(intervalMs)` call when the sampler is running. `startAutoCaptureSampler()` remains responsible for `stop -> immediate baseline sample -> setInterval(5000)` and `stopAutoCaptureSampler()` clears the timer.

This makes the runtime gate the sole authority for whether the sampler is running.

- [ ] **Step 4: Run frontend static validation**

There is no existing Vue unit-test runner, so do not add one for this task.

Run:

```bash
pnpm run typecheck
pnpm run build:check
```

Expected: pass.

- [ ] **Step 5: Commit Task 4**

```bash
git add src/api/types.ts src/api.ts src/api/commands/tracking.ts src/api/commands/window.ts src/api/commands/index.ts src/composables/useTrackingState.ts src/composables/useAutoCaptureSampler.ts
git commit -m "feat(ui): add persisted tracking state client"
```

---

### Task 5: Gate the Vue product runtime and persist Guard pause/resume

**Files:**
- Modify: `src/composables/useMainWindowLifecycle.ts`
- Modify: `src/composables/useGuardData.ts`
- Modify: `src/lib/guardFeedback.ts`
- Modify: `src/App.vue` script wiring

**Interfaces:**
- Consumes: `loadTrackingState`, `onboardingCompleted`, `autoCaptureEnabled`, `persistAutoCaptureEnabled` from Task 4.
- Produces from `useMainWindowLifecycle`: `startProductRuntime(captureEnabled: boolean)`, `syncCaptureRuntime(captureEnabled: boolean)`, `stopProductRuntime()`.
- Guard consumes one authoritative `autoCaptureEnabled` ref/computed and async `setAutoCaptureEnabled(next)` callback.

- [ ] **Step 1: Make product runtime start idempotent**

Refactor `useMainWindowLifecycle` so `onMounted` performs only locale/theme initialization plus `await loadTrackingState()`. Add internal `let productRuntimeStarted = false` and a returned function:

```ts
async function startProductRuntime(captureEnabled: boolean) {
  if (!productRuntimeStarted) {
    productRuntimeStarted = true;
    await loadHeatmapGoalSecondsSetting();
    resetGuardFeedback();
    resetPrivacyFeedback();
    void refreshHomeData();
    startSettingsWarmup();
    startMainRefreshPolling();
    await startInsightsSectionNavigationListener();
  }
  syncCaptureRuntime(captureEnabled);
}

function syncCaptureRuntime(captureEnabled: boolean) {
  if (!productRuntimeStarted) return;
  if (captureEnabled) startAutoCaptureSampler();
  else stopAutoCaptureSampler();
}
```

If the loaded state is already onboarded, `onMounted` calls `startProductRuntime(state.auto_capture_enabled)`. If incomplete, it returns without starting Home polling/settings warmup/sampler/listeners.

`onUnmounted` must stop polling, sampler, warmups/listeners, and reset `productRuntimeStarted=false`.

- [ ] **Step 2: Make Guard consume persisted state instead of `ref(true)`**

Change `UseGuardDataOptions` to accept:

```ts
autoCaptureEnabled: Readonly<Ref<boolean>>;
setAutoCaptureEnabled: (enabled: boolean) => Promise<void>;
```

Delete:

```ts
const autoCaptureEnabled = ref(true);
```

Change `onAutoCaptureToggle` to parse the checkbox state and call the injected async setter. Do not flip the shared state locally; the backend response updates it through `useTrackingState`.

Add a feedback helper in `guardFeedback.ts`:

```ts
export function guardAutoCaptureStateFeedback(enabled: boolean, tx: TranslateFn): string {
  return enabled
    ? tx("自动采样已开启", "Auto capture enabled")
    : tx("记录已暂停；重启后仍保持暂停", "Tracking paused; pause persists after restart");
}
```

Update feedback only after successful persistence.

- [ ] **Step 3: Wire persistence to sampler start/stop in `App.vue`**

At the composition root, create one async bridge:

```ts
async function handleSetAutoCaptureEnabled(enabled: boolean) {
  const state = await persistAutoCaptureEnabled(enabled);
  syncCaptureRuntime(state.auto_capture_enabled);
}
```

Pass `autoCaptureEnabled` and this callback into `useGuardData`.

Do not let `useGuardData`, `useTrackingState`, or `useAutoCaptureSampler` own all three responsibilities; `App.vue` remains the orchestration boundary.

- [ ] **Step 4: Run frontend static gates**

Run:

```bash
pnpm run typecheck
pnpm run build:check
```

Expected: pass and there is no remaining `ref(true)` tracking authority in `useGuardData`.

- [ ] **Step 5: Commit Task 5**

```bash
git add src/composables/useMainWindowLifecycle.ts src/composables/useGuardData.ts src/lib/guardFeedback.ts src/App.vue
git commit -m "feat(tracking): gate runtime on persisted state"
```

---

### Task 6: Add the one-time onboarding surface and activation transition

**Files:**
- Create: `src/components/OnboardingView.vue`
- Modify: `src/App.vue`
- Modify: `src/styles/app.css`

**Interfaces:**
- Consumes: `trackingStateLoading`, `onboardingCompleted`, `trackingActionLoading`, `activateTracking`.
- Emits from `OnboardingView`: `activate` only; there is no persistent browse-without-tracking branch.
- Activation order: backend state commit -> product runtime start -> pet reveal.

- [ ] **Step 1: Create a single-screen disclosure component**

Use the existing `tx(zh, en)` translation pattern. Component API:

```ts
const props = defineProps<{
  tx: (zh: string, en: string) => string;
  loading: boolean;
}>();
const emit = defineEmits<{ activate: [] }>();
```

Required visible content, expressed concisely rather than as a carousel:

```text
欢迎使用 TimePrism

自动记录你的电脑时间流向，数据保存在本机。

开始后，TimePrism 会：
• 读取当前前台应用及窗口信息，用于计算使用时长
• 默认对浏览器标题做隐私处理，并跳过无痕/隐私窗口
• 将核心数据保存在本机 SQLite；当前版本不会上传到云端

TimePrism 不会：
• 截屏
• 记录键盘输入内容

“数据看板”展示观察到的应用活动；桌宠的“学 / 休”只统计已分类为学习或休息的应用。

[开始使用 TimePrism]
```

English copy must communicate the same claims. The primary button is disabled while `loading` is true.

- [ ] **Step 2: Style onboarding as the first product surface**

In `app.css`, add focused classes such as `.onboarding-shell`, `.onboarding-card`, `.onboarding-points`, `.onboarding-primary`. Reuse existing background, card, text, hint, button radii, and spacing tokens; do not introduce a new visual system.

The content must fit the existing 888x900 main-window minimum without scrolling on the default Windows scale used by the app.

- [ ] **Step 3: Conditionally render onboarding before the normal product shell**

In `App.vue` template, structure the top-level condition as:

```vue
<OnboardingView
  v-if="!trackingStateLoading && !onboardingCompleted"
  :tx="tx"
  :loading="trackingActionLoading"
  @activate="handleActivateTracking"
/>

<main v-else-if="!trackingStateLoading" class="layout">
  <!-- existing nav and product views unchanged -->
</main>
```

Do not start Home/Insights/Guard data flows merely because the normal DOM exists; Task 5's runtime gate remains authoritative.

- [ ] **Step 4: Implement the activation order without optimistic tracking**

In `App.vue`:

```ts
async function handleActivateTracking() {
  try {
    const state = await activateTracking();
    await startProductRuntime(state.auto_capture_enabled);
    await summonPetWindow();
  } catch (error) {
    setErrorMessage(error);
  }
}
```

If the backend state write fails, onboarding remains visible and the sampler remains stopped. If the later pet reveal fails, tracking state remains valid; report the pet error without rolling the persisted activation back.

- [ ] **Step 5: Run frontend and Rust static gates**

Run:

```bash
pnpm run typecheck
pnpm run build:check
cd src-tauri
cargo check --all-targets
cargo test
```

Expected: pass.

- [ ] **Step 6: Commit Task 6**

```bash
git add src/components/OnboardingView.vue src/App.vue src/styles/app.css
git commit -m "feat(onboarding): require first-run tracking activation"
```

---

### Task 7: Make the pet communicate persisted tracking state

**Files:**
- Modify: `src/pet.ts`

**Interfaces:**
- Consumes: `getTrackingState()` and existing `getTodaySummary()`.
- Behavior: pet remains hidden pre-onboarding by Rust startup policy; when present, mood says active vs paused while the counters remain `LEARN` / `REST` totals.

- [ ] **Step 1: Read summary and tracking state in the same existing 5-second refresh**

Change `refreshSummary()` to use:

```ts
const [summary, tracking] = await Promise.all([
  getTodaySummary(),
  getTrackingState(),
]);
```

Keep `renderPetSummary` totals exactly based on `summary.learn_seconds` and `summary.rest_seconds`.

Set mood text from persisted state:

```ts
const trackingMood = tracking.auto_capture_enabled
  ? tx("自动记录中", "Auto tracking")
  : tx("记录已暂停", "Tracking paused");
```

Pass `trackingMood` into `renderPetSummary`.

- [ ] **Step 2: Do not change timer frequency**

Keep the existing:

```ts
window.setInterval(..., 5000)
```

No 1-second animation or synthetic counter increment is introduced.

- [ ] **Step 3: Run frontend validation**

Run:

```bash
pnpm run typecheck
pnpm run build:check
```

Expected: pass.

- [ ] **Step 4: Commit Task 7**

```bash
git add src/pet.ts
git commit -m "feat(pet): show persisted tracking status"
```

---

### Task 8: Clear recovered Home read errors without masking write failures

**Files:**
- Modify: `src/composables/useErrorMessage.ts`
- Modify: `src/composables/useHomeData.ts`
- Modify: `src/App.vue` wiring

**Interfaces:**
- Produces: `clearErrorMessage()`.
- Home core refresh clears the global read error only after its own successful `Promise.all` completes.
- Tracking/onboarding action failures stay visible until another explicit user action or relevant successful operation replaces/clears them.

- [ ] **Step 1: Add an explicit clear function**

In `useErrorMessage.ts`:

```ts
function clearErrorMessage() {
  error.value = "";
}

return {
  error,
  setErrorMessage,
  clearErrorMessage,
};
```

- [ ] **Step 2: Clear only after successful Home core data assignment**

Add `clearErrorMessage: () => void` to `UseHomeDataOptions`. In `refreshHomeCoreData()`, call it only after all five reads succeed and the refs are assigned:

```ts
todaySummary.value = summary;
recentLogs.value = logs;
pendingRuleProcesses.value = pendingRules;
idlePrompts.value = pendingIdle;
reminders.value = reminderRows;
clearErrorMessage();
```

Do not clear in `finally`, and do not clear on a failed refresh.

- [ ] **Step 3: Wire `clearErrorMessage` from `App.vue` and run gates**

Run:

```bash
pnpm run typecheck
pnpm run build:check
```

Expected: pass.

- [ ] **Step 4: Commit Task 8**

```bash
git add src/composables/useErrorMessage.ts src/composables/useHomeData.ts src/App.vue
git commit -m "fix(ui): clear recovered home read errors"
```

---

### Task 9: Update acceptance docs and perform final automated verification

**Files:**
- Modify: `docs/SMOKE_TESTS.md`
- Modify: `docs/RELEASE_CHECKLIST.md`
- Review only: `.github/workflows/ci.yml`, `.github/workflows/release.yml`

**Interfaces:**
- Produces: explicit fresh-install and upgrade smoke checklist tied to the corrected installer.
- Does not alter the release workflow unless verification exposes a separate workflow defect.

- [ ] **Step 1: Add a first-run smoke section**

Document these exact manual assertions in `docs/SMOKE_TESTS.md`:

```text
Fresh install / clean app-data directory
1. Launch TimePrism.
2. Confirm no `no such table: app_usage_logs` message appears.
3. Before activation, switch foreground apps for >10 seconds; confirm no usage rows are captured.
4. Confirm the onboarding disclosure states foreground app/window metadata, local SQLite storage, browser privacy, no screenshots/keystroke contents, and Insights vs 学/休 semantics.
5. Click 开始使用 TimePrism.
6. Confirm capture begins without restart.
7. Confirm Insights shows observed usage after sampling.
8. Confirm pet 学/休 remains classification-based and updates on the existing several-second cadence.
9. Pause capture; wait >10 seconds while switching apps; confirm no new capture is appended.
10. Fully quit and reopen; confirm capture remains paused and onboarding does not reappear.
11. Resume capture; quit/reopen again; confirm capture automatically resumes.
12. Confirm close-to-background, pet behavior, and Quit TimePrism remain correct.
```

Add a separate upgrade assertion: a pre-change DB without the two new keys starts as onboarded + capture enabled, without showing onboarding.

- [ ] **Step 2: Add the first-run gate to `RELEASE_CHECKLIST.md`**

Before public release, require both a clean-data fresh install and an upgrade-from-old-DB smoke. Keep the unsigned/SmartScreen note.

- [ ] **Step 3: Run the full repository gates from a clean dependency/build state as practical**

Run:

```bash
pnpm install --frozen-lockfile
pnpm run typecheck
pnpm run build:check
cd src-tauri
cargo check --all-targets
cargo test
```

Expected: all pass. Record the Rust test count from actual output; do not reuse the previous `35 passed` claim if the suite count changes.

- [ ] **Step 4: Push and require both Windows workflows to pass on the final HEAD**

Verify GitHub Actions:

```text
CI
- typecheck
- Vite build
- cargo check --all-targets
- cargo test

Windows Release PR packaging
- synchronized version check
- frontend typecheck
- cargo test
- Tauri NSIS build
- Tauri MSI build
- installer artifact upload
```

The PR packaging run must generate both `*-setup.exe` and `*.msi` on the final code HEAD.

- [ ] **Step 5: Commit docs if they were not included with an earlier task**

```bash
git add docs/SMOKE_TESTS.md docs/RELEASE_CHECKLIST.md
git commit -m "docs: add first-run tracking release gates"
```

- [ ] **Step 6: Stop before merge and perform the real Windows acceptance test**

Use a clean/sandboxed app-data directory for the fresh path and a backed-up pre-change database for the upgrade path. Do not mark PR #2 ready or publish a release until both pass.

After the corrected build passes, capture sanitized current-build screenshots for Home, Insights, Focus Guard, and the desktop pet, then wire those real files into README as the separate product-media gate already documented in `docs/media/README.md`.

---

## Final Review Checklist

Before claiming implementation complete:

- [ ] Fresh database defaults are `false / false`.
- [ ] Existing recognizable pre-change database defaults are `true / true`.
- [ ] Re-initialization preserves an explicit persisted pause.
- [ ] Backend rejects enabling capture before onboarding completion.
- [ ] Onboarding activation writes both state keys atomically.
- [ ] `tauri.conf.json` has `create: false` for `main`, `pet`, and `pet-panel`.
- [ ] `.setup()` initializes/migrates DB before creating the main WebView.
- [ ] DB initialization failure prevents normal runtime startup.
- [ ] Pre-onboarding main UI does not start foreground sampling or normal product polling.
- [ ] Pre-onboarding pet cannot be summoned via startup or global shortcut.
- [ ] Successful activation starts normal runtime without restart and shows the pet.
- [ ] Guard pause/resume is persisted before sampler state changes.
- [ ] Restart preserves pause; restart after resume automatically tracks.
- [ ] Sampler and UI refresh intervals remain 5 seconds.
- [ ] First capture remains baseline-only.
- [ ] Pet counters remain classified `LEARN / REST` totals and mood shows active vs paused.
- [ ] A recovered transient Home read error does not remain as a stale red footer.
- [ ] `pnpm run typecheck`, `pnpm run build:check`, `cargo check --all-targets`, and `cargo test` pass on final HEAD.
- [ ] Final Windows PR packaging produces both NSIS and MSI installers.
- [ ] Fresh-install and upgrade manual acceptance tests pass before PR/release promotion.
