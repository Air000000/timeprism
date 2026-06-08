# API Contracts

This file records the current Tauri command surface. Early refactor work should preserve these names and payload shapes.

The frontend wrapper currently lives in `src/api.ts`.

## Source And Caller Map

Backend command definitions currently live in `src-tauri/src/lib.rs`.

Frontend business wrappers currently live in `src/api.ts`.

High-level caller map:

| Surface | Caller file | Notes |
| --- | --- | --- |
| Main app | `src/App.vue` | Uses most business APIs for Home, Insights, Guard, Settings, reminders |
| Home view | `src/components/HomeView.vue` | Calls actions indirectly through `ctx` |
| Insights view | `src/components/InsightsView.vue` | Consumes data indirectly through `ctx` |
| Guard view | `src/components/GuardView.vue` | Calls actions indirectly through `ctx` |
| Pet | `src/pet.ts` | Uses `getTodaySummary`; calls reminder/rule/idle/window commands directly for lightweight prompts |
| Pet panel | `src/pet-panel.ts` | Uses `getLearnHeatmap` and `getUsageStack` |

Backend command line references as of 2026-06-08:

| Command | Rust function starts near |
| --- | --- |
| `list_reminders` | `src-tauri/src/lib.rs:1571` |
| `list_due_reminders` | `src-tauri/src/lib.rs:1582` |
| `save_reminder` | `src-tauri/src/lib.rs:1597` |
| `delete_reminder` | `src-tauri/src/lib.rs:1722` |
| `set_reminder_done` | `src-tauri/src/lib.rs:1731` |
| `set_reminder_order` | `src-tauri/src/lib.rs:1785` |
| `snooze_reminder` | `src-tauri/src/lib.rs:1803` |
| `list_categories` | `src-tauri/src/lib.rs:1821` |
| `create_category` | `src-tauri/src/lib.rs:1852` |
| `start_session` | `src-tauri/src/lib.rs:1880` |
| `stop_active_session` | `src-tauri/src/lib.rs:1900` |
| `capture_foreground_once` | `src-tauri/src/lib.rs:1927` |
| `list_pending_idle_prompts` | `src-tauri/src/lib.rs:2109` |
| `resolve_idle_prompt` | `src-tauri/src/lib.rs:2133` |
| `get_idle_memory_state` | `src-tauri/src/lib.rs:2184` |
| `list_recent_logs` | `src-tauri/src/lib.rs:2203` |
| `list_foreground_capture_diagnostics` | `src-tauri/src/lib.rs:2239` |
| `list_app_rules` | `src-tauri/src/lib.rs:2288` |
| `list_pending_rule_processes` | `src-tauri/src/lib.rs:2320` |
| `get_today_summary` | `src-tauri/src/lib.rs:2365` |
| `list_top_apps_today` | `src-tauri/src/lib.rs:2414` |
| `list_top_apps_all_time` | `src-tauri/src/lib.rs:2456` |
| `get_learn_heatmap` | `src-tauri/src/lib.rs:2561` |
| `get_heatmap_goal_seconds_setting` | `src-tauri/src/lib.rs:2697` |
| `set_heatmap_goal_seconds_setting` | `src-tauri/src/lib.rs:2704` |
| `get_usage_stack` | `src-tauri/src/lib.rs:2717` |
| `save_app_rule` | `src-tauri/src/lib.rs:2852` |
| `check_focus_deviation` | `src-tauri/src/lib.rs:2880` |
| `snooze_focus_guard` | `src-tauri/src/lib.rs:2993` |
| `get_privacy_settings` | `src-tauri/src/lib.rs:3006` |
| `update_privacy_settings` | `src-tauri/src/lib.rs:3016` |
| `get_auto_start_enabled` | `src-tauri/src/lib.rs:3065` |
| `set_auto_start_enabled` | `src-tauri/src/lib.rs:3070` |
| `list_whitelist` | `src-tauri/src/lib.rs:3075` |
| `set_whitelist_item` | `src-tauri/src/lib.rs:3093` |
| `summon_pet_window` | `src-tauri/src/lib.rs:3338` |
| `sync_pet_window_layout` | `src-tauri/src/lib.rs:3377` |
| `settle_pet_window` | `src-tauri/src/lib.rs:3383` |
| `show_pet_panel` | `src-tauri/src/lib.rs:3391` |
| `hide_pet_panel` | `src-tauri/src/lib.rs:3422` |
| `sync_pet_panel_position` | `src-tauri/src/lib.rs:3433` |
| `resize_pet_panel` | `src-tauri/src/lib.rs:3441` |
| `move_pet_window` | `src-tauri/src/lib.rs:3460` |
| `close_pet_window` | `src-tauri/src/lib.rs:3473` |
| `hide_pet_window` | `src-tauri/src/lib.rs:3485` |
| `begin_pet_drag` | `src-tauri/src/lib.rs:3497` |
| `show_main_window` | `src-tauri/src/lib.rs:3508` |
| `show_main_window_section` | `src-tauri/src/lib.rs:3537` |

Line numbers are guidance only. Update this table when functions move.

## Categories And Sessions

### `list_categories`

Input: none.

Returns: `Category[]`.

`Category` fields:

- `id: number`
- `parent_id: number | null`
- `name: string`
- `root_type: "LEARN" | "REST"`
- `color_hex: string`

### `create_category`

Input: `{ input: { parent_id: number, name: string, color_hex?: string } }`

Returns: created category id.

### `start_session`

Input: `{ categoryId: number }`

Returns: created session id.

Compatibility note: this exists today, but product decisions say traditional manual timing is not a v1 core source. Do not expand this feature without a decision.

### `stop_active_session`

Input: none.

Returns: `boolean`.

## Foreground Capture And Logs

### `append_app_usage_log`

Input:

- `processName: string`
- `windowTitle: string`
- `startTimestamp: number`
- `durationMs: number`

Returns: `boolean`.

### `capture_foreground_once`

Input: `{ durationMs?: number }`

Returns: `boolean`.

Behavior:

- Samples current foreground window.
- Applies privacy before storage.
- Merges with latest same process/title segment when possible.
- Adds bounded diagnostics.

### `list_foreground_capture_diagnostics`

Input:

- `limit?: number`
- `uniqueByProcess?: boolean`

Returns: `ForegroundCaptureDiagnostic[]`.

### `list_recent_logs`

Input: `{ limit?: number }`

Returns: `RecentLog[]`.

## Idle

### `list_pending_idle_prompts`

Input: `{ limit?: number }`

Returns: `IdlePrompt[]`.

Current risk: pending idle prompts are process-memory state.

### `resolve_idle_prompt`

Input:

```ts
{
  input: {
    prompt_id: number;
    decision: "LEARN" | "REST" | "IDLE" | "SKIP";
    remember_this_session?: boolean;
  }
}
```

Returns: `boolean`.

### `get_idle_memory_state`

Input: none.

Returns: `{ remembered_decision: "LEARN" | "REST" | "IDLE" | null }`.

### `clear_idle_memory_state`

Input: none.

Returns: `void`.

## Rules

### `list_app_rules`

Input: `{ limit?: number }`

Returns: `AppRule[]`.

### `list_pending_rule_processes`

Input: `{ limit?: number }`

Returns: `PendingRuleProcess[]`.

### `save_app_rule`

Input:

```ts
{
  input: {
    process_name: string;
    mapped_type: "LEARN" | "REST" | "IGNORE";
    privacy_level?: "NORMAL" | "BLUR_TITLE" | "WHITELIST_ONLY";
  }
}
```

Returns: `void`.

## Reminders

### `list_reminders`

Input:

- `limit?: number`
- `includeCompleted?: boolean`

Returns: `Reminder[]`.

### `list_due_reminders`

Input: `{ limit?: number }`

Returns: `Reminder[]`.

### `save_reminder`

Input:

```ts
{
  input: {
    id?: number;
    content: string;
    repeat_rule: "NONE" | "DAILY" | "WEEKLY";
    remind_at?: number;
    daily_time_minutes?: number;
    weekly_days?: number[];
  }
}
```

Returns: reminder id.

### `delete_reminder`

Input: `{ id: number }`

Returns: `boolean`.

### `set_reminder_done`

Input: `{ input: { id: number, done: boolean } }`

Returns: `boolean`.

### `set_reminder_order`

Input: `{ input: { ordered_ids: number[] } }`

Returns: `boolean`.

### `snooze_reminder`

Input: `{ id: number, snoozeSeconds?: number }`

Returns: `boolean`.

## Analytics

### `get_today_summary`

Input: none.

Returns:

- `learn_seconds: number`
- `rest_seconds: number`
- `active_session_id: number | null`

### `list_top_apps_today`

Input: `{ limit?: number }`

Returns: `TopApp[]`.

### `list_top_apps_all_time`

Input:

- `limit?: number`
- `rootFilter?: "ALL" | "LEARN" | "REST"`
- `includeIgnore?: boolean`

Returns: `TopApp[]`.

### `get_learn_heatmap`

Input:

- `days?: number`
- `goalSeconds?: number`

Returns: `LearnHeatmapCell[]`.

### `get_heatmap_goal_seconds_setting`

Input: none.

Returns: goal seconds.

### `set_heatmap_goal_seconds_setting`

Input: `{ goalSeconds: number }`

Returns: saved goal seconds.

### `get_usage_stack`

Input:

- `days?: number`
- `rootFilter?: "ALL" | "LEARN" | "REST"`

Returns: `UsageStackDay[]`.

## Focus Guard

### `check_focus_deviation`

Input:

- `processName: string`
- `debounceSeconds?: number`

Returns: `DeviationCheck`.

### `snooze_focus_guard`

Input: `{ cooldownSeconds?: number }`

Returns: `void`.

## Settings And Privacy

### `get_privacy_settings`

Input: none.

Returns:

- `curtain_enabled: boolean`
- `browser_title_mode: "FULL" | "BLUR" | "NONE"`
- `whitelist_only_enabled: boolean`

### `update_privacy_settings`

Input:

```ts
{
  input: {
    curtain_enabled: boolean;
    browser_title_mode: "FULL" | "BLUR" | "NONE";
    whitelist_only_enabled: boolean;
  }
}
```

Returns: `void`.

### `get_auto_start_enabled`

Input: none.

Returns: `boolean`.

### `set_auto_start_enabled`

Input: `{ enabled: boolean }`

Returns: `boolean`.

### `list_whitelist`

Input: none.

Returns: `string[]`.

### `set_whitelist_item`

Input: `{ input: { process_name: string, enabled: boolean } }`

Returns: `void`.

## Window And Pet

Commands:

- `show_main_window`
- `show_main_window_section`
- `summon_pet_window`
- `sync_pet_window_layout`
- `settle_pet_window`
- `show_pet_panel`
- `hide_pet_panel`
- `sync_pet_panel_position`
- `resize_pet_panel`
- `move_pet_window`
- `hide_pet_window`
- `close_pet_window`
- `begin_pet_drag`

Compatibility note: pet and window commands are part of user-visible behavior. Split backend implementation first; rename only after all callers are updated.
