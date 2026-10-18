"""Copy original dependency license/notice files into an assembled app."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("destination", type=Path)
parser.add_argument("--target", required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
output = args.destination
output.mkdir(parents=True, exist_ok=True)
patterns = ["LICENSE*", "LICENCE*", "COPYING*", "NOTICE*"]
count = 0

def copy_notices(directory, name):
    global count
    seen = set()
    for pattern in patterns:
        for file in directory.glob(pattern):
            if file.is_file() and file not in seen:
                seen.add(file)
                destination = output / name / file.name
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(file, destination)
                count += 1

metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--format-version", "1", "--filter-platform", args.target], cwd=root))
for package in metadata["packages"]:
    directory = Path(package["manifest_path"]).parent
    copy_notices(directory, f"rust-{package['name']}-{package['version']}")
for store in (root / "node_modules/.pnpm").glob("*/node_modules"):
    for directory in store.iterdir():
        if directory.name.startswith("@"):
            candidates = list(directory.iterdir())
        else:
            candidates = [directory]
        for package in candidates:
            manifest = package / "package.json"
            if package.is_dir() and manifest.exists():
                try:
                    data = json.loads(manifest.read_text())
                    name = f"js-{data['name'].replace('/', '__')}-{data['version']}"
                    copy_notices(package, name)
                except (KeyError, ValueError, OSError):
                    continue
print(f"Retained {count} dependency license/notice files")
