# Release Checklist

TimePrism publishes Windows installers from GitHub Actions. Releases are created as **drafts** first so the generated installers can be smoke-tested before they become public.

## Release artifacts

The Windows release workflow builds both Tauri-supported installer formats:

- NSIS setup executable: `*-setup.exe`
- WiX/MSI installer: `*.msi`

The current project is unsigned. Windows SmartScreen can therefore show an unknown-publisher warning until code signing is added.

## Two-layer installer gate

Installer validation is deliberately split into two layers:

1. **Automated packaging gate:** release-related pull requests build both NSIS and MSI installers on `windows-latest` and upload them as workflow artifacts. This proves the current tree can reach real installer artifacts before merge.
2. **Interactive Windows smoke:** a generated installer is downloaded to a real Windows desktop, installed, launched, exercised, and checked for restart/data/lifecycle behavior. CI cannot replace this interaction-level gate.

A green packaging job is therefore necessary evidence, but it is not sufficient by itself to publish a Release.

## Before tagging

1. Confirm `main` CI is green.
2. Confirm the latest relevant packaging PR produced both installer formats successfully.
3. Update the same semantic version in:
   - `package.json`
   - `src-tauri/Cargo.toml`
   - `src-tauri/tauri.conf.json`
4. Run or verify:

   ```text
   pnpm run typecheck
   pnpm run build:check
   cargo test --manifest-path src-tauri/Cargo.toml
   ```

5. Smoke-test the desktop flows listed in `docs/SMOKE_TESTS.md`.
6. Commit the synchronized version bump to `main`.

The release workflow rejects mismatched Node/Cargo/Tauri versions. Tag-triggered runs also reject a tag that does not equal `v<app-version>`.

## Pull-request packaging validation

When a pull request changes release/package inputs, **Windows Release** runs automatically.

The pull-request path:

1. validates version synchronization;
2. runs frontend typecheck and Rust tests;
3. builds both NSIS and MSI installers on `windows-latest`;
4. uploads both installers as one workflow artifact;
5. does **not** create a GitHub Release.

The packaging job uses `if-no-files-found: error`, so a green upload step also verifies that both configured installer paths resolved to real files.

## Manual dry-run installer build

After the workflow is on `main`, **Windows Release** can also be started manually from GitHub Actions before creating a tag.

The `workflow_dispatch` path performs the same version checks, tests, installer build, and artifact upload without creating a GitHub Release. Use this when you want a fresh installer candidate from the default branch even if no packaging-related pull request is open.

Download the generated artifact and perform the interactive Windows smoke before publishing a version tag.

## Interactive installer smoke

At minimum, verify one generated installer end to end on Windows:

- install from the NSIS `-setup.exe` or MSI artifact;
- launch TimePrism and confirm the main window renders normally;
- verify Home, Insights, Focus Guard, reminders, privacy settings, and desktop pet flows;
- verify main-window close/background behavior and full-app quit behavior;
- restart the app and confirm local data survives;
- uninstall the tested installer and confirm the uninstall flow completes normally;
- retain the unsigned/SmartScreen note until Windows code signing exists.

If the installer or installed application fails this gate, fix the issue and produce a fresh artifact before tagging.

## Publish a release draft

Create and push a tag that exactly matches the app version, for example:

```text
git tag v0.2.0
git push origin v0.2.0
```

The tag-triggered workflow validates the tag/version match and uses `tauri-apps/tauri-action` to create a draft GitHub Release with the NSIS and MSI assets attached.

## Verify the draft

Before publishing the draft Release:

- confirm the expected NSIS and MSI assets are attached;
- install at least one draft asset if it differs from the previously smoke-tested artifact;
- confirm Release notes describe any known limitations;
- explicitly retain the unsigned/SmartScreen note until Windows code signing exists.

Only after these checks should the GitHub Release be published.
