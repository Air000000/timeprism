<h1 align="center">TimePrism</h1>

<p align="center">
  Windows-first, local-first desktop time/rhythm tracker built with Vue 3 + Tauri 2 + Rust + SQLite.
</p>

<p align="center">
  <img alt="CI" src="https://github.com/Air000000/timeprism/actions/workflows/ci.yml/badge.svg?branch=main">
  <img alt="Tauri" src="https://img.shields.io/badge/Tauri-2-24C8DB?style=for-the-badge&logo=tauri&logoColor=white">
  <img alt="Vue" src="https://img.shields.io/badge/Vue-3-42B883?style=for-the-badge&logo=vuedotjs&logoColor=white">
  <img alt="TypeScript" src="https://img.shields.io/badge/TypeScript-5-3178C6?style=for-the-badge&logo=typescript&logoColor=white">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-backend-B7410E?style=for-the-badge&logo=rust&logoColor=white">
  <img alt="License" src="https://img.shields.io/github/license/Air000000/timeprism?style=for-the-badge">
</p>

<p align="center">
  <a href="https://github.com/Air000000/timeprism/releases"><strong>Windows Releases</strong></a> |
  <a href="#what-it-demonstrates">What It Demonstrates</a> |
  <a href="#architecture">Architecture</a> |
  <a href="#engineering-highlights">Engineering Highlights</a> |
  <a href="#features">Features</a> |
  <a href="#getting-started">Getting Started</a>
</p>

---

## Overview

TimePrism is a personal desktop tracker for understanding how computer time actually flows across learning, rest, reminders, idle periods, and app usage.

Rather than relying on manual start/stop timers, it samples the foreground application, applies privacy rules before persistence, maps activity through local classification rules, and stores the resulting facts in SQLite for review across the main window, Focus Guard, reminders, and a lightweight desktop pet.

The project is intentionally scoped as a Windows-first, local-first desktop application. It does not depend on cloud accounts, remote sync, or external services for its core workflow.

> **Release status:** the currently published `v0.1.0-preview` is an early portable build and predates the latest architecture/reliability work. The repository now includes a gated Windows release pipeline for NSIS `-setup.exe` and MSI installers; packaging changes are validated on Windows before merge, while public releases remain draft-first and require an interactive installer smoke test before publication.

## What It Demonstrates

TimePrism is primarily a desktop full-stack engineering project. The repository demonstrates:

- **Tauri desktop application architecture** across a main window, desktop pet, and pet side panel.
- **Vue 3 + TypeScript frontend modularization** using views, composables, typed command wrappers, and reusable domain helpers.
- **Rust backend structure** split into command boundaries, services, domain models, database access, and platform-specific window/foreground behavior.
- **Local data correctness** around foreground sampling, idle reclassification, reminders, and SQLite persistence.
- **Privacy-at-ingestion** rather than post-hoc masking of already stored sensitive data.
- **Runtime reliability** including startup sequencing, UI-thread protection, full-app exit behavior, CI, and Windows smoke validation.

## Architecture

```text
                     TimePrism desktop surfaces

       +----------------+  +---------------+  +----------------+
       |  Main WebView  |  |  Desktop Pet  |  |   Pet Panel    |
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
```

The frontend keeps business-facing state and interaction logic in focused Vue components and composables, while `src/api/commands/*` provides typed access to Tauri commands. The Rust side keeps command functions thin and pushes behavior into `services/*`, with domain data structures in `domain/*` and database concerns in `db/*`.

Foreground capture, privacy processing, reminder scheduling, idle handling, analytics, startup behavior, and window management are separated into dedicated service modules rather than accumulated inside `src-tauri/src/lib.rs`.

## Engineering Highlights

### 1. Privacy-at-ingestion

Foreground usage may contain sensitive window titles, so privacy decisions are applied before persistence.

- Browser title handling supports `FULL`, `BLUR`, and `NONE` modes.
- Whitelist-only capture blocks unlisted processes from normal usage persistence instead of storing a generic replacement row.
- Diagnostic sampling state redacts raw window titles before they enter the diagnostic path, so observability does not bypass the storage policy.

### 2. Idle reclassification without double counting

Foreground samples recorded before the idle threshold are provisional. Once an idle prompt is resolved, TimePrism transactionally rewrites the overlapping usage interval instead of simply appending an idle row.

This avoids counting the same interval once as foreground activity and again as idle/rest/away time.

### 3. Reminder wall-clock semantics

Analytics uses a 04:00 business-day boundary, but recurring reminders are user-facing wall-clock events. Their time bases are deliberately separated.

Daily and weekly reminders are scheduled from local calendar midnight and local weekday semantics, preventing a reminder such as `09:17` from being shifted by the analytics business-day anchor.

### 4. Startup and desktop runtime reliability

The main WebView remains hidden until Vue has mounted, avoiding a visible blank window during startup. Non-critical frontend work is deferred, and heavier read commands are moved away from the Tauri UI thread.

Desktop lifecycle semantics are also explicit:

- Main-window close hides the application to the background.
- Pet hide only hides the pet window.
- `Quit TimePrism` exits the full Tauri application.

## Features

| Area | What it does |
| --- | --- |
| **Home** | Shows today's learn/rest totals, progress, reminders, and pending signals. |
| **Insights** | Reviews top apps, recent logs, historical usage, heatmaps, and day composition. |
| **Focus Guard** | Classifies unknown apps, resolves idle segments, reviews rules, and exposes sampling diagnostics. |
| **Reminders** | Supports one-time, daily, and weekly reminders with done/restore and snooze flows. |
| **Privacy Controls** | Configures title handling, whitelist-only capture, and local persistence behavior. |
| **Desktop Pet** | Provides lightweight prompts, reminders, quick actions, and compact analytics panels. |

## Data Flow

```text
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
      `------> Desktop pet / panels
```

## Tech Stack

| Layer | Technology |
| --- | --- |
| Desktop shell | Tauri v2 |
| Frontend | Vue 3, TypeScript, Vite |
| Backend | Rust |
| Storage | SQLite / rusqlite |
| Platform integration | Windows + Tauri window APIs |
| Validation | GitHub Actions, `vue-tsc`, Vite build, `cargo check --all-targets`, `cargo test` |

## Project Structure

```text
timeprism/
|- src/
|  |- components/          # Main desktop views and UI surfaces
|  |- composables/         # Feature state, orchestration, and view workflows
|  |- api/
|  |  `- commands/         # Typed Tauri command wrappers
|  |- lib/                 # Reusable domain/UI helpers
|  |- App.vue              # Main desktop shell
|  |- main.ts              # Main window entry
|  |- pet.ts               # Desktop pet entry
|  `- pet-panel.ts         # Pet panel entry
|- src-tauri/
|  |- src/
|  |  |- db/               # SQLite connection and migrations
|  |  |- domain/           # Backend data models and command payload types
|  |  |- services/         # Analytics, privacy, idle, reminders, windows, etc.
|  |  |- lib.rs            # Thin Tauri command registration / app bootstrap
|  |  `- main.rs           # Native executable entry
|  |- capabilities/        # Tauri permissions
|  `- tauri.conf.json      # Desktop window/application configuration
|- docs/                   # Architecture, contracts, privacy, tests, ADRs
|- index.html              # Main WebView page
|- pet.html                # Pet WebView page
|- pet-panel.html          # Pet panel WebView page
`- README.md
```

## Validation

Every push to `main` and `refactor/**`, plus pull requests, runs the repository CI workflow on Windows. Rust build output is cached between runs and all test targets are compiled before the test phase:

```text
pnpm run typecheck
pnpm run build:check
cargo check --all-targets
cargo test
```

Release-related pull requests also run an independent Windows packaging gate that builds both NSIS and MSI installers and uploads them as workflow artifacts. This verifies that the current tree can reach real Tauri installer artifacts without publishing a Release.

The current desktop flow has also been manually smoke-tested on Windows for startup behavior, main views, reminder workflows, pet interactions, and full-app quit behavior.

CI, packaging validation, and smoke tests are used as project-level regression evidence; they are not a claim of production-grade hardening.

## Distribution

`.github/workflows/release.yml` separates installer generation from publication on `windows-latest`:

- **Packaging PR:** release-related pull requests validate synchronized versions, run frontend/Rust checks, build both NSIS and MSI installers, and upload the real installer artifacts without publishing anything.
- **Manual dry run:** `workflow_dispatch` performs the same installer build on demand after merge and uploads the installers as workflow artifacts without creating a Release.
- **Version tag:** requires the tag to exactly match the synchronized Cargo/Node/Tauri app version, then creates a **draft** GitHub Release with both installer formats.

Automated packaging proves that the installers can be generated; it does not replace the interactive Windows gate. At least one generated installer must still be downloaded, installed, launched, and smoke-tested before the draft Release is made public.

The project is currently unsigned, so Windows SmartScreen may warn about an unknown publisher. See [`docs/RELEASE_CHECKLIST.md`](docs/RELEASE_CHECKLIST.md) for the full publication gate.

## Privacy

TimePrism is designed around local storage and explicit privacy boundaries.

- Usage records are stored locally in SQLite.
- There is no cloud sync in the current version.
- Browser title handling can be configured as `FULL`, `BLUR`, or `NONE`.
- Whitelist-only mode can prevent non-whitelisted application usage from being stored.
- Privacy processing happens before normal usage persistence.
- Diagnostic paths do not retain raw titles after the privacy boundary.

Because active-window data can be sensitive, privacy is treated as part of the data model and ingestion pipeline rather than only as a UI setting.

## Getting Started

### Prerequisites

Recommended local development environment:

- Node.js 24 LTS
- pnpm 9
- Rust toolchain
- Tauri v2 system prerequisites
- Windows for the full foreground-capture/runtime path

### Install

```bash
git clone https://github.com/Air000000/timeprism.git
cd timeprism
pnpm install
```

### Run in development

```bash
pnpm tauri dev
```

### Frontend only

```bash
pnpm dev
```

### Validation

```bash
pnpm run typecheck
pnpm run build:check
cd src-tauri
cargo check --all-targets
cargo test
```

### Build desktop application

```bash
pnpm tauri build
```

## Current Scope

Implemented core scope:

- Automatic foreground sampling
- Learn/rest/ignore app rules
- Daily overview and usage insights
- Focus Guard and idle confirmation workflow
- Local one-time/daily/weekly reminders
- Desktop pet and lightweight analytics panel
- Local privacy controls
- Windows startup/window lifecycle handling
- CI-backed frontend and Rust validation

Current limitations are explicit rather than hidden behind roadmap claims: foreground capture is Windows-first, cloud/account features are intentionally absent, cross-platform capture is incomplete, and export/backup/restore flows are not yet part of the finished MVP.

## Documentation

Engineering notes and design rationale live under `docs/`, including:

- current system map
- API contracts
- database and privacy notes
- smoke-test catalog
- testing strategy
- release checklist
- product-media capture guide
- ADRs and refactor traceability

Real product screenshots/GIF are only added from sanitized current Windows builds; the capture/privacy requirements are documented in [`docs/media/README.md`](docs/media/README.md) so generated or mock UI is not presented as implementation evidence.

## License

TimePrism is licensed under the MIT License.
