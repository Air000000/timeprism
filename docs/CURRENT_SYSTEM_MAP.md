# Current System Map

Last reviewed: 2026-06-08.

## Repository Shape

The active application lives in this `timeprism` directory.

Important sibling folders in the larger workspace:

- `../Time_Prism`: product and rewrite reference docs.
- `../prismy`: pet character assets and production notes.
- `../未上传文件`: local planning notes and older working documents.

## Tech Stack

- Desktop shell: Tauri 2.
- Frontend: Vue 3, TypeScript, Vite.
- Backend: Rust.
- Storage: SQLite through `rusqlite`.
- Platform priority: Windows.
- Pet assets: static PNGs in `public`.

## Runtime Entries

| Surface | HTML | Script | Tauri window |
| --- | --- | --- | --- |
| Main app | `index.html` | `src/main.ts` | `main` |
| Pet | `pet.html` | `src/pet.ts` | `pet` |
| Pet panel | `pet-panel.html` | `src/pet-panel.ts` | `pet-panel` |

## Key Files

- `src/App.vue`: main app state, view routing, data orchestration, template, and large CSS block.
- `src/api.ts`: TypeScript wrapper around Tauri commands.
- `src/components/HomeView.vue`: home surface.
- `src/components/InsightsView.vue`: history and usage views.
- `src/components/GuardView.vue`: pending apps, idle queue, rules, diagnostics.
- `src/pet.ts`: pet UI, prompt bubble, quick actions, context menu.
- `src/pet-panel.ts`: mini heatmap and weekly activity panel.
- `src-tauri/src/lib.rs`: Tauri commands, analytics, capture, privacy, reminders, and window management.
- `src-tauri/src/db/connection.rs`: database path resolution, legacy DB migration, SQLite connection opening.
- `src-tauri/src/db/migrations.rs`: SQLite schema creation, table upgrade helpers, default seed data, and heatmap snapshot table creation.
- `src-tauri/src/domain/*.rs`: backend DTOs and command input/output structs grouped by feature area.
- `src-tauri/tauri.conf.json`: app and window configuration.

## Main Data Flow

```text
Windows foreground window
-> process/title normalization
-> shell/noise filtering
-> privacy settings and whitelist check
-> app rule mapping
-> SQLite usage log or bounded diagnostic
-> Home / Guard / Insights / Pet APIs
```

## Current Windows

Main window:

- Label: `main`.
- Size: `888 x 900`.
- Hidden instead of closed.

Pet window:

- Label: `pet`.
- Frameless, transparent, always on top, hidden from taskbar.
- Supports summon, drag, hide, close, edge settling, and context menu.

Pet panel:

- Label: `pet-panel`.
- Frameless, transparent, always on top, hidden by default.
- Follows pet position and resizes for heatmap or stack mode.

## Current Data Tables

- `categories`: root and child categories.
- `task_sessions`: legacy/manual session-compatible table.
- `app_usage_logs`: foreground activity segments after privacy processing.
- `app_rules`: process mapping and privacy level.
- `app_config`: key-value settings.
- `app_whitelist`: process whitelist.
- `reminders`: one-time, daily, weekly reminders.
- `daily_heatmap_snapshot`: sealed historical heatmap cells.

## Known Architecture Debt

- `src/App.vue` is too broad and owns too many feature concerns.
- `src-tauri/src/lib.rs` is too broad and mixes unrelated layers.
- Feature components consume `ctx: any`.
- Backend SQL, migrations, services, and commands are not separated.
- Windows APIs live in the same file as Tauri setup and SQL.
- Idle prompts are currently in process memory, so restart behavior needs a product decision.
- Heavy startup data requests need review.
- `task_sessions` exists despite the v1 decision that automatic sampling should be the core time source.

## Current Validation Notes

Known working checks:

- `pnpm.cmd run typecheck`
- `pnpm.cmd run build:check`
- `cargo check` from `src-tauri`

`pnpm.cmd build` may fail if old `dist` files are locked by a running process. Use a temporary output directory for source validation or stop the locking process before a normal build.
