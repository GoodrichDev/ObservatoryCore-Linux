#!/usr/bin/env python3
"""Verify failure is explicit and no xdg popup fallback is created on Weston."""
import argparse,json,os,pathlib,re,subprocess
p=argparse.ArgumentParser();p.add_argument('--build-log',type=pathlib.Path,required=True);p.add_argument('--runtime',type=pathlib.Path,required=True);p.add_argument('--display',default='wayland-no-layer');p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
records=[json.loads(s)for s in a.build_log.read_text().splitlines()if s.startswith('{')];binary=next(x['executable']for x in reversed(records)if x.get('executable')and x.get('profile',{}).get('test'))
env=dict(os.environ)
for key in ['DISPLAY','WAYLAND_SOCKET']:env.pop(key,None)
env.update(XDG_RUNTIME_DIR=str(a.runtime),WAYLAND_DISPLAY=a.display,LIBGL_ALWAYS_SOFTWARE='1',GALLIUM_DRIVER='llvmpipe',OBSERVATORY_WAYLAND_QA=str(a.output),WAYLAND_DEBUG='client')
with(a.output/'protocol.log').open('w')as log:
 proc=subprocess.run([binary,'--exact','passive_windows::wayland::tests::passive_overlay_native_rendering_probe','--ignored','--nocapture'],env=env,stdout=log,stderr=subprocess.STDOUT,timeout=25)
report=json.loads((a.output/'overlay.json').read_text());protocol=(a.output/'protocol.log').read_text();report.update(exit_code=proc.returncode,xdg_toplevel_creations=len(re.findall(r'get_toplevel\(',protocol)))
assert proc.returncode==0,protocol[-2000:]
assert report['max_layers']==0 and 'does not support wlr-layer-shell'in report['capability_error'],report
assert report['xdg_toplevel_creations']==1,report
(a.output/'result.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
