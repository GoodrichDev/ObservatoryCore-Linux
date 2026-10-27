#!/usr/bin/env python3
"""Run actual native/compositor fullscreen acceptance on private displays only.

The foreground game is an instrumented synthetic client. This is deliberately
not an Elite/Proton acceptance claim. No user desktop, journal or physical audio
server is used. Run Wayland as an ordinary Linux user; run X11 as root so the
existing PrivateDisplay helper can isolate WSLg's read-only X11 socket mount.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--kind', choices=['wayland', 'x11'], required=True)
    parser.add_argument('--build-log', type=Path, required=True)
    parser.add_argument('--app-binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    scripts = Path(__file__).resolve().parent
    base = dict(os.environ, PULSE_SERVER='unix:/nonexistent-observatory-private-audio', LIBGL_ALWAYS_SOFTWARE='1', GALLIUM_DRIVER='llvmpipe')
    for name in ['DISPLAY','WAYLAND_DISPLAY','WAYLAND_SOCKET','SWAYSOCK']:
        base.pop(name, None)
    receipt = {'actual_game_tested':False,'synthetic_foreground_client':True,'user_desktop_accessed':False,
               'physical_audio_connected':False,'platform':sys.platform,
               'app_binary_sha256':hashlib.sha256(args.app_binary.read_bytes()).hexdigest(),'cases':{}}
    if args.kind == 'x11':
        assert os.geteuid() == 0, 'X11 mount/network isolation requires root'
        from x11_qa import PrivateDisplay
        display = PrivateDisplay(width=1000,height=700)
        try:
            env = dict(base, **{k:v for k,v in display.env.items() if k in ['DISPLAY','XDG_RUNTIME_DIR','GDK_BACKEND','WEBKIT_DISABLE_DMABUF_RENDERER']})
            for name, flags in [('stellar', []), ('core-fallback', ['--core-fallback'])]:
                result = output / (name+'.json')
                subprocess.run([sys.executable,str(scripts/'notification_focus_test.py'),str(args.app_binary.resolve()),'--output',str(result),*flags],env=env,check=True,timeout=25)
                receipt['cases'][name] = json.loads(result.read_text())
        finally:
            display.close()
    else:
        assert os.geteuid() != 0, 'Sway requires an ordinary Linux user'
        with tempfile.TemporaryDirectory(prefix='observatory-fullscreen-acceptance-') as temporary:
            private = Path(temporary)
            runtime = private/'runtime'
            runtime.mkdir(mode=0o700)
            config = private/'sway.conf'
            config.write_text('output HEADLESS-1 resolution 1920x1080\nseat seat0 fallback true\ndefault_border none\nxwayland disable\n')
            env = dict(base,XDG_RUNTIME_DIR=str(runtime),WLR_BACKENDS='headless',WLR_RENDERER='pixman',WLR_LIBINPUT_NO_DEVICES='1',WLR_NO_HARDWARE_CURSORS='1')
            log = (output/'sway.log').open('w')
            sway = subprocess.Popen(['sway','-c',str(config)],env=env,stdout=log,stderr=subprocess.STDOUT)
            try:
                deadline = time.monotonic()+10
                while not list(runtime.glob('sway-ipc.*.sock')) and time.monotonic()<deadline:
                    assert sway.poll() is None, 'private Sway exited'
                    time.sleep(.025)
                assert list(runtime.glob('sway-ipc.*.sock')), 'private Sway did not start'
                for scale in [1,2]:
                    destination = output/('wayland-scale'+str(scale))
                    subprocess.run([sys.executable,str(scripts/'wayland_focus_test.py'),'--scale',str(scale),'--build-log',str(args.build_log.resolve()),'--runtime',str(runtime),'--output',str(destination)],env=env,check=True,timeout=45)
                    receipt['cases']['wayland-scale'+str(scale)] = json.loads((destination/'result.json').read_text())
            finally:
                sway.terminate()
                sway.wait(timeout=5)
                log.close()
            weston_runtime = private/'weston'
            weston_runtime.mkdir(mode=0o700)
            env['XDG_RUNTIME_DIR'] = str(weston_runtime)
            log = (output/'weston.log').open('w')
            weston = subprocess.Popen(['weston','--backend=headless-backend.so','--use-pixman','--width=1920','--height=1080','--idle-time=0','--no-config','--socket=wayland-no-layer'],env=env,stdout=log,stderr=subprocess.STDOUT)
            try:
                deadline = time.monotonic()+10
                while not (weston_runtime/'wayland-no-layer').exists() and time.monotonic()<deadline:
                    assert weston.poll() is None, 'private Weston exited'
                    time.sleep(.025)
                subprocess.run([sys.executable,str(scripts/'wayland_capability_test.py'),'--build-log',str(args.build_log.resolve()),'--runtime',str(weston_runtime),'--output',str(output/'no-layer-shell')],env=env,check=True,timeout=30)
                receipt['cases']['no-layer-shell'] = json.loads((output/'no-layer-shell'/'result.json').read_text())
            finally:
                weston.terminate()
                weston.wait(timeout=5)
                log.close()
    (output/'result.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps(receipt),flush=True)


if __name__=='__main__':
    main()
