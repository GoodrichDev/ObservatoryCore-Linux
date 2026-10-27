#!/usr/bin/env python3
"""Compile copied pure source theme classes; never load an installed plugin DLL."""
from pathlib import Path
import hashlib
import json
import os
import re
import shutil
import subprocess

root = Path(__file__).resolve().parents[2]
source = Path("/home/loic/.cache/observatory-readable/bio/EDJP.Observatory")
project = Path("/home/loic/.cache/observatory-bio-symbol-css-oracle")
project.mkdir(exist_ok=True)
files = ["Theme.cs", "ThemeColor.cs", "ThemeSettings.cs", "ThemeSymbol.cs", "SymbolContentType.cs", "OutputSymbol.cs"]
hashes = {}
for name in files:
    content = (source / name).read_bytes()
    hashes[name] = hashlib.sha256(content).hexdigest()
    (project / name).write_bytes(content)
utils = (source / "Utils.cs").read_text()
def method(name):
    found=re.search(r"^\t(?:public\s+)?static [^\n]+\b"+re.escape(name)+r"\(",utils,re.M)
    assert found,name
    start=found.start();brace=utils.index("{",start);depth=1;end=brace+1
    while depth:
        if utils[end]=="{":depth+=1
        elif utils[end]=="}":depth-=1
        end+=1
    return utils[start:end]
copied={}
pending=["MungeURL","Base64Decode","Base64Encode"]
while pending:
    name=pending.pop()
    if name in copied:continue
    body=method(name);copied[name]=body
    pending.extend(re.findall(r"\b(_\w+)\(",body))
(project / "Utils.cs").write_text("using System;using System.Text;namespace EDJP.Observatory;static class Utils{\n"+"\n".join(copied.values())+"\n}")
hashes["Utils.cs"]=hashlib.sha256(utils.encode()).hexdigest()
template = Path("/home/loic/.cache/observatory-plugins/bio-project/BioInsights.Resources.symbols.css")
shutil.copyfile(template,root / "native/observatory-plugins/src/bioinsights/symbols-template.css")
(project / "oracle.csproj").write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>Exe</OutputType><TargetFramework>net10.0</TargetFramework><Nullable>enable</Nullable></PropertyGroup></Project>')
driver = r'''using System;using System.IO;using System.Collections.Generic;using System.Text.Json;using EDJP.Observatory;
class Program {static void Main(string[] args){var cases=new List<object>();var template=File.ReadAllText(args[0]);foreach(var file in args[1..])foreach(var height in new[]{18,24,42})foreach(var custom in new[]{false,true}){var theme=Theme.Load(File.ReadAllText(file));theme.Settings.RowHeight=height;if(custom){var s=theme.Symbols["RB"];s.Font="Arial";s.FontSize=19;s.FontWeight="normal";s.PaddingLeft=3;s.PaddingRight=7;s.PaddingTop=2;s.PaddingBottom=4;s.Width=27;s.VerticalAlign="middle";s.SupressMouseOver=true;}cases.Add(new{theme=Path.GetFileName(file),height,custom,css=theme.generateSymbolCSS(".",template,".")});}Console.WriteLine(JsonSerializer.Serialize(new{source="Original Theme.generateSymbolCSS with original theme model/wrappers, no vendor assembly",cases}));}}
'''
(project / "Program.cs").write_text(driver)
env = dict(os.environ,DOTNET_ROOT="/home/loic/.cache/observatory-dotnet",DOTNET_ROLL_FORWARD="Major")
subprocess.run([env["DOTNET_ROOT"]+"/dotnet","build","--nologo","-v:q"],cwd=project,env=env,check=True,stdout=subprocess.DEVNULL)
result = subprocess.check_output([env["DOTNET_ROOT"]+"/dotnet",str(project / "bin/Debug/net10.0/oracle.dll"),str(template),str(root / "native/observatory-plugins/src/bioinsights/theme.json"),str(root / "native/observatory-plugins/src/bioinsights/theme-orange.json")],env=env)
data = json.loads(result)
data["source_sha256"] = hashes
(root / "native/fixtures/bioinsights-symbol-css-oracle.json").write_text(json.dumps(data,indent=2)+"\n")
print(json.dumps({"cases":len(data["cases"]),"original_source_sha256":hashes}))
