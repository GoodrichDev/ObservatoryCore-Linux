from pathlib import Path
import json,random
defaults=dict(NotifyPrivate=True,NotifyWing=True,NotifySquadron=True,NotifySystem=True,NotifyLocal=True,NotifyNPC=False,SuppressVoice=False,ShowPrivate=True,ShowWing=True,ShowSquadron=True,ShowSystem=True,ShowLocal=True,ShowNPC=False)
base=dict(timestamp='2026-10-25T01:02:03Z')
channels=['local','starsystem','system','wing','squadron','npc','Private CMDR','WING','SYSTEM','SQUADRON','LOCAL','NPC','']
cases=[]
for channel in channels:
 for localized in [None,'','Localized']:
  for commander in [None,'','CMDR Test']:
   events=[base|dict(event='LoadGame',Commander=commander),base|dict(event='SendText',To=channel,To_Localised=localized,Message='Raw outgoing',Message_Localised='Ignore outgoing localized'),base|dict(event='ReceiveText',Channel=channel,From='Raw sender',From_Localised=localized,Message='Raw incoming',Message_Localised=localized)]
   cases.append(dict(Label=f'{channel}/{localized}/{commander}',Settings=defaults,Events=events))
random.seed(152)
for i in range(128):
 config={k:bool(random.getrandbits(1)) for k in defaults}
 cases.append(dict(Label='settings-'+str(i),Settings=config,Events=[base|dict(event='ReceiveText',Channel=channel,From='Pilot',Message=channel) for channel in channels]+[dict(event='Batch',Enabled=True),base|dict(event='ReceiveText',Channel='wing',From='Batch sender',Message='Read all')]))
for timestamp in ['2026-10-25T01:02:03Z','2026-10-25T01:02:03.123Z','2026-10-25T01:02:03+01:00','','invalid']:
 cases.append(dict(Label='timestamp-'+timestamp,Settings=defaults,Events=[dict(event='SendText',timestamp=timestamp,To='wing',Message='time')]))
Path('/home/loic/.cache/observatory-communicator-inputs.json').write_text(json.dumps(cases),encoding='utf-8')
print(len(cases))
