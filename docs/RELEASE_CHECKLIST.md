# Release Checklist

TimePrism publishes Windows installers from GitHub Actions. Releases are created as **drafts** first so the generated installers can be smoke-tested before they become public.

## Release artifacts

The Windows release workflow builds both Tauri-supported installer formats:

- NSIS setup executable: `*-setup.exe`
- WiX/MSI installer: `*.msi`

The current project is unsigned. Windows SmartScreen can therefore show an unknown-publisher warning until code signing is added.

## Before tagging

1. Confirm `main` CI is green.
2. Update the same semantic version in:
   - `package.json`
   - `src-tauri/Cargo.toml`
   - `src-tauri/tauri.conf.json`
3. Run or verify:

   ```text
   pnpm run typecheck
   pnpm run build:check
   cargo test --manifest-path src-tauri/Cargo.toml
   ```

4. Smoke-test the desktop flows listed in `docs/SMOKE_TESTS.md`.
5. Commit the synchronized version bump to `main`.

The release workflow rejects mismatched Node/Cargo/Tauri versions. Tag-triggered runs also reject a tag that does not equal `v<app-version>`.

## Dry-run installer build

Run **Windows Release** manually from GitHub Actions before creating a tag.

The `workflow_dispatch` path:

1. validates version synchronization;
2. runs frontend typecheck and Rust tests;
3. builds both NSIS and MSI installers on `windows-latest`;
4. uploads the installers as workflow artifacts;
5. does **not** create a GitHub Release.

Install and smoke-test at least one generated installer on Windows before publishing a version tag.

## Publish a release draft

Create and push a tag that exactly matches the app version, for example:

```text
git tag v0.2.0
git push origin v0.2.0
```

The tag-triggered workflow validates the tag/version match and uses `tauri-apps/tauri-action` to create a draft GitHub Release with the NSIS and MSI assets attached.

## Verify the draft

Before publishing the draft Release:

- install from the NSIS `-setup.exe` asset;
- install or inspect the `.msi` asset;
- launch TimePrism and confirm the main window renders normally;
- verify Home, Insights, Focus Guard, reminders, privacy settings, and desktop pet flows;
- verify full-app quit behavior;
- confirm local data survives app restart;
- confirm Release notes describe any known limitations;
- explicitly retain the unsigned/SmartScreen note until Windows code signing exists.

Only after this verification should the GitHub Release be published.
