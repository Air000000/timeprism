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

## S-050 Fresh Install And First-Run Tracking

Use a clean or sandboxed TimePrism app-data directory so this path exercises a truly fresh database.

1. Launch TimePrism with no existing app database.
2. Confirm no `no such table: app_usage_logs` or other pre-initialization database error appears.
3. Before activation, switch foreground apps for more than 10 seconds and confirm no usage rows are captured.
4. Confirm onboarding explains:
   - foreground application/window metadata is observed for duration tracking;
   - core data is stored in local SQLite;
   - browser titles receive privacy processing and private/incognito windows are skipped;
   - TimePrism does not capture screenshots or keystroke contents;
   - Insights shows observed application activity while pet `学 / 休` counters include only activity classified as `LEARN / REST`.
5. Click `开始使用 TimePrism` and confirm tracking becomes active without restarting the app.
6. Switch foreground apps and confirm Insights begins showing observed activity after the normal sampling interval.
7. Confirm pet `学 / 休` remains classification-based and refreshes on the existing several-second cadence rather than synthetic per-second increments.
8. Pause capture from Focus Guard, switch foreground apps for more than 10 seconds, and confirm no new samples are appended.
9. Fully quit and reopen TimePrism; confirm the pause persists and onboarding does not reappear.
10. Resume capture, fully quit and reopen again, and confirm automatic tracking resumes.
11. While capture is paused, confirm the pet mood reports `记录已暂停` / `Tracking paused`.
12. Confirm close-to-background, pet summon/hide, and `Quit TimePrism` behavior still works.

Expected:

- Database initialization completes before product WebViews query SQLite.
- Fresh installs remain capture-off until explicit onboarding activation succeeds.
- Pause/resume is persisted, not reset by restart.
- Tracking, Home, and pet updates retain the existing approximately 5-second cadence.

## S-060 Upgrade From A Pre-Tracking-State Database

Use a backed-up database created by a TimePrism build from before the `onboarding_completed` and `auto_capture_enabled` keys existed.

1. Launch the current build against the backed-up pre-change database.
2. Confirm existing records remain present.
3. Confirm onboarding does not appear.
4. Confirm tracking starts enabled by default for this recognized legacy database.
5. Pause capture, fully quit, and reopen; confirm the explicit pause is preserved.

Expected:

- Recognizable legacy databases migrate to onboarded + capture-enabled defaults without overwriting later explicit tracking choices.
- Existing user data survives migration.

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

## S-500 Guard And Idle Attribution

1. Open Focus Guard.
2. Confirm pending apps list renders.
3. Save a pending app as Learn, another as Break, and another as Unclassified when available.
4. Create or encounter an idle segment after spending time in a known foreground app.
5. When a previous-app candidate exists, confirm the prompt shows only the app/process identity and never the previous raw window title.
6. Confirm the compact choice set is `Continue <app>`, `Away`, and `Other…` rather than five peer actions.
7. Choose `Continue <app>` and confirm the replacement interval is attributed to that process while its existing Learn/Break/Unclassified rule is unchanged.
8. Confirm classification semantics remain separate from attribution: a Learn/Break app contributes to the corresponding pet counter; an Unclassified app can gain usage time without increasing pet `学 / 休`.
9. Create another candidate idle segment, open `Other…`, and confirm Learn, Break, and Decide later remain available as semantic fallbacks.
10. Confirm app attribution is never remembered for later idle segments even when the remember checkbox is enabled for Learn/Break/Away decisions.
11. When no trustworthy previous-app candidate exists, confirm the prompt falls back directly to Learn / Break / Away with Decide later secondary.
12. Resolve an idle prompt as Away and defer another prompt if available.
13. Search existing rules, edit a rule, and save.
14. Review diagnostics.

Expected:

- Rule changes persist.
- `Continue <app>` changes interval attribution without rewriting that app's classification rule.
- Learn/Break/Away fallback remains available without requiring app attribution.
- App attribution is explicit per interval and is never auto-remembered.
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
11. With an idle prompt that offers previous-app attribution, enable the privacy curtain (or make the candidate fail whitelist-only policy) before choosing `Continue <app>`.
12. Confirm attribution is rejected rather than bypassing privacy, and confirm the idle prompt remains available so it can still be resolved as Learn / Break / Away.

Expected:

- Privacy behavior affects future storage and user-confirmed app attribution.
- App attribution never bypasses curtain or whitelist policy.
- A privacy rejection does not silently discard the unresolved idle prompt.
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
4. When an idle prompt has a previous-app candidate, confirm the pet bubble offers `Continue <app>`, `Away`, and `Other…` only.
5. Choose `Other…` and confirm TimePrism opens Focus Guard for the full fallback choices instead of expanding a dense five-button pet prompt.
6. Right-click pet and open context menu.
7. Open main window from pet.
8. Hide pet.
9. Summon pet again.
10. Close pet.

Expected:

- Pet appears quickly.
- Pet window remains transparent/frameless.
- Idle confirmation stays compact on the pet surface.
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
