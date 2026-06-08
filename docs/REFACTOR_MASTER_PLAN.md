# Refactor Master Plan

## Intent

Turn the current working prototype into a maintainable application without losing working behavior or local data compatibility.

This is not a full rewrite from zero. The current app stays runnable while modules are extracted and rebuilt.

## Current Strategy

Use `main` as the stable branch and perform refactor work on a dedicated branch such as `refactor/architecture`.

Recommended safety order:

1. Commit or stash unrelated local edits.
2. Tag the pre-refactor baseline, for example `pre-refactor-2026-06-08`.
3. Create a refactor branch.
4. Keep each phase buildable.
5. Merge only after a phase has a clear validation result.

## Operating Documents

Before starting implementation, use:

- `docs/REFACTOR_PLAYBOOK.md` for the exact batch workflow.
- `docs/FIRST_REFACTOR_BATCHES.md` for the first concrete implementation slices.
- `docs/PHASE_GATES.md` for phase-by-phase evaluation gates.
- `docs/TRACEABILITY.md` for process records and auditability.
- `docs/TESTING_STRATEGY.md` for automated safety nets.
- `docs/SMOKE_TESTS.md` for manual validation.
- `docs/DECISIONS.md` and `docs/ADR_TEMPLATE.md` for decisions.

## Professional Refactor Principles Adopted

- Refactor in small behavior-preserving steps.
- Characterize risky existing behavior before changing it.
- Prefer branch-by-abstraction and compatibility wrappers over big-bang replacement.
- Use strangler-style feature extraction: keep the old surface working while replacing one domain at a time.
- Keep important decisions explicit through decision records.
- Keep the stable branch releasable.

## Non-Goals

- Do not rebuild every feature before shipping any improvement.
- Do not change user data shape unless a migration is planned.
- Do not rename Tauri commands during early extraction.
- Do not add new product features while splitting architecture.
- Do not replace Vue, Tauri, Rust, or SQLite without a separate decision.

## Phase 1: Safety And Contracts

Goal: make the current behavior easy to compare.

Tasks:

- Keep this documentation set current.
- Record existing Tauri command contracts in `docs/API_CONTRACTS.md`.
- Record database and privacy facts in `docs/DATABASE_AND_PRIVACY.md`.
- Add or preserve validation commands.
- Add small tests only where they reduce refactor risk.

Acceptance:

- Current app typechecks.
- Rust backend checks.
- Existing commands are documented enough to preserve compatibility.

## Phase 2: Backend Split Without Behavior Change

Goal: reduce `src-tauri/src/lib.rs` without changing behavior.

Suggested extraction order:

1. `db`: paths, connection, migrations, table helpers.
2. `domain`: shared serializable types and value normalization.
3. `services/privacy`: browser title mode, whitelist behavior, shell filters.
4. `services/rules`: rule lookup, save, pending process queries.
5. `services/reminders`: reminder recurrence and ordering.
6. `services/analytics`: today summary, top apps, heatmap, usage stack.
7. `services/foreground`: Windows capture, idle boundary coordination, diagnostics.
8. `services/window`: main, pet, and pet-panel window behavior.
9. `commands`: thin command wrappers.

Acceptance:

- Command names and payloads remain compatible.
- `cargo check` passes after each extraction batch.
- No schema changes are introduced by extraction-only commits.

## Phase 3: Frontend API And Shell

Goal: make feature UI independent from the giant root component.

Tasks:

- Split `src/api.ts` into `src/api/types.ts`, `src/api/client.ts`, and command files.
- Add `src/app/AppShell.vue` and navigation metadata.
- Move shared formatting/date helpers into `src/lib`.
- Introduce typed view models for one feature at a time.

Acceptance:

- `App.vue` shrinks by moving orchestration out, not by hiding it in another giant file.
- Feature components receive typed props or use feature composables.
- No new `ctx: any` is introduced.

## Phase 4: Feature Extraction

Recommended order:

1. Settings and privacy.
2. Reminders.
3. Guard.
4. Insights.
5. Home.
6. Pet and pet-panel.

Reason:

- Settings/privacy and reminders have clearer boundaries.
- Guard validates capture/rules/idle integration.
- Home aggregates many domains, so it should move after supporting modules are stable.
- Pet depends on shared reminder/idle/rule APIs and should stay lightweight.

Acceptance:

- Each feature keeps its existing user-visible behavior.
- Feature state lives in feature modules, not in `App.vue`.
- Heavy analytics remain lazy-loaded.

## Phase 5: Data Management And Hardening

Goal: make the app safer for personal long-term use.

Tasks:

- Show database path and version.
- Add export/backup flow.
- Add recent error or diagnostics surface.
- Review migration failure behavior.
- Review privacy copy and defaults.

Acceptance:

- User can find and back up local data.
- Major database/privacy failures are not silent.

## Phase 6: Cleanup

Goal: remove compatibility scaffolding only after replacement modules are stable.

Tasks:

- Delete dead code.
- Remove temporary wrappers.
- Update screenshots and README.
- Ensure docs reflect the final architecture.

Acceptance:

- Build and checks pass.
- No major architecture red flags remain.
- `docs/REFACTOR_LOG.md` records what changed.
