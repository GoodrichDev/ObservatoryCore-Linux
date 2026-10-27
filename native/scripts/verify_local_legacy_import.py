#!/usr/bin/env python3
"""Verify a read-only original-profile import into disposable native XDG roots.
The receipt contains aggregate counts and hashes, never original rows/settings.
"""
import argparse, base64, hashlib, json, os, sqlite3, subprocess, tempfile
from pathlib import Path

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()
def inventory(root):
    return {str(p.relative_to(root)):sha(p) for p in root.rglob('*') if p.is_file()}
def database(path):
    result={}
    with sqlite3.connect(path.resolve().as_uri()+'?mode=ro',uri=True) as conn:
        for (name,) in conn.execute("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"):
            quoted='"'+name.replace('"','""')+'"'
            rows=[]
            for row in conn.execute('SELECT * FROM '+quoted):
                rows.append(json.dumps([{'blob':base64.b64encode(v).decode()} if isinstance(v,bytes) else v for v in row],ensure_ascii=True,separators=(',',':')))
            result[name]={'rows':len(rows),'sha256':hashlib.sha256('\n'.join(sorted(rows)).encode()).hexdigest()}
    return result

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--binary',type=Path,required=True);parser.add_argument('--settings',type=Path,required=True);parser.add_argument('--source-data',type=Path,required=True);parser.add_argument('--receipt',type=Path,required=True);args=parser.parse_args()
    before=inventory(args.source_data);settings_hash=sha(args.settings)
    with tempfile.TemporaryDirectory(prefix='observatory-import-verify-') as temp:
        root=Path(temp);env=os.environ.copy();env.update(HOME=str(root),XDG_CONFIG_HOME=str(root/'config'),XDG_DATA_HOME=str(root/'data'),XDG_CACHE_HOME=str(root/'cache'))
        native=root/'data/observatory-core';runs=[];first_bio=None
        for index in range(2):
            process=subprocess.run([str(args.binary.resolve()),'--import-settings',str(args.settings.resolve())],env=env,text=True,capture_output=True,timeout=90)
            if process.returncode:raise RuntimeError(process.stderr)
            receipt=json.JSONDecoder().raw_decode(process.stdout)[0]
            runs.append({'files':len(receipt['files']),'rows_added':sum(item['rows_added'] for item in receipt['files']),'commanders':receipt['commanders'],'codex_entries':receipt['codex_entries']})
            bio_path=native/'BioInsights/state.json'
            bio=json.loads(bio_path.read_text()) if bio_path.exists() else None
            if index==0:first_bio=bio
            else:
                assert bio==first_bio, 'Second import changed BioInsights state'
                assert runs[-1]['rows_added']==0, 'Second import added duplicate records'
        dbs={}
        for source in args.source_data.glob('4d617474-4707-0000-0000-484543415445/*.db'):
            original=database(source);copied=database(native/'Hecate'/source.name);assert original==copied,source.name+' logical rows changed';dbs[source.name]=original
        config=json.loads((root/'config/observatory-core/settings.json').read_text());assert all(name in config['plugins'] for name in ['BioInsights','Evaluator','Hecate','StellarOverlay'])
        assert before==inventory(args.source_data), 'Original storage changed'
        assert settings_hash==sha(args.settings), 'Original settings changed'
        output={'binary_sha256':sha(args.binary),'original_files_unchanged':len(before),'original_settings_unchanged':True,'import_runs':runs,'hecate_databases':dbs,'native_profile_temporary':True,'plugin_settings_present':4}
        args.receipt.parent.mkdir(parents=True,exist_ok=True);args.receipt.write_text(json.dumps(output,indent=2)+'\n');print(json.dumps(output,indent=2))
if __name__=='__main__':main()
