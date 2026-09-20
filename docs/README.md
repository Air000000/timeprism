# TimePrism Engineering Documentation

This directory contains the engineering reference for the current TimePrism codebase. The v0.1.0 release is complete; documents created during the refactor are retained as engineering history, while current architecture, data, testing, and release documents should be treated as the primary reference.

## Start Here

| Document | Use it for |
| --- | --- |
| [CURRENT_SYSTEM_MAP.md](CURRENT_SYSTEM_MAP.md) | Current application surfaces, backend modules, data flow, commands, and known boundaries |
| [API_CONTRACTS.md](API_CONTRACTS.md) | Tauri command payload/return contracts and frontend callers |
| [DATABASE_AND_PRIVACY.md](DATABASE_AND_PRIVACY.md) | SQLite schema/lifecycle, migrations, capture policy, and privacy behavior |
| [DECISIONS.md](DECISIONS.md) | Accepted/open architecture and product decisions |

## Quality and Release Evidence

| Document | Purpose |
| --- | --- |
| [TESTING_STRATEGY.md](TESTING_STRATEGY.md) | Static, unit, integration/contract, DB, and manual validation strategy |
| [SMOKE_TESTS.md](SMOKE_TESTS.md) | Manual Windows smoke catalog for product/runtime changes |
| [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) | Draft-first Windows release gate and installer acceptance |
| [media/README.md](media/README.md) | Privacy-safe capture guide for README/Release screenshots and GIFs |

### Current automation

<code>.github/workflows/ci.yml</code> runs the main Windows CI path:

~~~text
pnpm run typecheck
pnpm run build:check
cargo check --all-targets
cargo test
~~~

<code>.github/workflows/release.yml</code> validates synchronized versions and Windows packaging. Release-related pull requests can build NSIS/MSI validation artifacts without publishing; a matching version tag creates a draft GitHub Release for interactive installer smoke before publication.

## Architecture Decisions

Detailed ADRs live in [adr/](adr/). Use [ADR_TEMPLATE.md](ADR_TEMPLATE.md) when a new decision needs durable context, alternatives, and consequences.

A decision belongs in an ADR when it changes a durable boundary such as persistence semantics, privacy policy, window/runtime behavior, or release architecture. Routine implementation details do not need an ADR.

## Engineering History

The following documents describe the staged hardening/refactor that led to v0.1.0. They are useful for traceability, but they are not the first place to learn the current system.

| Document | Historical role |
| --- | --- |
| [REFACTOR_MASTER_PLAN.md](REFACTOR_MASTER_PLAN.md) | Overall staged refactor plan |
| [REFACTOR_PLAYBOOK.md](REFACTOR_PLAYBOOK.md) | Operating procedure, validation, rollback, and stop rules |
| [FIRST_REFACTOR_BATCHES.md](FIRST_REFACTOR_BATCHES.md) | Initial implementation batches |
| [PHASE_GATES.md](PHASE_GATES.md) | Refactor phase acceptance gates |
| [TRACEABILITY.md](TRACEABILITY.md) | Batch/process traceability rules |
| [ACCEPTANCE_CHECKLIST.md](ACCEPTANCE_CHECKLIST.md) | Historical phase acceptance checklist |
| [REFACTOR_LOG.md](REFACTOR_LOG.md) | Detailed chronological work log |

These files are retained because they show how risky desktop/runtime changes were decomposed, validated, and promoted rather than rewritten in one step.

## Document Precedence

For the current repository, use this order when documents disagree:

1. <code>../AGENTS.md</code>
2. current source code and automated tests on <code>main</code>
3. this index
4. <code>CURRENT_SYSTEM_MAP.md</code>, <code>API_CONTRACTS.md</code>, <code>DATABASE_AND_PRIVACY.md</code>, and <code>DECISIONS.md</code>
5. task-specific quality/release documents
6. historical refactor documents

Historical plans should not override current implementation evidence.

## Documentation Hygiene

- Keep current-state docs factual and executable.
- Update behavior, contract, schema, privacy, test, and release docs in the same change that modifies those boundaries.
- Prefer links to source files and command names over copied implementation blocks.
- Keep synthetic/test data separate from real user data.
- Treat screenshots and GIFs as product evidence: capture the real application and pass the media privacy gate before committing them.
