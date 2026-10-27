"""Synthetic orbital-tree inputs, independent of expected enrichment values."""
import copy,json
from pathlib import Path
AU=149597870700

def star(id,kind='G',temp=5800,radius=700000000,mag=4.83,orbit=0,parents=None):
 return dict(event='Scan',BodyID=id,BodyName=f'Synthetic {id}',StarType=kind,SurfaceTemperature=temp,Radius=radius,AbsoluteMagnitude=mag,Luminosity='V',SemiMajorAxis=orbit,Parents=parents or [],DistanceFromArrivalLS=0)
def planet(id,orbit,parents):
 return dict(event='Scan',BodyID=id,BodyName=f'Synthetic {id}',PlanetClass='Rocky body',Landable=True,SurfaceTemperature=180,Atmosphere='thin carbon dioxide atmosphere',AtmosphereType='CarbonDioxide',SurfacePressure=1000,SurfaceGravity=1,Radius=1000000,SemiMajorAxis=orbit,Parents=parents,DistanceFromArrivalLS=500)
def generate():
 out=[]
 def add(name,scans,id):out.append(dict(Name=name,Scans=scans,BodyID=id))
 for dist in [.1,1,10]:
  for kind in ['G','M','T','Y','L','D','N','AeBe','H']:
   add('solo '+kind,[star(0,kind),planet(1,dist*AU,[{'Star':0}])],1)
  add('moon',[star(0),planet(1,dist*AU,[{'Star':0}]),planet(2,.01*AU,[{'Planet':1},{'Star':0}])],2)
  for kind,temp,radius,mag in [('M',3000,100000000,10),('B',20000,3000000000,-2),('T',1000,70000000,20)]:
   bary=dict(event='ScanBaryCentre',BodyID=0,BodyName='Synthetic AB',SemiMajorAxis=0,Parents=[])
   a=star(1,'G',5800,700000000,4.83,.02*AU,[{'Null':0}]);b=star(2,kind,temp,radius,mag,.03*AU,[{'Null':0}]);p=planet(3,dist*AU,[{'Null':0}]);add('circumbinary '+kind,[bary,a,b,p],3)
   p=planet(3,dist*AU,[{'Star':1},{'Null':0}]);add('binary planet '+kind,[bary,a,b,p],3)
 add('no scans',[planet(1,AU,[{'Star':0}])],1)
 add('missing barycentre',[star(1,parents=[{'Null':0}]),star(2,'M',3000,100000000,10,.1*AU,[{'Null':0}]),planet(3,AU,[{'Null':0}])],3)
 for dist in [.1,1,10]:
  base=[star(0),planet(1,dist*AU,[{'Star':0}]),planet(2,.01*AU,[{'Planet':1},{'Star':0}])]
  add('deep moon',base+[planet(3,.001*AU,[{'Planet':2},{'Planet':1},{'Star':0}])],3)
  add('other root star',[star(0),star(10,'M',3000,100000000,10),planet(1,dist*AU,[{'Star':0}])],1)
  root=dict(event='ScanBaryCentre',BodyID=0,BodyName='Root',SemiMajorAxis=0,Parents=[])
  inner=dict(event='ScanBaryCentre',BodyID=10,BodyName='Inner',SemiMajorAxis=5*AU,Parents=[{'Null':0}])
  a=star(1,'G',5800,700000000,4.83,.02*AU,[{'Null':10},{'Null':0}]);b=star(2,'M',3000,100000000,10,.03*AU,[{'Null':10},{'Null':0}]);outer=star(20,'A',10000,2000000000,0,10*AU,[{'Null':0}])
  for parents in [[{'Null':10},{'Null':0}],[{'Star':1},{'Null':10},{'Null':0}],[{'Star':20},{'Null':0}]]:
   add('nested barycentres '+str(parents),[root,inner,a,b,outer,planet(3,dist*AU,parents)],3)
 for hint in ['G','M','T','Y','L','D','N','AEBE','H']:
  add('hint '+hint,[planet(1,AU,[{'Star':0}])],1);out[-1]['StarHint']=hint
 for parent in [[{'Star':9}],[{'Planet':9},{'Star':0}],[{'Null':9}]]:
  add('incomplete parents '+str(parent),[star(0),planet(1,AU,parent)],1)
 for kind in ['M_RedSuperGiant','M_RedGiant','K_OrangeGiant','F_WhiteSuperGiant','G_WhiteSuperGiant','A_BlueWhiteSuperGiant','B_BlueWhiteSuperGiant','DAB','DAZ','WNC','WC','CN','CJ','CH','CHd','CS','MS','S','SupermassiveBlackHole','Unknown']:
  add('classification '+kind,[star(0,kind),planet(1,AU,[{'Star':0}])],1)
 for count in range(4):
  for parents in [[],[{'Star':0}],[{'Star':2}],[{'Null':0}],[{'Null':2}],[{'Planet':9},{'Star':0}]]:
   for proc in [False,True]:
    for hint in ['', 'G', 'M']:
     scans=[star(i,['G','M','B'][i],[5800,3000,20000][i],[700000000,100000000,3000000000][i],[4.83,10,-2][i])for i in range(count)]
     scans.append(planet(20,AU,parents))
     add('fallback '+str((count,parents,proc,hint)),scans,20)
     out[-1]['StarHint']=hint
     if proc:
      out[-1]['SystemName']='Test AA-A a1-1'
      for scan in scans:scan['BodyName']=out[-1]['SystemName']+(' 1'if scan['BodyID']==20 else' '+chr(65+scan['BodyID']))
 return out
if __name__=='__main__': print(json.dumps(generate()))
