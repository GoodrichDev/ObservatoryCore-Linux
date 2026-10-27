"""Capture original Core NativeVoice.AddVoiceToSsml, no synthesizer or DLL."""
from pathlib import Path
s=Path('/home/loic/.cache/observatory-readable/core/Observatory.NativeNotification/NativeVoice.cs').read_text();a=s.index('private static string AddVoiceToSsml(');b=s.index('{',a);i=b+1;d=1
while d:d+=(s[i]=='{')-(s[i]=='}');i+=1
method=s[a:i].replace('private static','public static')
program='''using System;using System.Xml;using System.Linq;using System.Collections.Generic;using System.Text.Json;
class Voice {METHOD}
class Program {static void Main(){var rows=new List<object>();foreach(var ssml in new[]{"<speak xmlns='http://www.w3.org/2001/10/synthesis'><voice name='Old'>Hello</voice></speak>","<speak>Hello</speak>","<speak xmlns='http://www.w3.org/2001/10/synthesis'>Hello</speak>","<speak xmlns='http://www.w3.org/2001/10/synthesis'><voice name='First'>A</voice><voice name='Second'>B</voice></speak>","<s:speak xmlns:s='urn:custom'><s:voice name='Old'>A</s:voice></s:speak>","<speak xmlns='http://www.w3.org/2001/10/synthesis'><p><voice name='Nested'>Hello</voice></p></speak>","<speak xmlns='http://www.w3.org/2001/10/synthesis'><voice>Hello</voice></speak>","<speak><voice name='Old'>A &amp; B</voice></speak>","<speak><voice>","not XML"}){try{rows.Add(new{Input=ssml,Voice="en-us",Output=Voice.AddVoiceToSsml(ssml,"en-us"),Error=""});}catch(Exception e){rows.Add(new{Input=ssml,Voice="en-us",Output="",Error=e.GetType().Name});}}Console.WriteLine(JsonSerializer.Serialize(rows));}}
'''.replace('METHOD',method)
out=Path('/home/loic/.cache/observatory-core-ssml-oracle');out.mkdir(exist_ok=True);(out/'Program.cs').write_text(program);(out/'oracle.csproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><NoWarn>8632</NoWarn></PropertyGroup></Project>')
