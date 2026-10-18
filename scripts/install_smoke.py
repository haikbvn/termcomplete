"""Verify install/reinstall/rollback against owned fixtures, never real dotfiles."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--bin", type=Path, required=True)
parser.add_argument("--root", type=Path, required=True)
args = parser.parse_args()
root = args.root.resolve()
if root.exists():
    raise SystemExit("Choose a new fixture directory; existing data is never deleted")
config = root / "config"
config.mkdir(parents=True)
original = "# keep my settings\nexport MY_EXISTING_SETTING=unchanged\n# Amazon Q pre block. Keep at the top of this file.\neval \"$(q init zsh pre)\"\n"
files = [".bashrc", ".bash_profile", ".zshrc", ".zprofile"]
for name in files:
    (config / name).write_text(original)
env = dict(os.environ, TCOMP_DATA_DIR=str(root / "data"), TCOMP_BACKUP_DIR=str(root / "backups"),
           TCOMP_RUNTIME_DIR=str(root / "run"), TCOMP_SHELL_CONFIG_DIR=str(config))
binary = args.bin.resolve()
def run(*command):
    return subprocess.check_output([str(binary), *command], env=env, text=True)
for shell, executable in [("bash", "/bin/bash"), ("zsh", "/bin/zsh")]:
    for when in ["pre", "post"]:
        subprocess.run([executable, "-n"], input=run("init", shell, when), text=True, check=True)
run("install", "--shell-only")
first = {name: (config / name).read_text() for name in files}
for name, text in first.items():
    assert "Termcomplete pre block" in text, (name, text)
    assert "Termcomplete post block" in text, (name, text)
    assert original.strip() in text, (name, "unrelated settings or Q hooks changed")
run("install", "--shell-only")
assert first == {name: (config / name).read_text() for name in files}, "Second install changed startup files"
run("uninstall", "--shell-only")
for name in files:
    restored = (config / name).read_text()
    assert "Termcomplete pre block" not in restored
    assert "Termcomplete post block" not in restored
    assert restored.strip() == original.strip(), (name, restored)
backups = list((root / "backups").rglob(".zshrc"))
assert backups and any(path.read_text() == original for path in backups), "Original backup missing"
# Verify settings retain JSON value types, preserve other keys, and support removal.
run("settings", "autocomplete.disable", "true")
run("settings", "appearance.theme", "dark")
assert run("settings", "autocomplete.disable").strip() == "true"
run("settings", "autocomplete.disable", "--delete")
settings = json.loads(run("settings"))
assert settings["appearance.theme"] == "dark" and "autocomplete.disable" not in settings
print(json.dumps({"root": str(root), "passed": ["install", "repeat install unchanged", "Q hooks preserved", "rollback", "original backup", "typed settings", "Bash/Zsh syntax"]}, indent=2))
