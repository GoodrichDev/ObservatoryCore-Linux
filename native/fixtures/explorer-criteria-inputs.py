"""Independent threshold, formatting, history and disabled-switch inputs."""
from pathlib import Path
import json,re,random
source=Path('/home/loic/.cache/observatory-readable/explorer/Observatory.Explorer/ExplorerSettings.cs').read_text()
settings={key:True for key in re.findall(r'public bool (\w+)\s*\{',source)}
base=dict(BodyID=5,SystemAddress=1,StarSystem='Test',BodyName='Test 1 a',PlanetClass='Rocky body',Radius=1000000,Materials=[],Parents=[{'Planet':2},{'Star':0}])
parent=dict(BodyID=2,Radius=10000000,Rings=[dict(Name='Test 1 A Ring',InnerRad=12000000,OuterRad=22000000),dict(Name='Test 1 B Ring',InnerRad=30000000,OuterRad=40000000)])
cases=[]
def add(label,changes=None,history=None,signals=None,config=None):
 scan=base| (changes or {})
 cases.append(dict(Label=label,Scan=scan,History=(history if history is not None else [parent])+[scan],Signals=signals or [],Settings=config or settings))
add('baseline')
for field,values in {
 'SurfaceGravity':[0,29.39999,29.4,29.400001,29.40001,40,52.729],
 'Radius':[0,299999,300000,300001,17999998,18000000,18000002,123456789],
 'SemiMajorAxis':[0,1,11999999,12000000,13000000,19999999,22000000,29999998,30000000,30000002,40000000,50000000],
 'Eccentricity':[0,.8999999,.9,.9000001,.912345,.9999999],
 'RotationPeriod':[-28800.1,-28800,-28799.99,-3599.9,0,3599.9,28799.99,28800,28800.1],
 'OrbitalPeriod':[-28800.1,-28800,-28799.99,-3599.9,0,3599.9,28799.99,28800,28800.1],
 'Atmosphere':['','thin argon atmosphere','thin argon, "trace" atmosphere','hELLo WORLD',"o'BRIEN",'earth-LIKE body','1abc 2DEF'],
 'PlanetClass':[None,'','Barycentre','Earthlike body','Ammonia world','Water world','earth-LIKE body'],
 'StarType':[None,'','O','B','A','F','G','K','M','L','T','Y','TTS','AeBe','N','H','D','DA','DC','W','WN','WC','WO','WNC','C','CN','CS','CJ','CH','CHd','S','MS','X','RoguePlanet','SupermassiveBlackHole','B_BlueWhiteSuperGiant','A_BlueWhiteSuperGiant','F_WhiteSuperGiant','G_WhiteSuperGiant','K_OrangeGiant','M_RedGiant','M_RedSuperGiant','newtype'],
}.items():
 for value in values:
  add(f'{field}={value}',{'Landable':True,'DistanceFromArrivalLS':1234567.5,field:value})
for flags in range(32):
 add('high-value-'+str(flags),dict(PlanetClass=['Earthlike body','Water world'][flags&1],TerraformState='Terraformable' if flags&2 else '',WasMapped=bool(flags&4),WasDiscovered=bool(flags&8),Landable=bool(flags&16)))
for radius in [0,1000,1000000,16777216]:
 for width in [radius*5-1,radius*5,radius*5+1,299792450,3e11]:
  add(f'ring-{radius}-{width}',dict(Radius=radius,Rings=[dict(Name='Test 1 a A Ring',InnerRad=10000000,OuterRad=10000000+width)]))
for count in range(11):
 add('biosignals-'+str(count),signals=[dict(BodyID=5,Signals=[dict(Type='$SAA_SignalType_Biological;',Count=count)])])
boost=['Carbon','Germanium','Arsenic','Niobium','Yttrium','Polonium']
surface=['Antimony','Arsenic','Cadmium','Carbon','Chromium','Germanium','Iron','Manganese','Mercury','Molybdenum','Nickel','Niobium','Phosphorus','Polonium','Ruthenium','Selenium','Sulphur','Technetium','Tellurium','Tin','Tungsten','Vanadium','Yttrium','Zinc','Zirconium']
for materials in [[],boost,boost[:-1],surface,surface[:-1]]:
 for tail in [[],[dict(BodyID=8,Materials=[])],[dict(BodyID=8,Materials=[dict(Name='iron')])]]:
  add('materials-'+','.join(materials)+'-tail'+str(len(tail)),dict(Landable=True,Materials=[dict(Name=x.lower()) for x in materials]),history=[dict(BodyID=1,Materials=[dict(Name=x) for x in materials])]+tail)
for p in [[{'Null':2},{'Star':0}],[{'Planet':2},{'Planet':0}],[{'Star':2}]]:
 for ratio in [.39999,.4,.40001,1.0]:
  add('binary-'+str(p)+str(ratio),dict(Parents=p,SemiMajorAxis=1000000,Radius=ratio*1000000,Eccentricity=.8),history=[dict(BodyID=4,BodyName='Test Partner',Parents=p,SemiMajorAxis=1000000,Radius=ratio*1000000,Eccentricity=.8)])
for close in [False,True]:
 for colliding in [False,True]:
  for eccentricity in [0,.8]:
   add(f'binary-independent-{close}-{colliding}-{eccentricity}',dict(Parents=[{'Null':2}],SemiMajorAxis=1000000,Radius=500000,Eccentricity=eccentricity),history=[dict(BodyID=4,BodyName='Partner',Parents=[{'Null':2}],SemiMajorAxis=1000000,Radius=500000,Eccentricity=eccentricity)],config=settings|dict(CloseBinary=close,CollidingBinary=colliding))
for body_name in ['Test 1 a','Test 1 a Ring']:
 for ring_name in ['Test 1 A Ring','Test 1 A Belt']:
  add('ring-exclusions-'+body_name+'-'+ring_name,dict(BodyName=body_name,SemiMajorAxis=22000000),history=[parent|dict(Rings=[dict(Name=ring_name,InnerRad=12000000,OuterRad=22000000)])])
random.seed(152)
for i in range(512):
 add('mixed-'+str(i),dict(Landable=bool(i&1),TerraformState='Terraformable' if i&2 else '',Atmosphere='thin argon atmosphere' if i&4 else '',Radius=random.choice([299999,300000,19000000,500000]),RotationPeriod=random.choice([0,1000,-1000,28800]),TidalLock=bool(i&8),SemiMajorAxis=random.choice([100000,12000000,30000000]),Eccentricity=random.choice([0,.9,.99]),OrbitalPeriod=random.choice([1000,28800,0]),StarType=random.choice([None,'','N','M']),DistanceFromArrivalLS=1e6,Materials=[dict(Name=x) for x in random.sample(surface,random.randrange(26))]),config={k:bool(random.getrandbits(1)) for k in settings})
Path('/home/loic/.cache/observatory-explorer-criteria-inputs.json').write_text(json.dumps(cases),encoding='utf-8')
print(len(cases))
