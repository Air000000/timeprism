# ADR-0002: Idle Prompt Persistence

## Status

Proposed

## Context

Current pending idle prompts are held in process memory.

This means:

- Guard, Home, and Pet can share prompts while the app is running.
- Pending prompts may be lost on restart.

Product docs say idle/AFK confirmation is important for accurate data. Losing prompts may be acceptable in an early prototype, but it is risky for a personal long-term tracker.

## Decision

Proposed decision:

Do not change idle persistence during early architecture extraction.

After backend DB/services are split, add a persisted `idle_prompts` table or equivalent repository if the product confirms pending prompts should survive restart.

## Options Considered

### Option A: Keep In Memory For Early Refactor

Pros:

- Avoids schema change during architecture cleanup.
- Keeps first extraction batches smaller.

Cons:

- Restart can lose important pending prompts.
- Data behavior remains less trustworthy.

### Option B: Persist All Idle Prompts Now

Pros:

- Better data integrity.
- Aligns with long-term product expectations.

Cons:

- Requires schema migration.
- Requires careful duplicate prevention and status semantics.
- Increases risk during initial refactor.

### Option C: Persist Only Long Prompts

Pros:

- Limits noise.
- Captures more important away periods.

Cons:

- Adds threshold semantics that need product design.

## Consequences

Positive:

- Early refactor remains behavior-preserving.
- Persistence can be designed after DB module boundaries exist.

Negative:

- Known data-loss-on-restart risk remains for now.

Follow-up:

- Decide persistence semantics before hardening/data-management phase.
- If accepted, add schema, repository, migration, and smoke tests.

## Validation

During compatibility period:

- Existing idle prompt flows still work in a single app session.
- Docs continue to mark restart persistence as an open risk.

After persistence implementation:

- Restart preserves pending prompts.
- Duplicate prompts are avoided.
- Guard/Home/Pet see the same persisted source.

## Review Date

Before backend idle service extraction or data-management hardening.

