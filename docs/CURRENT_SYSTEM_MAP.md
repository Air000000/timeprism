# Current System Map

Last reviewed: 2026-06-09.

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

- `src/App.vue`: main app shell, global lifecycle/refresh routing, view context assembly, and top-level view mounting.
- `src/api.ts`: compatibility API surface that keeps existing frontend imports stable.
- `src/api/client.ts`: thin typed wrapper around Tauri `invoke`.
- `src/api/types.ts`: shared frontend API response/input-adjacent types.
- `src/api/commands/`: frontend Tauri command wrappers grouped by feature domain.
- `src/components/AppTopNav.vue`: main-window brand block and primary navigation tabs.
- `src/components/viewContexts.ts`: typed component context contracts shared by `App.vue` and feature views.
- `src/components/HomeView.vue`: home surface.
- `src/components/IdlePromptBanner.vue`: top-level idle prompt confirmation banner.
- `src/components/InsightsView.vue`: history subviews for today's top apps, all-time app usage, and recent logs.
- `src/components/GuardView.vue`: pending apps, idle queue, rules, diagnostics.
- `src/components/SettingsView.vue`: interface, startup, privacy sampling policy, and whitelist settings UI.
- `src/composables/useAutoCaptureSampler.ts`: auto-capture foreground sampling timer, capture feedback text, and timer cleanup.
- `src/composables/useAppNavigation.ts`: main-window navigation state, tab labels, and insights history subview state.
- `src/composables/useAppSettingsSection.ts`: App-level Settings privacy state, lazy mount coordination, and Settings view context assembly.
- `src/composables/useDisplayFormatters.ts`: locale/translation adapters for shared UI display formatter helpers.
- `src/composables/useErrorMessage.ts`: top-level main-window error message state and setter.
- `src/composables/useGuardData.ts`: Guard data loading, pending rules, idle prompts, rule mutations, idle resolution, and Guard input handlers.
- `src/composables/useGuardViewContext.ts`: computed Guard view context assembly from Guard data, workflow state, formatters, and handlers.
- `src/composables/useGuardWorkflow.ts`: guard workflow step completion/unlock state, labels, review completion, and reset watcher.
- `src/composables/useHeatmapCalendar.ts`: heatmap month navigation, calendar computed adapters, helper adapters for cell classes/progress/fetch windows, and Home heatmap summaries.
- `src/composables/useHeatmapGoalSetting.ts`: heatmap goal slider state, saved goal loading, 15-minute normalization, debounced persistence, and timer cleanup.
- `src/composables/useHomeData.ts`: Home data refresh orchestration for summary, logs, heatmap, pending work, reminders, and rhythm stack.
- `src/composables/useHomeOverview.ts`: Home summary state, goal progress, recent activity summary, pending counts, and status-helper binding.
- `src/composables/useHomeReminderPanel.ts`: Home reminder modal draft state, feedback, quick actions, weekly-day chips, and drag/drop event handling.
- `src/composables/useHomeRhythm.ts`: Home seven-day rhythm bar computed adapter from usage-stack data.
- `src/composables/useHomeRhythmTooltip.ts`: Home rhythm summary adapter plus hover-tooltip event state/placement.
- `src/composables/useHomeSchedule.ts`: Home reminder schedule computed binding and locale-aware due-text adapter.
- `src/composables/useHomeViewContext.ts`: computed Home view context assembly from Home data, reminders, heatmap, rhythm, and status state.
- `src/composables/useIdlePromptBannerContext.ts`: computed top-level idle prompt banner context assembly.
- `src/composables/useInsightsData.ts`: Insights top-app/all-time/recent-log loading, all-time filters, and visible Insights derived view data.
- `src/composables/useInsightsSectionNavigation.ts`: Insights section Tauri event listening, target switching, DOM flash highlighting, and listener/timer cleanup.
- `src/composables/useInsightsViewContext.ts`: computed Insights view context assembly from Insights data, navigation, and formatters.
- `src/composables/useLazySettingsMount.ts`: settings view lazy mount state, immediate refresh-on-open, warmup refresh timer, and cleanup.
- `src/composables/useLocale.ts`: main-window locale state, translation helper, Settings language select handler, document language sync, change watcher, and localStorage persistence.
- `src/composables/useMainDataBuffers.ts`: shared main-window data buffers for recent logs, heatmap cells, and Home usage stack.
- `src/composables/useMainDataRefresh.ts`: stable main-window global refresh callback and late-bound refresh handler coordination.
- `src/composables/useMainRefreshPolling.ts`: main-window 5-second Home refresh polling and Guard refresh while Guard is active.
- `src/composables/useMainViewActions.ts`: main-window tab/subview selection actions and their refresh side effects.
- `src/composables/useMainWindowLifecycle.ts`: main-window mount/unmount startup and cleanup sequence.
- `src/composables/useReminders.ts`: reminder list state, upsert/delete/done/reorder/snooze actions, and reminder action loading.
- `src/composables/useSettingsPrivacy.ts`: settings/privacy state, auto-start loading/saving, whitelist actions, and settings refresh cache.
- `src/composables/useSettingsViewContext.ts`: computed Settings view context assembly from settings/privacy state and handlers.
- `src/composables/useThemeMode.ts`: main-window theme state, safe initialization, body theme attribute sync, system preference fallback, and localStorage persistence.
- `src/lib/displayFormatters.ts`: shared UI display formatting helpers for clock text, mapped type labels, process names, and idle prompt spans.
- `src/lib/locale.ts`: shared locale code, storage key, browser fallback detection, and locale-specific text selection helpers.
- `src/lib/guardDiagnostics.ts`: Guard foreground-capture diagnostic reason text, diagnostic rule text, and save eligibility helpers.
- `src/lib/guardIdle.ts`: Guard idle-decision feedback tone/text helpers.
- `src/lib/guardRules.ts`: Guard app-rule search/sort helpers plus rule sort-key and mapped-type guards.
- `src/lib/heatmapCalendarClasses.ts`: heatmap cell class-name construction for level, green intensity, future/past, and today markers.
- `src/lib/heatmapCalendarGrid.ts`: heatmap full-month cell backfill and fixed 42-slot calendar grid construction helpers.
- `src/lib/heatmapCalendarProgress.ts`: heatmap green maximum, current-month goal progress, active streak, and fetch-window helpers.
- `src/lib/heatmapCalendarText.ts`: heatmap month keys, localized month/week labels, and day-number label helpers.
- `src/lib/homeOverviewStatus.ts`: Home current-status label, status tone, and pending-summary text helpers.
- `src/lib/homeReminderReorder.ts`: Home reminder drag/drop ordering helper that preserves undone/done grouping.
- `src/lib/homeRhythmBars.ts`: Home seven-day rhythm bar construction, scale, and segment-height helpers.
- `src/lib/homeRhythmTooltip.ts`: Home rhythm summary, compact duration, segment labels, tooltip palettes, and tone text helpers.
- `src/lib/petCharacter.ts`: pet character image paths, legacy path normalization, and localStorage persistence helpers.
- `src/lib/petContextMenu.ts`: pet-window context menu item definitions and panel-mode toggle wiring.
- `src/lib/petDock.ts`: pure pet-window dock state types, dock-edge mapping, and settle-state conversion helpers.
- `src/lib/petDom.ts`: pet-window initial shell template rendering, required DOM element lookup helpers, localized static text updates, and summary DOM updates.
- `src/lib/petPanelDom.ts`: pet-panel initial shell template rendering, required DOM element lookup, mode display updates, week-header rendering, heatmap-grid DOM helpers, and usage-stack DOM helpers.
- `src/lib/petPanelMetrics.ts`: pure pet-panel heatmap sizing/fetch-window, month cell data, and usage-stack day selection helpers.
- `src/lib/petPanelStack.ts`: pure pet-panel stack signature, segment, color, and percent helpers.
- `src/lib/petPanelText.ts`: pet-panel locale-specific week headers, mode titles, empty-state text, and stack summary labels.
- `src/lib/petPromptDom.ts`: pet prompt bubble DOM show/hide/render helpers and action button disabled-state handling.
- `src/lib/petProcessName.ts`: pet-window process name cleanup for idle pseudo-processes and `.exe` suffixes.
- `src/lib/petPromptDescriptors.ts`: pet prompt descriptor construction for reminders, idle prompts, and pending app-rule prompts.
- `src/lib/petPrompts.ts`: pet prompt DTO types plus pure snooze pruning, availability filtering, and active prompt selection helpers.
- `src/lib/petReminderText.ts`: pet reminder due-text formatting for daily, weekly, and one-time reminders.
- `src/lib/reminderSchedule.ts`: Home reminder visibility rules plus due-text and weekly-day formatting helpers.
- `src/lib/reminderSort.ts`: shared reminder ordering comparator and sorted reminder list helper.
- `src/lib/reminderUpsert.ts`: reminder form input normalization and save-payload construction for one-time, daily, and weekly reminders.
- `src/lib/reminderWeekdays.ts`: shared weekday options, default weekly reminder days, weekday labels, and weekly-day normalization.
- `src/lib/time.ts`: shared frontend time/date formatting and parsing helpers.
- `src/styles/app.css`: global main-window CSS previously embedded in `src/App.vue`.
- `src/pet.ts`: pet UI orchestration, prompt action side effects, context menu orchestration, and drag/dock behavior.
- `src/pet-panel.ts`: mini heatmap and weekly activity panel.
- `src-tauri/src/lib.rs`: Tauri command shells, app setup, command registration, and global shortcut wiring.
- `src-tauri/src/db/connection.rs`: database path resolution, legacy DB migration, SQLite connection opening.
- `src-tauri/src/db/migrations.rs`: SQLite schema creation, table upgrade helpers, default seed data, and heatmap snapshot table creation.
- `src-tauri/src/domain/*.rs`: backend DTOs and command input/output structs grouped by feature area.
- `src-tauri/src/services/analytics.rs`: recent logs, today summary, top-app, heatmap, and usage-stack analytics query bodies.
- `src-tauri/src/services/categories.rs`: category listing and child-category creation SQL.
- `src-tauri/src/services/focus.rs`: focus deviation state machine, debounce/cooldown handling, and snooze behavior.
- `src-tauri/src/services/foreground.rs`: foreground-window capture, current-idle-time platform wrappers, foreground sampling state, diagnostics, and idle prompt coordination.
- `src-tauri/src/services/privacy.rs`: privacy settings, whitelist CRUD, config parsing, process-name normalization, browser title protection, whitelist filtering, and related characterization tests.
- `src-tauri/src/services/reminders.rs`: reminder recurrence, due-time calculation, CRUD, ordering, snooze behavior, and related characterization tests.
- `src-tauri/src/services/rules.rs`: app rule lookup, save validation, rule listing, pending-rule process queries, and related characterization tests.
- `src-tauri/src/services/sessions.rs`: legacy/manual task-session start, stop, and active-root lookup SQL.
- `src-tauri/src/services/startup.rs`: Windows startup registry query/update wrappers plus non-Windows fallbacks.
- `src-tauri/src/services/time.rs`: business-day timestamp/key helpers plus shared interval overlap and merge helpers.
- `src-tauri/src/services/usage.rs`: app usage log append/merge behavior, privacy-filtered log persistence, and idle-decision persistence.
- `src-tauri/src/services/window.rs`: main, pet, and pet-panel window creation, positioning, visibility, dragging, and settle behavior.
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

- `src/App.vue` still owns global lifecycle/refresh orchestration and context assembly.
- `src-tauri/src/lib.rs` is too broad and mixes unrelated layers.
- Feature views now consume typed context contracts, but `src/App.vue` still assembles broad feature contexts and owns too much orchestration.
- Backend SQL, migrations, services, and commands are not separated.
- Tauri command registration still lives in `src-tauri/src/lib.rs`; command wrappers can move to feature command modules later.
- Idle prompts are currently in process memory, so restart behavior needs a product decision.
- Heavy startup data requests need review.
- `task_sessions` exists despite the v1 decision that automatic sampling should be the core time source.

## Current Validation Notes

Known working checks:

- `pnpm.cmd run typecheck`
- `pnpm.cmd run build:check`
- `cargo check` from `src-tauri`
- `cargo test` from `src-tauri`

`pnpm.cmd build` may fail if old `dist` files are locked by a running process. Use a temporary output directory for source validation or stop the locking process before a normal build.
