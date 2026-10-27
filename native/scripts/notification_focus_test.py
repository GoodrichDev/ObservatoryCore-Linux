#!/usr/bin/env python3
"""An X11 window manager that focuses every ordinary mapped window.

The real application must declare notification surfaces passive BEFORE mapping.
This deliberately reproduces the activation policy that interrupted gameplay;
checking only the final focus or a ViewportBuilder flag would miss the bug.
Run under xvfb-run, never against a user's desktop.
"""
import argparse
import ctypes as c
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

DRIVER=r'''return {id='test.focus',name='Notification Focus Test',version='1',on_status=function(s)
 local p=s.TestPhase;if not p or p==0 then return end
 if p==1 or p==4 then
  core.notify({id='focus-notice',action='show',title='Gameplay must keep focus',detail='Source notification',timeout=1000})
  core.send_message({target='StellarOverlay',command='RadarSetPlanet',payload={ID64=42,BodyID=1,RadiusKM=250}})
  core.send_message({target='StellarOverlay',command='RadarAddBiological',payload={UseCurrentPosition=false,X=0.1,Y=0.1,ContextID=7,Radius=500,Label='Sample',MeasureDistance=true}})
  core.send_message({target='StellarOverlay',command='ShowBitmapNotification',payload={ImageKey='focus-bitmap',Title='Bitmap',Predictions={},TTL=1000,ForceShow=true}})
 elseif p==2 then core.notify({id='focus-notice',action='update',title='Updated without activation',detail='The game still receives input'})
 elseif p==3 then
  core.notify({id='focus-notice',action='cancel'})
  core.send_message({target='StellarOverlay',command='HideBitmapNotification',payload={ImageKey='focus-bitmap'}})
  core.send_message({target='StellarOverlay',command='RadarHide',payload={}})
 end
end}'''

def main():
    parser=argparse.ArgumentParser();parser.add_argument('binary',type=Path);parser.add_argument('--output',required=True,type=Path)
    parser.add_argument('--core-fallback',action='store_true',help='Disable StellarOverlay and verify actual Core native notifications')
    parser.add_argument('--expect-focus-loss',action='store_true',help='Validate the pre-fix binary as a negative control')
    args=parser.parse_args()
    assert os.environ.get('DISPLAY','').startswith(':'), 'Use a private Xvfb display'
    x=c.CDLL('libX11.so.6');D=c.c_void_p;W=c.c_ulong
    x.XOpenDisplay.argtypes=[c.c_char_p];x.XOpenDisplay.restype=D
    display=x.XOpenDisplay(None);assert display
    x.XDefaultRootWindow.argtypes=[D];x.XDefaultRootWindow.restype=W;root=x.XDefaultRootWindow(display)
    x.XInternAtom.argtypes=[D,c.c_char_p,c.c_int];x.XInternAtom.restype=W
    atom=lambda name:x.XInternAtom(display,name.encode(),0)
    x.XSelectInput.argtypes=[D,W,c.c_long];x.XSelectInput(display,root,(1<<20)|(1<<19))
    x.XCreateSimpleWindow.argtypes=[D,W,c.c_int,c.c_int,c.c_uint,c.c_uint,c.c_uint,W,W];x.XCreateSimpleWindow.restype=W
    x.XStoreName.argtypes=[D,W,c.c_char_p];x.XMapWindow.argtypes=[D,W]
    x.XSetInputFocus.argtypes=[D,W,c.c_int,W];x.XGetInputFocus.argtypes=[D,c.POINTER(W),c.POINTER(c.c_int)]
    x.XSync.argtypes=[D,c.c_int];x.XPending.argtypes=[D]
    x.XFree.argtypes=[c.c_void_p]
    x.XGetWindowProperty.argtypes=[D,W,W,c.c_long,c.c_long,c.c_int,W,c.POINTER(W),c.POINTER(c.c_int),c.POINTER(W),c.POINTER(W),c.POINTER(c.c_void_p)]
    class Map(c.Structure):_fields_=[('type',c.c_int),('serial',W),('send_event',c.c_int),('display',D),('parent',W),('window',W)]
    class Event(c.Union):_fields_=[('type',c.c_int),('mapping',Map),('pad',c.c_long*24)]
    x.XNextEvent.argtypes=[D,c.POINTER(Event)]
    def property(window,name):
        actual,fmt,count,after,data=W(),c.c_int(),W(),W(),c.c_void_p()
        x.XGetWindowProperty(display,window,atom(name),0,1024,0,0,c.byref(actual),c.byref(fmt),c.byref(count),c.byref(after),c.byref(data))
        if not data:return []
        values=list(c.cast(data,c.POINTER(W))[:count.value]) if fmt.value==32 else c.string_at(data,count.value)
        x.XFree(data);return values
    game=x.XCreateSimpleWindow(display,root,0,0,1000,700,0,0,0)
    x.XStoreName(display,game,b'Elite Dangerous focus fixture');x.XMapWindow(display,game);x.XSync(display,0)
    maps=[];violations=[];polls=0;checking=False
    def pump(seconds):
        nonlocal polls
        deadline=time.monotonic()+seconds
        while time.monotonic()<deadline:
            while x.XPending(display):
                event=Event();x.XNextEvent(display,c.byref(event))
                if event.type==20:
                    window=event.mapping.window
                    hints=property(window,'WM_HINTS');kind=property(window,'_NET_WM_WINDOW_TYPE')
                    passive=bool(hints and hints[0]&1 and hints[1]==0)
                    title=property(window,'_NET_WM_NAME')
                    x.XMapWindow(display,window)
                    if not passive:x.XSetInputFocus(display,window,2,0)
                    if checking:maps.append({'title':title.decode(errors='replace') if isinstance(title,bytes) else '',
                                            'passive_before_map':passive,'notification_type':atom('_NET_WM_WINDOW_TYPE_NOTIFICATION') in kind})
                    x.XSync(display,0)
            if checking:
                focus,revert=W(),c.c_int();x.XGetInputFocus(display,c.byref(focus),c.byref(revert));polls+=1
                if focus.value!=game:violations.append(int(focus.value))
            time.sleep(0.005)
    with tempfile.TemporaryDirectory(prefix='observatory-focus-') as directory:
        folder=Path(directory);journals=folder/'journals';journals.mkdir()
        (journals/'Journal.2026-10-25T000000.01.log').write_text(json.dumps({'timestamp':'2026-10-25T00:00:00Z','event':'Location','SystemAddress':42,'StarSystem':'Focus Fixture'})+'\n')
        status=journals/'Status.json'
        def phase(number):
            temporary=status.with_suffix('.new');temporary.write_text(json.dumps({'timestamp':f'2026-10-25T00:00:{number:02}Z','event':'Status','TestPhase':number,'Latitude':0,'Longitude':0,'Heading':0,'PlanetRadius':250000}));temporary.replace(status)
        phase(0)
        env=os.environ.copy();env.pop('WAYLAND_DISPLAY',None);env['LIBGL_ALWAYS_SOFTWARE']='1'
        for kind in ('CONFIG','DATA','CACHE'):env[f'XDG_{kind}_HOME']=str(folder/kind.lower())
        plugins=folder/'data'/'observatory-core'/'plugins';plugins.mkdir(parents=True);(plugins/'focus.lua').write_text(DRIVER)
        config=folder/'settings.json';config.write_text(json.dumps({'journal_dir':str(journals),'auto_start':True,'poll_interval_ms':50,'popup':True,'speech':False,
            'disabled_plugins':['StellarOverlay'] if args.core_fallback else [],
            'plugins':{'Hecate':{'RemoteLookups':False},'BioInsights':{'GalacticCodexRemote':False},'StellarOverlay':{'DisplayOverride':True,'TTL':1000,'NotificationDelayMS':0}}}))
        args.output.parent.mkdir(parents=True,exist_ok=True)
        with args.output.with_suffix('.log').open('w') as log:
            process=subprocess.Popen([str(args.binary.resolve()),'--config',str(config)],env=env,stdout=log,stderr=log)
            try:
                pump(2);x.XSetInputFocus(display,game,2,0);x.XSync(display,0);checking=True
                for number,seconds in [(1,0.7),(2,0.4),(3,0.7),(4,2.5)]:phase(number);pump(seconds)
                assert process.poll() is None,'Native application exited'
                assert len(maps)>=(2 if args.core_fallback else 4),('Expected real native popup/recreated surfaces',maps)
                if args.core_fallback:assert all('Observatory Core Notification' in m['title']for m in maps),maps
                if args.expect_focus_loss:
                    assert violations,'Negative-control binary did not reproduce focus stealing'
                    assert any(not m['passive_before_map'] for m in maps),maps
                else:
                    assert all(m['passive_before_map'] and m['notification_type'] for m in maps),maps
                    assert not violations,('Foreground focus changed',violations,maps)
            finally:process.terminate();process.wait(timeout=8)
    result={'display':'private X11','foreground_samples':polls,'focus_changes':len(violations),'mapped_surfaces':maps,'cycles':['show','update','cancel','recreate','expire'],'synthetic_only':True,'negative_control':args.expect_focus_loss,'core_fallback':args.core_fallback}
    args.output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))

if __name__=='__main__':main()
