#!/usr/bin/env python3
"""Check real Core journal dispatch and idle CLR callbacks without a desktop."""
import argparse, hashlib, json, os, subprocess, tempfile
from pathlib import Path

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--binary',type=Path,required=True);parser.add_argument('--clr-host',type=Path);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
    with tempfile.TemporaryDirectory(prefix='observatory-clr-app-') as temporary:
        root=Path(temporary);journals=root/'journals';journals.mkdir()
        script=root/'criteria.lua'
        script.write_text("""---@Global
luanet.load_assembly('System.ComponentModel.TypeConverter')
Timer=luanet.import_type('System.Timers.Timer')
Math=luanet.import_type('System.Math')
timer=Timer(500)
timer.AutoReset=false
timer.Elapsed:Add(function(sender,args) notify('Idle CLR callback','CLR timer',tostring(Math.Sqrt(81))) end)
timer:Start()
---@End
---@Complex typed-object
scan.BodyName='CLR mutated body'
return true,'CLR scan',scan:GetType().FullName
---@End
""",encoding='utf-8')
        events=[{'timestamp':'2026-10-26T00:00:00Z','event':'Location','StarSystem':'CLR acceptance','SystemAddress':42},
                {'timestamp':'2026-10-26T00:00:01Z','event':'Scan','BodyName':'Original body','BodyID':1,'SystemAddress':42,'StarSystem':'CLR acceptance','Parents':[],'Radius':250000}]
        (journals/'Journal.2026-10-26T000000.01.log').write_text(''.join(json.dumps(event)+'\n'for event in events))
        config=root/'settings.json';config.write_text(json.dumps({'journal_dir':str(journals),'poll_interval_ms':50,'popup':False,'speech':False,'auto_start':False,'start_read_all':False,'disabled_plugins':['BioInsights','Evaluator','Hecate','StellarOverlay','Botanist','Communicator'],'plugins':{'Explorer':{'EnableCustomCriteria':True,'CustomCriteriaFile':str(script)},'BioInsights':{'GalacticCodexRemote':False},'Hecate':{'RemoteLookups':False,'ServerSettings':{'EnableWebServer':False}}}}))
        env=dict(os.environ,HOME=str(root),XDG_CONFIG_HOME=str(root/'config'),XDG_DATA_HOME=str(root/'data'),XDG_CACHE_HOME=str(root/'cache'),TZ='UTC',PULSE_SERVER='unix:/nonexistent-observatory-acceptance')
        for key in ['DISPLAY','WAYLAND_DISPLAY','WAYLAND_SOCKET']:env.pop(key,None)
        env.pop('OBSERVATORY_CLR_CRITERIA_HOST',None)
        if args.clr_host:env['OBSERVATORY_CLR_CRITERIA_HOST']=str(args.clr_host.resolve())
        original_journals={path.name:path.read_bytes() for path in journals.iterdir()}
        process=subprocess.run([str(args.binary.resolve()),'--headless','--watch-seconds','3','--config',str(config)],env=env,capture_output=True,text=True,timeout=30)
        assert process.returncode==0,process.stderr
        result=json.loads(process.stdout);assert not result['errors'],result['errors']
        scan=next(row for row in result['findings']if row['title']=='CLR scan')
        timer=next(row for row in result['findings']if row['title']=='CLR timer')
        assert scan['body']=='CLR mutated body' and scan['detail']=='Observatory.Framework.Files.Journal.Scan',scan
        assert timer['body']=='Idle CLR callback' and float(timer['detail'])==9,timer
        report={'core_sha256':hashlib.sha256(args.binary.read_bytes()).hexdigest(),'desktop_accessed':False,'physical_audio_connected':False,'source_user_files_accessed':False,'synthetic_only':True,'runtime_discovery':'explicit host'if args.clr_host else 'installed relative path','typed_journal_finding':scan,'idle_callback_finding':timer,'reported_live_event_counter':result['live_events'],'journal_files_unchanged':True,'runtime_errors':result['errors']}
        # The diagnostic watch counter can publish its initial total after the
        # Monitoring transition. Immutable input files independently prove that
        # no new journal input was supplied for the asynchronous callback.
        assert original_journals=={path.name:path.read_bytes()for path in journals.iterdir()},'Journal input changed during idle callback test'
        args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
if __name__=='__main__':main()
