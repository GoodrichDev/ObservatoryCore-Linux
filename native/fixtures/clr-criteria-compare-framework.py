"""Compare all public Framework types and members to the original installed DLL."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import shutil

HERE = Path(__file__).resolve().parent
CACHE = Path.home() / ".cache/observatory-clr-criteria-framework-api"
SDK = Path.home() / ".cache/observatory-dotnet/dotnet"
ORIGINAL = Path("/mnt/c/Program Files/Elite Observatory/ObservatoryFramework.dll")
REBUILT = Path.home() / ".cache/observatory-clr-criteria-net8/ObservatoryFramework.dll"
CACHE.mkdir(parents=True, exist_ok=True)
shutil.copyfile(HERE / "clr-criteria-framework-api.cs", CACHE / "Program.cs")
(CACHE / "api.csproj").write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><ImplicitUsings>enable</ImplicitUsings></PropertyGroup></Project>')
env = os.environ | {"DOTNET_ROOT": str(SDK.parent), "DOTNET_ReadyToRun": "0"}
subprocess.run([str(SDK), "build", str(CACHE / "api.csproj"), "-c", "Release", "--nologo", "-v:q"], check=True, env=env)
def inventory(path):
    return json.loads(subprocess.check_output([str(SDK), str(CACHE / "bin/Release/net10.0/api.dll"), str(path)], env=env, text=True))
original, rebuilt = inventory(ORIGINAL), inventory(REBUILT)
left, right = original["types"], rebuilt["types"]
changes = []
for name in sorted(left.keys() & right.keys()):
    if left[name] != right[name]:
        a, b = set(left[name].splitlines()), set(right[name].splitlines())
        changes.append(dict(type=name, removed=sorted(a - b), added=sorted(b - a)))
result = dict(original_identity=original["identity"], rebuilt_identity=rebuilt["identity"],
              original_sha256=hashlib.sha256(ORIGINAL.read_bytes()).hexdigest(),
              original_types=len(left), rebuilt_types=len(right),
              missing_types=sorted(left.keys() - right.keys()), added_types=sorted(right.keys() - left.keys()), changed_types=changes,
              scope="All exported public types, declared public member signatures, type flags, base types, interfaces, constants, getter/setter visibility, parameter names/flags/defaults, and non-compiler type/member attributes; metadata reflection only")
(HERE / "clr-criteria-framework-api.json").write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps(result, indent=2))
