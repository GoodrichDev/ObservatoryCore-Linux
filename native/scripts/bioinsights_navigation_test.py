#!/usr/bin/env python3
"""Click the real release UI in a private Xvfb session, using XTest.

Run with: xvfb-run -a python3 native/scripts/bioinsights_navigation_test.py BIN
Only synthetic journals and disposable settings are used. No desktop session,
network data or installed Windows application is controlled by this test.
"""
import argparse
import ctypes as c
import hashlib
import json
import os
from pathlib import Path
import subprocess
import struct
import tempfile
import time
import zlib
from plugin_smoke_test import fixture


def main():
    args = argparse.ArgumentParser()
    args.add_argument("binary", type=Path)
    args.add_argument("--output", type=Path, required=True)
    args.add_argument("--browser",action="store_true")
    args = args.parse_args()
    x = c.CDLL("libX11.so.6")
    test = c.CDLL("libXtst.so.6")
    display_t, window_t = c.c_void_p, c.c_ulong
    x.XOpenDisplay.argtypes = [c.c_char_p]
    x.XOpenDisplay.restype = display_t
    display = x.XOpenDisplay(None)
    assert display, "A private X11 test display is required"
    x.XDefaultRootWindow.argtypes = [display_t]
    x.XDefaultRootWindow.restype = window_t
    root = x.XDefaultRootWindow(display)
    x.XQueryTree.argtypes = [display_t, window_t, c.POINTER(window_t), c.POINTER(window_t),
                            c.POINTER(c.POINTER(window_t)), c.POINTER(c.c_uint)]
    x.XFetchName.argtypes = [display_t, window_t, c.POINTER(c.c_char_p)]
    class Attributes(c.Structure):
        _fields_=[("x",c.c_int),("y",c.c_int),("width",c.c_int),("height",c.c_int),
                  ("border_width",c.c_int),("depth",c.c_int),("visual",c.c_void_p),
                  ("root",window_t),("window_class",c.c_int),("bit_gravity",c.c_int),
                  ("win_gravity",c.c_int),("backing_store",c.c_int),("backing_planes",c.c_ulong),
                  ("backing_pixel",c.c_ulong),("save_under",c.c_int),("colormap",c.c_ulong),
                  ("map_installed",c.c_int),("map_state",c.c_int),("all_event_masks",c.c_long),
                  ("your_event_mask",c.c_long),("do_not_propagate_mask",c.c_long),
                  ("override_redirect",c.c_int),("screen",c.c_void_p)]
    x.XGetWindowAttributes.argtypes=[display_t,window_t,c.POINTER(Attributes)]
    class XImage(c.Structure):
        _fields_=[("width",c.c_int),("height",c.c_int),("xoffset",c.c_int),("format",c.c_int),
                  ("data",c.c_void_p),("byte_order",c.c_int),("bitmap_unit",c.c_int),
                  ("bitmap_bit_order",c.c_int),("bitmap_pad",c.c_int),("depth",c.c_int),
                  ("bytes_per_line",c.c_int),("bits_per_pixel",c.c_int)]
    x.XGetImage.argtypes=[display_t,window_t,c.c_int,c.c_int,c.c_uint,c.c_uint,c.c_ulong,c.c_int]
    x.XGetImage.restype=c.POINTER(XImage)
    x.XDestroyImage.argtypes=[c.POINTER(XImage)]
    x.XFree.argtypes = [c.c_void_p]
    x.XRaiseWindow.argtypes = [display_t, window_t]
    x.XSetInputFocus.argtypes = [display_t, window_t, c.c_int, c.c_ulong]
    x.XSync.argtypes = [display_t, c.c_int]
    x.XTranslateCoordinates.argtypes = [display_t, window_t, window_t, c.c_int, c.c_int,
                                       c.POINTER(c.c_int), c.POINTER(c.c_int), c.POINTER(window_t)]
    test.XTestFakeMotionEvent.argtypes = [display_t, c.c_int, c.c_int, c.c_int, c.c_ulong]
    test.XTestFakeButtonEvent.argtypes = [display_t, c.c_uint, c.c_int, c.c_ulong]
    error_handler_type = c.CFUNCTYPE(c.c_int, display_t, c.c_void_p)
    error_handler = error_handler_type(lambda *_: 0)
    x.XSetErrorHandler.argtypes = [error_handler_type]
    x.XSetErrorHandler(error_handler)

    def windows():
        parent, top = window_t(), window_t()
        children, count = c.POINTER(window_t)(), c.c_uint()
        x.XQueryTree(display, root, c.byref(top), c.byref(parent), c.byref(children), c.byref(count))
        found = {}
        for index in range(count.value):
            attributes=Attributes()
            if not x.XGetWindowAttributes(display,children[index],c.byref(attributes)) or attributes.map_state!=2:
                continue
            name = c.c_char_p()
            if x.XFetchName(display, children[index], c.byref(name)) and name.value:
                title = name.value.decode("utf-8", errors="replace")
                if "Observatory" in title or "BioInsights" in title:
                    found[int(children[index])] = title
                x.XFree(name)
        if children:
            x.XFree(children)
        return found

    def expect(count):
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            found = windows()
            if len(found) == count:
                time.sleep(0.2)
                return found
            time.sleep(0.1)
        raise AssertionError((count, windows()))

    def click(window, px, py):
        ox, oy, child = c.c_int(), c.c_int(), window_t()
        x.XRaiseWindow(display, window)
        x.XSetInputFocus(display, window, 2, 0)
        x.XTranslateCoordinates(display, window, root, 0, 0, c.byref(ox), c.byref(oy), c.byref(child))
        test.XTestFakeMotionEvent(display, -1, ox.value + px, oy.value + py, 0)
        x.XSync(display, 0)
        time.sleep(0.15)
        test.XTestFakeButtonEvent(display, 1, 1, 0)
        x.XSync(display, 0)
        time.sleep(0.08)
        test.XTestFakeButtonEvent(display, 1, 0, 0)
        x.XSync(display, 0)
        time.sleep(0.35)

    def capture(window,path):
        attributes=Attributes()
        x.XGetWindowAttributes(display,window,c.byref(attributes))
        image=x.XGetImage(display,window,0,0,attributes.width,attributes.height,c.c_ulong(-1).value,2)
        assert image and image.contents.bits_per_pixel==32
        info=image.contents
        raw=c.string_at(info.data,info.bytes_per_line*info.height)
        rows=[]
        for y in range(info.height):
            row=raw[y*info.bytes_per_line:y*info.bytes_per_line+info.width*4]
            rgb=bytearray(info.width*3)
            rgb[0::3],rgb[1::3],rgb[2::3]=row[2::4],row[1::4],row[0::4]
            rows.append(b"\0"+rgb)
        def chunk(kind,data):
            return struct.pack(">I",len(data))+kind+data+struct.pack(">I",zlib.crc32(kind+data))
        path.write_bytes(b"\x89PNG\r\n\x1a\n"+chunk(b"IHDR",struct.pack(">IIBBBBB",info.width,info.height,8,2,0,0,0))+
                         chunk(b"IDAT",zlib.compress(b"".join(rows)))+chunk(b"IEND",b""))
        # Exclude toolbar hover and Core status controls: a page must actually
        # replace its content, not merely leave one window on screen.
        digest=hashlib.sha256(raw[100*info.bytes_per_line:(info.height-50)*info.bytes_per_line]).hexdigest()
        x.XDestroyImage(image)
        return digest

    receipt = []
    with tempfile.TemporaryDirectory(prefix="observatory-navigation-") as directory:
        folder = Path(directory)
        journals = folder / "journals"
        journals.mkdir()
        (journals / "Journal.2026-10-25T000000.01.log").write_text(
            "".join(json.dumps(event) + "\n" for event in fixture()))
        settings = {"journal_dir": str(journals), "auto_start": True, "start_read_all": True,
                    "popup": False, "speech": False, "theme": "dark", "plugins": {
                        "Hecate": {"RemoteLookups": False},
                        "BioInsights": {"GalacticCodexRemote": False, "StellarOverlayUseRadar": False,
                                        "StellarOverlaySendFSSNotification": False,
                                        "StellarOverlaySendDSSNotification": False}}}
        if args.browser:settings["bioinsights_renderer"]="webkit"
        config = folder / "settings.json"
        config.write_text(json.dumps(settings))
        env = os.environ.copy()
        env.pop("WAYLAND_DISPLAY", None)
        env["LIBGL_ALWAYS_SOFTWARE"]="1"
        env["WEBKIT_DISABLE_DMABUF_RENDERER"]="1"
        for kind in ("CONFIG", "DATA", "CACHE"):
            env[f"XDG_{kind}_HOME"] = str(folder / kind.lower())
        args.output.parent.mkdir(parents=True,exist_ok=True)
        with args.output.with_suffix(".log").open("w+") as log:
            process = subprocess.Popen([str(args.binary.resolve()), "--config", str(config),
                                        "--view", "BioInsights", "--window-size", "1200x800"],
                                       env=env, stdout=log, stderr=log)
            try:
                initial = expect(1)
                main_window = next(iter(initial))
                time.sleep(1)
                page_hashes={}
                capture(main_window,args.output.with_name(args.output.stem+"-initial.png"))
                coordinates=[("Unsold",82),("About",120),("Codex Companion",158),("Main",44),("Zoom",234)] if args.browser else [("Unsold",74),("About",108),("Codex Companion",142),("Main",40),("Zoom",210)]
                for page, px in coordinates:
                    click(main_window, px, 64)
                    found = expect(1)
                    receipt.append({"clicked": page, "native_windows": len(found)})
                    if page!="Zoom":
                        capture_path=args.output.parent/(args.output.stem+"-"+page.lower().replace(" ","-")+".png")
                        page_hashes[page]=capture(main_window,capture_path)
                assert len(set(page_hashes.values()))==4, page_hashes
                # Close the anchored zoom selector by clicking outside it.
                click(main_window, 700, 500)
                click(main_window, 196 if args.browser else 176, 68 if args.browser else 64)
                popped = expect(2)
                child = next(window for window, title in popped.items() if title == "BioInsights Popout")
                receipt.append({"clicked": "Popout", "native_windows": len(popped)})
                time.sleep(0.5)
                capture(child,args.output.with_suffix(".png"))
                click(child, 180 if args.browser else 160, 28 if args.browser else 24)
                receipt.append({"clicked": "Popout in child (reattach)", "native_windows": len(expect(1))})
                click(main_window, 74, 64)
                receipt.append({"clicked": "Unsold after reattach", "native_windows": len(expect(1))})
                click(main_window, 272 if args.browser else 244, 68 if args.browser else 64)
                settings_windows = expect(2)
                assert "BioInsights Settings" in settings_windows.values(), settings_windows
                receipt.append({"clicked": "Settings", "native_windows": len(settings_windows)})
                assert process.poll() is None
            except Exception:
                log.flush()
                log.seek(0)
                print(log.read())
                raise
            finally:
                process.terminate()
                process.wait(timeout=8)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps({"backend": "private X11/XTest", "synthetic_only": True,
                                       "button_clicks": receipt,"content_hashes":page_hashes}, indent=2) + "\n")
    print(json.dumps(receipt))


if __name__ == "__main__":
    main()
