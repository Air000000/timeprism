# Product Media Capture Guide

This directory is reserved for **real TimePrism application captures** used by the repository README and Releases. Do not use mockups or generated UI images as evidence of implemented behavior.

## Required captures

Capture these from a current Windows build using synthetic or sanitized data:

| File | Surface | What must be visible |
| --- | --- | --- |
| `timeprism-home.png` | Home | Today totals, rhythm/progress, reminders, navigation |
| `timeprism-insights.png` | Insights | Top apps plus at least one historical/heatmap view |
| `timeprism-focus-guard.png` | Focus Guard | Rule classification or idle-resolution workflow without private titles |
| `timeprism-pet.png` | Desktop pet | Pet together with its compact panel or prompt |
| `timeprism-demo.gif` | Short product flow | Home -> Insights -> Focus Guard -> desktop pet |

## Privacy gate

Foreground-window data can contain private information. Before committing any capture:

- use synthetic application/process names where practical;
- remove real window titles, document names, URLs, account names, and notification text;
- do not expose the real SQLite database path or local username;
- verify Focus Guard diagnostics contain no raw private titles;
- inspect every GIF frame, not only its first frame.

If sanitization would make a capture ambiguous, create a clean demo database instead of editing pixels after capture.

## Presentation rules

- Capture the actual application, not Figma/mockups/generated UI.
- Keep the main window at its normal aspect ratio; do not stretch screenshots.
- Prefer one coherent theme across all captures.
- Crop surrounding desktop chrome when it adds no information.
- Keep screenshots readable on a GitHub README at roughly 900-1200 px display width.
- Keep the GIF focused on interaction; avoid cursor wandering and long idle pauses.

## README layout after capture

Place the GIF directly below the repository badges, then use a compact two-column screenshot table:

```markdown
<p align="center">
  <img src="docs/media/timeprism-demo.gif" alt="TimePrism product walkthrough" width="900">
</p>

| Home | Insights |
| --- | --- |
| ![Home](docs/media/timeprism-home.png) | ![Insights](docs/media/timeprism-insights.png) |

| Focus Guard | Desktop pet |
| --- | --- |
| ![Focus Guard](docs/media/timeprism-focus-guard.png) | ![Desktop pet](docs/media/timeprism-pet.png) |
```

Do not add these references to the root README until the corresponding files exist, because broken image placeholders are worse than a text-only README.
