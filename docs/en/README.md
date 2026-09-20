<h1 align="center">TimePrism</h1>

<p align="center">
  <strong>See where your computer time actually goes — without living inside a timer.</strong>
</p>

<p align="center">
  Windows-first, local-first focus and activity tracking with privacy-aware capture, idle correction, reminders, and the Yin Yue desktop pet.
</p>

<p align="center">
  <img alt="Release" src="https://img.shields.io/github/v/release/Air000000/timeprism?style=for-the-badge&sort=semver">
  <img alt="CI" src="https://github.com/Air000000/timeprism/actions/workflows/ci.yml/badge.svg?branch=main">
  <img alt="Tauri" src="https://img.shields.io/badge/Tauri-2-24C8DB?style=for-the-badge&logo=tauri&logoColor=white">
  <img alt="Vue" src="https://img.shields.io/badge/Vue-3-42B883?style=for-the-badge&logo=vuedotjs&logoColor=white">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-backend-B7410E?style=for-the-badge&logo=rust&logoColor=white">
  <img alt="License" src="https://img.shields.io/github/license/Air000000/timeprism?style=for-the-badge">
</p>

<p align="center">
  <a href="../../README.md">中文</a> · <strong>English</strong> ← current
</p>

<p align="center">
  <a href="https://github.com/Air000000/timeprism/releases/tag/v0.1.0"><strong>Download v0.1.0</strong></a> ·
  <a href="#product">Product</a> ·
  <a href="#engineering-highlights">Engineering</a> ·
  <a href="#architecture">Architecture</a> ·
  <a href="#privacy">Privacy</a> ·
  <a href="#development">Development</a>
</p>

---

## Why TimePrism

Traditional timers only work when you remember to start and stop them.

TimePrism takes a different approach: it observes the foreground application, applies privacy rules before storage, classifies activity through local rules, and turns the result into a view of learning, rest, idle gaps, reminders, and app usage.

The core workflow runs locally on Windows. No account or cloud service is required.

| Automatic | Local-first | Correctable |
| --- | --- | --- |
| Foreground activity is sampled in the background instead of relying on manual timers. | Core data is stored in local SQLite and privacy rules are applied before normal persistence. | Idle periods are explicitly confirmed so time can be attributed to the previous app, marked Learn/Break/Away, or deferred. |

> **Current release:** <strong>v0.1.0</strong> is available as both an NSIS setup executable and MSI installer. TimePrism is currently unsigned, so Windows SmartScreen may show an unknown-publisher warning.

<!--
SHOWCASE MEDIA SLOT
After sanitized v0.1.0 captures exist, insert ../media/timeprism-demo.gif here,
then add the two-row screenshot grid described in ../media/README.md.
Do not reference missing media files before they are committed.
-->

<a id="product"></a>
## Product

| Area | What it does |
| --- | --- |
| **Home** | Shows today's learning/rest rhythm, editable focus goal, reminders, pending signals, and recent activity. |
| **Insights** | Reviews top apps, recent logs, historical usage, heatmaps, and day composition. |
| **Focus Guard** | Classifies unknown apps, resolves idle segments, edits rules, and exposes capture diagnostics. |
| **Reminders** | Supports one-time, daily, and weekly reminders with completion, restore, and snooze flows. |
| **Privacy Controls** | Configures browser-title handling, whitelist-only capture, and local persistence behavior. |
| **Yin Yue Desktop Pet** | Surfaces lightweight reminders, idle decisions, quick actions, learning/rest totals, and compact analytics. |

### Yin Yue desktop pet

The desktop pet is a separate always-on-top Tauri window rather than a decorative element inside the main UI.

It supports drag animation, left/right edge docking, compact prompt flows, a native context menu, and a small companion panel. Its window sizing adapts to mixed-resolution / mixed-DPI monitor setups so moving the pet between displays does not force a fixed physical-pixel size.

## How it works

~~~text
Foreground window
      |
      v
Privacy processing
      |
      v
Local app-rule classification
      |
      v
SQLite persistence
      |
      +------> Home / Insights
      +------> Focus Guard
      +------> Reminders
      `------> Yin Yue pet / panels
~~~

Foreground activity and classification are deliberately separate. A confirmed idle interval can be attributed back to an application without rewriting that application's Learn / Break / Unclassified rule.

<a id="engineering-highlights"></a>
## Engineering Highlights

### 1. Privacy at ingestion

Foreground data can contain sensitive window titles. TimePrism applies privacy decisions before normal persistence rather than storing raw data first and masking it later.

- Browser title handling supports <code>FULL</code>, <code>BLUR</code>, and <code>NONE</code>.
- Whitelist-only mode can prevent non-whitelisted application usage from being stored.
- Private/incognito browser windows are handled explicitly.
- Diagnostic paths do not retain raw titles after the privacy boundary.

### 2. Idle correction without double counting

Samples recorded immediately before an idle threshold are provisional. When the user resolves an idle prompt, TimePrism transactionally replaces the overlapping usage interval instead of appending a second row.

Confirmed corrections carry explicit provenance such as <code>FOREGROUND</code> and <code>IDLE_CONFIRMED</code>, while attribution and classification remain separate concepts.

### 3. Persisted tracking state and startup ordering

First-run tracking consent, Pause / Resume state, and legacy-database migration are persisted in SQLite.

Product WebViews are created only after database initialization succeeds, avoiding a startup race where the frontend could query tables before schema initialization finished.

### 4. Desktop runtime across displays

The pet uses logical sizing plus monitor-aware scaling rather than a fixed physical-pixel window. Regression tests cover high-DPI sizing and the case where a lower-resolution display has similar logical desktop space.

### 5. Release as an engineering gate

The Windows release workflow validates synchronized Node/Cargo/Tauri versions, frontend type safety, Rust tests, and real installer generation.

Release-related pull requests build NSIS and MSI packages as validation artifacts. Version tags create a draft GitHub Release, and publication remains gated on an interactive Windows installer smoke test.

<a id="architecture"></a>
## Architecture

~~~text
                     TimePrism desktop surfaces

       +----------------+  +---------------+  +----------------+
       |  Main WebView  |  |  Yin Yue Pet  |  |   Pet Panel    |
       +--------+-------+  +-------+-------+  +--------+-------+
                |                  |                   |
                +------------------+-------------------+
                                   |
                                   v
                  Vue views / composables / src/lib
                                   |
                                   v
                        typed Tauri command layer
                                   |
                                   v
             +---------------- Rust backend ----------------+
             |                                               |
             |   services   ->   domain   ->   db/SQLite     |
             |      |                                        |
             |      +------ Windows / Tauri APIs             |
             +-----------------------------------------------+
~~~

The frontend keeps interaction state in focused Vue components and composables, while <code>src/api/commands/*</code> provides typed access to Tauri commands.

The Rust side keeps command registration thin and pushes behavior into <code>services/*</code>, with domain structures in <code>domain/*</code> and database lifecycle/migrations in <code>db/*</code>.

## Validation

The main CI workflow runs on Windows for pull requests and pushes to <code>main</code>:

~~~text
pnpm run typecheck
pnpm run build:check
cargo check --all-targets
cargo test
~~~

The test suite covers database initialization and migration, persisted tracking state, privacy rules, idle attribution/provenance, reminder recurrence, analytics boundaries, pet window sizing, and Home goal wiring.

For v0.1.0, the final Windows installer was also installed and smoke-tested before the draft Release was published.

CI and smoke tests are regression evidence for this project; they are not a claim of production-grade hardening.

<a id="privacy"></a>
## Privacy

TimePrism is intentionally local-first:

- core usage records are stored locally in SQLite;
- there is no cloud sync or account requirement in v0.1.0;
- privacy processing happens before normal usage persistence;
- browser title handling is configurable;
- whitelist-only capture can restrict what is stored;
- TimePrism does not capture screenshots or keystroke contents.

Because active-window data can be sensitive, privacy is treated as part of the ingestion/data model rather than only as a presentation setting.

See [Database & Privacy](../DATABASE_AND_PRIVACY.md) for implementation details.

## Download

The current Windows release is [TimePrism v0.1.0](https://github.com/Air000000/timeprism/releases/tag/v0.1.0).

Release assets include:

- <code>TimePrism_0.1.0_x64-setup.exe</code> — NSIS installer
- <code>TimePrism_0.1.0_x64_en-US.msi</code> — MSI installer

The project is currently unsigned, so Windows SmartScreen may warn about an unknown publisher.

<a id="development"></a>
## Development

### Prerequisites

- Node.js 24
- pnpm 9.15.9
- Rust stable toolchain
- Tauri v2 system prerequisites
- Windows for the full foreground-capture/runtime path

### Run locally

~~~bash
git clone https://github.com/Air000000/timeprism.git
cd timeprism
pnpm install
pnpm tauri dev
~~~

### Validate

~~~bash
pnpm run typecheck
pnpm run build:check
cd src-tauri
cargo check --all-targets
cargo test
~~~

### Build installers

~~~bash
pnpm tauri build
~~~

## Tech Stack

| Layer | Technology |
| --- | --- |
| Desktop shell | Tauri 2 |
| Frontend | Vue 3, TypeScript, Vite |
| Backend | Rust |
| Storage | SQLite / rusqlite |
| Platform integration | Windows + Tauri window APIs |
| Validation | GitHub Actions, vue-tsc, Vite, Cargo |

## Repository Map

~~~text
timeprism/
|- src/
|  |- components/          # Main views and desktop UI surfaces
|  |- composables/         # Feature state and view workflows
|  |- api/commands/        # Typed Tauri command wrappers
|  |- lib/                 # Reusable UI/domain helpers
|  |- App.vue              # Main desktop shell
|  |- pet.ts               # Yin Yue desktop pet entry
|  `- pet-panel.ts         # Pet companion panel
|- src-tauri/
|  `- src/
|     |- db/               # SQLite connection, migrations, lifecycle
|     |- domain/           # Backend data models and command payloads
|     |- services/         # Tracking, privacy, idle, analytics, windows, etc.
|     `- lib.rs            # Tauri bootstrap / command registration
|- docs/                   # Architecture, privacy, tests, decisions, release evidence
`- .github/workflows/      # CI and Windows release pipelines
~~~

## Documentation

Start with [docs/README.md](../README.md). Key references include:

- [Current System Map](../CURRENT_SYSTEM_MAP.md)
- [API Contracts](../API_CONTRACTS.md)
- [Database & Privacy](../DATABASE_AND_PRIVACY.md)
- [Testing Strategy](../TESTING_STRATEGY.md)
- [Smoke Tests](../SMOKE_TESTS.md)
- [Release Checklist](../RELEASE_CHECKLIST.md)
- [Product Media Capture Guide](../media/README.md)
- [Architecture Decisions](../DECISIONS.md)

## Current Scope

v0.1.0 is intentionally Windows-first. Cloud sync/accounts, complete cross-platform foreground capture, and export/backup/restore workflows are outside the finished scope of this release.

## License

TimePrism is licensed under the [MIT License](../../LICENSE).
