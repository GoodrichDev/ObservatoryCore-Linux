"""Run the original C# criteria compiler against an inert NLua code recorder.

The recorder never executes Lua. Native differential tests execute both the
recorded original helper/functions and the Rust-generated version with Lua5.4.
"""
from pathlib import Path
import shutil
src=Path('/home/loic/.cache/observatory-readable/explorer/Observatory.Explorer')
shared=Path('/home/loic/.cache/observatory-explorer-events-oracle')
out=Path('/home/loic/.cache/observatory-explorer-lua-oracle');out.mkdir(exist_ok=True)
for file in ['Settings.cs','DefaultCriteria.cs','Explorer.cs','Grid.cs']:
 (out/file).write_text((shared/file).read_text(),encoding='utf-8')
text=(shared/'Program.cs').read_text().split('class Program{')[0]
a=text.index('public class CustomCriteriaManager{');b=text.index('public class ExplorerWorker:',a)
text=text[:a]+text[b:]
text=text.replace('public class FSSAllBodiesFound:JournalBase{}','public class FSSAllBodiesFound:JournalBase{public ulong SystemAddress{get;set;}}')
text+='''namespace NLua {public class LuaFunction{public object[] Call(params object[] args)=>Array.Empty<object>();}public class LuaState{public System.Text.Encoding Encoding{get;set;}}public class Lua{public static List<string> Captured=new();public static List<string> Registered=new();public bool UseTraceback{get;set;}public LuaState State=new();public void LoadCLRPackage(){}public object this[string key]=>new LuaFunction();public object[] DoString(string source){Captured.Add(source);return Array.Empty<object>();}public void RegisterFunction(string name,object target,System.Reflection.MethodInfo method)=>Registered.Add(name);}}
namespace NLua.Exceptions{public class LuaScriptException:Exception{}}
class Program{static void Main(){using var doc=JsonDocument.Parse(Console.In.ReadToEnd());var output=new List<object>();int i=0;foreach(var input in doc.RootElement.EnumerateArray()){NLua.Lua.Captured.Clear();NLua.Lua.Registered.Clear();var errors=new List<string>();var path=System.IO.Path.Combine("/home/loic/.cache/observatory-explorer-lua-oracle","input"+(i++)+".lua");System.IO.File.WriteAllText(path,input.GetProperty("Source").GetString());var settings=new Observatory.Explorer.ExplorerSettings{CustomCriteriaFile=path};var manager=new Observatory.Explorer.CustomCriteriaManager(settings,(e,s)=>errors.Add(s),(a,b,c,d,e)=>{});output.Add(new{Chunks=NLua.Lua.Captured.ToArray(),Registered=NLua.Lua.Registered.ToArray(),Errors=errors});}Console.WriteLine(JsonSerializer.Serialize(output));}}
'''
(out/'Program.cs').write_text(text,encoding='utf-8')
(out/'CustomCriteriaManager.cs').write_text((src/'CustomCriteriaManager.cs').read_text(),encoding='utf-8')
(out/'CriteriaLoadException.cs').write_text((src/'CriteriaLoadException.cs').read_text(),encoding='utf-8')
shutil.copy(shared/'oracle.csproj',out/'oracle.csproj')
