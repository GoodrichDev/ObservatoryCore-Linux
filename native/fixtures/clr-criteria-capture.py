"""Build independent source oracle against the *installed* original Framework.

Run under Linux/WSL. This executes only the installed Explorer criteria manager
source, its settings and load exception, NLua, Framework DTOs and synthetic Lua.
No original vendor application entry point or personal criteria is executed.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

HERE = Path(__file__).resolve().parent
CACHE = Path.home() / ".cache/observatory-clr-criteria-independent-oracle"
SDK = Path.home() / ".cache/observatory-dotnet/dotnet"
INSTALLED = Path("/mnt/c/Program Files/Elite Observatory/ObservatoryFramework.dll")
SOURCE = Path.home() / ".cache/observatory-readable/explorer/Observatory.Explorer"
CACHE.mkdir(parents=True, exist_ok=True)
sources = ["CustomCriteriaManager.cs", "ExplorerSettings.cs", "CriteriaLoadException.cs"]
for name in sources:
    shutil.copyfile(SOURCE / name, CACHE / name)
shutil.copyfile(HERE / "clr-criteria-oracle-driver.cs", CACHE / "Program.cs")
shutil.copyfile(INSTALLED, CACHE / "ObservatoryFramework.dll")
(CACHE / "oracle.csproj").write_text('''<Project Sdk="Microsoft.NET.Sdk">
<PropertyGroup><TargetFramework>net8.0</TargetFramework><OutputType>Exe</OutputType><ImplicitUsings>enable</ImplicitUsings><NoWarn>8632;0649</NoWarn></PropertyGroup>
<ItemGroup><PackageReference Include="NLua" Version="1.7.3"/><Reference Include="ObservatoryFramework"><HintPath>ObservatoryFramework.dll</HintPath></Reference></ItemGroup>
</Project>''')
env = os.environ | {"DOTNET_ROOT": str(SDK.parent), "DOTNET_ReadyToRun": "0", "TZ": "UTC"}
subprocess.run([str(SDK), "publish", str(CACHE / "oracle.csproj"), "-c", "Release", "-r", "linux-x64", "--self-contained", "true", "-o", str(CACHE / "publish"), "--nologo", "-v:q"], env=env, check=True)
executable = str(CACHE / "publish/oracle")
inputs = HERE / "clr-criteria-inputs.json"
result = subprocess.run([executable], input=inputs.read_text(), text=True, capture_output=True, env=env)
if result.returncode:
    raise RuntimeError(f"Oracle failed ({result.returncode}): {result.stderr}")
captured = json.loads(result.stdout)
metadata = json.loads(subprocess.check_output([executable, "--metadata"], env=env, text=True))
output = dict(
    provenance=dict(runtime=f"Linux .NET {metadata['runtime']} / NLua 1.7.3 / KeraLua 1.4.1; DOTNET_ReadyToRun=0 for original Windows R2R Framework image",
                    original_framework_identity=metadata["framework"],
                    original_framework_sha256=hashlib.sha256(INSTALLED.read_bytes()).hexdigest(),
                    original_source_sha256={name: hashlib.sha256((SOURCE / name).read_bytes()).hexdigest() for name in sources},
                    driver_sha256=hashlib.sha256((HERE / "clr-criteria-oracle-driver.cs").read_bytes()).hexdigest(),
                    inputs_sha256=hashlib.sha256(inputs.read_bytes()).hexdigest(),
                    scope="Original installed Framework DLL with unchanged decompiled installed Explorer criteria manager, settings and exception; synthetic inputs only"),
    cases=captured)
(HERE / "clr-criteria-oracle.json").write_text(json.dumps(output, indent=2, ensure_ascii=False) + "\n")
for case in captured:
    print(case["id"], json.dumps(case["responses"], ensure_ascii=False))
