"""Compile only the installed Explorer DefaultCriteria source with inert DTOs.

No original assembly is loaded or executed. stdin is a JSON array of independent
{Scan,History,Signals,Settings} cases; stdout preserves the original result order.
"""
from pathlib import Path
import re

source = Path('/home/loic/.cache/observatory-readable/explorer/Observatory.Explorer')
out = Path('/home/loic/.cache/observatory-explorer-criteria-oracle')
out.mkdir(exist_ok=True)
(out/'DefaultCriteria.cs').write_text((source/'DefaultCriteria.cs').read_text(), encoding='utf-8')
settings = (source/'ExplorerSettings.cs').read_text()
properties = re.findall(r'public (bool|string) (\w+)\s*\{', settings)
(out/'Settings.cs').write_text('namespace Observatory.Explorer {public class ExplorerSettings {' + ''.join('public '+t+' '+n+' {get;set;}'+(' = "";' if t=='string' else '') for t,n in properties)+'}}', encoding='utf-8')
(out/'Program.cs').write_text('''using System;using System.Linq;using System.Collections.Generic;using System.Collections.Immutable;using System.Text.Json;using System.Text.Json.Serialization;
namespace Observatory.Framework.Files.ParameterTypes { public enum ParentType{Null,Planet,Star} public class Ring {public string Name{get;set;}="";public float InnerRad{get;set;}public float OuterRad{get;set;}} public class MaterialComposition {public string Name{get;set;}="";}}
namespace Observatory.Framework.Files.Journal {using Observatory.Framework.Files.ParameterTypes;
public class ScanBaryCentre {public ulong SystemAddress{get;set;}public int BodyID{get;set;}public float SemiMajorAxis{get;set;}public float Eccentricity{get;set;}public float OrbitalPeriod{get;set;}}
public class Scan:ScanBaryCentre {public string BodyName{get;set;}=""; public bool Landable{get;set;}public bool WasMapped{get;set;}public bool WasDiscovered{get;set;}public bool TidalLock{get;set;}public string TerraformState{get;set;}="";public string Atmosphere{get;set;}="";public string PlanetClass{get;set;}public string StarType{get;set;}public float Radius{get;set;}public float SurfaceGravity{get;set;}public float RotationPeriod{get;set;}public double DistanceFromArrivalLS{get;set;}public ImmutableList<Ring> Rings{get;set;}public ImmutableList<MaterialComposition> Materials{get;set;}public List<Dictionary<ParentType,int>> Parents{get;set;}[JsonIgnore] public ImmutableList<(ParentType,int)> Parent=>Parents?.SelectMany(x=>x.Select(p=>(p.Key,p.Value))).ToImmutableList();}
public class Signal{public string Type{get;set;}="";public int Count{get;set;}}public class SAASignalsFound{public int BodyID{get;set;}public ImmutableList<Signal> Signals{get;set;}=ImmutableList<Signal>.Empty;}public class FSSBodySignals:SAASignalsFound{} }
class Program { static void Main(){System.Globalization.CultureInfo.CurrentCulture=System.Globalization.CultureInfo.GetCultureInfo("en-US");var opts=new JsonSerializerOptions{IncludeFields=true};opts.Converters.Add(new JsonStringEnumConverter());using var doc=JsonDocument.Parse(Console.In.ReadToEnd());var results=new List<object>();foreach(var row in doc.RootElement.EnumerateArray()){var scan=row.GetProperty("Scan").Deserialize<Observatory.Framework.Files.Journal.Scan>(opts);var history=row.GetProperty("History").Deserialize<List<Observatory.Framework.Files.Journal.Scan>>(opts).ToDictionary(x=>x.BodyID);var signals=row.GetProperty("Signals").Deserialize<List<Observatory.Framework.Files.Journal.FSSBodySignals>>(opts).ToDictionary(x=>x.BodyID);var settings=row.GetProperty("Settings").Deserialize<Observatory.Explorer.ExplorerSettings>();var found=Observatory.Explorer.DefaultCriteria.CheckInterest(scan,new(){{scan.SystemAddress,history}},new(){{scan.SystemAddress,signals}},settings);results.Add(found.Select(x=>new{Title=x.Description,Detail=x.Detail,SystemWide=x.SystemWide}));}Console.WriteLine(JsonSerializer.Serialize(results));}}
''', encoding='utf-8')
(out/'oracle.csproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><NoWarn>8632</NoWarn></PropertyGroup></Project>', encoding='utf-8')
