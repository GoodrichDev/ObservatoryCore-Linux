#!/usr/bin/env python3
"""Inventory the statically decoded installed sources; never load vendor code.

Usage: python3 native/scripts/inventory_vault_overlay.py --cache /home/loic/.cache/observatory-readable
"""
import argparse, hashlib, json, pathlib, re
p=argparse.ArgumentParser();p.add_argument('--cache',type=pathlib.Path,required=True);a=p.parse_args()
root=pathlib.Path(__file__).resolve().parents[2]
result={'provenance':'Statically decompiled installed plugin source and decoded string constants; no vendor executable code was run. File hashes identify the exact reviewed text.','components':{}}
configs={
 'Hecate':('hecate','Hecate/HecateWorker.cs',['Hecate/HecateSettings.cs','Hecate/ServerSettings.cs','Hecate.Settings/FormServerSettings.cs','Hecate/HecateServer2.cs','Hecate/DBH.cs','Hecate/SystemDBH.cs','com.github.fredjk_gh.PluginCommon.Data.Spansh.Converters/SpanshToJournal.cs'],'native/observatory-plugins/src/hecate.rs'),
 'StellarOverlay':('stellar','StellarOverlay/OverlayWorker.cs',['StellarOverlay/OverlaySettings.cs','StellarOverlay/RadarSettings.cs','StellarOverlay/FormNotificationSettings.cs','StellarOverlay/FormRadarSettings.cs','StellarOverlay/Radar.cs','StellarOverlay/NotificationQueue.cs','StellarOverlay/NotificationItem.cs','StellarOverlay/EliteOverlay.cs','StellarOverlay/BrushHandler.cs'],'native/observatory-plugins/src/stellar_overlay.rs')}
for name,(folder,worker,extra,native) in configs.items():
    base=a.cache/folder; text=(base/worker).read_text(); nt=(root/native).read_text()
    dispatch=text[:text.index('\n\tprivate ',text.index('private void SetStateMachine'))] if name=='StellarOverlay' else text[text.index('public override void HandlePluginMessage'):text.index('public override void HandleReady')]
    commands=sorted(set(re.findall(r'\btext, "([A-Za-z][A-Za-z0-9]+)"',dispatch)))
    commands=sorted(set(commands)|{'Ready','LegacyPluginMessage'})
    comp={'files':[], 'message_commands':[{'name':cmd,'native_dispatch':bool(re.search(r'"'+re.escape(cmd)+r'"',nt))} for cmd in commands]}
    for rel in [worker]+extra:
        source=(base/rel).read_text()
        members=[]
        for no,line in enumerate(source.splitlines(),1):
            if re.match(r'^\t(?:public|private|protected) (?:override |virtual |static |async )*[^_\n]*\b[A-Za-z][A-Za-z0-9_]*(?:<[^>]+>)?\(',line) and '_20' not in line:
                members.append({'line':no,'declaration':line.strip()})
        for index, member in enumerate(members):
            begin=member['line']-1;end=members[index+1]['line']-1 if index+1<len(members) else len(source.splitlines())
            body='\n'.join(source.splitlines()[begin:end])
            member['payload_keys']=sorted(set(re.findall(r'\b(?:payload|args)\["([^"\\]+)"\]',body)))
            member['string_literals']=sorted(set(re.findall(r'"((?:[^"\\]|\\.)*)"',body)))
        properties=[]
        for m in re.finditer(r'^\tpublic (?:override )?[A-Za-z0-9<>?, ]+ ([A-Za-z][A-Za-z0-9_]*)\s*(?:\{|=>|\n\t\{)',source,re.M):
            prefix=source[max(0,m.start()-500):m.start()]
            attrs=[]
            for line in reversed(prefix.splitlines()):
                if line.strip().startswith('['):attrs.insert(0,line.strip())
                elif line.strip():break
            properties.append({'name':m[1],'line':source.count('\n',0,m.start())+1,'attributes':attrs})
        types=sorted(set(re.findall(r'\b(?:journal|obj\d*) is ([A-Za-z0-9_]+)',source)))
        comp['files'].append({'path':rel,'sha256':hashlib.sha256(source.encode()).hexdigest(),'methods':members,'properties':properties,'journal_event_types':types})
    result['components'][name]=comp
out=root/'native/fixtures/hecate-stellar-source-inventory.json';out.write_text(json.dumps(result,indent=2)+'\n')
for k,v in result['components'].items():print(k,len(v['message_commands']),'commands;',sum(len(f['methods']) for f in v['files']),'methods;',sum(len(f['properties']) for f in v['files']),'properties; missing dispatch:',[x['name'] for x in v['message_commands'] if not x['native_dispatch']])
