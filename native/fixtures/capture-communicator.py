"""Compile original Communicator handlers/filters with inert Core journal DTOs."""
from pathlib import Path
import re
src=Path('/home/loic/.cache/observatory-readable/communicator/Observatory.Communicator')
s=(src/'Communicator.cs').read_text()
def method(name):
 a=s.index(name);b=s.index('{',a);i=b+1;d=1
 while d:d+=(s[i]=='{')-(s[i]=='}');i+=1
 return s[a:i]
methods='\n'.join(method(x) for x in ['public void JournalEvent<','private void Notify(','private bool ChannelShown(','private bool ChannelNotify(','private static string GetChannelDisplayName('])
program='''using System;using System.Linq;using System.Collections.Generic;using System.Text.Json;using System.Text.Json.Serialization;using Observatory.Framework;using Observatory.Framework.Files.Journal;using Observatory.Framework.Interfaces;
namespace Observatory.Framework.Interfaces{public interface IObservatoryCore{void SendNotification(NotificationArgs args);}}
namespace Observatory.Framework{public enum NotificationRendering{All=7}public class NotificationArgs{public string Title,Detail;public NotificationRendering Rendering=(NotificationRendering)7;}}
namespace Observatory.Framework.Files.Journal{public class JournalBase{[JsonPropertyName("timestamp")]public string Timestamp{get;set;}public DateTime TimestampDateTime=>DateTime.TryParseExact(Timestamp,"yyyy-MM-ddTHH:mm:ssZ",null,System.Globalization.DateTimeStyles.AssumeUniversal,out var date)?date:new DateTime();}public class LoadGame:JournalBase{public string Commander{get;set;}}public class SendText:JournalBase{public string To{get;set;}public string To_Localised{get;set;}public string Message{get;set;}}public class ReceiveText:JournalBase{public string Channel{get;set;}public string From{get;set;}public string From_Localised{get;set;}public string Message{get;set;}public string Message_Localised{get;set;}}}
namespace Observatory.Communicator {public class Communicator{string _currentCommanderName;CommunicatorSettings _settings;IObservatoryCore _core;List<CommunicatorGridItem> _gridItems=new();public Communicator(IObservatoryCore core,CommunicatorSettings settings){_core=core;_settings=settings;}void AddGridItem(CommunicatorGridItem item)=>_gridItems.Add(item);public object[] Rows()=>_gridItems.Where(x=>ChannelShown(x.Channel)).ToArray();METHODS}}
class CoreStub:IObservatoryCore{public List<object> Calls=new();public void SendNotification(NotificationArgs args)=>Calls.Add(new{args.Title,args.Detail,Rendering=(int)args.Rendering});}
class Program{static void Main(){System.Globalization.CultureInfo.CurrentCulture=System.Globalization.CultureInfo.GetCultureInfo("en-US");var opts=new JsonSerializerOptions();using var doc=JsonDocument.Parse(Console.In.ReadToEnd());var result=new List<object>();foreach(var row in doc.RootElement.EnumerateArray()){var core=new CoreStub();var worker=new Observatory.Communicator.Communicator(core,row.GetProperty("Settings").Deserialize<Observatory.Communicator.CommunicatorSettings>());foreach(var step in row.GetProperty("Events").EnumerateArray()){var type=Type.GetType("Observatory.Framework.Files.Journal."+step.GetProperty("event").GetString());if(type!=null)worker.JournalEvent((JournalBase)step.Deserialize(type));}result.Add(new{Rows=worker.Rows(),Calls=core.Calls});}Console.WriteLine(JsonSerializer.Serialize(result));}}
'''.replace('METHODS',methods)
out=Path('/home/loic/.cache/observatory-communicator-oracle');out.mkdir(exist_ok=True)
(out/'Program.cs').write_text(program,encoding='utf-8')
for name in ['CommunicatorSettings','CommunicatorGridItem']:
 text=(src/(name+'.cs')).read_text();text=re.sub(r'^using Observatory.Framework.Attributes;[^\S\n]*\n','',text,flags=re.M);text=re.sub(r'^[^\S\n]*\[Setting[^\n]*\][^\S\n]*\n','',text,flags=re.M).replace('internal class CommunicatorSettings','public class CommunicatorSettings')
 (out/(name+'.cs')).write_text(text,encoding='utf-8')
(out/'oracle.csproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><NoWarn>8632</NoWarn></PropertyGroup></Project>',encoding='utf-8')
