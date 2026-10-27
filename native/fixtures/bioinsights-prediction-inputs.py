"""Generate catalogue-guided positive and boundary inputs, never expectations."""
import copy,json,re,itertools
from pathlib import Path
def generate():
    codex=json.loads((Path(__file__).resolve().parents[1]/'observatory-plugins/src/bioinsights/codex.json').read_text())
    def enum(file):return re.findall(r'^\s*([A-Za-z][A-Za-z0-9_]*)\s*(?:=\s*[^,]+)?\s*,?$',(Path('/home/loic/.cache/observatory-plugins/bio-codex/CodexLib')/file).read_text(),re.M)
    atmos=enum('AtmosphereType.cs');planets=[s.replace('_',' ')for s in enum('PlanetClass.cs')];stars=enum('VariantStar.cs');chem=enum('AtmosphereComposition.cs')
    volcanos=['']+[a+b for a in ['','minor ','major ']for b in ['water geysers','metallic magma','silicate vapour geysers','silicate magma','rocky magma','nitrogen magma','ammonia magma','methane magma','carbon dioxide geysers']]
    def volcano(s,v):
        v=v.upper()
        return {'ANY':bool(v),'NONE':not v,'MAJOR':v.startswith('MAJOR '),'MINOR':v.startswith('MINOR '),'REGULAR':bool(v)and not v.startswith(('MAJOR ','MINOR ')),'NOT_MAJOR':bool(v)and not v.startswith('MAJOR '),'NOT_MINOR':bool(v)and not v.startswith('MINOR '),'IRON_BASED':'METALLIC 'in v,'ROCKY':'ROCKY'in v}.get(s,s.replace('_',' ').replace(' BASED','')in v)
    def apply(state,p):
        t=p['Type'];i=p.get('IValue',0);d=p.get('DValue',0);s=p.get('SValue','')
        if t in [1,15,3]:
            key={1:'Atmos',15:'Planet',3:'Star'}[t];value=s if t==3 else i
            if key in state and state[key]!=value:return False
            state[key]=value
        elif t in [7,8,9,10,11,12,19,21,31,32]:
            key={7:'Pressure',8:'Pressure',9:'Gravity',10:'Gravity',11:'Temperature',12:'Temperature',19:'Orbit',21:'Arrival',31:'StarDistance',32:'StarDistance'}[t]
            lo,hi=state.setdefault(key,[0.0,{'Pressure':1.0,'Gravity':4.0,'Temperature':1200.0,'Orbit':20.0,'StarDistance':20.0,'Arrival':20000.0}[key]])
            if t in [7,9,11,21,32]:lo=max(lo,d)
            else:hi=min(hi,d)
            if hi<=lo:return False
            state[key]=[lo,hi]
        elif t==2:state.setdefault('Composition',{})[i.bit_length()-1]=max(d,state.get('Composition',{}).get(i.bit_length()-1,0))
        elif t==5:
            state['Volcanos']=[v for v in state.get('Volcanos',volcanos)if volcano(s,v)]
            if not state['Volcanos']:return False
        elif t==17:state.setdefault('ExtraPlanets',set()).add(i)
        elif t==18:state.setdefault('ExtraStars',set()).add(i)
        elif t==33:state.setdefault('NotAtmos',set()).add(i)
        elif t not in [0,16,43]:raise ValueError(t)
        return state.get('Atmos')not in state.get('NotAtmos',set())
    def solve(requirements,state,index=0):
        if index==len(requirements):return state
        for conjunction in requirements[index]['Value']:
            candidate=copy.deepcopy(state)
            if all(apply(candidate,p)for p in conjunction):
                result=solve(requirements,candidate,index+1)
                if result:return result
        return None
    output=[]
    for g,genus in codex['Genus'].items():
      for species,data in genus['Species'].items():
        requirements=genus['Requirements']+data['Requirements']+data['PostRequirements']+data['StarRequirements'];state=solve(requirements,{})
        if state is None:continue
        choose=lambda k,default:sum(state[k])/2 if k in state else default
        at=state.get('Atmos',5);a=atmos[at];st=state.get('Star',stars[next((v['Star']for v in data['Variant']if 'Star'in v),5)]);st={'AEBE':'AeBe','UNKNOWN':''}.get(st,st)
        pressure=choose('Pressure',0.01)*101325;gravity=choose('Gravity',0.1)*9.798;temp=choose('Temperature',170);orbit=choose('Orbit',1.0)*149597870700;dist=choose('StarDistance',max(orbit/149597870700,1.0))*149597870700
        composition=[{'Name':chem[i],'Percent':value}for i,value in state.get('Composition',{}).items()]
        if not composition and a!='None':composition=[{'Name':a.replace('Rich',''),'Percent':100}]
        scan={'event':'Scan','timestamp':'2026-01-01T00:00:00Z','SystemAddress':42,'BodyID':1,'BodyName':'Synthetic 1','PlanetClass':planets[state.get('Planet',9)],'Landable':True,'AtmosphereType':''if a=='None'else a,'Atmosphere':''if a=='None'else'thin '+a.lower()+' atmosphere','SurfaceTemperature':temp,'SurfacePressure':pressure,'SurfaceGravity':gravity,'Radius':1_000_000,'DistanceFromArrivalLS':choose('Arrival',500),'SemiMajorAxis':orbit,'Parents':[{'Star':0}],'Volcanism':state.get('Volcanos',[''])[0],'Materials':[{'Name':m,'Percent':1}for m in ['cadmium','mercury','molybdenum','niobium','tungsten','tin','antimony','polonium','ruthenium','technetium','tellurium','yttrium']],'AtmosphereComposition':composition,'MassEM':0.05,'TerraformState':'','Rings':[]}
        case={'Target':species,'Scan':scan,'Position':[0,0,0],'Star':st,'Complete':True,'AllRegions':True,'AllSpheres':True,'ExtraPlanets':[planets[i]for i in sorted(state.get('ExtraPlanets',[]))],'ExtraStars':[{'AEBE':'AeBe','UNKNOWN':''}.get(stars[i],stars[i])for i in sorted(state.get('ExtraStars',[]))],'StarDistance':dist}
        output.append(case)
        # Original checker confidence and merged-colour reasons for binary parents.
        for secondary, delta in [('G',1.0),('G',3.0),('A',1.0),('A',3.0)]:
            binary=copy.deepcopy(case);binary['ParentStar']=secondary;binary['MagnitudeDelta']=delta;output.append(binary)
        for req in requirements:
          for conjunction in req['Value']:
            for p in conjunction:
              if p['Type']not in [7,8,9,10,11,12]:continue
              key={7:'SurfacePressure',8:'SurfacePressure',9:'SurfaceGravity',10:'SurfaceGravity',11:'SurfaceTemperature',12:'SurfaceTemperature'}[p['Type']];scale=101325 if p['Type']in[7,8]else 9.798 if p['Type']in[9,10]else 1
              for offset in [-0.000001,0,0.000001]:
                altered=copy.deepcopy(case);altered['Scan'][key]=(p['DValue']+offset)*scale;output.append(altered)
    return output
