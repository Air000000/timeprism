# TimePrism Refactor Documentation

This directory is the active refactor guidance set for the `timeprism` codebase.

## Document Precedence

When documents disagree, use this order:

1. `../AGENTS.md`
2. `docs/README.md`
3. `docs/DECISIONS.md`
4. `docs/REFACTOR_MASTER_PLAN.md`
5. Task-specific documents in this directory
6. Older workspace documents under `../Time_Prism` or `../未上传文件`

Older documents are reference material. They do not override this repository's active guidance unless a decision is copied into `docs/DECISIONS.md`.

## Active Documents

| File | Purpose | Update when |
| --- | --- | --- |
| `REFACTOR_MASTER_PLAN.md` | Overall staged refactor plan | Phase order or strategy changes |
| `REFACTOR_PLAYBOOK.md` | Step-by-step operating procedure | Workflow, validation, rollback, or stop rules change |
| `FIRST_REFACTOR_BATCHES.md` | Concrete first implementation batches | The next batch changes or finishes |
| `PHASE_GATES.md` | Per-phase evaluation and test gates | Phase acceptance criteria or validation scope changes |
| `TRACEABILITY.md` | Process records and audit trail rules | Batch IDs, logging, or trace requirements change |
| `CURRENT_SYSTEM_MAP.md` | Current architecture and debt map | Entries, windows, tables, commands, or debt change |
| `API_CONTRACTS.md` | Existing Tauri command contracts | Any command payload/return/caller changes |
| `DATABASE_AND_PRIVACY.md` | Schema, data path, migration, privacy rules | Schema or privacy behavior changes |
| `TESTING_STRATEGY.md` | Automated and manual safety net strategy | Test categories or commands change |
| `SMOKE_TESTS.md` | Manual smoke test checklist | User flows or expected behavior changes |
| `ACCEPTANCE_CHECKLIST.md` | Phase-level acceptance gates | Refactor stage gates change |
| `DECISIONS.md` | Accepted and open architecture/product decisions | Any important decision is made |
| `ADR_TEMPLATE.md` | Template for new decisions | ADR format changes |
| `adr/` | Proposed/accepted detailed decision records | A significant architecture/product decision needs traceable rationale |
| `REFACTOR_LOG.md` | Short work log | Every meaningful refactor batch |

## Automation

GitHub CI lives in `.github/workflows/ci.yml`.

It currently runs on Windows:

- `pnpm run typecheck`
- `pnpm run build:check`
- `cargo check` from `src-tauri`

The CI job is intentionally source-oriented. Full Tauri packaging can be added later when release automation is ready.

## Professional Sources Behind This Set

These docs adapt several established practices:

- Martin Fowler's refactoring guidance: small behavior-preserving transformations.
- Martin Fowler's Branch by Abstraction: gradual large changes through abstraction while the system keeps running.
- Strangler Fig modernization pattern: gradually replace parts of a legacy system instead of a big-bang rewrite.
- Michael Feathers's legacy-code approach: put code under feedback before changing risky areas.
- Architecture Decision Records: record context, decision, and consequences for important choices.
- C4-style architecture documentation: keep a lightweight system/container/component map.
- GitHub branch protection practices: keep important branches protected by checks/review when available.

## Minimum Before Refactor Work

Before starting a code batch:

1. Read `../AGENTS.md`.
2. Read `REFACTOR_PLAYBOOK.md`.
3. Read `PHASE_GATES.md`.
4. Read `TRACEABILITY.md`.
5. Read the relevant section of `FIRST_REFACTOR_BATCHES.md`.
6. Confirm `git status --short --branch`.
7. Identify the validation commands for the batch.

If GitHub checks are enabled for the branch, treat a failing CI run as a phase-gate failure unless the failure is documented as unrelated infrastructure breakage.

## Documentation Hygiene

- Keep docs factual and executable.
- Do not duplicate large code blocks in docs.
- Prefer links to local files and command names.
- Update docs in the same batch as behavior, command, schema, or workflow changes.
- Move obsolete guidance to an "Archived" section instead of silently deleting rationale.
