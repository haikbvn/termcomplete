"""Install isolated build tools. Does not edit shell startup files or global defaults."""
import hashlib
import io
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tarfile
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
TOOLS = Path(os.environ.get("TCOMP_TOOLS_DIR", ROOT / ".tools")).resolve()
ARCH = "arm64" if platform.machine() == "arm64" else "x64"
TRIPLE = "aarch64-apple-darwin" if ARCH == "arm64" else "x86_64-apple-darwin"

def fetch(url):
    with urllib.request.urlopen(url, timeout=120) as response:
        return response.read()

def run(*args, env=None):
    subprocess.run([str(a) for a in args], cwd=ROOT, env=env, check=True)

if platform.system() != "Darwin":
    raise SystemExit("This milestone builds macOS only.")
TOOLS.mkdir(parents=True, exist_ok=True)
node_name = f"node-v22.19.0-darwin-{ARCH}"
node_dir = TOOLS / node_name
if not node_dir.exists():
    filename = node_name + ".tar.gz"
    base = "https://nodejs.org/dist/v22.19.0/"
    checksums = fetch(base + "SHASUMS256.txt").decode().splitlines()
    expected = next(line.split()[0] for line in checksums if line.endswith("  " + filename))
    archive = fetch(base + filename)
    if hashlib.sha256(archive).hexdigest() != expected:
        raise SystemExit("Node archive checksum mismatch")
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:gz") as package:
        package.extractall(TOOLS, filter="data")

# Keep protoc cached across Cargo feature/profile builds.
protoc = TOOLS / "protoc/bin/protoc"
if not protoc.exists():
    import zipfile
    protoc_arch = "aarch_64" if ARCH == "arm64" else "x86_64"
    data = fetch(f"https://github.com/protocolbuffers/protobuf/releases/download/v26.1/protoc-26.1-osx-{protoc_arch}.zip")
    expected = "26a29befa8891ecc48809958c909d284f2b9539a2eb47f22cadc631fe6abe8fd" if ARCH == "arm64" else "febd8821c3a2a23f72f4641471e0ab6486f4fb07b68111490a27a31681465b3c"
    if hashlib.sha256(data).hexdigest() != expected:
        raise SystemExit("protoc archive checksum mismatch")
    with zipfile.ZipFile(io.BytesIO(data)) as package:
        package.extractall(TOOLS / "protoc")
    protoc.chmod(0o755)

env = dict(os.environ, RUSTUP_HOME=str(TOOLS / "rustup"), CARGO_HOME=str(TOOLS / "cargo"),
           COREPACK_HOME=str(TOOLS / "corepack"), BUF_CACHE_DIR=str(TOOLS / "buf-cache"),
           UV_PYTHON_INSTALL_DIR=str(TOOLS / "python"), UV_CACHE_DIR=str(TOOLS / "uv-cache"))
env["PATH"] = str(node_dir / "bin") + os.pathsep + env["PATH"]
rustup = shutil.which("rustup")
if not rustup:
    binary = TOOLS / "rustup-init"
    binary.write_bytes(fetch(f"https://static.rust-lang.org/rustup/dist/{TRIPLE}/rustup-init"))
    binary.chmod(0o755)
    run(binary, "-y", "--no-modify-path", "--default-toolchain", "none", env=env)
    rustup = TOOLS / "cargo/bin/rustup"
run(rustup, "toolchain", "install", "1.87.0", "--profile", "minimal", "--component", "rustfmt,clippy", "--no-self-update", env=env)
uv_name = f"uv-{'aarch64' if ARCH == 'arm64' else 'x86_64'}-apple-darwin"
uv_dir = TOOLS / uv_name
if not uv_dir.exists():
    with tarfile.open(fileobj=io.BytesIO(fetch(f"https://github.com/astral-sh/uv/releases/download/0.8.22/{uv_name}.tar.gz")), mode="r:gz") as package:
        package.extractall(TOOLS, filter="data")
run(uv_dir / "uv", "python", "install", "3.11.13", env=env)
run(node_dir / "bin/corepack", "prepare", "pnpm@10.0.0", "--activate", env=env)
(TOOLS / "bin").mkdir(exist_ok=True)
# Explicit shims prevent another global pnpm or Node version from taking over child builds.
for name, command in {
    "pnpm": [node_dir / "bin/node", TOOLS / "corepack/v1/pnpm/10.0.0/bin/pnpm.cjs"],
}.items():
    import shlex
    file = TOOLS / "bin" / name
    file.write_text("#!/bin/sh\nexec " + " ".join(shlex.quote(str(a)) for a in command) + ' "$@"\n')
    file.chmod(0o755)
python = next((TOOLS / "python").glob("cpython-3.11.13-*/bin/python3.11"))
paths = [TOOLS / "bin", TOOLS / "protoc/bin", node_dir / "bin", TOOLS / f"rustup/toolchains/1.87.0-{TRIPLE}/bin", python.parent]
import shlex
exports = {key: env[key] for key in ["RUSTUP_HOME", "CARGO_HOME", "COREPACK_HOME", "BUF_CACHE_DIR"]}
exports["TCOMP_TOOLS_DIR"] = str(TOOLS)
text = "\n".join(f"export {key}={shlex.quote(value)}" for key, value in exports.items())
text += '\nexport PATH=' + shlex.quote(os.pathsep.join(str(p) for p in paths)) + ':"$PATH"\n'
(TOOLS / "env.sh").write_text(text)
env["PATH"] = os.pathsep.join(str(p) for p in paths) + os.pathsep + env["PATH"]
run(TOOLS / "bin/pnpm", "install", "--frozen-lockfile", "--ignore-scripts", "--store-dir", TOOLS / "pnpm-store", env=env)
print(f"Tools ready. Run: source {shlex.quote(str(TOOLS / 'env.sh'))}")
