using System;using System.Linq;using System.Collections.Generic;using System.Text.Json;using CodexLib;using BioInsights.SystemDetails;using CodexLib.SystemDetails;using Observatory.Framework.Files.Journal;using Observatory.Framework.Files.ParameterTypes;using EDJP.Observatory;
class Program {
 static void Main() {
 var input=JsonDocument.Parse(System.IO.File.ReadAllText(System.IO.Path.Combine(AppContext.BaseDirectory,"../../../radiant-inputs.json"))).RootElement;
 var output=new List<object>();foreach(var item in input.EnumerateArray()){
 var system=new BSystem(item.TryGetProperty("SystemName",out var name)?name.GetString():"Synthetic",42,new StarPosition{x=0,y=0,z=0},new Logger(),0);
 foreach(var scan in item.GetProperty("Scans").EnumerateArray()){
 if(scan.GetProperty("event").GetString()=="ScanBaryCentre")system.Add(JsonSerializer.Deserialize<ScanBaryCentre>(scan.GetRawText()),BscanSource.Journal);
 else system.Add(JsonSerializer.Deserialize<Scan>(scan.GetRawText()),BscanSource.Journal);
 }
 if(item.TryGetProperty("StarHint",out var hint)&&hint.GetString()!="")system.StarHint=Enum.Parse<VariantStar>(hint.GetString());
 int id=item.GetProperty("BodyID").GetInt32();var body=system.Planets[id];system.EnrichStars(body);var rs=body.RadiantSources;
 var checker=new CodexChecker();checker.UpdateCurrentSystem(0,0,0);checker.IsSystemComplete=true;foreach(var st in system.Stars.Values)checker.AddSeenStarType(st.StarType);foreach(var pl in system.Planets.Values)checker.AddSeenPlanetType(pl.PlanetClass.ToString().Replace('_',' '));
 var checkedResults=checker.CheckScan(body,false,true);
 foreach(var result in checkedResults)if(checker.Codex.Genus[result.Genus].Species[result.Species].StarRequirements.Count>0&&!checker.CheckRadiantStarDetails(body,rs.RadiantStar,result.Genus,result.Species,false))foreach(var variant in result.VariantResult)if((int)variant.Likelihood<2)variant.Likelihood=(VariantResultLikelihood)((int)variant.Likelihood+1);
 var predictions=checkedResults.Select(r=>new{Genus=r.Genus.ToString(),Species=r.Species.ToString(),r.Value,r.Distance,Type=r.VariantType.ToString(),Variants=r.VariantResult.Select(v=>new{Colour=v.Colour.ToString(),Likelihood=v.Likelihood.ToString(),v.Reason,v.ShowInUI}).ToArray()}).ToArray();
 output.Add(new{Predictions=predictions,Input=item,Method=rs.Method.ToString(),Radiant=rs.RadiantStar.BodyID,RadiantType=rs.RadiantStar.VariantStar.ToString(),Distance=rs.RadiantStarDistance,Nearest=rs.NearestStar?.BodyID,NeedScans=rs.NeedScans.Order().ToArray(),Stars=rs.SourceStars.Select(s=>new{s.BodyID,Star=s.VariantStar.ToString(),s.CalculatedDistance,s.Distances,s.ApparentCalculatedMagnitude,s.ComputedFlux,s.BlackBodyEnergy}).ToArray()});
 }
 Console.WriteLine(JsonSerializer.Serialize(output));
 }
}

