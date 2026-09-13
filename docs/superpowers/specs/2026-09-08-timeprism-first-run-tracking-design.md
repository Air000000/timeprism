# TimePrism First-Run Tracking Design

Date: 2026-09-08
Status: Proposed / user-approved direction, pending written-spec review
Scope: First-run startup safety, tracking consent/activation, persistent tracking state, and first-run explanation

## 1. Problem statement

A real Windows installer smoke test exposed three related problems:

1. A fresh install can briefly render `failed to prepare summary query: no such table: app_usage_logs` even though the database becomes usable moments later.
2. Raw application usage can appear in Insights while the desktop pet remains at `0`, which is technically valid but unexplained: Insights includes observed application time while the pet only shows time classified as `LEARN` or `REST`.
3. A first-time user is dropped directly into the product with foreground capture already enabled, before the product explains what is collected, where it is stored, or how learning/rest classification works.

The current behavior also stores the auto-capture toggle only in frontend memory, so an explicit pause is lost after restart.

## 2. Goals

This change must:

- eliminate the fresh-install database/WebView startup race;
- ensure no foreground capture begins before the first-run explanation is acknowledged;
- match the common automatic-tracker model: one-time activation, then automatic tracking on future launches unless the user pauses it;
- persist the user's tracking pause/resume choice across restarts;
- keep the current approximately 5-second capture and UI refresh cadence;
- explain the difference between observed application activity and classified `LEARN` / `REST` time;
- preserve existing-user behavior during migration;
- fail closed if the database cannot be initialized rather than launching a partially initialized product shell.

## 3. Non-goals

This work will not:

- change the 5-second foreground sampling interval;
- change the 5-second Home/pet refresh cadence;
- redesign the analytics model;
- add cloud accounts, telemetry, or remote consent storage;
- add a multi-step tutorial carousel;
- add screenshots, keyboard logging, or any new capture source;
- re-architect the existing `services / domain / db` backend structure;
- change application classification thresholds except where needed to explain them in UI copy.

## 4. Product behavior

### 4.1 Fresh installation

A genuinely new TimePrism database starts with:

- `onboarding_completed = false`
- `auto_capture_enabled = false`

The first visible product surface is a one-time welcome / tracking-disclosure screen in the main window.

Before the user accepts:

- foreground capture is not started;
- no foreground application metadata is read by TimePrism's sampler;
- Home/Insights/Guard polling is not started;
- the desktop pet is not shown as an active tracking surface;
- normal product data reads that are unrelated to rendering the onboarding screen are not started.

The primary action is:

`开始使用 TimePrism`

The user may close/quit the application instead of accepting. No separate persistent "browse without tracking" mode is introduced.

### 4.2 First-run disclosure

The welcome screen must state, in plain language:

- TimePrism identifies the current foreground application and measures how long it remains active;
- application/window metadata can be stored locally for time analysis;
- browser titles are privacy-processed by default;
- private/incognito browser windows are not recorded by the current privacy rules;
- TimePrism does not capture screenshots or keystroke contents;
- core data is stored locally in SQLite and is not uploaded by the current product;
- Insights can show observed application activity, while the pet's `学 / 休` counters only include applications classified as `LEARN` or `REST`.

The disclosure is informational and activation-oriented rather than a long legal flow.

### 4.3 Activation

When the user selects `开始使用 TimePrism`, the backend performs one atomic product-state transition:

- `onboarding_completed = true`
- `auto_capture_enabled = true`

Only after that transition succeeds may the frontend start the capture sampler and normal product polling.

The first capture still behaves as a baseline-only sample; this is retained to avoid synthetic +5-second overcounting.

### 4.4 Subsequent launches

On later launches:

- if `onboarding_completed = true` and `auto_capture_enabled = true`, TimePrism automatically resumes normal capture;
- if the user previously paused capture, `auto_capture_enabled = false` remains false after restart;
- onboarding is not shown again after successful first activation.

This matches the intended automatic-tracker model: activate once, then run automatically unless explicitly paused.

### 4.5 Pause / resume semantics

The existing auto-capture control becomes persistent.

Changing the toggle updates the SQLite-backed tracking preference, not only a Vue `ref`.

Rules:

- pause: persist `auto_capture_enabled = false`, then stop the sampler;
- resume: persist `auto_capture_enabled = true`, then start the sampler;
- a failed persistence write must not silently change the runtime state;
- enabling capture before onboarding is complete is rejected by the backend invariant rather than trusted to the UI.

## 5. Startup architecture

### 5.1 Root cause to remove

The current configuration allows WebViews to be created before `.setup()` completes, while database initialization currently runs inside `.setup()`. Main and pet frontends can therefore invoke database-backed commands before `app_usage_logs` and the rest of the schema exist.

### 5.2 Required startup ordering

The startup order becomes:

```text
Tauri process starts
    -> resolve/open database path
    -> run schema initialization and migrations
    -> load tracking state
    -> create desktop WebViews
    -> render main window
    -> if onboarding incomplete: show onboarding only
    -> if onboarding complete: start normal product runtime according to persisted auto_capture_enabled
```

The key invariant is:

> No database-backed product WebView may issue product queries before database initialization has completed successfully.

### 5.3 Window creation

Configured windows should no longer auto-create before database initialization.

The Tauri window configs remain the source of truth for window shape/URL/behavior, but startup code creates the required WebViews only after successful database initialization.

Main-window reveal still occurs after Vue mount so the prior blank-startup regression remains fixed.

Desktop-pet behavior:

- on first run before activation: keep the pet hidden;
- after first activation: show/settle the pet using the existing window service;
- on subsequent launches: pet visibility follows the normal application behavior, independent of whether tracking is paused;
- when tracking is paused, the pet displays a paused/not-recording state rather than implying active capture.

### 5.4 Database initialization failure

Database initialization is no longer treated as a warning that allows the app to continue.

If initialization/migration fails:

- do not start normal product WebViews and polling;
- do not start foreground capture;
- fail startup with an explicit error path/log rather than showing a partially functional UI that later emits table-missing errors.

A richer recovery UI is out of scope for this change.

## 6. Persistent state model

Use the existing `app_config` table rather than introducing another settings database.

Required keys:

- `onboarding_completed`
- `auto_capture_enabled`

A backend tracking-state model should expose both values together, e.g. conceptually:

```text
TrackingState {
  onboarding_completed: bool,
  auto_capture_enabled: bool
}
```

Backend operations should be explicit rather than exposing arbitrary `app_config` writes to the frontend:

- read current tracking state;
- complete onboarding and enable tracking atomically;
- set auto-capture enabled/disabled after onboarding.

This preserves invariants in Rust rather than relying on frontend sequencing.

## 7. Migration semantics

Existing installations must not unexpectedly stop tracking after upgrading.

Initialization must distinguish a genuinely fresh database from a pre-existing TimePrism database before inserting the new tracking-state keys.

### Fresh database

```text
onboarding_completed = false
auto_capture_enabled = false
```

### Existing database without the new keys

```text
onboarding_completed = true
auto_capture_enabled = true
```

Rationale: existing users have already been using automatic capture; forcing them through a new consent screen or silently disabling capture would be a backward-incompatible behavior change.

### Existing database with the new keys

Never overwrite user choices.

In particular:

- a persisted pause remains paused;
- completed onboarding remains completed.

An empty/corrupt file with no recognizable TimePrism schema can be treated as a fresh database; database corruption recovery beyond that is out of scope.

## 8. Frontend runtime gate

The current main-window lifecycle starts Home refresh, polling, settings warmup, and the auto-capture sampler on mount. This must be split into two phases.

### Phase A: shell initialization

Allowed before onboarding completion:

- locale initialization;
- theme initialization;
- load tracking state;
- render/reveal the main window;
- render onboarding when needed.

### Phase B: product runtime

Started only after onboarding is complete:

- Home core data refresh;
- Home analytics refresh scheduling;
- settings warmup;
- main refresh polling;
- auto-capture sampler only if `auto_capture_enabled = true`;
- Insights/Guard listeners and normal product orchestration.

The transition from onboarding to normal product runtime occurs without requiring an app restart.

Runtime startup must be idempotent so the same polling/sampler is not started twice.

## 9. Tracking status and pet semantics

The 5-second cadence remains unchanged.

Current timing model remains:

```text
capture sampler: about every 5 seconds
main data polling: about every 5 seconds
pet summary refresh: about every 5 seconds
```

Because the first foreground sample is baseline-only, a newly classified LEARN/REST interval can take several seconds before it becomes visible. This is acceptable; the UI should communicate the tracking state rather than simulate second-by-second updates.

The pet should distinguish at least:

- active tracking: e.g. `自动记录中`;
- tracking paused: e.g. `记录已暂停`;
- pre-onboarding: pet hidden, so no ambiguous state is needed.

The pet continues to show only `LEARN` and `REST` totals. It does not become a raw observed-app timer.

## 10. Explain observed activity vs. classified time

The product should add concise first-run copy explaining:

- Insights may immediately show observed applications because it reports captured application activity;
- `学 / 休` totals remain zero until those applications map to `LEARN` or `REST`;
- unknown applications can later be classified in Focus Guard.

This explanation belongs in onboarding and may be repeated as lightweight helper text where the classification workflow is introduced. It does not require changing the analytics computation.

Obvious system/self processes that should never require meaningful user classification can be evaluated separately as a small cleanup, but changing default rule coverage is not required to complete this first-run design.

## 11. Error handling

### Startup/database

- initialization failure prevents normal runtime startup;
- no schema-missing message should be rendered as a persistent red footer during a normal fresh launch.

### Frontend data errors

The global error surface should no longer preserve a stale transient startup error forever after later refreshes succeed.

Minimum behavior:

- successful refresh after a transient data-read error clears the relevant global error;
- onboarding/runtime errors remain visible when they block the requested transition.

This is secondary to fixing the database race at the source; frontend error clearing is not a substitute for startup ordering.

### Tracking state writes

- failed activation leaves onboarding incomplete and capture off;
- failed pause/resume keeps the prior persisted/runtime state and reports an error;
- no optimistic state that contradicts SQLite should survive a failed write.

## 12. Backend boundaries

Keep the current architecture:

```text
Tauri command wrappers
    -> service layer
    -> db / app_config
```

Add a focused tracking-preferences boundary rather than placing SQL in command wrappers.

Expected responsibility split:

- `db/migrations`: fresh-vs-existing defaults and schema/config migration;
- tracking service: read state, complete onboarding, persist pause/resume invariant;
- `lib.rs`: thin commands plus startup/window ordering;
- existing foreground service: capture mechanics remain unchanged;
- existing window service: reuse current reveal/show/settle semantics.

Do not perform another broad backend directory refactor.

## 13. Frontend boundaries

Prefer a focused composable for persisted tracking/onboarding state rather than adding more startup booleans directly to `App.vue`.

Conceptual responsibilities:

- load backend tracking state;
- expose onboarding-complete and auto-capture state;
- perform activation;
- perform pause/resume with persistence;
- trigger product-runtime start only after state permits it.

The existing auto-capture sampler remains responsible only for periodic sampling; it should not own persistence or onboarding policy.

## 14. Tests

Implementation must be test-driven where practical.

### Rust/database tests

Add tests covering:

1. fresh DB receives `onboarding_completed=false` and `auto_capture_enabled=false`;
2. recognizable legacy/existing DB with missing keys migrates to `true / true`;
3. rerunning initialization never overwrites persisted `auto_capture_enabled=false`;
4. completing onboarding atomically produces `true / true`;
5. pause persists `false` while keeping onboarding complete;
6. resume persists `true` after onboarding;
7. backend rejects enabling capture when onboarding is incomplete;
8. existing database lifecycle/migration tests continue to pass.

### Frontend tests / pure-logic characterization

Where existing test infrastructure permits, cover runtime-state decisions as pure logic rather than requiring full WebView automation:

- incomplete onboarding -> no product runtime/capture start;
- completed + enabled -> product runtime + capture start;
- completed + paused -> product runtime starts but sampler stays stopped;
- activation transitions to normal runtime once;
- pause/resume does not create duplicate sampling intervals.

If the repository still lacks a Vue unit-test runner, keep the state-transition logic small/pure and cover the Rust invariant plus Windows smoke path; do not add a large new test framework solely for this change.

### CI

Final gates remain:

```text
pnpm run typecheck
pnpm run build:check
cargo check --all-targets
cargo test
```

Windows installer packaging must still produce both NSIS and MSI artifacts.

## 15. Manual Windows acceptance test

Use a clean/sandboxed TimePrism data directory.

A fresh install passes only if:

1. no `no such table: app_usage_logs` error appears;
2. no foreground activity is captured before first-run activation;
3. onboarding clearly explains foreground-app metadata, local storage, browser privacy, no screenshots/keystrokes, and classified `学 / 休` semantics;
4. clicking `开始使用 TimePrism` starts tracking without restart;
5. Insights begins to show observed application activity after sampling;
6. pet totals remain classification-based and update after LEARN/REST activity with the existing several-second cadence;
7. pausing capture stops new samples;
8. closing/reopening TimePrism preserves the pause;
9. resuming persists across a later restart;
10. onboarding does not reappear after successful activation;
11. pet shows an appropriate paused state when tracking is paused;
12. existing close-to-background / full-quit behavior still works.

An upgrade test with an existing pre-change database passes only if tracking remains enabled and onboarding is treated as already complete.

## 16. Rollout / release impact

The current `0.1.0` installer candidate exposed this first-run issue, so it should not be published as the polished recruiting/demo release.

After implementation:

- rerun Windows CI;
- rerun NSIS + MSI packaging smoke;
- perform the fresh-install and upgrade manual tests above;
- capture final sanitized Home / Insights / Focus Guard / pet screenshots only from the corrected build;
- then move the PR out of Draft, merge, and create the intended public release.

## 17. Acceptance summary

This design is complete when the product has the following observable behavior:

```text
Fresh install
  -> DB fully initialized before product WebViews query it
  -> one-time tracking disclosure
  -> no capture before activation
  -> Start TimePrism
  -> tracking ON and persisted
  -> future launches auto-track
  -> user Pause persists across restart
```

And the existing 5-second capture/refresh model remains intact.
