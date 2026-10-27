#!/usr/bin/env python3
"""Capture actual source Codex CSS geometry, never reconstructed native HTML."""
import argparse,json,os,re,tempfile,subprocess,hashlib
from pathlib import Path
from bioinsights_dom_probe import worker_html
from native_update_test import Profile
from x11_qa import PrivateDisplay

SCRIPT=r'''(()=>{
 const rect=e=>e.getBoundingClientRect().toJSON();
 const info=e=>{let s=getComputedStyle(e);return {id:e.id,tag:e.tagName,text:e.tagName==='SCRIPT'?'':e.textContent.slice(0,160),rect:rect(e),font:s.fontFamily,size:s.fontSize,weight:s.fontWeight,line:s.lineHeight,padding:s.padding,align:s.verticalAlign};};
 const tables=[...document.querySelectorAll('table')];
 const out={width:innerWidth,media:[...document.querySelectorAll('img,canvas')].map(info),content:info(document.getElementById('maincontent')),tables:tables.map(t=>({rect:rect(t),rows:[...t.rows].map(r=>({rect:rect(r),cells:[...r.cells].map(c=>({...info(c),span:c.colSpan,children:[...c.children].filter(e=>e.tagName!=='SCRIPT').map(info)}))}))})),back:[...document.querySelectorAll('.close')].map(info)};
 const t=tables[0];t.style.width='1px';out.minimum={rect:rect(t),top:[...t.rows[3].cells].map(info),variants:[...tables[1].rows[1].cells].map(info)};
 return JSON.stringify(out);
})()'''

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('worker',type=Path);p.add_argument('--output',type=Path,required=True);p.add_argument('--page',default='codexcompanion/2/11');p.add_argument('--width',type=int,default=1168);p.add_argument('--windows-fonts',action='store_true');p.add_argument('--theme-index',type=int,choices=[0,1],default=1);p.add_argument('--gallery-caption',action='store_true');args=p.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 private=PrivateDisplay()
 try:
  with tempfile.TemporaryDirectory(dir=args.output) as temporary:
   root=Path(temporary);profile=Profile(root,'source');profile.settings['plugins']['BioInsights']['ThemeSettings']={'ThemeIndex':args.theme_index};html=worker_html(args.worker.resolve(),profile.settings['plugins']['BioInsights'],root,args.page)
   if args.gallery_caption:
    caption='<br /><span class="h3cc">Synthetic Site</span><br /><i>CMDR Synthetic Commander</i>'
    html,count=re.subn(r'(<img\b(?=[^>]*\bid="galleryimage")[^>]*>)',lambda match:match.group(1)+caption,html,count=1)
    assert count==1,'Original gallery image not found'
   # Explicit original fallback-gallery state: metadata is absent. Disable only
   # the asynchronous chooser script so its later network result cannot change
   # this deterministic CSS measurement; markup/image dimensions are untouched.
   html=re.sub(r'<script>\(function\(\)\{let image=.*?</script>','',html,flags=re.S)
   os.environ.update(private.env)
   if args.windows_fonts:
    config=root/'fontconfig.conf'
    config.write_text(f'<?xml version="1.0"?><!DOCTYPE fontconfig SYSTEM "fonts.dtd"><fontconfig><include>/etc/fonts/fonts.conf</include><dir>/mnt/c/Windows/Fonts</dir><cachedir>{root/"font-cache"}</cachedir></fontconfig>',encoding='utf-8')
    os.environ['FONTCONFIG_FILE']=str(config)
   font_environment=[]
   for requested in ['Verdana','Verdana:style=Bold','Segoe UI Emoji','Segoe UI Emoji:style=Bold']:
    resolved=subprocess.check_output(['fc-match','-f','%{family}|%{file}',requested],text=True).split('|',1)
    font_environment.append({'requested':requested,'family':resolved[0],'file':resolved[1],'sha256':hashlib.sha256(Path(resolved[1]).read_bytes()).hexdigest()})
   import gi
   gi.require_version('Gtk','3.0');gi.require_version('WebKit2','4.1')
   from gi.repository import Gtk,WebKit2,GLib
   view=WebKit2.WebView();window=Gtk.Window();window.set_default_size(args.width,900);window.add(view);data={}
   def complete(view,result,*unused):
    try:data.update(json.loads(view.evaluate_javascript_finish(result).to_string()))
    finally:Gtk.main_quit()
   def probe():view.evaluate_javascript(SCRIPT,-1,None,None,None,complete,None);return False
   def loaded(view,event):
    if event==WebKit2.LoadEvent.FINISHED:GLib.timeout_add(100,probe)
   view.connect('load-changed',loaded);window.show_all();view.load_html(html,'observatory://bioinsights/');GLib.timeout_add_seconds(15,lambda:Gtk.main_quit());Gtk.main();window.destroy();assert data,'WebKit timed out'
   data['theme_index']=args.theme_index
   data['gallery_caption']=args.gallery_caption
   data['font_environment']={'mode':'system+Windows' if args.windows_fonts else 'system','faces':font_environment};data['page']=args.page;data['gallery']='original fallback';(args.output/'dom.json').write_text(json.dumps(data,indent=2)+'\n',encoding='utf-8')
   print(json.dumps({'page':args.page,'width':args.width,'table':data['tables'][0]['rect'],'minimum':data['minimum']['rect'],'cells':[(c['rect']['width']) for c in data['tables'][0]['rows'][3]['cells']],'minimumCells':[(c['rect']['width'])for c in data['minimum']['top']]}))
 finally:private.close()

if __name__=='__main__':main()
