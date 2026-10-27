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


def worker_html(binary, settings, directory, page, alert):
    process=subprocess.Popen([str(binary),"--kind","Evaluator"],stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True)
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
        if alert: call("on_message",{"command":"SourceUIAlert","payload":{"Type":alert}})
        (directory.parent/"projection.json").write_text(json.dumps({"summary":call("summary"),"rows":call("rows"),"settings":settings},indent=2)+"\n")
        return call("browser_html",{"page":page})
    finally:
        process.terminate()
        process.wait(timeout=10)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("worker",type=Path)
    parser.add_argument("--output",type=Path,required=True)
    parser.add_argument("--width",type=int,default=1168)
    parser.add_argument("--page",default="main")
    parser.add_argument("--alert",default="")
    parser.add_argument("--windows-fonts",action="store_true")
    parser.add_argument("--settings-json",default="{}",help="Synthetic source settings overrides")
    args=parser.parse_args()
    args.output.mkdir(parents=True,exist_ok=True)
    private=PrivateDisplay()
    try:
        with tempfile.TemporaryDirectory(dir=args.output) as temporary:
            root=Path(temporary)
            profile=Profile(root,"probe")
            profile.settings["plugins"]["Evaluator"].update(json.loads(args.settings_json))
            html=worker_html(args.worker.resolve(),profile.settings["plugins"]["Evaluator"],root,args.page,args.alert)
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
            script="""(()=>{const rect=e=>e.getBoundingClientRect().toJSON();const style=e=>{const s=getComputedStyle(e);return Object.fromEntries(['fontFamily','fontSize','fontWeight','fontStyle','padding','margin','height','minWidth','maxWidth','borderSpacing','width','boxSizing'].map(k=>[k,s[k]]))};const result={width:innerWidth,body:rect(document.body),bodyStyle:style(document.body),buttons:rect(document.querySelector('.buttonheader')),buttonStyle:style(document.querySelector('.buttonheader')),buttonCells:[...document.querySelectorAll('.buttonheader button')].map(e=>({id:e.id,rect:rect(e),style:style(e)})),content:rect(document.querySelector('.content')),tables:[...document.querySelectorAll('table')].map(table=>({rect:rect(table),style:style(table),rows:[...table.rows].map(tr=>({rect:rect(tr),cells:[...tr.cells].map(e=>({text:e.textContent,span:e.colSpan,rect:rect(e),style:style(e),children:[...e.children].map(c=>({tag:c.tagName,text:c.textContent,rect:rect(c),style:style(c)}))}))}))}))};result.alerts=[...document.querySelectorAll(".alert,.alertbtn")].map(e=>({text:e.textContent,rect:rect(e),style:style(e)}));result.extras=[...document.querySelectorAll('.thanks,.h1thanks,a')].map(e=>({text:e.textContent,rect:rect(e),style:style(e)}));const table=document.querySelector('table');if(table){table.style.width='1px';result.intrinsic=[...table.rows].map(tr=>[...tr.cells].map(e=>({text:e.textContent,span:e.colSpan,width:rect(e).width})));}return JSON.stringify(result);})()"""
            def result(view,response,*unused):
                try:
                    value=view.evaluate_javascript_finish(response).to_string()
                    data=json.loads(value)
                    (args.output/"dom.json").write_text(json.dumps(data,indent=2)+"\n")
                    print(json.dumps({"width":data["width"],"rowCount":len(data["tables"][0]["rows"]),"output":str(args.output/"dom.json")}),flush=True)
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
            view.load_html(html,"observatory://evaluator/")
            GLib.timeout_add_seconds(15,lambda:Gtk.main_quit())
            Gtk.main()
            window.destroy()
            assert (args.output/"dom.json").is_file(),"WebKit probe timed out"
    finally:
        private.close()


if __name__=="__main__":
    main()
