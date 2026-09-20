# TimePrism First-Run Tracking Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make a fresh TimePrism install initialize SQLite before any product WebView can query it, show a one-time tracking disclosure before foreground capture starts, and persist tracking pause/resume state across restarts.

**Architecture:** Keep the existing `commands -> services -> domain/db` structure. Add a small SQLite-backed tracking-state service, manually create configured Tauri windows only after database initialization, and gate the Vue product runtime behind a persisted `TrackingState`. The foreground sampler keeps its current approximately 5-second cadence; persistence and onboarding policy stay outside the sampler.

**Tech Stack:** Tauri 2, Rust, rusqlite/SQLite, Vue 3, TypeScript, Vite, GitHub Actions on Windows.

**Spec:** `docs/superpowers/specs/2026-09-08-timeprism-first-run-tracking-design.md`

## Global Constraints

- Fresh databases start with `onboarding_completed=false` and `auto_capture_enabled=false`.
- Existing pre-change TimePrism databases migrate to `onboarding_completed=true` and `auto_capture_enabled=true` unless a key already exists.
- Existing values are never overwritten by migration defaults.
- No foreground capture may occur before onboarding activation succeeds.
- A user-initiated pause persists across restart; restart must never silently turn capture back on.
- Keep foreground sampling, Home refresh, and pet refresh at approximately 5 seconds.
- The first foreground sample remains baseline-only.
- No new frontend test framework is added solely for this feature.
- Database initialization failure prevents normal product runtime startup.
- Do not perform another broad backend/frontend directory refactor.
- Product copy must state foreground app/window metadata, local storage, browser privacy processing, no screenshots/keystroke contents, and the difference between observed activity and classified `LEARN` / `REST` time.

---

## File Structure

### Create

- `src-tauri/src/domain/tracking.rs` — serialized backend `TrackingState`.
- `src-tauri/src/services/tracking.rs` — SQLite-backed tracking invariants/transitions.
- `src/api/commands/tracking.ts` — typed tracking-state Tauri wrappers.
- `src/api/commands/window.ts` — typed `showMainWindow` / `summonPetWindow` wrappers.
- `src/composables/useTrackingState.ts` — persisted tracking state only; no timer ownership.
- `src/components/OnboardingView.vue` — one-screen first-run disclosure.

### Modify

- `src-tauri/src/db/migrations.rs`, `src-tauri/src/db/tests.rs`
- `src-tauri/src/domain/mod.rs`, `src-tauri/src/services/mod.rs`
- `src-tauri/src/lib.rs`, `src-tauri/src/services/window.rs`, `src-tauri/tauri.conf.json`
- `src/api/types.ts`, `src/api.ts`, `src/api/commands/index.ts`
- `src/composables/useAutoCaptureSampler.ts`, `src/composables/useGuardData.ts`, `src/composables/useMainWindowLifecycle.ts`
- `src/composables/useErrorMessage.ts`, `src/composables/useHomeData.ts`
- `src/lib/guardFeedback.ts`
- `src/App.vue`, `src/main.ts`, `src/pet.ts`, `src/styles/app.css`
- `docs/SMOKE_TESTS.md`, `docs/RELEASE_CHECKLIST.md`

---

### Task 1: Add migration-safe tracking defaults

**Files:**
- Modify: `src-tauri/src/db/migrations.rs` (`initialize_connection` and helpers)
- Modify/Test: `src-tauri/src/db/tests.rs` (fresh/idempotent/legacy tests)

**Interfaces:**
- Consumes: existing `app_config(key TEXT PRIMARY KEY, value TEXT NOT NULL)`.
- Produces: guaranteed `onboarding_completed` and `auto_capture_enabled` keys after successful DB initialization.

- [ ] **Step 1: Add RED assertions for a fresh DB**

Extend `fresh_file_database_initialization_seeds_schema_and_defaults_and_reopens`:

```rust
let onboarding: String = conn.query_row(
    "SELECT value FROM app_config WHERE key = 'onboarding_completed'",
    [],
    |row| row.get(0),
).expect("read onboarding default");
let auto_capture: String = conn.query_row(
    "SELECT value FROM app_config WHERE key = 'auto_capture_enabled'",
    [],
    |row| row.get(0),
).expect("read auto capture default");
assert_eq!(onboarding, "false");
assert_eq!(auto_capture, "false");
```

- [ ] **Step 2: Add RED assertions for a recognizable legacy DB**

Extend `legacy_schema_is_upgraded_without_losing_existing_rows` after `initialize_connection`:

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

- [ ] **Step 3: Add RED regression for a persisted pause**

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

- [ ] **Step 4: Run focused tests and verify RED**

```bash
cd src-tauri
cargo test db::tests::fresh_file_database_initialization_seeds_schema_and_defaults_and_reopens
cargo test db::tests::legacy_schema_is_upgraded_without_losing_existing_rows
cargo test db::tests::database_initialization_preserves_persisted_tracking_pause
```

Expected: missing tracking keys cause failure.

- [ ] **Step 5: Detect recognizable TimePrism schema before creating tables**

Add before `initialize_connection`:

```rust
fn has_recognizable_timeprism_schema(conn: &Connection) -> Result<bool, String> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*)
         FROM sqlite_master
         WHERE type = 'table'
           AND name IN ('categories', 'task_sessions', 'app_usage_logs', 'app_rules', 'app_config', 'reminders')",
        [],
        |row| row.get(0),
    ).map_err(|e| format!("failed to inspect existing TimePrism schema: {e}"))?;
    Ok(count > 0)
}
```

At the very start of `initialize_connection`, before the existing schema `CREATE TABLE` batch:

```rust
let was_existing_timeprism_db = has_recognizable_timeprism_schema(conn)?;
```

- [ ] **Step 6: Seed migration-safe defaults after `app_config` exists**

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
    ).map_err(|e| format!("failed to seed tracking config {key}: {e}"))?;
}
```

`INSERT OR IGNORE` is required so existing explicit values survive re-initialization.

- [ ] **Step 7: Verify GREEN and full regression**

```bash
cd src-tauri
cargo test db::tests::fresh_file_database_initialization_seeds_schema_and_defaults_and_reopens
cargo test db::tests::legacy_schema_is_upgraded_without_losing_existing_rows
cargo test db::tests::database_initialization_preserves_persisted_tracking_pause
cargo test
```

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/db/migrations.rs src-tauri/src/db/tests.rs
git commit -m "feat(db): persist first-run tracking defaults"
```

---

### Task 2: Add the backend tracking-state invariant

**Files:**
- Create: `src-tauri/src/domain/tracking.rs`
- Create/Test: `src-tauri/src/services/tracking.rs`
- Modify: `src-tauri/src/domain/mod.rs`, `src-tauri/src/services/mod.rs`
- Modify: `src-tauri/src/lib.rs` command imports/handlers

**Interfaces:**
- Produces: `TrackingState { onboarding_completed: bool, auto_capture_enabled: bool }`.
- Produces: `get_tracking_state_entry(&Connection) -> Result<TrackingState, String>`.
- Produces: `complete_onboarding_entry(&Connection) -> Result<TrackingState, String>`.
- Produces: `set_auto_capture_enabled_entry(&Connection, bool) -> Result<TrackingState, String>`.
- Produces Tauri commands: `get_tracking_state`, `complete_tracking_onboarding`, `set_auto_capture_enabled`.

- [ ] **Step 1: Create and expose the domain contract**

`src-tauri/src/domain/tracking.rs`:

```rust
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TrackingState {
    pub onboarding_completed: bool,
    pub auto_capture_enabled: bool,
}
```

Add `pub mod tracking;` to `domain/mod.rs`.

- [ ] **Step 2: Create the tracking module, expose it, and add RED tests before functions**

Create `src-tauri/src/services/tracking.rs` with the tests below and add `pub mod tracking;` to `services/mod.rs` immediately so `cargo test services::tracking::tests` actually compiles the new module.

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
        assert_eq!(state.onboarding_completed, true);
        assert_eq!(state.auto_capture_enabled, true);
        assert_eq!(get_tracking_state_entry(&conn).unwrap(), state);
    }

    #[test]
    fn pause_and_resume_persist_after_onboarding() {
        let conn = tracking_conn(true, true);
        assert_eq!(set_auto_capture_enabled_entry(&conn, false).unwrap().auto_capture_enabled, false);
        assert_eq!(set_auto_capture_enabled_entry(&conn, true).unwrap().auto_capture_enabled, true);
    }

    #[test]
    fn enabling_capture_before_onboarding_is_rejected() {
        let conn = tracking_conn(false, false);
        let err = set_auto_capture_enabled_entry(&conn, true).unwrap_err();
        assert!(err.contains("onboarding"));
        assert_eq!(get_tracking_state_entry(&conn).unwrap().auto_capture_enabled, false);
    }
}
```

- [ ] **Step 3: Run and verify RED**

```bash
cd src-tauri
cargo test services::tracking::tests
```

Expected: compile failure because the referenced tracking functions do not exist.

- [ ] **Step 4: Implement strict reads and state transitions**

```rust
use rusqlite::{params, Connection};
use crate::domain::tracking::TrackingState;

fn read_bool_config(conn: &Connection, key: &str) -> Result<bool, String> {
    let value: String = conn.query_row(
        "SELECT value FROM app_config WHERE key = ?1",
        [key],
        |row| row.get(0),
    ).map_err(|e| format!("failed to read tracking config {key}: {e}"))?;

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

Complete onboarding with one transaction and UPSERT both keys so the invariant is robust even if a key was unexpectedly missing:

```rust
pub(crate) fn complete_onboarding_entry(conn: &Connection) -> Result<TrackingState, String> {
    let tx = conn.unchecked_transaction()
        .map_err(|e| format!("failed to begin onboarding transaction: {e}"))?;
    for key in ["onboarding_completed", "auto_capture_enabled"] {
        tx.execute(
            "INSERT INTO app_config (key, value) VALUES (?1, 'true')
             ON CONFLICT(key) DO UPDATE SET value = 'true'",
            [key],
        ).map_err(|e| format!("failed to persist onboarding config {key}: {e}"))?;
    }
    tx.commit().map_err(|e| format!("failed to commit onboarding: {e}"))?;
    get_tracking_state_entry(conn)
}
```

Implement `set_auto_capture_enabled_entry` as:

```rust
pub(crate) fn set_auto_capture_enabled_entry(
    conn: &Connection,
    enabled: bool,
) -> Result<TrackingState, String> {
    let current = get_tracking_state_entry(conn)?;
    if enabled && !current.onboarding_completed {
        return Err("cannot enable auto capture before onboarding is complete".to_string());
    }
    conn.execute(
        "INSERT INTO app_config (key, value) VALUES ('auto_capture_enabled', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [if enabled { "true" } else { "false" }],
    ).map_err(|e| format!("failed to persist auto capture state: {e}"))?;
    get_tracking_state_entry(conn)
}
```

- [ ] **Step 5: Verify GREEN**

```bash
cd src-tauri
cargo test services::tracking::tests
cargo test
```

- [ ] **Step 6: Add thin Tauri commands and register them**

In `lib.rs`:

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

Add all three to `tauri::generate_handler!`.

- [ ] **Step 7: Compile all targets and commit**

```bash
cd src-tauri
cargo check --all-targets
cargo test
cd ..
git add src-tauri/src/domain/tracking.rs src-tauri/src/domain/mod.rs src-tauri/src/services/tracking.rs src-tauri/src/services/mod.rs src-tauri/src/lib.rs
git commit -m "feat(tracking): add persisted tracking state invariant"
```

---

### Task 3: Eliminate the DB/WebView startup race

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Modify/Test: `src-tauri/src/services/window.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `get_tracking_state_entry` from Task 2.
- Produces: `create_configured_window(app, label) -> Result<WebviewWindow, String>`.
- Invariant: no product WebView exists before `init_database` succeeds.

- [ ] **Step 1: Add RED config test**

In `services/window.rs` tests:

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

- [ ] **Step 2: Verify RED**

```bash
cd src-tauri
cargo test services::window::tests::configured_product_windows_require_manual_creation
```

- [ ] **Step 3: Set all configured product windows to manual creation**

Add to each `main`, `pet`, `pet-panel` object in `tauri.conf.json`:

```json
"create": false
```

Do not change their other config values.

- [ ] **Step 4: Add config-driven creation helper**

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

- [ ] **Step 5: Make `.setup()` fail closed and order DB before windows**

Required order in `lib.rs`:

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

    // register existing shortcut only after DB/window readiness
    Ok(())
})
```

Delete the current `eprintln!("database init warning...")` continue-on-error behavior.

- [ ] **Step 6: Gate both shortcut and Tauri pet command before onboarding**

For the global shortcut handler: open DB, read `TrackingState`, and summon only when `onboarding_completed=true`. On read failure, log and do nothing.

For the `summon_pet_window` Tauri command wrapper, enforce the same backend policy:

```rust
#[tauri::command]
fn summon_pet_window(app: AppHandle) -> Result<(), String> {
    let conn = open_connection(&app)?;
    let state = get_tracking_state_entry(&conn)?;
    if !state.onboarding_completed {
        return Err("pet is unavailable before onboarding is complete".to_string());
    }
    drop(conn);
    window_service::summon_pet_window(&app)
}
```

The window service itself remains DB-agnostic.

- [ ] **Step 7: Verify Rust**

```bash
cd src-tauri
cargo test services::window::tests::configured_product_windows_require_manual_creation
cargo check --all-targets
cargo test
```

- [ ] **Step 8: Commit**

```bash
git add src-tauri/tauri.conf.json src-tauri/src/services/window.rs src-tauri/src/lib.rs
git commit -m "fix(startup): initialize database before webviews"
```

---

### Task 4: Add typed frontend tracking/window APIs and make the sampler mechanism-only

**Files:**
- Modify: `src/api/types.ts`, `src/api.ts`, `src/api/commands/index.ts`
- Create: `src/api/commands/tracking.ts`, `src/api/commands/window.ts`
- Create: `src/composables/useTrackingState.ts`
- Modify: `src/composables/useAutoCaptureSampler.ts`

**Interfaces:**
- Produces TS `TrackingState`.
- Produces `getTrackingState`, `completeTrackingOnboarding`, `setAutoCaptureEnabled`.
- Produces `showMainWindow`, `summonPetWindow`.
- Produces `useTrackingState` refs/methods.
- Sampler owns only immediate sample + 5-second interval + stop.

- [ ] **Step 1: Add typed contracts/wrappers**

`src/api/types.ts`:

```ts
export type TrackingState = {
  onboarding_completed: boolean;
  auto_capture_enabled: boolean;
};
```

Export `TrackingState` from `src/api.ts`.

`src/api/commands/tracking.ts`:

```ts
import { invokeCommand } from "../client";
import type { TrackingState } from "../types";

export const getTrackingState = (): Promise<TrackingState> =>
  invokeCommand("get_tracking_state");

export const completeTrackingOnboarding = (): Promise<TrackingState> =>
  invokeCommand("complete_tracking_onboarding");

export const setAutoCaptureEnabled = (enabled: boolean): Promise<TrackingState> =>
  invokeCommand("set_auto_capture_enabled", { enabled });
```

`src/api/commands/window.ts`:

```ts
import { invokeCommand } from "../client";

export async function showMainWindow(): Promise<void> {
  await invokeCommand("show_main_window");
}

export async function summonPetWindow(): Promise<void> {
  await invokeCommand("summon_pet_window");
}
```

Export both new modules from `src/api/commands/index.ts`.

- [ ] **Step 2: Create persistence-only `useTrackingState`**

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

  async function loadTrackingState(): Promise<TrackingState> {
    trackingStateLoading.value = true;
    try {
      const next = await getTrackingState();
      trackingState.value = next;
      return next;
    } catch (error) {
      setErrorMessage(error);
      throw error;
    } finally {
      trackingStateLoading.value = false;
    }
  }

  async function activateTracking(): Promise<TrackingState> {
    trackingActionLoading.value = true;
    try {
      const next = await completeTrackingOnboarding();
      trackingState.value = next;
      return next;
    } catch (error) {
      setErrorMessage(error);
      throw error;
    } finally {
      trackingActionLoading.value = false;
    }
  }

  async function persistAutoCaptureEnabled(enabled: boolean): Promise<TrackingState> {
    trackingActionLoading.value = true;
    try {
      const next = await setAutoCaptureEnabled(enabled);
      trackingState.value = next;
      return next;
    } catch (error) {
      setErrorMessage(error);
      throw error;
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

No optimistic ref mutation is allowed.

- [ ] **Step 3: Remove policy ownership from `useAutoCaptureSampler`**

Delete `autoCaptureEnabled` from `AutoCaptureSamplerOptions` and delete the early return in `sampleAutoCapture()`. The sampler runs only when `startAutoCaptureSampler()` has been called. Preserve:

```ts
function startAutoCaptureSampler() {
  stopAutoCaptureSampler();
  sampleAutoCapture();
  captureTimer = window.setInterval(sampleAutoCapture, intervalMs);
}
```

Keep default `intervalMs = 5000`.

- [ ] **Step 4: Static verification and commit**

```bash
pnpm run typecheck
pnpm run build:check
git add src/api/types.ts src/api.ts src/api/commands/tracking.ts src/api/commands/window.ts src/api/commands/index.ts src/composables/useTrackingState.ts src/composables/useAutoCaptureSampler.ts
git commit -m "feat(ui): add persisted tracking state client"
```

---

### Task 5: Gate Vue runtime, delay main reveal until state is loaded, and persist Guard pause/resume

**Files:**
- Modify: `src/composables/useMainWindowLifecycle.ts`
- Modify: `src/composables/useGuardData.ts`
- Modify: `src/lib/guardFeedback.ts`
- Modify: `src/App.vue`
- Modify: `src/main.ts`

**Interfaces:**
- Consumes: Task 4 tracking state/APIs.
- Produces: `startProductRuntime(captureEnabled)`, `syncCaptureRuntime(captureEnabled)`, `stopProductRuntime`.
- Main WebView remains hidden until tracking state has loaded; first visible UI is onboarding or normal product shell, not an empty shell.

- [ ] **Step 1: Remove unconditional main reveal from `src/main.ts`**

Replace current mount-plus-`invoke("show_main_window")` code with:

```ts
import { createApp } from "vue";
import App from "./App.vue";
import "./styles/app.css";

createApp(App).mount("#app");
```

Main window is already created hidden by Rust after DB init. It will be revealed only after tracking state loads.

- [ ] **Step 2: Refactor lifecycle into shell-init + idempotent product runtime**

Add `loadTrackingState` and `showMainWindow` dependencies. `onMounted` order:

```ts
onMounted(async () => {
  initLocale();
  initThemeModeSafely();

  const tracking = await loadTrackingState();
  await showMainWindow();

  if (tracking.onboarding_completed) {
    await startProductRuntime(tracking.auto_capture_enabled);
  }
});
```

Use:

```ts
let productRuntimeStarted = false;

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

function stopProductRuntime() {
  stopMainRefreshPolling();
  stopAutoCaptureSampler();
  cleanupSettingsWarmup();
  cleanupInsightsSectionNavigation();
  cleanupHeatmapGoalSetting();
  productRuntimeStarted = false;
}
```

Return `startProductRuntime`, `syncCaptureRuntime`, `stopProductRuntime`; call `stopProductRuntime()` on unmount.

- [ ] **Step 3: Make Guard consume the authoritative persisted state**

Change `UseGuardDataOptions` to include:

```ts
autoCaptureEnabled: Readonly<Ref<boolean>>;
setAutoCaptureEnabled: (enabled: boolean) => Promise<void>;
```

Import `type Ref` from Vue and delete local `const autoCaptureEnabled = ref(true)`.

`onAutoCaptureToggle` becomes async:

```ts
async function onAutoCaptureToggle(event: Event) {
  const enabled = guardCheckedFromEvent(event);
  try {
    await setAutoCaptureEnabled(enabled);
    autoCaptureFeedback.value = guardAutoCaptureStateFeedback(enabled, tx);
  } catch (error) {
    setErrorMessage(error);
  }
}
```

Add to `guardFeedback.ts`:

```ts
export function guardAutoCaptureStateFeedback(enabled: boolean, tx: TranslateFn): string {
  return enabled
    ? tx("自动采样已开启", "Auto capture enabled")
    : tx("记录已暂停；重启后仍保持暂停", "Tracking paused; pause persists after restart");
}
```

- [ ] **Step 4: Wire persistence to sampler only after backend success**

In `App.vue`:

```ts
async function handleSetAutoCaptureEnabled(enabled: boolean) {
  const state = await persistAutoCaptureEnabled(enabled);
  syncCaptureRuntime(state.auto_capture_enabled);
}
```

Pass `autoCaptureEnabled` and `handleSetAutoCaptureEnabled` into `useGuardData`.

- [ ] **Step 5: Verify and commit**

```bash
pnpm run typecheck
pnpm run build:check
git add src/main.ts src/composables/useMainWindowLifecycle.ts src/composables/useGuardData.ts src/lib/guardFeedback.ts src/App.vue
git commit -m "feat(tracking): gate runtime on persisted state"
```

---

### Task 6: Add one-time onboarding and activation transition

**Files:**
- Create: `src/components/OnboardingView.vue`
- Modify: `src/App.vue`, `src/styles/app.css`

**Interfaces:**
- Emits `activate` only; no persistent browse-without-tracking branch.
- Activation order: backend commit -> start runtime -> reveal/create pet.

- [ ] **Step 1: Create disclosure component**

```ts
const props = defineProps<{
  tx: (zh: string, en: string) => string;
  loading: boolean;
}>();
const emit = defineEmits<{ activate: [] }>();
```

Required Chinese meaning (English copy must match):

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

Disable the primary button while `loading`.

- [ ] **Step 2: Add focused onboarding styles**

Use `.onboarding-shell`, `.onboarding-card`, `.onboarding-points`, `.onboarding-primary`; reuse existing colors/radii/spacing. Fit within 888x900 without default-scale scrolling.

- [ ] **Step 3: Render onboarding before normal product shell**

```vue
<OnboardingView
  v-if="!trackingStateLoading && !onboardingCompleted"
  :tx="tx"
  :loading="trackingActionLoading"
  @activate="handleActivateTracking"
/>

<main v-else-if="!trackingStateLoading" class="layout">
  <!-- existing nav/views -->
</main>
```

The hidden main window is revealed by Task 5 only after tracking state resolves, so users do not see an empty shell during this conditional decision.

- [ ] **Step 4: Implement activation without optimistic tracking**

```ts
async function handleActivateTracking() {
  try {
    const state = await activateTracking();
    await startProductRuntime(state.auto_capture_enabled);
    try {
      await summonPetWindow();
    } catch (petError) {
      setErrorMessage(petError);
    }
  } catch (error) {
    setErrorMessage(error);
  }
}
```

A backend activation failure leaves onboarding visible and sampler stopped. A later pet-show failure does not roll back successfully persisted activation.

- [ ] **Step 5: Verify and commit**

```bash
pnpm run typecheck
pnpm run build:check
cd src-tauri
cargo check --all-targets
cargo test
cd ..
git add src/components/OnboardingView.vue src/App.vue src/styles/app.css
git commit -m "feat(onboarding): require first-run tracking activation"
```

---

### Task 7: Make pet show active vs paused tracking state

**Files:**
- Modify: `src/pet.ts`

**Interfaces:**
- Consumes `getTrackingState` + `getTodaySummary`.
- Counters remain classified `LEARN` / `REST`; only mood reflects tracking state.

- [ ] **Step 1: Read state in existing summary refresh**

```ts
const [summary, tracking] = await Promise.all([
  getTodaySummary(),
  getTrackingState(),
]);
const trackingMood = tracking.auto_capture_enabled
  ? tx("自动记录中", "Auto tracking")
  : tx("记录已暂停", "Tracking paused");
```

Pass `formatSeconds(summary.learn_seconds)`, `formatSeconds(summary.rest_seconds)`, and `trackingMood` to `renderPetSummary`.

- [ ] **Step 2: Preserve 5-second cadence**

Keep the existing `window.setInterval(..., 5000)` and baseline-only capture behavior. Do not synthesize per-second increments.

- [ ] **Step 3: Verify and commit**

```bash
pnpm run typecheck
pnpm run build:check
git add src/pet.ts
git commit -m "feat(pet): show persisted tracking status"
```

---

### Task 8: Clear only the recovered Home error, never a newer tracking error

**Files:**
- Modify: `src/composables/useErrorMessage.ts`
- Modify: `src/composables/useHomeData.ts`
- Modify: `src/App.vue`

**Interfaces:**
- Produces compare-and-clear API: `clearErrorMessage(expected?: unknown)`.
- Home remembers the exact error string it last published and clears only that same string after recovery.

- [ ] **Step 1: Add compare-and-clear semantics**

```ts
function clearErrorMessage(expected?: unknown) {
  if (expected === undefined || error.value === `${expected}`) {
    error.value = "";
  }
}
```

Return it with `error` and `setErrorMessage`.

- [ ] **Step 2: Track the last Home-owned read error**

In `useHomeData`:

```ts
let lastHomeCoreError = "";
```

Add `clearErrorMessage: (expected?: unknown) => void` to options. On successful core assignment:

```ts
todaySummary.value = summary;
recentLogs.value = logs;
pendingRuleProcesses.value = pendingRules;
idlePrompts.value = pendingIdle;
reminders.value = reminderRows;
if (lastHomeCoreError) {
  clearErrorMessage(lastHomeCoreError);
  lastHomeCoreError = "";
}
```

On failure:

```ts
lastHomeCoreError = `${e}`;
setErrorMessage(e);
```

If a newer tracking write error has replaced the global message, `clearErrorMessage(lastHomeCoreError)` does nothing, so polling cannot erase the newer failure.

- [ ] **Step 3: Wire and verify**

```bash
pnpm run typecheck
pnpm run build:check
git add src/composables/useErrorMessage.ts src/composables/useHomeData.ts src/App.vue
git commit -m "fix(ui): clear only recovered home read errors"
```

---

### Task 9: Update acceptance docs and run final automated gates

**Files:**
- Modify: `docs/SMOKE_TESTS.md`, `docs/RELEASE_CHECKLIST.md`
- Review only unless a new defect appears: `.github/workflows/ci.yml`, `.github/workflows/release.yml`

- [ ] **Step 1: Add exact fresh-install smoke assertions**

Document:

```text
1. Clean/sandbox the TimePrism app-data directory and launch.
2. Confirm no `no such table: app_usage_logs` error appears.
3. Before activation, switch foreground apps for >10 seconds and confirm no usage rows are captured.
4. Confirm onboarding explains foreground app/window metadata, local SQLite, browser privacy, no screenshots/keystroke contents, and Insights vs 学/休 semantics.
5. Click 开始使用 TimePrism; confirm capture begins without restart.
6. Confirm Insights shows observed activity after sampling.
7. Confirm pet 学/休 remains classification-based and updates on the existing several-second cadence.
8. Pause capture; switch apps for >10 seconds; confirm no new samples are appended.
9. Fully quit/reopen; confirm pause persists and onboarding does not reappear.
10. Resume; quit/reopen; confirm automatic tracking resumes.
11. Confirm pet says paused while paused.
12. Confirm close-to-background, pet, and Quit TimePrism behavior still works.
```

Add upgrade test: pre-change DB without the new keys starts onboarded + capture enabled without showing onboarding.

- [ ] **Step 2: Add fresh + upgrade gates to release checklist**

Require both before public release; keep unsigned/SmartScreen note.

- [ ] **Step 3: Run repository gates**

```bash
pnpm install --frozen-lockfile
pnpm run typecheck
pnpm run build:check
cd src-tauri
cargo check --all-targets
cargo test
```

Record actual test count from output; do not reuse `35 passed` if it changes.

- [ ] **Step 4: Push final HEAD and require both Windows workflows GREEN**

CI must pass typecheck, Vite build, `cargo check --all-targets`, `cargo test`.

Windows Release PR packaging must pass synchronized-version check, typecheck, Rust tests, `tauri build --bundles nsis msi`, and artifact upload. Verify both `*-setup.exe` and `*.msi` exist on the final code HEAD.

- [ ] **Step 5: Commit docs**

```bash
git add docs/SMOKE_TESTS.md docs/RELEASE_CHECKLIST.md
git commit -m "docs: add first-run tracking release gates"
```

- [ ] **Step 6: Stop before merge for real Windows acceptance**

Use a clean app-data directory for fresh install and a backed-up pre-change DB for upgrade. Do not mark PR #2 ready or publish a release until both pass.

After acceptance, capture sanitized Home / Insights / Focus Guard / pet screenshots from this corrected build and wire only those real product images into README.

---

## Final Review Checklist

- [ ] Fresh DB defaults are `false / false`.
- [ ] Existing recognizable pre-change DB defaults are `true / true`.
- [ ] Re-initialization preserves explicit pause.
- [ ] Backend rejects enabling capture before onboarding.
- [ ] Onboarding activation atomically persists `true / true`.
- [ ] `main`, `pet`, `pet-panel` all have `create:false`.
- [ ] DB initializes before main WebView creation.
- [ ] DB init failure prevents normal startup.
- [ ] Main stays hidden until tracking state loads.
- [ ] Pre-onboarding runtime starts no sampler/polling.
- [ ] Pre-onboarding pet cannot be created through startup, shortcut, or Tauri command.
- [ ] Activation starts runtime without restart and then reveals pet.
- [ ] Guard pause/resume persists before sampler changes.
- [ ] Restart preserves pause; restart after resume auto-tracks.
- [ ] Sampling/Home/pet intervals remain 5 seconds.
- [ ] First sample remains baseline-only.
- [ ] Pet counters remain `LEARN / REST`; mood shows active vs paused.
- [ ] Recovered Home error clears only if still the active Home-owned error.
- [ ] Final typecheck/build/Rust gates pass.
- [ ] Final Windows packaging produces NSIS + MSI.
- [ ] Fresh-install and upgrade manual acceptance pass before PR/release promotion.
