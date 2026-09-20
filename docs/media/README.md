# Product Media Capture Guide

This directory is reserved for **real TimePrism v0.1.0+ application captures** used by the repository README, Releases, and promotion material.

Do not use mockups, generated UI, or an older pre-v0.1.0 build as evidence of implemented behavior.

## Capture Set

Create these five assets from a current Windows build using synthetic or sanitized data:

| File | Surface | Required content |
| --- | --- | --- |
| <code>timeprism-home.png</code> | Home | Today totals, editable goal/progress, calendar/recent rhythm, navigation, and Yin Yue visible beside the app if framing permits |
| <code>timeprism-insights.png</code> | Insights | Top apps plus a readable historical/heatmap or day-composition view |
| <code>timeprism-focus-guard.png</code> | Focus Guard | App classification or idle-resolution workflow with no private titles |
| <code>timeprism-pet.png</code> | Yin Yue desktop pet | Pet plus one meaningful state: compact prompt, docked pose, or companion panel |
| <code>timeprism-demo.gif</code> | Short walkthrough | Home → Insights → Focus Guard → Yin Yue interaction |

## Recommended Walkthrough

Keep the GIF short and intentional. A good sequence is roughly:

1. Home: show today's rhythm and adjust the learning goal.
2. Insights: switch to a historical/usage view.
3. Focus Guard: show an app rule or idle-correction decision.
4. Return to the desktop pet: drag/dock Yin Yue or show a compact prompt/panel.

Avoid waiting for timers or recording long cursor travel. The purpose is to demonstrate the product model, not every setting.

## Privacy Gate

Foreground-window data can contain private information. Before committing any capture:

- use synthetic application/process names where practical;
- remove real window titles, document names, URLs, account names, notifications, and personal calendar/reminder text;
- do not expose the real SQLite database path, Windows username, browser profile, or local filesystem path;
- verify Focus Guard diagnostics contain no raw private titles;
- inspect every GIF frame, not only its first frame;
- prefer a clean demo database over pixel-level redaction when possible.

If a capture contains sensitive information, recapture it. Do not rely on a blur that can be reversed or missed in a later frame.

## Framing

- Capture the actual installed/current application, not Figma or generated UI.
- Use one coherent theme across the set.
- Keep the main window at its normal aspect ratio; do not stretch it.
- Crop browser chrome, unrelated desktop space, taskbar content, and terminal windows unless they are part of the point being shown.
- Keep screenshots readable at roughly 900–1200 px README display width.
- For the pet image, leave enough desktop context to make the always-on-top/docking behavior understandable.
- For the GIF, prefer 10–20 seconds over a long tutorial.

## README Insertion

The root README intentionally does **not** reference these files until they exist.

After all five assets pass the privacy gate, insert the GIF below the top badges/links and add this screenshot grid after the product introduction:

~~~markdown
<p align="center">
  <img src="docs/media/timeprism-demo.gif" alt="TimePrism product walkthrough" width="900">
</p>

| Home | Insights |
| --- | --- |
| ![Home](docs/media/timeprism-home.png) | ![Insights](docs/media/timeprism-insights.png) |

| Focus Guard | Yin Yue desktop pet |
| --- | --- |
| ![Focus Guard](docs/media/timeprism-focus-guard.png) | ![Yin Yue desktop pet](docs/media/timeprism-pet.png) |
~~~

Do not add image references before the corresponding files are committed; broken placeholders are worse than a text-only README.

## Promotion Copies

README captures should be the clean source assets. Platform-specific posts may crop them differently, but keep the original repository versions free of platform watermarks, annotations, or promotional text.
