"""Extract original Botanist event/state/grid/math methods, with inert Core I/O."""
from pathlib import Path
import re
src=Path('/home/loic/.cache/observatory-readable/botanist/Observatory.Botanist')
s=(src/'Botanist.cs').read_text()
def method(name):
 a=s.index(name);b=s.index('{',a);i=b+1;d=1
 while d:d+=(s[i]=='{')-(s[i]=='}');i+=1
 return s[a:i]
prefix=s[s.index('public class Botanist'):s.index('\tpublic AboutInfo AboutInfo')].replace(': IObservatoryWorker, IObservatoryPlugin',': IObservatoryWorker')
methods='\n'.join(method(x) for x in ['public void JournalEvent<','public void StatusChange(','private string GetDistanceText(','private static double CalculateGreatCircleDistance(','private static double ToRadians(','private object GetColonyDistance(','private void MaybeCloseSamplerStatusNotification(','private void UpdateUIGrid('])
methods=re.sub(r'(\w+)\.set_(\w+)\(([^;]*)\);',r'\1.\2 = \3;',methods)
program='''using System;using System.Linq;using System.Collections.Generic;using System.Collections.ObjectModel;using System.Runtime.CompilerServices;using System.Text.Json;using System.Text.Json.Serialization;
using Observatory.Framework;using Observatory.Framework.Files;using Observatory.Framework.Files.Journal;using Observatory.Framework.Files.ParameterTypes;using Observatory.Framework.Interfaces;
namespace Observatory.Framework.Interfaces{public interface IObservatoryWorker{}public interface IObservatoryCore{bool IsLogMonitorBatchReading{get;}Status GetStatus();void CancelNotification(Guid id);void SendNotification(NotificationArgs v);void UpdateNotification(NotificationArgs v);void ClearGrid(IObservatoryWorker worker,object placeholder);void AddGridItems(IObservatoryWorker worker,IEnumerable<object> items);}}
namespace Observatory.Framework{public class PluginUI{}public class AboutInfo{public string ShortName="Botanist";}[Flags]public enum NotificationRendering{NativeVisual=1}public class NotificationArgs{public string Title,Detail,Sender;public int Timeout;public Guid? Guid;public NotificationRendering Rendering;}}
namespace Observatory.Framework.Files {public class Status{public double Latitude{get;set;}public double Longitude{get;set;}public double PlanetRadius{get;set;}}}
namespace Observatory.Framework.Files.ParameterTypes{public enum ScanOrganicType{Log,Sample,Analyse}public class Signal{public string Type{get;set;}public int Count{get;set;}}}
namespace Observatory.Framework.Files.Journal{public class JournalBase{}public class LoadGame:JournalBase{public bool Odyssey{get;set;}}public class SAASignalsFound:JournalBase{public ulong SystemAddress{get;set;}public int BodyID{get;set;}public string BodyName{get;set;}public List<Signal> Signals{get;set;}=new();}public class FSSBodySignals:SAASignalsFound{}public class ScanOrganic:JournalBase{public ulong SystemAddress{get;set;}public int Body{get;set;}public string Species_Localised{get;set;}public string Genus{get;set;}="";public string Genus_Localised{get;set;}public ScanOrganicType ScanType{get;set;}}public class LeaveBody:JournalBase{}public class FSDJump:JournalBase{}public class CarrierJump:FSDJump{}public class Shutdown:JournalBase{}}
namespace Observatory.Botanist {
CLASS
 public AboutInfo AboutInfo=>new();public Botanist(CoreStub core){Core=core;BioPlanets=new();}public void SetSettings(bool enabled,bool sticky){botanistSettings.OverlayEnabled=enabled;botanistSettings.OverlayIsSticky=sticky;}public void Draw()=>UpdateUIGrid();}
public class CoreStub:IObservatoryCore{public bool IsLogMonitorBatchReading{get;set;}public Status Status;public List<object> Rows=new();public List<object> Calls=new();public Status GetStatus()=>Status;public void CancelNotification(Guid id)=>Calls.Add(new{Action="cancel"});public void SendNotification(NotificationArgs v)=>Record("show",v);public void UpdateNotification(NotificationArgs v)=>Record("update",v);void Record(string action,NotificationArgs v)=>Calls.Add(new{Action=action,v.Title,v.Detail,v.Timeout,Rendering=(int)v.Rendering});public void ClearGrid(IObservatoryWorker w,object p)=>Rows.Clear();public void AddGridItems(IObservatoryWorker w,IEnumerable<object> rows)=>Rows.AddRange(rows);}
}
class Program {static void Main(){System.Globalization.CultureInfo.CurrentCulture=System.Globalization.CultureInfo.GetCultureInfo("en-US");var opts=new JsonSerializerOptions{IncludeFields=true};opts.Converters.Add(new JsonStringEnumConverter());using var doc=JsonDocument.Parse(Console.In.ReadToEnd());var result=new List<object>();foreach(var scenario in doc.RootElement.EnumerateArray()){var core=new Observatory.Botanist.CoreStub();var worker=new Observatory.Botanist.Botanist(core);var steps=new List<object>();foreach(var step in scenario.GetProperty("Steps").EnumerateArray()){core.Calls.Clear();string kind=step.GetProperty("event").GetString();if(kind=="Status"){core.Status=step.Deserialize<Status>(opts);worker.StatusChange(core.Status);}else if(kind=="Batch"){core.IsLogMonitorBatchReading=step.GetProperty("Enabled").GetBoolean();if(core.IsLogMonitorBatchReading)core.Rows.Clear();else worker.Draw();}else if(kind=="Settings")worker.SetSettings(step.GetProperty("OverlayEnabled").GetBoolean(),step.GetProperty("OverlayIsSticky").GetBoolean());else{var type=Type.GetType("Observatory.Framework.Files.Journal."+kind);if(type!=null)worker.JournalEvent((JournalBase)step.Deserialize(type,opts));}steps.Add(JsonSerializer.SerializeToElement(new{Rows=core.Rows,Calls=core.Calls},opts));}result.Add(steps);}Console.WriteLine(JsonSerializer.Serialize(result,opts));}}
'''.replace('CLASS',prefix+methods)
out=Path('/home/loic/.cache/observatory-botanist-oracle');out.mkdir(exist_ok=True)
(out/'Program.cs').write_text(program,encoding='utf-8')
for name in ['BotanistSettings','BotanistGrid','BodyAddress','BioPlanetDetail','BioSampleDetail']:
 text=(src/(name+'.cs')).read_text()
 text=re.sub(r'^using Observatory.Framework.Attributes;\s*','',text,flags=re.M)
 text=re.sub(r'^\s*\[(?:Setting|Column)[^\n]*\]\s*','\n',text,flags=re.M)
 (out/(name+'.cs')).write_text(text,encoding='utf-8')
(out/'oracle.csproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><NoWarn>8632;0169;0649</NoWarn></PropertyGroup></Project>',encoding='utf-8')
