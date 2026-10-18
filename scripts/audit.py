"""Fail if an active binary depends on the removed Q service clients."""
import json
import subprocess
import platform
from pathlib import Path

root = Path(__file__).resolve().parents[1]
metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--format-version", "1", "--filter-platform", "aarch64-apple-darwin" if platform.machine() == "arm64" else "x86_64-apple-darwin"], cwd=root))
packages = {item["id"]: item["name"] for item in metadata["packages"]}
nodes = {item["id"]: item["dependencies"] for item in metadata["resolve"]["nodes"]}
starts = [key for key, value in packages.items() if value in {"fig_desktop", "figterm", "q_cli", "fig_input_method"}]
seen = set()
while starts:
    item = starts.pop()
    if item not in seen:
        seen.add(item)
        starts.extend(nodes[item])
banned = {"fig_auth", "fig_api_client", "fig_install", "fig_telemetry", "fig_telemetry_core", "fig_aws_common", "aws-config", "aws-types"}
violations = [packages[item] for item in seen if packages[item] in banned or packages[item].startswith("amzn-")]
if violations:
    raise SystemExit(f"Removed service clients remain reachable: {violations}")
for file in (root / "crates/fig_desktop/src").rglob("*.rs"):
    source = file.read_text()
    if "midway_cookie_path" in source or "subscribe_midway" in source:
        raise SystemExit(f"AWS authentication watcher remains in {file}")
for name in ["autocomplete-app", "dashboard-app"]:
    for file in (root / "packages" / name / "dist").rglob("*.js"):
        source = file.read_text()
        for endpoint in ["specs.q.us-east-1.amazonaws.com", "jupiter.ai.aws.dev", "desktop-release.q.us-east-1.amazonaws.com", "client-telemetry.us-east-1.amazonaws.com"]:
            if endpoint in source:
                raise SystemExit(f"Q endpoint remains in {file}: {endpoint}")
print(f"Runtime boundary passed: {len(seen)} reachable packages; no Q service clients or legacy Q endpoints.")
