"""Capture source journal dispatcher with inert downstream processors.
The readable source only expands transparent one-line obfuscation wrappers.
No installed assembly or game file is executed/read by this harness.
"""
from pathlib import Path
import re
source=Path('/home/loic/.cache/observatory-readable/bio-semantic.cs').read_text()
a=source.index('public void processJournalEvent<');b=source.index('{',a);d=1;e=b+1
while d:d+=(source[e]=='{')-(source[e]=='}');e+=1
method=source[a:e]
# The first readable-source helper expander did not parenthesize a string
# inequality below unary not. Correct that textual expansion to !(a != b).
method=method.replace('!(commanderFIDAndSaveDetails) != (currentFID)','!((commanderFIDAndSaveDetails) != (currentFID))')
types=['Commander','LoadGame','Location','FSDJump','CarrierJump','StartJump','Scan','ScanBaryCentre','FSSBodySignals','SAASignalsFound','SAAScanComplete','FSSAllBodiesFound','ApproachBody','SupercruiseExit','SupercruiseEntry','Touchdown','Liftoff','LaunchSRV','DockSRV','Embark','Disembark','ReceiveText','ScanOrganic','CodexEntry','SellOrganicData','Died','DockingRequested','Docked','Shutdown']
classes='\n'.join('public class '+t+':'+{'CarrierJump':'FSDJump','SupercruiseExit':'SupercruiseEntry','Embark':'Disembark'}.get(t,'JournalBase')+' {}' for t in types)
stub=r'''
using System;using System.Linq;using System.Collections.Generic;using System.Diagnostics;using System.Runtime.CompilerServices;using System.Text.Json;
public enum BscanSource{Journal,Hecate,Spansh}public enum LogLevel{Verbose,Debug}
public class JournalEventProperties {public BscanSource Source;public bool IsCurrentCommander=true;}
public class JournalBase {public int BodyID=1;public ulong SystemAddress=42;public string BodyType="Planet",Message_Localised="",Message="",Commander="Source",FID="F123",Ship="test",StarClass="G",JumpType="Hyperspace",StarSystem="Source",Event="test",Json="{}";public bool Odyssey=true,InSRV,OnFoot,SRV,OnStation,OnPlanet=true,Docked;public double Latitude=1,Longitude=2;public double[] StarPos={0,0,0};public long Population;public DateTime TimestampDateTime=DateTime.UnixEpoch;}
CLASSES
public class Dummy {public bool IsLogMonitorBatchReading;public string GetCommanderFIDAndSaveDetails(LoadGame j)=>j.FID;public void WriteLog(string a,LogLevel b){}public void WriteException(string s,Exception e){throw new Exception(s,e);}public void ResetData(int fid){}public void SaveFile(){}public void Save(){}public void SetCacheFilename(string s){}public void SetCommander(string f,string n){}}
public class Bio {
 public List<string> Trace=new();public ulong currentID64=42;public int approachBodyID=1,currentFIDAsInt=123;public bool OdysseyLoaded,isPreReading;public string currentCommander="Source",currentFID="F123",starhint="";
 public string debugevent="";public int jCnt;public bool updateUI,submitErrors;public bool DoPostChecks()=>false;
 public Dummy Core=new(),logger=new(),commanderLookup=new(),UIHandler=new(),distanceNotifier=new(),seenCodexEntries=new(),unsoldDataHandler=new();
 public Dictionary<string,Dictionary<ulong,DateTime>> LastVisited=new();public Stopwatch swatch=new();
 public string DistanceNotifierFilename(string f)=>"";public void LoadPlayerCodexEntries(string f){}public void RequestUnsoldTotal(bool forceUpdate){}
 public void SetCurrentSystem(double[] p,ulong s,string n,DateTime t,long pop,JournalEventProperties e){currentID64=s;}
 public void RadarHide()=>Trace.Add("RadarHide");public void RadarShow(ulong id,int body)=>Trace.Add("RadarShow:"+id+":"+body);
 public void RadarForceHidden(bool val)=>Trace.Add("RadarForceHidden:"+val);public void RadarAddWaypoint()=>Trace.Add("RadarAddWaypoint");
 public void RadarAddVehicle(int t,double? x,double? y)=>Trace.Add("RadarAddVehicle:"+t+":"+x+":"+y);public void RadarRemoveVehicle(int t)=>Trace.Add("RadarRemoveVehicle:"+t);
HANDLERS
METHOD
}
public class Program {public static void Main(){var cases=new List<object>();foreach(var type in new Type[]{TYPES})for(int f=0;f<128;f++){
 var bio=new Bio();var j=(JournalBase)Activator.CreateInstance(type);j.OnPlanet=(f&1)!=0;j.OnStation=(f&2)!=0;j.SRV=(f&4)!=0;j.OnFoot=(f&8)!=0;j.InSRV=(f&16)!=0;
 j.Docked=(f&32)!=0;if((f&64)!=0)bio.currentID64=0;
 j.Message_Localised="CMDR SOURCE points";j.Message="test$HumanoidEmote_point_Action;;";
 bio.processJournalEvent(j,new JournalEventProperties());cases.Add(new{Event=type.Name,Flags=f,Actions=bio.Trace});
 }Console.WriteLine(JsonSerializer.Serialize(cases));}}
'''
handlers='\n'.join('public void '+name+'(JournalBase j,JournalEventProperties p){Trace.Add("'+name+'");}' for name in re.findall(r'\b(handleEvent\w+)\(',method))
handlers+='public void handleEventSAAScanComplete(JournalBase j){Trace.Add("handleEventSAAScanComplete");}'
program=stub.replace('CLASSES',classes).replace('HANDLERS',handlers).replace('METHOD',method).replace('TYPES',','.join('typeof('+t+')'for t in types))
out=Path('/home/loic/.cache/observatory-bio-journal-oracle');out.mkdir(exist_ok=True);(out/'Program.cs').write_text(program)
(out/'oracle.csproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>Exe</OutputType><TargetFramework>net10.0</TargetFramework><NoWarn>0162;0168;0219</NoWarn></PropertyGroup></Project>')
