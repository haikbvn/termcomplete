"""Build and package Termcomplete without AWS or Tauri release infrastructure."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import plistlib
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
VERSION = "0.1.0"
BINS = ["q_cli", "figterm", "fig_desktop", "fig_input_method"]

def run(*args, cwd=ROOT, env=None):
    print("+", " ".join(str(arg) for arg in args), flush=True)
    subprocess.run([str(arg) for arg in args], cwd=cwd, env=env, check=True)

def assemble(target, release, output):
    profile = "release" if release else "debug"
    binaries = ROOT / "target" / target / profile
    if not binaries.exists():
        binaries = ROOT / "target" / profile
    expected_arch = "arm64" if target == "aarch64-apple-darwin" else "x86_64"
    for binary in ["tcomp", "tcomp-term", "tcomp-desktop", "fig_input_method"]:
        actual_arch = subprocess.check_output(["lipo", "-archs", str(binaries / binary)], text=True).strip()
        if actual_arch != expected_arch:
            raise SystemExit(f"{binary}: expected {expected_arch}, found {actual_arch}; refusing to mislabel the package")
    app = output / "Termcomplete.app"
    if app.exists():
        shutil.rmtree(app)
    contents = app / "Contents"
    macos = contents / "MacOS"
    resources = contents / "Resources"
    helpers = contents / "Helpers"
    for folder in [macos, resources, helpers]:
        folder.mkdir(parents=True)
    for binary in ["tcomp", "tcomp-term", "tcomp-desktop"]:
        shutil.copy2(binaries / binary, macos / binary)
    for source, destination in [("packages/autocomplete-app/dist", "autocomplete"),
                                ("packages/dashboard-app/dist", "dashboard"),
                                ("build/specs", "specs"), ("build/themes", "themes")]:
        shutil.copytree(ROOT / source, resources / destination)
    for name in ["LICENSE.MIT", "LICENSE.APACHE", "THIRD_PARTY_NOTICES.md", "PROVENANCE.json"]:
        shutil.copy2(ROOT / name, resources / name)
    source_commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    source_dirty = bool(subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True).strip())
    manifest = {"managed_by": "none", "target_triple": target, "variant": "full", "default_channel": "stable",
                "source_commit": source_commit, "source_dirty": source_dirty,
                "packaged_at": datetime.datetime.now(datetime.timezone.utc).isoformat(), "packaged_by": "Termcomplete"}
    (resources / "manifest.json").write_text(json.dumps(manifest))
    info = {"CFBundleName": "Termcomplete", "CFBundleDisplayName": "Termcomplete", "CFBundleIdentifier": "io.github.haikbvn.termcomplete",
            "CFBundleExecutable": "tcomp-desktop", "CFBundlePackageType": "APPL", "CFBundleVersion": VERSION,
            "CFBundleShortVersionString": VERSION, "LSMinimumSystemVersion": "14.0", "LSUIElement": True,
            "CFBundleIconFile": "Termcomplete.icns", "NSHighResolutionCapable": True,
            "CFBundleURLTypes": [{"CFBundleURLName": "Termcomplete settings", "CFBundleURLSchemes": ["tcomp"]}]}
    (contents / "Info.plist").write_bytes(plistlib.dumps(info))
    icon = ROOT / "build/Termcomplete.icns"
    if not icon.exists():
        run("swift", ROOT / "scripts/icon.swift", ROOT / "build", env=dict(os.environ, SWIFT_MODULECACHE_PATH=str(ROOT / "build/swift-cache"), CLANG_MODULE_CACHE_PATH=str(ROOT / "build/clang-cache")))
        run("iconutil", "-c", "icns", ROOT / "build/Termcomplete.iconset", "-o", icon)
    shutil.copy2(icon, resources / icon.name)
    ime = helpers / "TermcompleteInputMethod.app" / "Contents"
    (ime / "MacOS").mkdir(parents=True)
    (ime / "Resources").mkdir()
    shutil.copy2(binaries / "fig_input_method", ime / "MacOS/fig_input_method")
    ime_info = plistlib.loads((ROOT / "crates/fig_input_method/Info.plist").read_bytes())
    ime_info["LSMinimumSystemVersion"] = "14.0"
    (ime / "Info.plist").write_bytes(plistlib.dumps(ime_info))
    shutil.copy2(icon, ime / "Resources/AppIcon.icns")
    import sys
    run(sys.executable, ROOT / "scripts/licenses.py", resources / "licenses", "--target", target)
    # Remove copied Finder/resource-fork metadata from this freshly assembled bundle.
    run("xattr", "-cr", app)
    # Sign nested executables before their containing bundles, with an ad hoc identity.
    for binary in macos.iterdir():
        if binary.name != "tcomp-desktop":
            run("codesign", "--force", "--sign", "-", binary)
    run("codesign", "--force", "--sign", "-", ime.parent)
    run("codesign", "--force", "--sign", "-", app)
    run("codesign", "--verify", "--deep", "--strict", app)
    return app

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("action", choices=["build", "package", "test"])
parser.add_argument("--target", choices=["x86_64-apple-darwin", "aarch64-apple-darwin"], default="aarch64-apple-darwin" if platform.machine() == "arm64" else "x86_64-apple-darwin")
parser.add_argument("--release", action="store_true")
parser.add_argument("--out", type=Path, default=ROOT / "dist")
args = parser.parse_args()
if platform.system() != "Darwin":
    raise SystemExit("This milestone builds macOS only")
env = dict(os.environ, MACOSX_DEPLOYMENT_TARGET="14.0", CARGO_BUILD_JOBS=os.environ.get("CARGO_BUILD_JOBS", "2"), CARGO_INCREMENTAL="0", SWIFT_MODULECACHE_PATH=str(ROOT / "build/swift-cache"), CLANG_MODULE_CACHE_PATH=str(ROOT / "build/clang-cache"))
packages = [arg for package in BINS for arg in ["-p", package]]
if args.action == "test":
    env["TCOMP_BUILD_SKIP_FISH_TESTS"] = "1"
    if shutil.which("shellcheck") is None:
        env["TCOMP_BUILD_SKIP_SHELLCHECK_TESTS"] = "1"
        print("ShellCheck is unavailable; upstream ShellCheck fixtures are skipped.")
    run("cargo", "test", "--locked", "--target", args.target, *packages, "-p", "fig_integrations", "-p", "fig_ipc", env=env)
    run("cargo", "test", "--locked", "--target", args.target, "-p", "fig_util", "directories::", env=env)
    run("pnpm", "exec", "vitest", "run", "--maxWorkers=2", "--minWorkers=1", env=env)
    run("node", "--test", "scripts/specs.test.mjs", env=env)
    import sys
    profile = "release" if args.release else "debug"
    binary_dir = ROOT / "target" / args.target / profile
    run(sys.executable, ROOT / "scripts/pty_smoke.py", "--bin", binary_dir / "tcomp-term", env=env)
    run(sys.executable, ROOT / "scripts/install_smoke.py", "--bin", binary_dir / "tcomp", "--root", ROOT / "build" / ("install-smoke-" + datetime.datetime.now().strftime("%Y%m%d%H%M%S%f")), env=env)
else:
    if args.action == "build":
        if shutil.disk_usage(ROOT).free < 5 * 1024**3:
            raise SystemExit("At least 5 GiB free space is required before compiling; no files were deleted")
        run("pnpm", "build", env=env)
        run("pnpm", "build:specs", env=env)
        run("cargo", "build", "--locked", "--target", args.target, *(["--release"] if args.release else []), *packages, env=env)
    output = args.out.resolve()
    output.mkdir(parents=True, exist_ok=True)
    import tempfile
    # Sign outside File Provider / synced Documents directories, which can add
    # Finder metadata while codesign is sealing the bundle.
    with tempfile.TemporaryDirectory(prefix="tcomp-package-", dir="/private/tmp") as temporary:
        temporary = Path(temporary)
        app = assemble(args.target, args.release, temporary / "app")
        staging = temporary / "dmg-stage"
        staging.mkdir()
        shutil.copytree(app, staging / app.name, symlinks=True)
        (staging / "Applications").symlink_to("/Applications")
        dmg = output / f"Termcomplete-{VERSION}-{args.target}.dmg"
        dmg.unlink(missing_ok=True)
        run("hdiutil", "create", "-volname", "Termcomplete", "-srcfolder", staging, "-ov", "-format", "UDZO", dmg)
        run("hdiutil", "verify", dmg)
        (output / (dmg.name + ".sha256")).write_text(hashlib.sha256(dmg.read_bytes()).hexdigest() + "  " + dmg.name + "\n")
        print(f"Packaged {dmg}")
