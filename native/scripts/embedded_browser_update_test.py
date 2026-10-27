#!/usr/bin/env python3
"""Exercise the actual host DOM update in WebKit on a private Xvfb display."""
import argparse
import hashlib
import json
import os
from pathlib import Path
from x11_qa import PrivateDisplay


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    args.output.mkdir(parents=True,exist_ok=True)
    script_path=Path(__file__).parents[1]/'observatory-desktop/src/embedded_browser_update.js'
    script=script_path.read_text(encoding='utf-8')
    host_path=script_path.with_name('embedded_browser.js')
    host_script=host_path.read_text(encoding='utf-8').replace('$$POPOUT$$','true').replace('$$HIDE_BUTTONS$$','false')
    private=PrivateDisplay()
    try:
        os.environ.update(private.env)
        import gi
        gi.require_version('Gtk','3.0')
        gi.require_version('WebKit2','4.1')
        from gi.repository import Gtk,WebKit2,GLib
        view=WebKit2.WebView()
        window=Gtk.Window()
        window.set_default_size(800,600)
        window.add(view)
        initial='<html><head></head><body><div class="buttonheader"></div><div id="maincontent" style="height:3000px"><img id="galleryimage" data-loaded="1" src="data:image/svg+xml,original"/><span id="gallerycredit">Original selected credit</span><span id="mark">Off</span></div></body></html>'
        updated='<html><head><style>body{color:rgb(1,2,3)}</style></head><body><div id="maincontent" style="height:3000px"><img id="galleryimage" src="data:image/svg+xml,fallback"/><span id="gallerycredit"></span><span id="mark">On</span><script>window.replacedScripts=(window.replacedScripts||0)+1;if(!document.getElementById("galleryimage").dataset.loaded)window.galleryReloaded=true;</script></div></body></html>'
        evaluate=host_script+";for(const id of ['native-zoom-menu','observatory-popout-menu']){const node=document.createElement('div');node.id=id;document.body.appendChild(node);}window.dispatchEvent(new Event('blur'));"+"window.oldImage=document.getElementById('galleryimage');window.oldCredit=document.getElementById('gallerycredit');window.scrollTo(0,200);"+script.replace('$$DOCUMENT$$',json.dumps(updated))+";JSON.stringify({blurRemovedZoom:!document.getElementById('native-zoom-menu'),blurRemovedPopoutMenu:!document.getElementById('observatory-popout-menu'),imagePreserved:oldImage===document.getElementById('galleryimage'),creditPreserved:oldCredit===document.getElementById('gallerycredit'),credit:oldCredit.textContent,image:oldImage.getAttribute('src'),mark:document.getElementById('mark').textContent,scroll:scrollY,scripts:window.replacedScripts,reloaded:!!window.galleryReloaded,color:getComputedStyle(document.body).color})"
        result_data={}
        def complete(view,response,*unused):
            try:
                result_data.update(json.loads(view.evaluate_javascript_finish(response).to_string()))
            finally:
                Gtk.main_quit()
        def probe():
            view.evaluate_javascript(evaluate,-1,None,None,None,complete,None)
            return False
        def loaded(view,event):
            if event==WebKit2.LoadEvent.FINISHED:
                GLib.timeout_add(100,probe)
        view.connect('load-changed',loaded)
        window.show_all()
        view.load_html(initial,'observatory://bioinsights/')
        GLib.timeout_add_seconds(10,lambda:Gtk.main_quit())
        Gtk.main()
        window.destroy()
        assert result_data.get('blurRemovedZoom') and result_data.get('blurRemovedPopoutMenu'),result_data
        assert result_data.get('imagePreserved') and result_data.get('creditPreserved'),result_data
        assert result_data['credit']=='Original selected credit',result_data
        assert result_data['image']=='data:image/svg+xml,original',result_data
        assert result_data['mark']=='On' and result_data['scroll']==200,result_data
        assert result_data['scripts']==1 and not result_data['reloaded'],result_data
        assert result_data['color']=='rgb(1, 2, 3)',result_data
        receipt={'scriptSha256':hashlib.sha256(script.encode()).hexdigest(),'hostScriptSha256':hashlib.sha256(host_script.encode()).hexdigest(),'checks':result_data}
        (args.output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
        print(json.dumps(receipt))
    finally:
        private.close()


if __name__=='__main__':
    main()
