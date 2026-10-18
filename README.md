# Termcomplete

Standalone terminal autocomplete for macOS, derived from Amazon Q Developer CLI Autocomplete. There is no account, AI assistant, telemetry emitter, Q spec server, or automatic updater.

This development milestone targets macOS 14 or later on Intel and Apple Silicon, with Bash/Zsh in Terminal.app and iTerm2. Linux, Fish, IDE integrations, SSH integration, and production signing are deferred. Build success does not establish interactive terminal support; see [verification](docs/VERIFICATION.md).

## Build

Xcode and its command-line tools are required. Bootstrap installs Rust 1.87, Node 22.19, pnpm 10, and Python 3.11.13 under `.tools/`, without editing shell startup files or global defaults.

```sh
bash scripts/bootstrap.sh
source .tools/env.sh
python3.11 scripts/standalone.py build
python3.11 scripts/standalone.py test
python3.11 scripts/audit.py
```

The host architecture is selected by default. Pass `--target x86_64-apple-darwin` or `--target aarch64-apple-darwin` for an explicit target with its Rust target installed. GitHub Actions uses matching native runners. Pass `--release` for an optimized build. DMGs and checksums go to `dist/`; `--out /absolute/directory` selects another output directory.

## Install and use

1. Open the DMG and drag `Termcomplete.app` to Applications.
2. Open Termcomplete and use Setup to install shell integration, grant Accessibility permission, and register the cursor-positioning helper.
3. Open a new Terminal.app or iTerm2 window and type `git `.

The app prepares `~/.local/bin/tcomp`, `tcomp-term`, and `tcomp-desktop` links. If that directory is not on PATH, invoke `~/.local/bin/tcomp` directly or add it to PATH yourself. Existing files at those names are preserved.

```sh
tcomp                   # open settings
tcomp doctor            # installation and permission checks
tcomp settings          # list configured settings
tcomp settings autocomplete.disable true
tcomp settings autocomplete.disable --delete
tcomp theme --list
tcomp install --shell-only  # install hooks without registering the input method
tcomp install --input-method-only # register/activate only the cursor helper
tcomp install           # install Bash/Zsh hooks and input method
tcomp uninstall         # remove owned hooks and input-method registration
```

Shell files are backed up before changes under `~/.termcomplete.dotfiles.bak/`. Repeated installs must not add duplicate blocks. Termcomplete owns distinct bundle IDs, data, sockets, and shell variables, and does not migrate an existing Fig/Q installation. Uninstall retains settings, backups, and the app bundle.

## Definitions and networking

718 command definitions are bundled from a pinned public Fig catalog, including versioned specs. AI generators and Q-only specs are omitted. Definitions and common icons load locally. Normal generators execute the relevant CLI and may query services such as npm or an account configured in the AWS CLI. A Q login is never required. Source builds require network access for tools and dependencies.

Data is stored in `~/Library/Application Support/termcomplete`. Absolute `TCOMP_DATA_DIR` and `TCOMP_RUNTIME_DIR` paths override data and IPC directories for isolated diagnostics. `TCOMP_SHELL_CONFIG_DIR` and `TCOMP_BACKUP_DIR` select isolated startup-file and backup directories for install/rollback tests. `tcomp-desktop --no-install` starts without preparing helper links or shell scripts.

## Development packages

Packages are ad hoc signed and are not Apple-notarized releases. macOS may require an explicit user decision to open a downloaded development app. Do not disable Gatekeeper. Accessibility permission may need to be granted again after rebuilding an ad hoc signed binary.

Source revisions are recorded in [PROVENANCE.json](PROVENANCE.json). Original licenses and copyright notices are retained; see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
