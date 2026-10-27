#!/usr/bin/env python3
"""Build/package the four independently installable Rust worker artifacts.

The artifact URL is explicit: this creates a feed for the chosen release
location; it neither publishes files nor invents a public update endpoint.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile
from urllib.parse import urlsplit

PLUGINS = ("BioInsights", "Evaluator", "Hecate", "StellarOverlay")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, help="Already built native worker ELF")
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--base-url", required=True, help="HTTPS release directory, or loopback HTTP for QA")
    parser.add_argument("--revision", type=int, default=1)
    parser.add_argument("--channel", choices=("stable", "beta"), default="stable")
    parser.add_argument("--core-min", default="2.0.0")
    parser.add_argument("--plugin", choices=PLUGINS, action="append")
    args = parser.parse_args()
    if sys.platform != "linux":
        parser.error("Native worker packaging/metadata validation must run on Linux")
    url = urlsplit(args.base_url)
    if url.scheme not in ("https", "http") or not url.hostname or url.username or url.password or url.query or url.fragment:
        parser.error("--base-url must be a release directory URL without credentials/query/fragment")
    if url.scheme == "http" and url.hostname not in ("localhost", "127.0.0.1", "::1"):
        parser.error("HTTP is supported only for an explicit loopback QA feed")
    if args.revision < 1:
        parser.error("--revision must be positive")
    root = Path(__file__).resolve().parents[2]
    binary = args.binary
    if binary is None:
        subprocess.run(["cargo", "build", "--release", "--locked", "-p", "observatory-plugins", "--bin", "observatory-native-worker"], cwd=root, check=True)
        binary = Path(os.environ.get("CARGO_TARGET_DIR", root / "target")) / "release/observatory-native-worker"
    binary = binary.resolve(strict=True)
    with binary.open("rb") as stream:
        header = stream.read(64)
    if len(header) != 64 or header[:6] != b"\x7fELF\x02\x01" or struct.unpack_from("<H", header, 16)[0] not in (2, 3):
        parser.error("--binary must be a native 64-bit little-endian Linux executable")
    args.out.mkdir(parents=True, exist_ok=True)
    manifest = {"schema": 1, "core_min": args.core_min, "plugins": []}
    with tempfile.TemporaryDirectory(prefix=".native-workers-", dir=args.out) as temporary:
        stage = Path(temporary)
        for plugin in dict.fromkeys(args.plugin or PLUGINS):
            info = json.loads(subprocess.check_output([str(binary), "--kind", plugin, "--hello"], timeout=10))
            if info["protocol"] != 1 or info["plugin"] != plugin:
                raise RuntimeError(f"Incompatible worker metadata for {plugin}")
            platform = info["platform"]
            if manifest.setdefault("platform", platform) != platform:
                raise RuntimeError("Worker artifacts must target one platform")
            name = f"{plugin.lower()}-worker-{info['source_version']}-r{args.revision}-{platform}"
            artifact = stage / name
            shutil.copyfile(binary, artifact)
            artifact.chmod(0o755)
            manifest["plugins"].append({
                "plugin": plugin, "version": info["source_version"], "revision": args.revision,
                "channel": args.channel, "protocol": info["protocol"],
                "url": f"{args.base_url.rstrip('/')}/{name}",
                "sha256": hashlib.sha256(artifact.read_bytes()).hexdigest(), "size": artifact.stat().st_size,
            })
        (stage / "native-plugins.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
        hashes = [f"{item['sha256']}  {Path(urlsplit(item['url']).path).name}" for item in manifest["plugins"]]
        (stage / "NATIVE-SHA256SUMS").write_text("\n".join(hashes) + "\n", encoding="utf-8")
        # The feed is replaced last; clients cannot discover an absent artifact
        # during a local release-directory update.
        for artifact in stage.iterdir():
            if artifact.name != "native-plugins.json":
                os.replace(artifact, args.out / artifact.name)
        os.replace(stage / "native-plugins.json", args.out / "native-plugins.json")
    print(json.dumps({"manifest": str((args.out / "native-plugins.json").resolve()), "plugins": [item["plugin"] for item in manifest["plugins"]]}))


if __name__ == "__main__":
    main()
