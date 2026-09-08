# TimePrism Agent Guide

This file is the highest-priority project guide for work inside this repository.

## Product Rules

- TimePrism is a Windows-first, local-first desktop rhythm tracker.
- Automatic foreground sampling is the primary source of time facts.
- Privacy processing must happen before storing window titles or usage logs.
- The pet is a lightweight companion surface, not a second full application.
- Do not add cloud accounts, sync, team features, billing, or strict blocking flows unless the product decision changes.

## Refactor Rules

- Keep the current app runnable while refactoring.
- Prefer staged extraction over a big-bang rewrite.
- Preserve existing Tauri command names and payload shapes until a compatibility layer exists.
- Do not expand `src/App.vue` or `src-tauri/src/lib.rs` with new feature logic.
- Do not introduce new broad `ctx: any` state bags.
- Keep SQL in database/repository/service modules, not in UI or window code.
- Keep Windows APIs behind a platform/service boundary.
- Do not delete or overwrite user data without explicit confirmation.

## Before Editing

For non-trivial code changes, identify:

- Primary domain.
- Affected files and windows.
- Affected Tauri commands.
- Affected database/schema.
- Privacy impact.
- Startup/performance impact.
- Validation command.

## Validation Defaults

- Frontend typecheck: `pnpm.cmd run typecheck`
- Frontend build check: `pnpm.cmd run build:check`
- Backend check: `cargo check` from `src-tauri`
- Full Tauri build only when packaging/runtime behavior is relevant.

Remove temporary validation output such as `dist-codex-check` after use.
