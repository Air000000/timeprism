# Refactor Playbook

This playbook defines how to execute refactor work safely.

## Core Rule

Every refactor batch must be one of these:

1. **Move-only**: code moves to new files with no behavior change.
2. **Extract-only**: code is wrapped in smaller functions/modules with no behavior change.
3. **Compatibility-preserving replacement**: old caller shape remains while implementation changes.
4. **Behavior change**: explicit product/API/data change backed by a decision record.

If a batch mixes multiple categories, split it.

## Standard Batch Flow

### 1. Prepare

- Check branch: `git status --short --branch`.
- Confirm whether local uncommitted changes are related.
- Read relevant docs.
- Name the primary domain.
- State non-goals.

### 2. Characterize

Capture current behavior before changing code:

- For backend logic, identify command names and query outputs.
- For frontend UI, identify visible states and event paths.
- For database/privacy code, identify stored fields and privacy mode.
- For pet/window code, identify current window labels, sizes, and event names.

Use `docs/API_CONTRACTS.md` and `docs/SMOKE_TESTS.md` as the baseline.

### 3. Change

Rules:

- Keep the batch small enough to review in one sitting.
- Prefer extracting pure helpers before changing orchestration.
- Preserve Tauri command names unless the batch is explicitly an API migration.
- Preserve database schema unless the batch is explicitly a migration.
- Keep old and new implementations side by side only when there is a clear switch point.

### 4. Validate

Use the smallest meaningful validation set.

Common checks:

```powershell
pnpm.cmd run typecheck
pnpm.cmd run build:check
```

From `src-tauri`:

```powershell
cargo check
```

Remove temporary output after validation:

```powershell
Remove-Item -LiteralPath .\dist-codex-check -Recurse -Force
```

Only run full Tauri builds when packaging, Rust/Tauri integration, or binary behavior changed.

### 5. Record

Update:

- `docs/REFACTOR_LOG.md`
- `docs/TRACEABILITY.md` if batch IDs, file movement, or trace rules change
- `docs/API_CONTRACTS.md` if command shape changed
- `docs/DATABASE_AND_PRIVACY.md` if schema/privacy changed
- `docs/DECISIONS.md` or an ADR if an architectural decision was made

## Definition Of Ready

A refactor batch is ready when:

- Primary domain is named.
- Target files are known.
- Expected command/schema compatibility is known.
- Validation commands are known.
- Rollback point exists through Git.
- Data/privacy impact is understood.

## Definition Of Done

A refactor batch is done when:

- Relevant checks pass.
- No temporary output remains.
- The app remains runnable or the limitation is documented.
- Docs that describe changed contracts are updated.
- Risk and next step are logged.
- The batch can be mapped to a traceability ID or log entry.

## Stop Conditions

Stop and ask before proceeding if:

- A user-data-destructive change appears necessary.
- A schema change is required but no migration plan exists.
- Privacy defaults or title handling would change.
- Existing Tauri command compatibility would break.
- `task_sessions` semantics become relevant.
- A refactor batch would require changing more than one major domain at once.
- Manual validation finds a behavior change that was not intended.

## Branching Guidance

Recommended:

```powershell
git switch main
git pull --ff-only
git tag pre-refactor-YYYY-MM-DD
git switch -c refactor/architecture
```

For side-by-side reference:

```powershell
git worktree add ..\timeprism-reference origin/main
```

Guidelines:

- Keep `main` stable.
- Do refactor work on a named branch.
- Prefer small commits with a single purpose.
- Avoid long-lived unmerged rewrites.
- Use pull requests and branch protection on GitHub when collaborating or when checks are configured.

## Commit Message Pattern

Use concise messages:

```text
refactor(backend-db): move database path helpers
refactor(api): split command types
docs(refactor): add smoke test checklist
test(analytics): characterize business day windows
```

## Batch Size Guide

Prefer:

- 1 domain.
- 1 to 5 files for early extractions.
- No schema change unless the whole batch is about schema.
- No UI redesign inside backend extraction.
- No new feature inside architecture cleanup.

Accept a larger batch only when moving a type requires callers to update imports mechanically.

## Migration Techniques For This Project

Use these patterns:

- **Extract module**: move functions from `lib.rs` while command behavior remains.
- **Thin wrapper**: keep command functions in place but delegate to services.
- **Branch by abstraction**: add a stable interface around API/client/db access before replacing implementation.
- **Strangler-style feature extraction**: move one feature surface at a time while the old root still hosts the rest.
- **Compatibility layer**: keep old command names while the internals change.
