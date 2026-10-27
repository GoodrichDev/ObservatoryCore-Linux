from pathlib import Path
import json
root=Path('/home/loic/.cache')
criteria=json.loads((root/'observatory-explorer-criteria-inputs.json').read_text())
cases=[]
def scan(v):return dict(event='Scan',timestamp='2026-10-25T01:02:03Z',BodyName='Test '+str(v.get('BodyID',0)),SystemAddress=1,StarSystem='Test')|v
for row in criteria:
 settings=row['Settings']|dict(EnableCustomCriteria=False)
 history=[scan(v) for v in row['History'] if v['BodyID']!=row['Scan']['BodyID']]
 signals=[dict(event='FSSBodySignals',SystemAddress=1)|v for v in row['Signals']]
 cases.append(dict(Label=row['Label'],Settings=settings,Events=history+signals+[scan(row['Scan'])]))
settings=criteria[0]['Settings']|dict(EnableCustomCriteria=False)
for system,body in [('Test','Test'),('Test','Test 1 a'),('Test','Test AB 12 c'),('Test','Test 1 A Ring'),('Test','Test Test A Ring'),('Test','Unknown Body'),('Test','Test Belt Cluster 1')]:
 cases.append(dict(Label='formatter-'+body,Settings=settings,Events=[scan(dict(BodyName=body,StarSystem=system,Radius=100000,PlanetClass='Rocky body'))]))
for suffix in ['A','B','Z','AA','AZ','1','9','10','20','1 a','1 9']:
 for delta in [1,2]:
  cases.append(dict(Label=f'barycentre-{suffix}-{delta}',Settings=settings,Events=[dict(event='ScanBaryCentre',timestamp='2026-10-25T01:02:03Z',StarSystem='Test',SystemAddress=1,BodyID=1,OrbitalPeriod=1000),scan(dict(BodyID=1+delta,BodyName='Test '+suffix,PlanetClass='Rocky body',Radius=1000000,Parents=[{'Null':1},{'Star':0}]))]))
for signal_kind in ['SAASignalsFound','FSSBodySignals']:
 cases.append(dict(Label='signal-history-'+signal_kind,Settings=settings,Events=[dict(event=signal_kind,SystemAddress=1,BodyID=5,Signals=[dict(Type='$SAA_SignalType_Biological;',Count=8)]),dict(event=signal_kind,SystemAddress=1,BodyID=5,Signals=[dict(Type='$SAA_SignalType_Biological;',Count=2)]),scan(dict(BodyID=5,Radius=1000000))]))
for batch in [False,True]:
 cases.append(dict(Label='onlycurrent-'+str(batch),Settings=settings|dict(OnlyShowCurrentSystem=True),Events=[dict(event='Batch',Enabled=batch),dict(event='Location',StarSystem='Test',SystemAddress=1),scan(dict(BodyID=5,Radius=100000)),dict(event='Location',StarSystem='Next',SystemAddress=2),dict(event='Location',StarSystem='Test',SystemAddress=1),scan(dict(BodyID=5,Radius=100000))]))
cases.append(dict(Label='codex-body-lookup',Settings=settings,Events=[scan(dict(BodyID=5,Radius=1000000)),dict(event='CodexEntry',timestamp='2026-10-25T01:02:03Z',SystemAddress=1,BodyID=5,System='Test',IsNewEntry=True,Category_Localised='Organic',Name_Localised='Tussock')]))
cases.append(dict(Label='batch-clear-reprocess',Settings=settings,Events=[scan(dict(BodyID=5,Radius=100000)),dict(event='Batch',Enabled=True),scan(dict(BodyID=5,Radius=100000)),dict(event='Batch',Enabled=False),scan(dict(BodyID=5,Radius=100000))]))
cases.append(dict(Label='scan-history-prune-keeps-signal-cache',Settings=settings,Events=[dict(event='FSSBodySignals',SystemAddress=1,BodyID=5,Signals=[dict(Type='$SAA_SignalType_Biological;',Count=8)])]+[scan(dict(SystemAddress=i,BodyID=1,Radius=1000000)) for i in range(1,1003)]+[scan(dict(BodyID=5,Radius=100000))]))
(root/'observatory-explorer-events-inputs.json').write_text(json.dumps(cases),encoding='utf-8')
print(len(cases))
