# Verification status

This file distinguishes compilation, automated regression tests, and real terminal validation.

## Required interactive checks

Run the following matrix separately for Terminal.app and iTerm2, using both Bash and Zsh:

- Start a fresh profile without any Q credentials. Complete shell/helper/Accessibility setup.
- Type `git `, `git checkout `, and `cd `; verify suggestions follow the caret.
- Move the selection, accept a suggestion, press Escape, move the cursor, and edit quoted paths.
- Check Unicode text, a multiline prompt, terminal resizing, focus changes, and app restart.
- Disable networking; verify static commands, local files, and Git branches still complete.
- Install twice, inspect startup files for duplicate blocks, uninstall, and confirm unrelated content and existing Fig/Q hooks remain.

Do not mark a scenario passed from a browser mock, PTY test, or successful compilation. Record OS, terminal/shell versions, architecture, observations, and any failed scenarios.

## Local Intel evidence (2026-10-17–18)

Host: macOS 14.8.9, x86_64, Xcode 16.2. Isolated tools: Rust 1.87.0, Node 22.19.0, pnpm 10.0.0, Python 3.11.13.

- Native CLI, desktop, PTY and input-method helper compiled; both UI bundles built.
- Directory-isolation tests passed (15 tests), including the macOS socket-path length limit. The private runtime subdirectory is `tcomp`; data remains under `termcomplete`.
- Six native crate suites passed (70 tests, 10 ignored). Input-method tests requiring OS registration remain ignored.
- JavaScript parser, suggestion and generator suites passed (179 tests). Upstream TODO suites remain explicitly skipped; they are not validation evidence.
- Five added generator regressions cover Unicode output, missing tools, failed/offline generators, timeouts and out-of-order responses.
- Four bundled-catalog tests passed; 718 commands and all four versioned catalogs are present.
- Real Bash and Zsh PTYs passed startup, quoted Vietnamese Unicode, cursor movement, resizing, interrupt and exit checks.
- Fixture-only shell install passed initial/repeated install, existing Q-hook preservation, backup verification, uninstall rollback and typed settings. Actual user dotfiles were not changed by those tests.
- Dependency/bundle audit passed without Q service clients, auth watchers or legacy Q service endpoints.
- Ad hoc signatures verified deeply and strictly before packaging, and again from the mounted development DMG. DMG integrity and SHA-256 checks passed.
- Native fresh-profile startup without AWS/Q environment credentials passed IPC and bare `tcomp` settings-open checks. No process network sockets were observed in the startup snapshot; this is not a full packet trace. Doctor correctly reported missing Accessibility/helper permissions.

## Remaining validation

The Terminal.app/iTerm2 × Bash/Zsh popup matrix above is **pending**. Accessibility and input-method registration require macOS user interaction. Placement, focus, selection, insertion, dismissal and end-to-end offline popup behavior have not been proven by the automated PTY or parser tests.

Apple Silicon compilation and native automated tests are recorded separately in GitHub Actions. A successful CI run does not validate interactive behavior on either architecture. Packages use ad hoc development signing; Developer ID signing, notarization and public releases remain deferred.
