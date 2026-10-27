from pathlib import Path
import json,re
src=Path('/home/loic/.cache/observatory-readable/botanist/Observatory.Botanist/Botanist.cs').read_text()
genera=re.findall(r'\{ "(\$[^"]+)", "([^"]+)" \}',src)
cases=[]
load=dict(event='LoadGame',Odyssey=True)
signal=dict(event='SAASignalsFound',SystemAddress=42,BodyID=5,BodyName='Test 5',Signals=[dict(Type='$SAA_SignalType_Biological;',Count=3)])
status=dict(event='Status',Latitude=0,Longitude=0,PlanetRadius=1000000)
scan=dict(event='ScanOrganic',SystemAddress=42,Body=5,Species_Localised='Tussock Pennata',Genus='$Codex_Ent_Tussocks_Genus_Name;',ScanType='Log')
def add(label,steps):cases.append(dict(Label=label,Steps=steps))
for identifier,genus in genera+[('future-genus','Future genus')]:
 add('genus-'+identifier,[load,signal,status,scan|dict(Genus=identifier,Genus_Localised=genus)])
for stage in ['Log','Sample','Analyse']:
 add('first-unknown-'+stage,[status,scan|dict(ScanType=stage)])
for sticky in [False,True]:
 for enabled in [False,True]:
  add(f'lifecycle-{sticky}-{enabled}',[load,signal,dict(event='Settings',OverlayEnabled=enabled,OverlayIsSticky=sticky),status,scan,status|dict(Longitude=.01),scan|dict(ScanType='Sample'),status|dict(Longitude=.02),scan,scan|dict(ScanType='Analyse'),status|dict(Longitude=.03)])
for close in ['LeaveBody','FSDJump','CarrierJump','Shutdown','Location','Scan','ApproachBody','SupercruiseEntry']:
 add('close-'+close,[load,signal,status,scan,dict(event=close,BodyName='Changed'),status|dict(Longitude=.01),scan|dict(ScanType='Sample')])
add('scan-does-not-rename',[load,signal,dict(event='Scan',SystemAddress=42,BodyID=5,BodyName='Changed')])
add('signals-first-retained',[load,signal,signal|dict(BodyName='Changed',Signals=[dict(Type='$SAA_SignalType_Biological;',Count=9)])])
add('organic-before-signals',[load,scan,signal,scan])
add('fss-subclass',[load,signal|dict(event='FSSBodySignals')])
add('signals-odyssey-gate',[signal,load,signal])
add('batch-keeps-open-notification',[load,signal,status,scan,dict(event='Batch',Enabled=True),status|dict(Longitude=.1),scan|dict(ScanType='Sample'),dict(event='Batch',Enabled=False),status|dict(Longitude=.2)])
add('disable-keeps-open-notification',[load,signal,status,scan,dict(event='Settings',OverlayEnabled=False,OverlayIsSticky=False),status|dict(Longitude=.1),scan|dict(ScanType='Sample')])
add('row-insertion-order',[load,signal|dict(BodyID=9,BodyName='Test 9'),signal,scan|dict(Species_Localised='Z First'),scan|dict(Species_Localised='A Second'),scan|dict(Body=9,Species_Localised='Old Body')])
for lat,lon in [(0,0),(0,.02),(0,179.9),(45,45),(89.9,120),(-40,-179),(90,120)]:
 for lat2,lon2 in [(0,0),(0,.1),(45,-45),(-89.9,-120)]:
  add(f'distance-{lat}-{lon}-{lat2}-{lon2}',[load,signal,status|dict(Latitude=lat,Longitude=lon),scan,status|dict(Latitude=lat2,Longitude=lon2),scan|dict(ScanType='Sample'),status|dict(Latitude=0,Longitude=0)])
Path('/home/loic/.cache/observatory-botanist-inputs.json').write_text(json.dumps(cases),encoding='utf-8')
print(len(cases))
