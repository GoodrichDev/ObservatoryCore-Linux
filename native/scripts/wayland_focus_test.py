#!/usr/bin/env python3
"""Exercise actual native passive layers in an isolated Sway Wayland session.

The native test executable contains ignored probes. No production focus restore
or synthetic input is used outside the explicitly supplied isolated compositor.
"""
import argparse,json,os,pathlib,subprocess,time
p=argparse.ArgumentParser();p.add_argument('--scale',type=int,default=1);p.add_argument('--build-log',type=pathlib.Path,required=True);p.add_argument('--runtime',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args()
artifacts=[json.loads(line)for line in a.build_log.read_text().splitlines()if line.startswith('{')]
binary=next(x['executable']for x in reversed(artifacts)if x.get('executable')and x.get('profile',{}).get('test'))
a.output.mkdir(parents=True,exist_ok=True)
baseenv=dict(os.environ)
for k in ['DISPLAY','WAYLAND_SOCKET']:baseenv.pop(k,None)
baseenv.update(XDG_RUNTIME_DIR=str(a.runtime),WAYLAND_DISPLAY=next(x.name for x in a.runtime.iterdir()if x.name.startswith('wayland-')and not x.name.endswith('.lock')),SWAYSOCK=str(next(a.runtime.glob('sway-ipc.*.sock'))),LIBGL_ALWAYS_SOFTWARE='1',GALLIUM_DRIVER='llvmpipe',OBSERVATORY_QA_SCALE=str(a.scale))
namespace='passive_windows::wayland::tests::'
def ipc(*args):return json.loads(subprocess.check_output(['swaymsg','-r',*args],env=baseenv,text=True))
def nodes(tree):
 yield tree
 for child in tree.get('nodes',[])+tree.get('floating_nodes',[]):yield from nodes(child)
def wait_window(app):
 start=time.monotonic()
 while time.monotonic()-start<8:
  found=next((n for n in nodes(ipc('-t','get_tree'))if n.get('app_id')==app),None)
  if found:return found
  time.sleep(.05)
 raise AssertionError('window never mapped: '+app)
def run_case(negative):
 name='negative-control'if negative else'passive';case=a.output/name;case.mkdir(exist_ok=True);env=dict(baseenv,OBSERVATORY_WAYLAND_QA=str(case),OBSERVATORY_WAYLAND_WAIT_FOR_START='1');processes=[];logs=[]
 start_marker=case/'probe-start';start_marker.unlink(missing_ok=True)
 def start(test,log):
  f=(case/log).open('w');logs.append(f);proc=subprocess.Popen([binary,'--exact',namespace+test,'--ignored','--nocapture'],env=env,stdout=f,stderr=subprocess.STDOUT);processes.append(proc);return proc
 overlay=start('unsafe_xdg_toplevel_negative_control'if negative else'passive_overlay_native_rendering_probe','overlay.log');wait_window('observatory-overlay-probe')
 game=start('fullscreen_game_input_probe','game.log');wait_window('observatory-game-probe');
 if not negative:ipc('[app_id="observatory-game-probe"] fullscreen enable')
 # Initial arrangement ends here. Never focus or restore any window during popup sampling.
 start_marker.touch()
 keyboard_log=(case/'keyboard.log').open('w');logs.append(keyboard_log);keyboard=subprocess.Popen(['wtype','-s','1200','a','-s','1500','b','-s','1500','c','-s','1500','d','-s','8000'],env=env,stdout=keyboard_log,stderr=subprocess.STDOUT);processes.append(keyboard)
 start('virtual_input::game_pointer_probe','pointer.log')
 began=time.monotonic();samples=[];screenshot=False
 while overlay.poll()is None and time.monotonic()-began<14:
  focused=[{'id':n['id'],'app_id':n.get('app_id'),'name':n.get('name')}for n in nodes(ipc('-t','get_tree'))if n.get('focused')and n.get('type')=='con']
  elapsed=time.monotonic()-began
  if elapsed>1.8 and elapsed<7.5:samples.append(focused)
  if elapsed>4 and not screenshot:
   subprocess.run(['grim',str(case/'compositor.png')],env=env,check=True);screenshot=True
  time.sleep(.025)
 if overlay.poll()is None:overlay.terminate();raise AssertionError('overlay probe did not exit')
 time.sleep(.15)
 for proc in processes:
  if proc.poll()is None:proc.terminate()
 for proc in processes:
  try:proc.wait(timeout=3)
  except subprocess.TimeoutExpired:proc.kill();proc.wait()
 for f in logs:f.close()
 assert overlay.returncode==0,(case/'overlay.log').read_text()
 game_report=json.loads((case/'game.json').read_text());overlay_report=json.loads((case/('negative.json'if negative else'overlay.json')).read_text())
 violations=[s for s in samples if not any(n['app_id']=='observatory-game-probe'for n in s)]
 report={'scale':a.scale,'samples':len(samples),'focus_violations':len(violations),'first_violation':violations[0]if violations else None,'game':game_report,'overlay':overlay_report}
 if negative:assert violations,'negative control did not detect xdg_toplevel focus theft'
 else:
  assert len(samples)>70 and not violations,report
  assert game_report['clicks']>=5,game_report
  assert all(c in game_report['text']for c in 'abcd'),game_report
  assert overlay_report['max_layers']>=4,overlay_report
  assert overlay_report.get('foreign_seen_game') and overlay_report.get('foreign_seen_other'),overlay_report
  from PIL import Image
  image=Image.open(case/'compositor.png').convert('RGB');pixels=list(image.getdata());magenta=sum(r>210 and g<70 and b>130 for r,g,b in pixels);green=sum(g>180 and r<100 and b<150 for r,g,b in pixels)
  report.update(bitmap_magenta_pixels=magenta,green_pixels=green)
  assert magenta>1000 and green>1000,'real PNG did not render: '+str(report)
  for capture_name in ['StellarOverlay','StellarOverlay-Radar','Core-Notification']:
   path=case/(capture_name+'.png');assert path.exists(),'native callback capture missing: '+str(path)
   img=Image.open(path).convert('RGBA');assert sum(1 for r,g,b,alpha in img.getdata()if alpha and max(r,g,b)>30)>100,'blank native capture: '+str(path)
 if not negative:
  core=Image.open(case/'Core-Notification.png').convert('RGB');core=core.crop((8,8,core.width-8,core.height-8));core_text=sum(r>150 and g>80 and b<80 for r,g,b in core.getdata());radar=Image.open(case/'StellarOverlay-Radar.png').convert('RGB');radar_text=sum(r>220 and g>220 and b>220 for r,g,b in radar.getdata());assert core_text>50 and radar_text>50,('missing native font glyphs',core_text,radar_text)
  report.update(core_text_pixels=core_text,radar_white_text_pixels=radar_text)
 (case/'result.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'case':name,**report}),flush=True);return report
ipc('output HEADLESS-1 scale '+str(a.scale))
results={'passive':run_case(False),'negative_control':run_case(True)}
(a.output/'result.json').write_text(json.dumps(results,indent=2)+'\n')
