# Refactor Acceptance Checklist

This checklist is for refactor phases, not final product perfection.

## Always Required

- [ ] Frontend typecheck passes.
- [ ] Backend check passes when Rust changes.
- [ ] The main window can still open.
- [ ] Existing Tauri command names remain compatible unless a migration note exists.
- [ ] No user data is deleted or overwritten.
- [ ] Privacy-sensitive fields are not stored raw after privacy settings forbid them.
- [ ] `src/App.vue` does not grow with new feature logic.
- [ ] `src-tauri/src/lib.rs` does not grow with new domain logic.

## Backend Extraction

- [ ] Commands are thin wrappers.
- [ ] SQL has moved toward database/repository/service modules.
- [ ] Windows API calls are isolated from command registration.
- [ ] Foreground capture and idle logic are separate.
- [ ] Migration logic is centralized.
- [ ] Error paths are explicit and user-safe.

## Frontend Extraction

- [ ] New components use typed props or composables.
- [ ] No new `ctx: any` is introduced.
- [ ] Raw `invoke` calls remain centralized through the API layer, except small window-only pet commands.
- [ ] Formatting helpers are not duplicated per feature.
- [ ] Heavy analytics are not loaded on first render unless required by the current view.

## Feature Parity

Home:

- [ ] Today's learn/rest summary still appears.
- [ ] Pending app, idle, and reminder counts still match Guard/reminder state.

Guard:

- [ ] Pending app can be classified.
- [ ] Idle prompt can be resolved or deferred.
- [ ] Rules can be searched and edited.
- [ ] Diagnostics still explain stored/blocked samples.

Reminders:

- [ ] Create/edit/delete works.
- [ ] Done/restore works.
- [ ] Snooze works.
- [ ] One-time/daily/weekly semantics still work.

Insights:

- [ ] Today's top apps works.
- [ ] Historical app usage works.
- [ ] Recent logs works.
- [ ] Heatmap and usage stack still use backend analytics.

Pet:

- [ ] Pet opens quickly.
- [ ] Pet can show due reminders.
- [ ] Pet can resolve idle prompts.
- [ ] Pet can classify pending apps.
- [ ] Pet panel follows the pet and remains lightweight.

## Release Readiness Later

- [ ] Database path is visible.
- [ ] Export or backup exists.
- [ ] Recent errors or diagnostics are visible.
- [ ] Startup path has been reviewed.
- [ ] Product decisions are reflected in UI and docs.

