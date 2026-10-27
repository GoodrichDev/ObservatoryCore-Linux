// Synthetic cases are evaluated by the entire installed CodexChecker source.
// The generation script statically decodes constants and compiles inert DTOs.
using System;using System.Linq;using System.Collections.Generic;using System.Text.Json;
using CodexLib;using CodexLib.SystemDetails;using Observatory.Framework.Files.Journal;using RegionLookup;using System.Reflection;
class Program{
 static object Result(CodexCheckerResult r)=>new{Genus=r.Genus.ToString(),Species=r.Species.ToString(),Value=r.Value,Distance=r.Distance,Type=r.VariantType.ToString(),Variants=r.VariantResult.Select(v=>new{Colour=v.Colour.ToString(),Likelihood=v.Likelihood.ToString(),v.Reason,v.ShowInUI}).ToArray()};
 static List<object> output=new();static CodexChecker checker=new();static int counter;
 static void Evaluate(Dictionary<string,object> scan,double[]position,string star,bool complete,JsonElement? extra=null){
   bool allRegions=extra?.GetProperty("AllRegions").GetBoolean()??false,allSpheres=extra?.GetProperty("AllSpheres").GetBoolean()??false;
   var morePlanets=extra?.GetProperty("ExtraPlanets").EnumerateArray().Select(x=>x.GetString()).ToArray()??Array.Empty<string>();
   var moreStars=extra?.GetProperty("ExtraStars").EnumerateArray().Select(x=>x.GetString()).ToArray()??Array.Empty<string>();
   var distance=extra?.GetProperty("StarDistance").GetDouble()??149_597_870_700d;
   var source=JsonSerializer.Deserialize<Scan>(JsonSerializer.Serialize(scan));var b=new BScanPlanet(source,BscanSource.Journal);
   checker.UpdateCurrentSystem(position[0],position[1],position[2]);checker.IsSystemComplete=complete;checker.SystemStarTypes.Clear();checker.SystemPlanetClasses.Clear();checker.AddSeenPlanetType(source.PlanetClass);
   foreach(var type in morePlanets)checker.AddSeenPlanetType(type);foreach(var type in moreStars)checker.AddSeenStarType(type);
   if(allRegions)typeof(CodexChecker).GetFields(BindingFlags.NonPublic|BindingFlags.Instance).Single(f=>f.FieldType==typeof(List<Region>)).SetValue(checker,Enumerable.Range(1,42).Select(i=>new Region{Id=i,Name="Synthetic"}).ToList());
   if(allSpheres){checker.IsNearNebula=true;checker.IsInBrainTreeSphere=true;typeof(CodexChecker).GetFields(BindingFlags.NonPublic|BindingFlags.Instance).Single(f=>f.FieldType==typeof(HashSet<POISphereType>)).SetValue(checker,Enum.GetValues<POISphereType>().ToHashSet());}
   var radiant=new RadiantSources();
   if(star!=""){
     checker.AddSeenStarType(star);var ss=new Scan{BodyID=0,BodyName="Synthetic A",StarType=star,SurfaceTemperature=5800,Radius=700_000_000,AbsoluteMagnitude=4.83f,Luminosity="V"};
     var st=new RadiantSourceStar(new BScanStar(ss,BscanSource.Journal)){CalculatedDistance=distance,ApparentCalculatedMagnitude=1.0,ComputedFlux=1.0};
     radiant.RadiantStar=st;radiant.NearestStar=st;radiant.SourceStars.Add(st);radiant.RadiantStarDistance=distance;radiant.Method=VariantStarMethod.Good;
   }
   string parentStar=extra.HasValue&&extra.Value.TryGetProperty("ParentStar",out var other)?other.GetString():"";
   double magnitudeDelta=extra.HasValue&&extra.Value.TryGetProperty("MagnitudeDelta",out var md)?md.GetDouble():0;
   if(parentStar!=""){
     checker.AddSeenStarType(parentStar);
     var otherScan=new Scan{BodyID=10,BodyName="Synthetic B",StarType=parentStar,SurfaceTemperature=5800,Radius=(float)(700_000_000*Math.Pow(10,-magnitudeDelta/5)),AbsoluteMagnitude=(float)(4.83+magnitudeDelta),Luminosity="V"};
     radiant.SourceStars.Add(new RadiantSourceStar(new BScanStar(otherScan,BscanSource.Journal)){CalculatedDistance=distance,ApparentCalculatedMagnitude=1.0+magnitudeDelta,ComputedFlux=Math.Pow(10,-magnitudeDelta/2.5)});
   }
   b.RadiantSources=radiant;
   var checkedResults=checker.CheckScan(b,false,true);
   // BioInsights.CheckBody applies this original one-step confidence downgrade
   // after CodexChecker.CheckScan (installed BioInsights.cs, star checks block).
   foreach(var result in checkedResults)
     if(checker.Codex.Genus[result.Genus].Species[result.Species].StarRequirements.Count>0 && !checker.CheckRadiantStarDetails(b,radiant.RadiantStar,result.Genus,result.Species,false))
       foreach(var variant in result.VariantResult) if((int)variant.Likelihood<2) variant.Likelihood=(VariantResultLikelihood)((int)variant.Likelihood+1);
   var results=checkedResults.Select(Result).ToArray();bool parentAmbiguous=(bool)typeof(CodexChecker).GetMethods(BindingFlags.NonPublic|BindingFlags.Instance).Single(m=>m.Name.StartsWith("q_002CTK70") ).Invoke(checker,new object[]{b});
   output.Add(new{Case=counter++,Scan=scan,Position=position,Star=star,Complete=complete,AllRegions=allRegions,AllSpheres=allSpheres,ExtraPlanets=morePlanets,ExtraStars=moreStars,StarDistance=distance,ParentStar=parentStar,MagnitudeDelta=magnitudeDelta,ParentAmbiguous=parentAmbiguous,Target=extra?.GetProperty("Target").GetString()??"",Results=results});
 }
 static void Main(){

 var atmospheres=new[]{"","Ammonia","AmmoniaRich","Argon","ArgonRich","CarbonDioxide","CarbonDioxideRich","Helium","Methane","MethaneRich","Neon","NeonRich","Nitrogen","Oxygen","SulphurDioxide","Water","WaterRich"};
 var planets=new[]{"Rocky body","High metal content body","Rocky ice body","Icy body"};
 var stars=new[]{"G","O","B","A","F","K","M","L","T","TTS","W","D","N","Y","AeBe","H","SupermassiveBlackHole","C",""};
 for(int index=0;index<1800;index++){
   var atmosphere=atmospheres[index%atmospheres.Length];var planet=planets[index/atmospheres.Length%planets.Length];
   var temp=new[]{70d,120d,150d,170d,180d,190d,200d,250d,300d,400d,500d,600d}[index/68%12];
   var pressure=new[]{0d,500d,1500d}[index/816%3];var star=stars[index%stars.Length];
   var gravity=new[]{0.3,0.5,0.6,1.0,2.0}[index/7%5]*9.798;
   var scan=new Dictionary<string,object>{{"event","Scan"},{"timestamp","2026-01-01T00:00:00Z"},{"SystemAddress",42UL},{"BodyID",1},{"BodyName","Synthetic 1"},{"PlanetClass",planet},{"Landable",true},{"AtmosphereType",atmosphere},{"Atmosphere",atmosphere==""?"":"thin "+atmosphere.ToLower()+" atmosphere"},{"SurfaceTemperature",temp},{"SurfacePressure",pressure},{"SurfaceGravity",gravity},{"Radius",1_000_000d},{"DistanceFromArrivalLS",500d},{"SemiMajorAxis",149_597_870_700d},{"Parents",new[]{new Dictionary<string,int>{{"Star",0}}}},{"Volcanism",new[]{"","water geysers","minor water geysers","major silicate vapour geysers","iron magma","nitrogen magma"}[index/13%6]},{"Materials",new[]{new{Name="cadmium",Percent=1d},new{Name="iron",Percent=20d},new{Name="tellurium",Percent=0.1d}}},{"AtmosphereComposition",atmosphere==""?Array.Empty<object>():new object[]{new{Name=atmosphere.Replace("Rich",""),Percent=100d}}},{"MassEM",0.05d},{"TerraformState",""},{"Rings",Array.Empty<object>()}};
   var position=index%3==0?new[]{0d,0d,0d}:index%3==1?new[]{-719.23047,-17.45312,694.55273}:new[]{850.85938,57.59375,5433.48047};
   Evaluate(scan,position,star,index%2==0);

 }
 var file=System.IO.Path.Combine(AppContext.BaseDirectory,"../../../inputs.json");
 foreach(var e in JsonDocument.Parse(System.IO.File.ReadAllText(file)).RootElement.EnumerateArray())Evaluate(JsonSerializer.Deserialize<Dictionary<string,object>>(e.GetProperty("Scan").GetRawText()),e.GetProperty("Position").EnumerateArray().Select(x=>x.GetDouble()).ToArray(),e.GetProperty("Star").GetString(),e.GetProperty("Complete").GetBoolean(),e);
 Console.WriteLine(JsonSerializer.Serialize(output));
 }
}
