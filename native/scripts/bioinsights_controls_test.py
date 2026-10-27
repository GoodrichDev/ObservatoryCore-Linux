#!/usr/bin/env python3
"""Real Bio zoom/disabled-output controls on an owned private Xvfb display."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import time
import numpy as np
from native_update_test import Profile
from x11_qa import PrivateDisplay
from bioinsights_visual_parity_test import wait,geometry


def drag(private,window,start,end):
    private.x.move(window,*start)
    private.x.test.XTestFakeButtonEvent(private.x.display,1,1,0)
    private.x.x.XSync(private.x.display,0)
    time.sleep(.035)
    private.x.move(window,*end)
    time.sleep(.035)
    private.x.test.XTestFakeButtonEvent(private.x.display,1,0,0)
    private.x.x.XSync(private.x.display,0)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary',type=Path)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    args.output.mkdir(parents=True,exist_ok=True)
    binary=args.output.resolve()/'observatory-core-tested'
    shutil.copy2(args.binary.resolve(),binary)
    receipt={'core_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'synthetic_only':True,'display':'owned private Xvfb','cases':{}}
    private=PrivateDisplay(1600,900)
    try:
        with tempfile.TemporaryDirectory(dir=args.output) as temporary:
            for renderer in ['rust','webkit']:
                for disabled in [False,True]:
                    label=renderer+('-disabled' if disabled else '-zoom')
                    profile=Profile(Path(temporary),label)
                    for path in profile.plugins.iterdir():path.unlink()
                    profile.settings.update(auto_start=True,start_read_all=True,bioinsights_renderer=renderer)
                    profile.settings['plugins']['StellarOverlay']['DisplayOverride']=False
                    if disabled:profile.settings['plugins']['BioInsights']['NativeOutput']='Disabled'
                    profile.save()
                    env=dict(private.env)
                    env.update({k:v for k,v in profile.env.items() if k.startswith('XDG_')})
                    with (args.output/(label+'.log')).open('w') as log:
                        process=subprocess.Popen([str(binary),'--config',str(profile.config),'--view','BioInsights','--window-size','1200x800'],env=env,stdout=log,stderr=log)
                        try:
                            def mapped():
                                assert process.poll() is None,'Core exited '+label
                                return next((window for window,state in private.x.windows().items() if state['mapped'] and 'Observatory' in state['title']),None)
                            window=wait(mapped,label+' mapped')
                            time.sleep(2)
                            private.x.move(window,1100,700)
                            time.sleep(.1)
                            initial=private.x.capture(window)
                            initial.save(args.output/(label+'-initial.png'))
                            original=geometry(initial)
                            if disabled:
                                assert original['toolbar'] is None and not original['orange_bands'],original
                                receipt['cases'][label]={'no_custom_toolbar':True,'no_main_table':True,'visible_label':'Native Output Disabled (source UI string; screenshot retained)'}
                                continue
                            assert original['toolbar'],original
                            anchor=(original['toolbar']['centers'][5],original['toolbar']['y']+12)
                            roi=(int(anchor[0])-8,int(anchor[1])-8,530,76)
                            before=np.asarray(private.x.capture(window,roi)).copy()
                            private.x.click(window,*anchor)
                            def menu_visible():
                                pixels=np.asarray(private.x.capture(window,roi))
                                return np.count_nonzero(np.max(abs(pixels.astype(np.int16)-before),axis=2)>20)>1500
                            wait(menu_visible,label+' zoom opens inline')
                            private.x.capture(window).save(args.output/(label+'-open.png'))
                            def zoom():
                                try:return json.loads(profile.config.read_text())['plugins']['BioInsights']['ZoomLevel']
                                except (KeyError,ValueError):return None
                            y=anchor[1]+24
                            drag(private,window,(anchor[0]+180,y),(anchor[0]-35,y))
                            wait(lambda:zoom()==50,label+' minimum50%')
                            private.x.capture(window).save(args.output/(label+'-50.png'))
                            drag(private,window,(anchor[0]+180,y),(anchor[0]+440,y))
                            wait(lambda:zoom()==600,label+' maximum600%')
                            private.x.capture(window).save(args.output/(label+'-600.png'))
                            private.x.click(window,anchor[0]+465,anchor[1]+27)
                            wait(lambda:zoom()==100,label+' reset100%')
                            private.x.move(window,1100,700)
                            time.sleep(.1)
                            private.x.capture(window).save(args.output/(label+'-reset.png'))
                            dummy=private.x.x.XCreateSimpleWindow(private.x.display,private.x.root,1250,0,100,100,0,0,0)
                            private.x.x.XMapWindow(private.x.display,dummy)
                            private.x.x.XSetInputFocus(private.x.display,dummy,2,0)
                            private.x.x.XSync(private.x.display,0)
                            time.sleep(.15)
                            wait(lambda:not menu_visible(),label+' zoom closes on focus loss')
                            private.x.capture(window).save(args.output/(label+'-focus-loss.png'))
                            private.x.x.XDestroyWindow(private.x.display,dummy)
                            windows=[state for state in private.x.windows().values() if state['mapped'] and ('Observatory' in state['title'] or 'BioInsights' in state['title'])]
                            assert len(windows)==1,windows
                            receipt['cases'][label]={'minimum':50,'maximum':600,'reset':100,'settings_persist_immediately':True,'closes_on_focus_loss':True,'native_windows':1}
                        finally:
                            process.terminate()
                            process.wait(timeout=10)
                    (args.output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    finally:private.close()
    (args.output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps(receipt),flush=True)


if __name__=='__main__':main()
