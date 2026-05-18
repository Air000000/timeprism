# TimePrism

TimePrism is a desktop time-tracking app with dual-track logging:

- Track A: manual task sessions (learn/rest category tree)
- Track B: objective app usage fragments for later analytics

This repository currently contains the first implementation milestone.

## Tech Stack

- Tauri v2
- Vue 3 + TypeScript + Vite
- Rust backend
- SQLite (WAL mode)

## Current Milestone (Implemented)

- Project bootstrapped with Tauri + Vue3 + TS
- SQLite schema created on app startup:
	- `categories`
	- `task_sessions`
	- `app_usage_logs`
	- `app_rules`
- Built-in immutable root categories:
	- `id=1` Learn
	- `id=2` Rest
- Backend commands:
	- `list_categories`
	- `create_category`
	- `start_session`
	- `stop_active_session`
	- `append_app_usage_log`
	- `get_today_summary`
	- `list_top_apps_today`
- Frontend MVP panel:
	- category selection
	- start/stop timer controls
	- create child category under Learn/Rest
	- today's learn/rest totals
	- top apps list

## Run Locally

1. Install dependencies:

```powershell
pnpm install
```

2. Start desktop app:

```powershell
pnpm tauri dev
```

3. Build frontend only (sanity check):

```powershell
pnpm build
```

## Notes

- On this machine, development currently uses a local Node runtime and user-scoped Rust toolchain.
- If folder rename from `TimeNest` to `TimePrism` is blocked, close handles to the folder and rename after shutdown.
