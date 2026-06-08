# Database And Privacy

## Principles

- SQLite is the local source of truth.
- Privacy-sensitive values must be processed before persistence.
- Schema changes require an explicit migration plan.
- Refactor-only changes should not alter schema.
- Data deletion, restore, and overwrite flows require explicit user confirmation.

## Database Location

The backend currently prefers `app_local_data_dir()/timeprism.db`.

It can fall back to a legacy `app_data_dir()/timeprism.db` path if migration/opening fails.

Migration behavior:

- If preferred DB exists, use it.
- If preferred DB does not exist and legacy DB exists, copy legacy DB and SQLite sidecars.
- If migration fails, try opening the legacy DB.

## Current Tables

### `categories`

Purpose: category tree.

Key fields:

- `id`
- `parent_id`
- `name`
- `color_hex`
- `root_type`

Current schema:

```sql
CREATE TABLE IF NOT EXISTS categories (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  parent_id INTEGER,
  name TEXT NOT NULL,
  color_hex TEXT NOT NULL,
  root_type TEXT NOT NULL CHECK(root_type IN ('LEARN','REST')),
  FOREIGN KEY(parent_id) REFERENCES categories(id)
);
```

### `task_sessions`

Purpose: legacy/manual-session compatibility.

Key fields:

- `id`
- `category_id`
- `start_time`
- `end_time`
- `is_flow_target`

Decision needed: whether this remains compatibility-only or is removed from v1 UI.

Current schema:

```sql
CREATE TABLE IF NOT EXISTS task_sessions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  category_id INTEGER NOT NULL,
  start_time INTEGER NOT NULL,
  end_time INTEGER,
  is_flow_target INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY(category_id) REFERENCES categories(id)
);
```

### `app_usage_logs`

Purpose: foreground activity records.

Key fields:

- `id`
- `process_name`
- `window_title`
- `start_timestamp`
- `duration_ms`

Privacy requirement: `window_title` must already be privacy-processed before insertion.

Current schema:

```sql
CREATE TABLE IF NOT EXISTS app_usage_logs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  process_name TEXT NOT NULL,
  window_title TEXT NOT NULL,
  start_timestamp INTEGER NOT NULL,
  duration_ms INTEGER NOT NULL
);
```

### `app_rules`

Purpose: process classification and per-rule privacy.

Key fields:

- `process_name`
- `mapped_type`: `LEARN`, `REST`, `IGNORE`
- `privacy_level`: `NORMAL`, `BLUR_TITLE`, `WHITELIST_ONLY`
- `created_at`
- `updated_at`

Current schema:

```sql
CREATE TABLE IF NOT EXISTS app_rules (
  process_name TEXT PRIMARY KEY,
  mapped_type TEXT NOT NULL CHECK(mapped_type IN ('LEARN','REST','IGNORE')),
  privacy_level TEXT NOT NULL DEFAULT 'NORMAL',
  created_at INTEGER NOT NULL DEFAULT 0,
  updated_at INTEGER NOT NULL DEFAULT 0
);
```

### `app_config`

Purpose: key-value settings.

Known keys:

- `curtain_enabled`
- `browser_title_mode`
- `browser_blur_enabled`
- `whitelist_only_enabled`
- `heatmap_goal_seconds`

Current schema:

```sql
CREATE TABLE IF NOT EXISTS app_config (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
```

### `app_whitelist`

Purpose: allowed processes when whitelist-only capture is enabled.

Key field:

- `process_name`

Current schema:

```sql
CREATE TABLE IF NOT EXISTS app_whitelist (
  process_name TEXT PRIMARY KEY
);
```

### `reminders`

Purpose: local schedule/reminder items.

Key fields:

- `id`
- `content`
- `repeat_rule`
- `sort_order`
- `remind_at`
- `daily_time_minutes`
- `weekly_days`
- `is_completed`
- `completed_day_key`
- `completed_at`
- `snooze_until`
- `created_at`
- `updated_at`

Current schema:

```sql
CREATE TABLE IF NOT EXISTS reminders (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  content TEXT NOT NULL,
  repeat_rule TEXT NOT NULL CHECK(repeat_rule IN ('NONE','DAILY','WEEKLY')),
  sort_order INTEGER NOT NULL DEFAULT 0,
  remind_at INTEGER,
  daily_time_minutes INTEGER,
  weekly_days TEXT,
  is_completed INTEGER NOT NULL DEFAULT 0,
  completed_day_key TEXT,
  completed_at INTEGER,
  snooze_until INTEGER,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
```

### `daily_heatmap_snapshot`

Purpose: sealed historical heatmap state.

Key fields:

- `day_key`
- `learn_seconds`
- `goal_seconds`
- `level`
- `sealed_at`

Current schema:

```sql
CREATE TABLE IF NOT EXISTS daily_heatmap_snapshot (
  day_key TEXT PRIMARY KEY,
  learn_seconds INTEGER NOT NULL,
  goal_seconds INTEGER NOT NULL,
  level TEXT NOT NULL CHECK(level IN ('GRAY', 'YELLOW', 'GREEN')),
  sealed_at INTEGER NOT NULL
);
```

## Current Indexes

```sql
CREATE INDEX IF NOT EXISTS idx_task_sessions_start_time
  ON task_sessions(start_time);

CREATE INDEX IF NOT EXISTS idx_app_usage_logs_start
  ON app_usage_logs(start_timestamp);

CREATE INDEX IF NOT EXISTS idx_app_usage_logs_process
  ON app_usage_logs(process_name);

CREATE INDEX IF NOT EXISTS idx_reminders_due_none
  ON reminders(repeat_rule, remind_at, is_completed);

CREATE INDEX IF NOT EXISTS idx_reminders_due_daily
  ON reminders(repeat_rule, daily_time_minutes, completed_day_key);

CREATE INDEX IF NOT EXISTS idx_reminders_due_weekly
  ON reminders(repeat_rule, daily_time_minutes, weekly_days, completed_day_key);

CREATE INDEX IF NOT EXISTS idx_daily_heatmap_snapshot_day_key
  ON daily_heatmap_snapshot(day_key);
```

## Current Compatibility Migrations

Current helper behavior:

- `ensure_app_rules_time_columns`: adds `created_at` and `updated_at` to old `app_rules`.
- `ensure_reminders_weekly_columns`: rebuilds old `reminders` table when weekly/snooze/order fields are missing.
- `ensure_reminders_sort_order`: adds `sort_order` to old `reminders`.
- `ensure_heatmap_snapshot_table`: creates the derived heatmap snapshot table and index.

During backend extraction, keep SQL equivalent unless the batch is explicitly a migration batch.

## Privacy Flow

Required order:

```text
sample foreground process/title
-> normalize process and title
-> filter shell/noise windows
-> load privacy settings and whitelist
-> block non-whitelisted process when whitelist-only is enabled
-> apply browser/private-title protection
-> apply per-rule privacy level
-> classify through app rules
-> write usage log or diagnostic
```

Do not store raw browser/private titles and sanitize later.

## Browser Title Modes

- `FULL`: store allowed browser title.
- `BLUR`: store a generalized browser title.
- `NONE`: store no browser title.

Protective behavior should remain the default.

## Incognito / Private Windows

If a title indicates private browsing, avoid storing the raw title even if full-title mode is enabled.

## Whitelist-Only Mode

When enabled:

- Only whitelisted processes should be stored.
- Blocked samples may create diagnostics.
- Diagnostics must not leak sensitive raw titles.

## Refactor Checklist For Data Changes

Before changing database or privacy code, answer:

1. Does this change alter schema?
2. Is there a migration?
3. Can old data still be opened?
4. Could raw sensitive data be stored?
5. Is privacy applied before insert/update?
6. What happens if migration/open fails?
7. Is backup/export affected?
