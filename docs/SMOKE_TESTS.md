# Smoke Tests

Use this checklist for manual validation after meaningful refactor batches.

## S-000 Preflight

Commands:

```powershell
git status --short --branch
pnpm.cmd run typecheck
pnpm.cmd run build:check
```

If Rust changed:

```powershell
cd src-tauri
cargo check
```

Cleanup:

```powershell
Remove-Item -LiteralPath .\dist-codex-check -Recurse -Force
```

## S-100 Main Window

1. Launch the app.
2. Confirm the main window opens.
3. Confirm close hides the window instead of exiting the app.
4. Reopen main window through pet or command path if available.

Expected:

- Main window is usable.
- No blank white screen.
- Navigation is visible.

## S-200 Navigation

1. Open Home.
2. Open Insights.
3. Open Focus Guard.
4. Open Settings.
5. Switch language if supported.
6. Switch theme if supported.

Expected:

- No view crashes.
- Labels fit their controls.
- Main state survives navigation.

## S-300 Home

1. Open Home.
2. Confirm today's learn/rest summary appears.
3. Confirm pending app, idle, and reminder counts appear.
4. Confirm today's schedule/reminders list renders.
5. Confirm learning calendar renders.

Expected:

- Home renders without waiting for all historical analytics.
- Counts match Guard and reminder state.

## S-400 Reminders

1. Create a one-time reminder.
2. Edit its content.
3. Mark it done.
4. Restore it.
5. Snooze it for 10 minutes.
6. Delete it.
7. Create a daily reminder.
8. Create a weekly reminder.

Expected:

- List updates after each action.
- Due text is understandable.
- Pet can see due reminders when due.

## S-500 Guard

1. Open Focus Guard.
2. Confirm pending apps list renders.
3. Save a pending app as Learn.
4. Save a pending app as Break.
5. Save a pending app as Unclassified.
6. Resolve an idle prompt as Learn/Break/Away if one exists.
7. Defer an idle prompt if one exists.
8. Search existing rules.
9. Edit a rule and save.
10. Review diagnostics.

Expected:

- Rule changes persist.
- Guard steps unlock correctly.
- Diagnostics show stored/blocked status.

## S-600 Privacy And Capture

1. Set browser title mode to `BLUR`.
2. Capture a browser foreground window.
3. Confirm stored/recent title is generalized.
4. Set browser title mode to `NONE`.
5. Capture again.
6. Confirm stored/recent title is empty or safe.
7. Enable whitelist-only mode.
8. Confirm non-whitelisted processes do not store normal logs.
9. Add a process to whitelist.
10. Confirm whitelisted process can be stored.

Expected:

- Privacy behavior affects future storage.
- Diagnostics do not leak sensitive raw titles.

## S-700 Insights

1. Open today's top apps.
2. Open all-time app usage.
3. Toggle Learn/Rest/All filter.
4. Toggle include unclassified.
5. Open recent logs.
6. Confirm heatmap and usage stack areas render if present.

Expected:

- Queries complete.
- Filtered totals are plausible.
- Recent logs respect privacy settings.

## S-800 Pet

1. Summon pet.
2. Drag pet.
3. Dock near left or right screen edge.
4. Right-click pet and open context menu.
5. Open main window from pet.
6. Hide pet.
7. Summon pet again.
8. Close pet.

Expected:

- Pet appears quickly.
- Pet window remains transparent/frameless.
- Context menu actions work.
- No taskbar pollution.

## S-900 Pet Panel

1. From pet context menu, open learning calendar panel.
2. Switch month in panel.
3. Open weekly activity panel.
4. Move pet while panel is visible.
5. Hide panel.

Expected:

- Panel follows pet.
- Panel avoids screen overflow.
- Panel content remains small and fast.

## S-999 Failure Handling

If a smoke step fails:

1. Record exact step id.
2. Record branch and commit.
3. Record error text or screenshot.
4. Decide whether it is a refactor regression or existing behavior.
5. Do not continue broad refactor work until regression status is known.
