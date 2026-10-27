#!/usr/bin/env python3
"""Record the exact original source inputs linked into the CLR criteria build."""
import hashlib
import json
import pathlib
import sys

root = pathlib.Path(__file__).resolve().parents[2]
files = [
    root / "ObservatoryExplorer" / "CustomCriteriaManager.cs",
    root / "ObservatoryExplorer" / "CriteriaLoadException.cs",
    root / "ObservatoryExplorer" / "ExplorerSettings.cs",
    root / "ObservatoryCore" / "Utils" / "JournalReader.cs",
]
files += [path for path in (root / "ObservatoryFramework").rglob("*.cs")
          if not {"obj", "bin"}.intersection(path.relative_to(root).parts)]
sources = [{"path": path.relative_to(root).as_posix(), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
           for path in sorted(files)]
digest = hashlib.sha256("".join(item["path"] + "\0" + item["sha256"] + "\n" for item in sources).encode()).hexdigest()
manifest = {"schema": 1, "source_tree_sha256": digest, "source_count": len(sources), "sources": sources}
output = json.dumps(manifest, indent=2) + "\n"
if len(sys.argv) > 1:
    pathlib.Path(sys.argv[1]).write_text(output, encoding="utf-8")
else:
    print(output, end="")
