#!/usr/bin/env python3
"""Inspect actual WebKit source CSS layout on an owned private Xvfb display."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile
from native_update_test import Profile
from plugin_smoke_test import fixture
from x11_qa import PrivateDisplay


def worker_html(binary, settings, directory, page):
    process=subprocess.Popen([str(binary),"--kind","BioInsights"],stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True)
    counter=0
    def call(method,args=None):
        nonlocal counter
        counter+=1
        process.stdin.write(json.dumps({"protocol":1,"id":counter,"now_ms":1000000,"method":method,"args":args or {}})+"\n")
        process.stdin.flush()
        response=json.loads(process.stdout.readline())
        assert response.get("error") is None,response
        return response["result"]
    try:
        call("initialize",{"settings":settings,"data_dir":str(directory)})
        call("on_peer_message",{"sender":"Hecate","command":"Ready","payload":{"Version":"1.3.444.761"}})
        call("set_batch",{"batch":True})
        call("on_events",{"events":[{"event":event["event"],"timestamp":event["timestamp"],"data":event} for event in fixture()]})
        call("set_batch",{"batch":False})
        call("on_message",{"command":"BrowserNavigate","payload":{"Url":"observatory://bioinsights/"+page}})
        return call("browser_html")
    finally:
        process.terminate()
        process.wait(timeout=10)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("worker",type=Path)
    parser.add_argument("--output",type=Path,required=True)
    parser.add_argument("--width",type=int,default=1168)
    parser.add_argument("--page",default="main")
    parser.add_argument("--windows-fonts",action="store_true")
    args=parser.parse_args()
    args.output.mkdir(parents=True,exist_ok=True)
    private=PrivateDisplay()
    try:
        with tempfile.TemporaryDirectory(dir=args.output) as temporary:
            root=Path(temporary)
            profile=Profile(root,"probe")
            html=worker_html(args.worker.resolve(),profile.settings["plugins"]["BioInsights"],root,args.page)
            (args.output/"source.html").write_text(html)
            os.environ.update(private.env)
            if args.windows_fonts:
                config=root/"fontconfig.conf"
                config.write_text(f'<?xml version="1.0"?><!DOCTYPE fontconfig SYSTEM "fonts.dtd"><fontconfig><include>/etc/fonts/fonts.conf</include><dir>/mnt/c/Windows/Fonts</dir><cachedir>{root/"font-cache"}</cachedir></fontconfig>')
                os.environ["FONTCONFIG_FILE"]=str(config)
            import gi
            gi.require_version("Gtk","3.0")
            gi.require_version("WebKit2","4.1")
            from gi.repository import Gtk,WebKit2,GLib
            view=WebKit2.WebView()
            window=Gtk.Window()
            window.set_default_size(args.width,700)
            window.add(view)
            script="""(()=>{const canvas=document.createElement('canvas').getContext('2d');const metrics={},textMetrics={};for(const family of ['serif','sans-serif','Segoe UI Emoji','Verdana','Arial','DejaVu Sans','Liberation Sans']){canvas.font='15px "'+family+'"';metrics[family]=canvas.measureText('Aurasus').width;textMetrics[family]=Object.fromEntries(['Aurasus','Bacterium','20.01M','1M','500m','Native Smoke A 1'].map(t=>[t,canvas.measureText(t).width]));}const result={width:innerWidth,metrics,textMetrics,headings:[...document.querySelectorAll('h1,h2,h3')].map(e=>({text:e.textContent,rect:e.getBoundingClientRect().toJSON(),font:getComputedStyle(e).font,size:getComputedStyle(e).fontSize,margin:getComputedStyle(e).margin})),tables:[...document.querySelectorAll('table')].map(e=>({classes:e.className,rect:e.getBoundingClientRect().toJSON(),padding:getComputedStyle(e).padding,margin:getComputedStyle(e).margin})),rows:[...document.querySelectorAll('table tr')].map(tr=>({rect:tr.getBoundingClientRect().toJSON(),cells:[...tr.children].map(e=>{const s=getComputedStyle(e),r=e.getBoundingClientRect();return {text:e.textContent,span:e.colSpan,rect:r.toJSON(),font:s.fontFamily,size:s.fontSize,weight:s.fontWeight,padding:s.padding,fontStyle:s.fontStyle,children:[...e.children].map(c=>({tag:c.tagName,text:c.textContent,font:getComputedStyle(c).fontFamily,fontStyle:getComputedStyle(c).fontStyle,rect:c.getBoundingClientRect().toJSON()}))}})}))};const table=[...document.querySelectorAll('table')].find(t=>t.rows.length>4)||document.querySelector('table');table.style.width='1px';result.intrinsic=[...(table.rows[4]||table.rows[0]).cells].map(e=>({text:e.textContent,span:e.colSpan,width:e.getBoundingClientRect().width}));return JSON.stringify(result);})()"""
            def result(view,response,*unused):
                try:
                    value=view.evaluate_javascript_finish(response).to_string()
                    data=json.loads(value)
                    (args.output/"dom.json").write_text(json.dumps(data,indent=2)+"\n")
                    print(json.dumps(data),flush=True)
                finally:
                    Gtk.main_quit()
            def probe():
                view.evaluate_javascript(script,-1,None,None,None,result,None)
                return False
            def loaded(view,event):
                if event==WebKit2.LoadEvent.FINISHED:
                    GLib.timeout_add(400,probe)
            view.connect("load-changed",loaded)
            window.show_all()
            view.load_html(html,"observatory://bioinsights/")
            GLib.timeout_add_seconds(15,lambda:Gtk.main_quit())
            Gtk.main()
            window.destroy()
            assert (args.output/"dom.json").is_file(),"WebKit probe timed out"
    finally:
        private.close()


if __name__=="__main__":
    main()
