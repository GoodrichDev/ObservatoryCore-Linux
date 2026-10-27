from pathlib import Path
import json
cases=[]
def add(label,source):cases.append(dict(Label=label,Source=source))
for prefix in ['::','---@']:
 for sep in ['=',' ']:
  for label in ['Example','  leading and trailing  ','UPPER label','a::b']:
   add(prefix+sep+label,prefix+'Description'+sep+label+'\nscan.Radius < 300000\n'+prefix+'Detail\nstring.format("%.0f km",distanceAsKm(scan.Radius))')
for tag in ['Complex','Criteria','cOmPlEx']:
 add(tag,'---@'+tag+' sample\nreturn true,"Complex",tostring(scan.Radius)\n---@End')
helpers={
 'materials':'local n=0 for m in materials(scan.Materials) do n=n+m.percent end return true,"materials",tostring(n)',
 'rings':'local s="" for r in rings(scan.Rings) do s=s..r.name..":"..r.ringclass..":"..r.massmt..":"..r.innerrad..":"..r.outerrad..";" end return true,"rings",s',
 'ringsOnly':'local n=0 for r in ringsOnly(scan.Rings) do n=n+1 end return true,"ringsOnly",tostring(n)',
 'beltsOnly':'local n=0 for r in beltsOnly(scan.Rings) do n=n+1 end return true,"beltsOnly",tostring(n)',
 'bodies':'local s="" for body in bodies(system) do s=s..body.BodyName..";" end return true,"bodies",s',
 'allparents':'local s="" for p in allparents(parents) do s=s..p.parenttype..":"..p.body..":"..tostring(p.scan and p.scan.BodyName)..";" end return true,"allparents",s',
 'predicates':'return true,"predicates",tostring(isStar(scan))..":"..tostring(isPlanet(scan))..":"..tostring(isBarycentre(scan))..":"..tostring(hasAtmosphere(scan))..":"..tostring(hasLandableAtmosphere(scan))..":"..tostring(hasRings(scan.Rings))..":"..tostring(hasBelts(scan.Rings))',
 'conversions':'return true,"units",distanceAsLs(299792458)..":"..distanceAsKm(1234)..":"..gravityAsG(9.81)..":"..pressureAsAtm(101325)..":"..periodAsDay(86400)..":"..periodAsHour(3600)',
 'signals':'return true,"signals",biosignals..":"..geosignals',
 'stdlib':'return true,"stdlib",type(os.time)..":"..type(io.open)..":"..type(package.path)..":"..type(require)..":"..type(dofile)..":"..type(loadfile)',
 'notify':'notify(scan.BodyName,"Notify","Detail") return true,"Return","Extended"',
 'notifyForBody':'notifyForBody("Test 1","Notify","Detail",2) return false,"Return","Extended"',
 'boolean-return':'return true,false,true',
}
for label,body in helpers.items():add(label,'---@Complex '+label+'\n'+body+'\n---@End')
for handler,arg in [('Jump','jump'),('Discovery','discovery'),('BodySignals','bodySignals'),('AllBodies','allBodies')]:
 field='StarSystem' if handler=='Jump' else 'BodyName' if handler=='BodySignals' else 'SystemName'
 add(handler,'---@'+handler+'\nnotify('+arg+'.'+field+',"'+handler+'",'+arg+'.Event)\n---@End')
add('global-state','---@Global\ncount=41\n---@End\n---@Complex\ncount=count+1 return true,"state",tostring(count)\n---@End')
add('global-after-definitions','---@Complex\nreturn true,"global",tostring(answer)\n---@End\n---@Global\nanswer=42\n---@End')
Path('/home/loic/.cache/observatory-explorer-lua-inputs.json').write_text(json.dumps(cases),encoding='utf-8')
print(len(cases))
