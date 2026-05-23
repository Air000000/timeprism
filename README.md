<h1 align="center">TimePrism</h1>

<p align="center">
  A local-first desktop companion for understanding how your computer time actually flows.
</p>

<p align="center">
  <img alt="Tauri" src="https://img.shields.io/badge/Tauri-2-24C8DB?style=for-the-badge&logo=tauri&logoColor=white">
  <img alt="Vue" src="https://img.shields.io/badge/Vue-3-42B883?style=for-the-badge&logo=vuedotjs&logoColor=white">
  <img alt="TypeScript" src="https://img.shields.io/badge/TypeScript-5-3178C6?style=for-the-badge&logo=typescript&logoColor=white">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-backend-B7410E?style=for-the-badge&logo=rust&logoColor=white">
  <img alt="License" src="https://img.shields.io/github/license/Air000000/timeprism?style=for-the-badge">
</p>

<p align="center">
  <a href="#features">Features</a> |
  <a href="#how-it-works">How It Works</a> |
  <a href="#getting-started">Getting Started</a> |
  <a href="#privacy">Privacy</a> |
  <a href="#roadmap">Roadmap</a>
</p>

---

## Overview

TimePrism is a Tauri desktop app for reviewing computer usage, work-rest rhythm, reminders, and focus signals.

Instead of asking you to start and stop a manual timer all day, TimePrism focuses on automatic foreground app capture, local classification rules, and lightweight daily review. It helps you answer a few practical questions:

- What did my computer usage actually look like today?
- Which apps shaped my learn/rest rhythm?
- What needs attention before future tracking becomes more accurate?
- Which reminders or idle segments still need a quick decision?

TimePrism is currently Windows-first and designed as a local-first personal desktop tool.

## Features

| Area | What it helps with |
| --- | --- |
| **Home** | See today's learn/rest totals, goal progress, reminders, and pending signals at a glance. |
| **Insights** | Review top apps, recent logs, historical usage, heatmaps, and day composition. |
| **Focus Guard** | Classify unknown apps, resolve idle segments, review rules, and inspect sampling diagnostics. |
| **Reminders** | Create one-time, daily, and weekly reminders, then mark them done or snooze them locally. |
| **Privacy Controls** | Configure browser title handling, whitelist-only capture, and local capture behavior. |
| **Desktop Pet** | Use a lightweight floating companion for quick prompts, reminders, and mini panels. |

## How It Works

```text
Foreground window
      |
      v
Privacy processing
      |
      v
App rule classification
      |
      v
Local SQLite storage
      |
      v
Home, Insights, Guard, reminders, and pet surfaces
```

TimePrism samples the active foreground window, applies privacy settings before storage, maps apps through local rules, and then uses the resulting records for daily summaries and review workflows.

## Tech Stack

| Layer | Technology |
| --- | --- |
| Desktop shell | Tauri v2 |
| Frontend | Vue 3, TypeScript, Vite |
| Backend | Rust |
| Storage | SQLite |
| Target platform | Windows-first desktop |

## Getting Started

### Prerequisites

Before running the project locally, install:

- Node.js 18+
- pnpm
- Rust toolchain
- Tauri system prerequisites for your platform

For Windows development, follow the standard Tauri setup requirements.

### Install

```bash
git clone https://github.com/Air000000/timeprism.git
cd timeprism
pnpm install
```

### Run In Development

```bash
pnpm tauri dev
```

### Run Frontend Only

```bash
pnpm dev
```

### Build

Build the frontend:

```bash
pnpm build
```

Build the desktop app:

```bash
pnpm tauri build
```

## Project Structure

```text
timeprism/
|- public/                 # Static assets
|- src/
|  |- components/          # Main desktop views
|  |- api.ts               # Tauri command bindings
|  |- App.vue              # Main desktop shell
|  |- main.ts              # Main window entry
|  |- pet.ts               # Pet window entry
|  `- pet-panel.ts         # Pet panel entry
|- src-tauri/
|  |- src/
|  |  |- lib.rs            # Rust commands and app logic
|  |  `- main.rs           # Tauri bootstrap
|  |- capabilities/        # Tauri permissions
|  `- tauri.conf.json      # Tauri app configuration
|- index.html              # Main webview entry
|- pet.html                # Pet window page
|- pet-panel.html          # Pet panel page
`- README.md
```

## Privacy

TimePrism is designed as a local-first app.

- Usage records are stored locally in SQLite.
- There is no cloud sync in the current version.
- Browser title handling can be configured as `FULL`, `BLUR`, or `NONE`.
- Whitelist-only mode can restrict what gets stored.
- Privacy processing happens before usage records are written.
- Incognito/private browser windows are treated conservatively.

Because TimePrism records active app usage and window titles when allowed, privacy behavior is treated as a core product concern rather than an afterthought.

## Current Status

TimePrism is an early desktop app and still evolving.

Implemented:

- Automatic foreground app capture
- Learn/rest/ignore app rules
- Daily overview
- Usage insights
- Focus Guard workflow
- Idle confirmation flow
- Local reminders
- Desktop pet and pet panel
- Privacy settings

Known limitations:

- Foreground capture is currently Windows-first.
- The UI is still being refined.
- There is no cloud sync or external account system.
- Data export, backup, and restore flows are not fully built yet.
- Cross-platform capture support is limited.

## Roadmap

Planned or under consideration:

- Add polished screenshots and demo media
- Improve replay/day timeline workflows
- Add data export and backup tools
- Strengthen database diagnostics and recovery flows
- Improve cross-platform capture behavior
- Continue refining desktop layout and interaction polish

## Contributing

Issues, ideas, and pull requests are welcome.

For larger changes, please open an issue first so the direction can be discussed clearly. TimePrism is especially sensitive around privacy, local data handling, and capture behavior, so changes in those areas should be reviewed carefully.

## License

TimePrism is licensed under the MIT License.
