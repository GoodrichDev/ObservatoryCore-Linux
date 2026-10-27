using System;using System.Collections.Generic;using System.Text.Json;using Evaluator;using Observatory.Framework.Files.Journal;using EDJP.Observatory;
class Program {static void Main(){var cases=new List<object>();string[] types={"Earthlike body","Ammonia world","Rocky body","Icy body","High metal content body"};
foreach(var type in types)foreach(var minimum in new[]{0,10000,100000,2000000})foreach(var filter in new[]{UIFilter.Default,UIFilter.MappingOnly,UIFilter.All})foreach(var mapped in new[]{false,true})foreach(var ov in new bool?[]{null,false,true})foreach(var visited in new[]{false,true})foreach(var flag in new[]{"","AlwaysShowAW","AlwaysShowELW","AlwaysShowAtmospherics","AlwaysShowMain"}){
var settings=new StubSettings{MinimumBodyValue=minimum};if(flag!="")typeof(StubSettings).GetProperty(flag).SetValue(settings,true);Evaluator.Evaluator.EvalSettings=settings;
var scan=new Scan{BodyName="Oracle 1",PlanetClass=type,MassEM=1,WasDiscovered=true,WasMapped=true,Landable=true,SurfaceGravity=1,SurfacePressure=0};var extra=new BodyExtra();var body=new Body(scan,true,extra){BodyID=1,IsMapped=mapped,OverrideIsWorthMapping=ov};body.Extra.PluginBodyVisited=visited;body.UpdateValues();
var system=new EDSystem("Oracle",42,DateTime.UnixEpoch,new Logger());system.Bodies.Add(1,body);system.TotalBodies=2;
var capture=new Evaluator.Evaluator{settings=settings,current=system,systems=new(){system},UIFilter=filter};capture.UpdateUI();cases.Add(new{type,minimum,filter=filter.ToString(),mapped,override_mapping=ov,visited,flag,visible=capture.uiHandler.Rows.Count>0,current=body.CurrentValue,potential=body.PotentialMaxValue});
}
Console.Write(JsonSerializer.Serialize(cases));}}
