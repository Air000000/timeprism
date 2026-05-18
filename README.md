# TimePrism

TimePrism is a Tauri desktop app for people who want to review work-rest rhythms, manage reminders, and build a cleaner understanding of how time is actually spent on the computer.

## Overview

TimePrism centers on automatic foreground app capture and lightweight daily planning in one desktop experience.

The app is designed to help users answer three practical questions:

- What did my computer usage actually look like today?
- Where are my work-rest rhythms drifting over time?
- What do I need to handle next to keep the day on track?

## Features

### Automatic Usage Tracking
- Capture foreground app usage fragments for later analytics
- Review today's learn and rest totals at a glance
- Track progress toward a configurable daily focus goal

### Daily Rhythm Insights
- View recent seven-day rhythm patterns
- Explore app usage history, top apps, and recent logs
- Inspect learn heatmaps and stacked usage summaries

### Focus Guard Workflow
- Review uncategorized apps and assign rules
- Resolve idle segments after away periods
- Audit saved rules and inspect sampling diagnostics

### Reminders and Planning
- Create one-time, daily, or weekly reminders
- Reorder and manage reminders directly from the desktop UI
- Mark reminders done or snooze them when plans shift

### Privacy Controls
- Curtain mode to stop app logging entirely
- Browser title privacy modes: `FULL`, `BLUR`, or `NONE`
- Whitelist-only capture mode for stricter logging control

### Desktop Companion
- Includes a floating pet window and companion panel
- Supports quick summon and lightweight desktop interactions

## Tech Stack

| Layer | Technology |
| --- | --- |
| Desktop Shell | Tauri v2 |
| Frontend | Vue 3, TypeScript, Vite |
| Backend | Rust |
| Local Storage | SQLite |
| UI Mode | Desktop-first, Windows-focused |

## Getting Started

### Prerequisites

- Node.js 18+
- pnpm
- Rust toolchain
- Tauri build requirements for your platform

For Windows development, make sure the standard Tauri prerequisites are installed.

### Installation

```bash
git clone https://github.com/your-username/timeprism.git
cd timeprism
pnpm install
```

### Environment Variables

This project does not currently require a `.env` file for local development.

### Run Locally

Start the desktop app in development mode:

```bash
pnpm tauri dev
```

Start the frontend only:

```bash
pnpm dev
```

Build the frontend:

```bash
pnpm build
```

Build the desktop app:

```bash
pnpm tauri build
```

## Project Structure

```txt
timeprism/
├─ public/                 # Static assets
├─ src/
│  ├─ components/          # Main desktop views
│  ├─ assets/              # Frontend assets
│  ├─ api.ts               # Tauri command bindings
│  ├─ App.vue              # Main desktop shell
│  ├─ main.ts              # Frontend entry
│  ├─ pet.ts               # Pet window entry
│  └─ pet-panel.ts         # Pet panel entry
├─ src-tauri/
│  ├─ src/
│  │  ├─ lib.rs            # Main Rust app logic and commands
│  │  └─ main.rs           # Tauri bootstrap
│  └─ tauri.conf.json      # Tauri configuration
├─ index.html              # Main webview entry
├─ pet.html                # Pet window page
├─ pet-panel.html          # Pet panel page
└─ README.md
```

## Available Scripts

| Command | Description |
| --- | --- |
| `pnpm dev` | Start the Vite frontend dev server |
| `pnpm build` | Type-check and build the frontend |
| `pnpm tauri dev` | Run the desktop app in development mode |
| `pnpm tauri build` | Build the desktop application |

## Current Views

- `Home`: daily overview, goal ring, reminders, calendar, and recent rhythm
- `Insights`: top apps, all-time usage, and recent logs
- `Focus Guard`: rule review, idle confirmation, and diagnostics
- `Settings`: privacy, startup behavior, and whitelist management

## Known Limitations

- Foreground capture is currently Windows-first; non-Windows behavior is more limited.
- The UI is still evolving and some layout polish is ongoing.
- README screenshots and demo assets have not been added yet.
- No external sync or cloud backup is included in the current version.
- Backend session commands exist, but the current desktop UI is centered on automatic tracking rather than a dedicated manual timer workflow.

## Roadmap

- [x] Automatic foreground capture
- [x] Reminder system
- [x] Focus guard workflow
- [x] Privacy controls
- [x] Desktop pet companion
- [ ] README screenshots and demo media
- [ ] Stronger cross-platform capture support
- [ ] Additional polish for responsive desktop layouts

## Contributing

Contributions, issue reports, and design feedback are welcome. If you plan to make a larger change, please open an issue first so the direction can be discussed before implementation.

## License

No license has been added yet.
