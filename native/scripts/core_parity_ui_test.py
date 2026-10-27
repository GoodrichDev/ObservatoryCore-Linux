#!/usr/bin/env python3
"""Exercise real Core widgets and GTK export in an isolated Xvfb display.

Uses disposable HOME/XDG roots and synthetic Lua rows; never controls the user's
session. The receipt distinguishes source semantic checks from screenshot
capture. No screenshot alone establishes pixel-for-pixel Windows parity.
"""
import argparse
import ctypes as c
import hashlib
import json
import os
import signal
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import time
import zipfile
import zlib


def eventually(predicate, description, seconds=30):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        value = predicate()
        if value:
            return value
        time.sleep(0.1)
    raise AssertionError(description)


class X11:
    class Attributes(c.Structure):
        _fields_ = [("x", c.c_int), ("y", c.c_int), ("width", c.c_int), ("height", c.c_int),
                    ("border_width", c.c_int), ("depth", c.c_int), ("visual", c.c_void_p),
                    ("root", c.c_ulong), ("window_class", c.c_int), ("bit_gravity", c.c_int),
                    ("win_gravity", c.c_int), ("backing_store", c.c_int), ("backing_planes", c.c_ulong),
                    ("backing_pixel", c.c_ulong), ("save_under", c.c_int), ("colormap", c.c_ulong),
                    ("map_installed", c.c_int), ("map_state", c.c_int), ("all_event_masks", c.c_long),
                    ("your_event_mask", c.c_long), ("do_not_propagate_mask", c.c_long),
                    ("override_redirect", c.c_int), ("screen", c.c_void_p)]

    class Image(c.Structure):
        _fields_ = [("width", c.c_int), ("height", c.c_int), ("xoffset", c.c_int), ("format", c.c_int),
                    ("data", c.c_void_p), ("byte_order", c.c_int), ("bitmap_unit", c.c_int),
                    ("bitmap_bit_order", c.c_int), ("bitmap_pad", c.c_int), ("depth", c.c_int),
                    ("bytes_per_line", c.c_int), ("bits_per_pixel", c.c_int)]

    class Client(c.Structure):
        _fields_ = [("type", c.c_int), ("serial", c.c_ulong), ("send_event", c.c_int),
                    ("display", c.c_void_p), ("window", c.c_ulong), ("message_type", c.c_ulong),
                    ("format", c.c_int), ("data", c.c_long * 5)]

    def __init__(self):
        self.x = c.CDLL("libX11.so.6")
        self.test = c.CDLL("libXtst.so.6")
        d, w = c.c_void_p, c.c_ulong
        for name, types, result in [
            ("XOpenDisplay", [c.c_char_p], d),
            ("XDefaultRootWindow", [d], w),
            ("XQueryTree", [d,w,c.POINTER(w),c.POINTER(w),c.POINTER(c.POINTER(w)),c.POINTER(c.c_uint)], c.c_int),
            ("XFetchName", [d,w,c.POINTER(c.c_char_p)], c.c_int),
            ("XGetWindowAttributes", [d,w,c.POINTER(self.Attributes)], c.c_int),
            ("XGetImage", [d,w,c.c_int,c.c_int,c.c_uint,c.c_uint,c.c_ulong,c.c_int], c.POINTER(self.Image)),
            ("XDestroyImage", [c.POINTER(self.Image)], c.c_int),
            ("XFree", [c.c_void_p], c.c_int),
            ("XRaiseWindow", [d,w], c.c_int),
            ("XSetInputFocus", [d,w,c.c_int,c.c_ulong], c.c_int),
            ("XSync", [d,c.c_int], c.c_int),
            ("XTranslateCoordinates", [d,w,w,c.c_int,c.c_int,c.POINTER(c.c_int),c.POINTER(c.c_int),c.POINTER(w)], c.c_int),
            ("XStringToKeysym", [c.c_char_p], c.c_ulong),
            ("XKeysymToKeycode", [d,c.c_ulong], c.c_uint),
            ("XInternAtom", [d,c.c_char_p,c.c_int], c.c_ulong),
            ("XSendEvent", [d,w,c.c_int,c.c_long,c.c_void_p], c.c_int),
            ("XMoveResizeWindow", [d,w,c.c_int,c.c_int,c.c_uint,c.c_uint], c.c_int),
            ("XCreateSimpleWindow",[d,w,c.c_int,c.c_int,c.c_uint,c.c_uint,c.c_uint,c.c_ulong,c.c_ulong],w),
            ("XConvertSelection",[d,c.c_ulong,c.c_ulong,c.c_ulong,w,c.c_ulong],c.c_int),
            ("XGetWindowProperty",[d,w,c.c_ulong,c.c_long,c.c_long,c.c_int,c.c_ulong,c.POINTER(c.c_ulong),c.POINTER(c.c_int),c.POINTER(c.c_ulong),c.POINTER(c.c_ulong),c.POINTER(c.POINTER(c.c_ubyte))],c.c_int),
            ("XDestroyWindow",[d,w],c.c_int),
        ]:
            fn = getattr(self.x, name)
            fn.argtypes, fn.restype = types, result
        self.test.XTestFakeMotionEvent.argtypes = [d,c.c_int,c.c_int,c.c_int,c.c_ulong]
        self.test.XTestFakeButtonEvent.argtypes = [d,c.c_uint,c.c_int,c.c_ulong]
        self.test.XTestFakeKeyEvent.argtypes = [d,c.c_uint,c.c_int,c.c_ulong]
        self.display = self.x.XOpenDisplay(None)
        assert self.display, "Cannot open the private Xvfb display"
        self.root = self.x.XDefaultRootWindow(self.display)
        error_type = c.CFUNCTYPE(c.c_int,d,c.c_void_p)
        self.error = error_type(lambda *_: 0)
        self.x.XSetErrorHandler.argtypes = [error_type]
        self.x.XSetErrorHandler(self.error)

    def geometry(self, window):
        a = self.Attributes()
        assert self.x.XGetWindowAttributes(self.display,window,c.byref(a))
        ox,oy,child = c.c_int(),c.c_int(),c.c_ulong()
        self.x.XTranslateCoordinates(self.display,window,self.root,0,0,c.byref(ox),c.byref(oy),c.byref(child))
        return [ox.value,oy.value,a.width,a.height]

    def windows(self):
        top,parent = c.c_ulong(),c.c_ulong()
        children,count = c.POINTER(c.c_ulong)(),c.c_uint()
        self.x.XQueryTree(self.display,self.root,c.byref(top),c.byref(parent),c.byref(children),c.byref(count))
        found = {}
        for index in range(count.value):
            child = children[index]
            a,name = self.Attributes(),c.c_char_p()
            if self.x.XGetWindowAttributes(self.display,child,c.byref(a)) and a.map_state == 2:
                if self.x.XFetchName(self.display,child,c.byref(name)) and name.value:
                    found[int(child)] = name.value.decode("utf-8",errors="replace")
                    self.x.XFree(name)
        if children:
            self.x.XFree(children)
        return found

    def wait_window(self, title):
        return eventually(lambda: next((w for w,t in self.windows().items() if title in t),None),
                          f"Missing window {title!r}; visible={self.windows()}")

    def focus(self, window):
        self.x.XRaiseWindow(self.display,window)
        self.x.XSetInputFocus(self.display,window,2,0)
        self.x.XSync(self.display,0)

    def move(self, window, px, py):
        ox,oy,_,_ = self.geometry(window)
        self.test.XTestFakeMotionEvent(self.display,-1,ox+int(px),oy+int(py),0)
        self.x.XSync(self.display,0)

    def click(self, window, px, py, button=1):
        self.focus(window)
        self.move(window,px,py)
        time.sleep(0.12)
        self.test.XTestFakeButtonEvent(self.display,button,1,0)
        self.x.XSync(self.display,0)
        time.sleep(0.08)
        self.test.XTestFakeButtonEvent(self.display,button,0,0)
        self.x.XSync(self.display,0)
        time.sleep(0.3)

    def drag(self,window,start,end):
        self.focus(window)
        self.move(window,*start)
        self.test.XTestFakeButtonEvent(self.display,1,1,0)
        self.x.XSync(self.display,0)
        time.sleep(0.15)
        for step in range(1,11):
            self.move(window,start[0]+(end[0]-start[0])*step/10,start[1]+(end[1]-start[1])*step/10)
            time.sleep(0.08)
        self.test.XTestFakeButtonEvent(self.display,1,0,0)
        self.x.XSync(self.display,0)
        time.sleep(0.5)

    def key(self, name, pressed=True):
        symbol = self.x.XStringToKeysym(name.encode("ascii"))
        assert symbol, name
        code = self.x.XKeysymToKeycode(self.display,symbol)
        assert code, name
        self.test.XTestFakeKeyEvent(self.display,code,int(pressed),0)
        self.x.XSync(self.display,0)

    def press(self, name):
        self.key(name)
        self.key(name,False)
        time.sleep(0.12)

    def chord(self, modifier, name):
        self.key(modifier)
        self.press(name)
        self.key(modifier,False)
        time.sleep(0.15)

    def text(self, text):
        punctuation = {"/":"slash",".":"period","-":"minus","_":"underscore",":":"colon"," ":"space"}
        for letter in text:
            name = punctuation.get(letter,letter)
            shifted = letter in "_:" or letter.isupper()
            if shifted:
                self.key("Shift_L")
            token = name.lower() if letter.isupper() else name
            self.key(token)
            self.key(token,False)
            time.sleep(0.004)
            if shifted:
                self.key("Shift_L",False)

    def clipboard(self):
        window=self.x.XCreateSimpleWindow(self.display,self.root,0,0,1,1,0,0,0)
        selection=self.x.XInternAtom(self.display,b"CLIPBOARD",False)
        target=self.x.XInternAtom(self.display,b"UTF8_STRING",False)
        prop=self.x.XInternAtom(self.display,b"OBSERVATORY_TEST_CLIPBOARD",False)
        self.x.XConvertSelection(self.display,selection,target,prop,window,0)
        self.x.XSync(self.display,0)
        def read():
            kind,format,items,remaining=c.c_ulong(),c.c_int(),c.c_ulong(),c.c_ulong()
            data=c.POINTER(c.c_ubyte)()
            self.x.XGetWindowProperty(self.display,window,prop,0,1048576,False,0,c.byref(kind),c.byref(format),c.byref(items),c.byref(remaining),c.byref(data))
            value=None
            if kind.value and format.value==8 and data:
                value=c.string_at(data,items.value).decode("utf-8")
            if data:self.x.XFree(data)
            return value
        try:return eventually(read,"Selected grid rows must reach the real private X11 clipboard")
        finally:self.x.XDestroyWindow(self.display,window)

    def pixel(self, window, px, py):
        image=self.x.XGetImage(self.display,window,px,py,1,1,c.c_ulong(-1).value,2)
        assert image and image.contents.bits_per_pixel==32
        pixel=c.string_at(image.contents.data,4)
        self.x.XDestroyImage(image)
        return list(pixel[2::-1])

    def close(self, window):
        event = self.Client()
        event.type,event.display,event.window,event.format = 33,self.display,window,32
        event.message_type = self.x.XInternAtom(self.display,b"WM_PROTOCOLS",False)
        event.data[0] = self.x.XInternAtom(self.display,b"WM_DELETE_WINDOW",False)
        event.data[1] = 0
        self.x.XSendEvent(self.display,window,False,0,c.byref(event))
        self.x.XSync(self.display,0)
        time.sleep(0.4)

    def capture(self, window, path):
        _,_,width,height = self.geometry(window)
        image = self.x.XGetImage(self.display,window,0,0,width,height,c.c_ulong(-1).value,2)
        assert image and image.contents.bits_per_pixel == 32
        info = image.contents
        raw = c.string_at(info.data,info.bytes_per_line*height)
        rows = []
        for y in range(height):
            row = raw[y*info.bytes_per_line:y*info.bytes_per_line+width*4]
            rgb = bytearray(width*3)
            rgb[0::3],rgb[1::3],rgb[2::3] = row[2::4],row[1::4],row[0::4]
            rows.append(b"\0"+rgb)
        def chunk(kind,data):
            return struct.pack(">I",len(data))+kind+data+struct.pack(">I",zlib.crc32(kind+data))
        path.parent.mkdir(parents=True,exist_ok=True)
        path.write_bytes(b"\x89PNG\r\n\x1a\n"+chunk(b"IHDR",struct.pack(">IIBBBBB",width,height,8,2,0,0,0))+
                         chunk(b"IDAT",zlib.compress(b"".join(rows)))+chunk(b"IEND",b""))
        self.x.XDestroyImage(image)
        return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary",type=Path)
    parser.add_argument("--output",type=Path,required=True)
    parser.add_argument("--capture-only",action="store_true",help="Diagnostic capture; does not claim interaction coverage")
    args = parser.parse_args()
    if not os.environ.get("OBSERVATORY_PRIVATE_XVFB"):
        env = os.environ.copy()
        env["OBSERVATORY_PRIVATE_XVFB"] = "1"
        env.pop("WAYLAND_DISPLAY",None)
        raise SystemExit(subprocess.call(["xvfb-run","-a","-s","-screen 0 1600x1100x24",sys.executable,
                                          str(Path(__file__).resolve()),*sys.argv[1:]],env=env))
    args.binary,args.output = args.binary.resolve(),args.output.resolve()
    args.output.parent.mkdir(parents=True,exist_ok=True)
    receipt = {"backend":"private X11/XTest/GTK","synthetic_only":True,
               "binary_sha256":hashlib.sha256(args.binary.read_bytes()).hexdigest(),"checks":[],"captures":{}}
    x = X11()
    with tempfile.TemporaryDirectory(prefix="observatory-core-ui-") as temporary:
        profile = Path(temporary)
        env = os.environ.copy()
        env.pop("WAYLAND_DISPLAY",None)
        runtime = profile/"run"
        runtime.mkdir(mode=0o700)
        env.update(HOME=str(profile),LIBGL_ALWAYS_SOFTWARE="1",GTK_USE_PORTAL="0",GDK_BACKEND="x11",
                   XDG_RUNTIME_DIR=str(runtime),GSETTINGS_BACKEND="memory",NO_AT_BRIDGE="1",
                   DBUS_SESSION_BUS_ADDRESS="",LANG="C.UTF-8")
        for kind in ("CONFIG","DATA","CACHE"):
            env[f"XDG_{kind}_HOME"] = str(profile/kind.lower())
        data = profile/"data"/"observatory-core"
        plugins = data/"plugins"
        plugins.mkdir(parents=True)
        (profile/"Documents").mkdir()
        journals = profile/"journals"
        journals.mkdir()
        (journals/"Journal.2026-10-26T000000.01.log").write_text(json.dumps({"timestamp":"2026-10-26T00:00:00Z","event":"Fileheader","gameversion":"4.0.0.0"})+"\n",encoding="utf-8")
        (plugins/"core-parity.lua").write_text("""return {name='Core Parity',id='test.core.parity',version='1.0.0',settings={Enabled=true},
on_ready=function()core.add_row{Title='Beta',Count='2',Detail='second'};core.add_row{Title='Alpha',Count='10',Detail='first'}end}
""",encoding="utf-8")
        config = profile/"settings.json"
        settings = {"journal_dir":str(journals),"auto_start":False,"start_read_all":False,
                    "popup":False,"speech":False,"theme":"dark","ExportFormat":0,
                    "tab_order":["Core","Core Parity","Explorer","Botanist","Communicator","BioInsights","Evaluator"],
                    "plugins":{"Hecate":{"RemoteLookups":False},"BioInsights":{"GalacticCodexRemote":False,
                    "StellarOverlayUseRadar":False,"StellarOverlaySendFSSNotification":False,
                    "StellarOverlaySendDSSNotification":False},"StellarOverlay":{"Enabled":False}}}
        config.write_text(json.dumps(settings),encoding="utf-8")
        log_path = args.output.with_suffix(".log")
        with log_path.open("w+",encoding="utf-8") as log:
            command = [str(args.binary),"--config",str(config),"--view","Core Parity","--window-size","1200x800"]
            if os.environ.get("OBSERVATORY_UI_GDB"):
                command = ["gdb","-batch","-ex","set pagination off","-ex",
                           "python import threading,os,signal; threading.Timer(8,lambda:os.kill(os.getpid(),signal.SIGINT)).start()",
                           "-ex","run","-ex","thread apply all bt 15","--args",*command]
            if os.environ.get("OBSERVATORY_UI_STRACE"):
                command = ["strace","-f","-o",str(args.output.with_suffix(".strace")),*command]
            process = subprocess.Popen(command,env=env,stdout=log,stderr=log,start_new_session=True)
            try:
                main_window = x.wait_window("Elite Observatory Core")
                eventually(lambda: len(x.windows())==1,"Only main window on startup")
                time.sleep(1.5)
                receipt["main_geometry"] = x.geometry(main_window)
                receipt["captures"]["initial"] = x.capture(main_window,args.output.with_name(args.output.stem+"-initial.png"))
                assert receipt["main_geometry"][2:] == [1200,800],receipt["main_geometry"]
                receipt["checks"].append("requested client geometry 1200x800")
                if not args.capture_only:
                    run_interactions(x,main_window,config,args,receipt)
                x.close(main_window)
                process.wait(timeout=10)
                assert process.returncode==0,process.returncode
                receipt["settings_after_exit"] = json.loads(config.read_text(encoding="utf-8"))
                if not args.capture_only:
                    saved=receipt["settings_after_exit"]
                    assert saved["tab_order"].index("Explorer")<saved["tab_order"].index("Core Parity"),saved["tab_order"]
                    assert saved["window_size"]==[1000.0,650.0],saved["window_size"]
                    assert saved["window_position"]==[70.0,50.0],saved["window_position"]
                    assert saved["plugin_windows"]["Core Parity"]["open"]
                    process=subprocess.Popen([str(args.binary),"--config",str(config)],env=env,stdout=log,stderr=log,start_new_session=True)
                    restored=x.wait_window("Elite Observatory Core")
                    popout=x.wait_window("Core Parity")
                    assert x.geometry(restored)==[70,50,1000,650],x.geometry(restored)
                    assert x.geometry(popout)==[190,140,730,480],x.geometry(popout)
                    receipt["captures"]["restart-main"]=x.capture(restored,args.output.with_name(args.output.stem+"-restart-main.png"))
                    receipt["captures"]["restart-popout"]=x.capture(popout,args.output.with_name(args.output.stem+"-restart-popout.png"))
                    receipt["checks"].append("actual tab drag and main geometry persist; restart reopens original generic popout geometry")
                    x.close(restored)
                    process.wait(timeout=10)
                    assert process.returncode==0

            except Exception:
                log.flush()
                log.seek(0)
                print(log.read(),file=sys.stderr)
                receipt["process_status_on_failure"] = process.poll()
                receipt["pid_on_failure"] = process.pid
                if os.environ.get("OBSERVATORY_UI_BACKTRACE") and process.poll() is None:
                    trace = subprocess.run(["gdb","-batch","-ex","set pagination off","-ex","thread apply all bt 12","-p",str(process.pid)],capture_output=True,text=True,timeout=15)
                    args.output.with_suffix(".backtrace").write_text(trace.stdout+trace.stderr,encoding="utf-8")
                for name in ("wchan","stack","status"):
                    try: receipt["process_"+name] = Path(f"/proc/{process.pid}/{name}").read_text()
                    except OSError: pass
                receipt["visible_windows_on_failure"] = x.windows()
                for window,title in x.windows().items():
                    x.capture(window,args.output.with_name(args.output.stem+f"-failure-{window}.png"))
                raise
            finally:
                if process.poll() is None:
                    os.killpg(process.pid,signal.SIGTERM)
                    try:
                        process.wait(timeout=8)
                    except subprocess.TimeoutExpired:
                        os.killpg(process.pid,signal.SIGKILL)
                        process.wait(timeout=3)
                args.output.write_text(json.dumps(receipt,indent=2)+"\n",encoding="utf-8")
    print(json.dumps({"checks":receipt["checks"],"receipt":str(args.output)}))


def run_interactions(x, main_window, config, args, receipt):
    def capture(name,window=main_window):
        receipt["captures"][name] = x.capture(window,args.output.with_name(args.output.stem+"-"+name+".png"))

    def exported(name):
        destination = config.parent/name
        _,_,width,height = x.geometry(main_window)
        x.click(main_window,width-314,height-23)
        chooser = x.wait_window("Export")
        assert chooser != main_window
        capture("chooser-"+name.replace(".","-"),chooser)
        x.focus(chooser)
        x.chord("Control_L","l")
        x.chord("Control_L","a")
        x.text(str(destination))
        x.press("Return")
        time.sleep(0.4)
        # GTK's location entry may first resolve a folder/name; Alt+S activates
        # the real native Save action without assuming theme-specific buttons.
        if not destination.exists():
            x.chord("Alt_L","s")
        eventually(destination.exists,"GTK Save must create "+str(destination))
        eventually(lambda: chooser not in x.windows(),"GTK chooser closes after saving")
        data = destination.read_bytes()
        retained = args.output.with_name(args.output.stem+"-"+name)
        retained.write_bytes(data)
        receipt.setdefault("exports",{})[name] = {"sha256":hashlib.sha256(data).hexdigest(),"bytes":len(data)}
        return data

    # Source CSV option is deliberately TSV, including final TAB and CRLF.
    initial = exported("initial.csv")
    lines = initial.decode("utf-8").splitlines()
    assert len(lines)==3,lines
    headers = lines[0].split("\t")[:-1]
    title_index = headers.index("Title")
    assert [line.split("\t")[title_index] for line in lines[1:]] == ["Beta","Alpha"],lines
    assert initial.endswith(b"\t\r\n")
    receipt["checks"].append("native GTK Save produces exact trailing-TAB/CRLF source CSV and initial append order")

    # With the saved tab order, Core Parity follows Core; use its actual header
    # at the right edge of the three-column synthetic grid to sort Title.
    x.click(main_window,145,47)
    sorted_csv = exported("sorted.csv")
    lines = sorted_csv.decode("utf-8").splitlines()
    assert [line.split("\t")[title_index] for line in lines[1:]] == ["Alpha","Beta"],lines
    x.click(main_window,145,47)
    descending = exported("descending.csv").decode("utf-8").splitlines()
    assert [line.split("\t")[title_index] for line in descending[1:]] == ["Beta","Alpha"],descending
    receipt["checks"].append("actual header clicks first sort ascending and then descending")
    x.click(main_window,25,67)
    x.key("Control_L")
    x.click(main_window,25,86)
    x.key("Control_L",False)
    x.chord("Control_L","c")
    expected="\r\n".join(descending[1:])+"\r\n"
    assert x.clipboard()==expected,(x.clipboard(),expected)
    x.click(main_window,25,67)
    x.key("Control_L");x.key("Shift_L");x.press("c");x.key("Shift_L",False);x.key("Control_L",False)
    assert x.clipboard()==expected,"Ctrl+Shift+C must not copy in the original grid"
    receipt["checks"].append("actual full-row Ctrl multiselect copies trailing-TAB/CRLF rows; Ctrl+Shift+C leaves clipboard unchanged")


    # The tab context must put Popout before Settings/About/Data Folder.
    x.click(main_window,92,24,button=3)
    capture("tab-context")
    x.click(main_window,150,45)
    popout = x.wait_window("Core Parity")
    assert len(x.windows())==2,x.windows()
    x.x.XMoveResizeWindow(x.display,popout,190,140,730,480)
    x.x.XSync(x.display,0)
    time.sleep(0.6)
    capture("popout",popout)
    x.close(popout)
    eventually(lambda: len(x.windows())==1,"Close generic popout")
    x.click(main_window,92,24,button=3)
    x.click(main_window,150,45)
    restored = x.wait_window("Core Parity")
    geometry = x.geometry(restored)
    assert geometry == [190,140,730,480],geometry
    receipt["checks"].append("actual tab context opens generic popout; move/resize survives close/reopen")
    x.close(restored)

    # Core row dropdown and Core settings are separate source entry points.
    x.click(main_window,32,24)
    capture("core")
    # Source Core Settings button is anchored to the Core panel's lower right.
    _,_,width,height = x.geometry(main_window)
    x.click(main_window,width-101,height-64)
    settings = x.wait_window("Core Settings")
    capture("core-settings",settings)
    # The source Export Format dropdown's second item selects XLSX.
    x.click(settings,230,70)
    x.click(settings,230,116)
    # Source built-in theme selector: choose Light and inspect real client fill.
    x.click(settings,230,98)
    x.click(settings,230,145)
    x.close(settings)
    eventually(lambda: len(x.windows())==1,"Close Core Settings")
    receipt["checks"].append("real Core Settings window opens and closes")
    x.click(main_window,92,24)
    assert x.pixel(main_window,1,1)==[240,240,240],x.pixel(main_window,1,1)
    capture("light-theme")
    workbook=exported("selected.xlsx")
    import io
    import xml.etree.ElementTree as xml
    with zipfile.ZipFile(io.BytesIO(workbook))as archive:
        document=xml.fromstring(archive.read("xl/worksheets/sheet1.xml"))
        ns={"s":"http://schemas.openxmlformats.org/spreadsheetml/2006/main"}
        cells=document.findall(".//s:c",ns)
        assert len(cells)==9
        assert all(cell.attrib["t"]=="inlineStr"for cell in cells)
        assert [cell.find("s:is/s:t",ns).text for cell in cells]==[part for line in descending for part in line.split("\t")[:-1]]
    receipt["checks"].append("actual Core Export Format dropdown selects XLSX; GTK Save preserves all9 text cells")
    receipt["checks"].append("actual Light theme selector changes client background to source RGB240")
    x.click(main_window,32,24)
    x.click(main_window,width-101,height-64)
    settings=x.wait_window("Core Settings")
    x.click(settings,230,70);x.click(settings,230,92)
    x.click(settings,230,98);x.click(settings,230,122)
    x.close(settings)
    eventually(lambda:len(x.windows())==1,"Close restored Dark/CSV preferences")
    receipt["checks"].append("actual settings restore source Dark/CSV choices")


    # Original Read All starts immediately with a progress/cancel form.
    # Use enough real replay work for optimized binaries as well as debug builds.
    # A fast import may correctly finish before XTest observes its progress form.
    event=json.dumps({"timestamp":"2026-10-26T00:00:00Z","event":"ParityProgress"})+"\n"
    journals=Path(json.loads(config.read_text(encoding="utf-8"))["journal_dir"])
    for index in range(60):
        (journals/f"Journal.2026-10-26T{index:06}.01.log").write_text(event*20000,encoding="utf-8")
    x.click(main_window,width-49,height-23)
    read_all = eventually(lambda:next((window for window,title in x.windows().items() if "Read All In Progress" in title or "Reticulating Splines" in title),None),"Missing source Read All progress window")
    capture("read-all",read_all)
    assert x.geometry(read_all)[2:]==[395,86]
    x.click(read_all,345,67)
    eventually(lambda: len(x.windows())==1,"Cancel Read All progress")
    receipt["checks"].append("actual 395x86 Read All progress form starts and Cancel stops it")

    # Clear acts on the selected synthetic Basic grid, not a database history.
    x.click(main_window,92,24)
    x.click(main_window,width-233,height-23)
    cleared = exported("cleared.csv")
    assert cleared.decode("utf-8").splitlines() == [initial.decode("utf-8").splitlines()[0]]
    receipt["checks"].append("actual Clear removes selected grid rows while retaining headers")
    capture("cleared")
    x.drag(main_window,(92,24),(165,24))
    capture("tab-drag")
    # Leave a generic Core popout open, as source OpenPopouts does on exit.
    x.click(main_window,160,24,button=3)
    x.click(main_window,190,45)
    x.wait_window("Core Parity")
    x.x.XMoveResizeWindow(x.display,main_window,70,50,1000,650)
    x.x.XSync(x.display,0)
    time.sleep(0.6)



if __name__ == "__main__":
    main()
