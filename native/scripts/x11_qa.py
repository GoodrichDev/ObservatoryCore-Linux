"""Owned private Xvfb/XTest surfaces for graphical native regression tests."""
import ctypes as c
import os
from pathlib import Path
import select
import subprocess
import tempfile
import time
from PIL import Image


class PrivateDisplay:
    def __init__(self, width=3840, height=2160):
        self.socket_directory = None
        if os.geteuid() == 0:
            # WSLg's socket directory may be a read-only mount. A private mount
            # namespace gives this test its own writable sockets without
            # changing or hiding any display in the user's mount namespace.
            libc = c.CDLL(None, use_errno=True)
            assert libc.unshare(0x20000|0x40000000) == 0, "Private Xvfb mount/network namespaces unavailable"
            libc.mount.argtypes = [c.c_char_p,c.c_char_p,c.c_char_p,c.c_ulong,c.c_char_p]
            assert libc.mount(None,b"/",None,16384|262144,None) == 0
            self.socket_directory = tempfile.TemporaryDirectory(prefix="observatory-x11-sockets-")
            os.chmod(self.socket_directory.name,0o1777)
            assert libc.mount(self.socket_directory.name.encode(),b"/tmp/.X11-unix",None,4096,None) == 0
        read, write = os.pipe()
        self.process = subprocess.Popen(["Xvfb", "-displayfd", str(write), "-screen", "0", f"{width}x{height}x24", "-nolisten", "tcp", "-ac"], pass_fds=[write], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        os.close(write)
        assert select.select([read], [], [], 10)[0], "Private Xvfb did not start"
        number = os.read(read, 64).decode().strip()
        os.close(read)
        assert number.isdigit(), "Private Xvfb exited without a display; use a writable private socket directory"
        self.name = ":" + number
        self.env = os.environ.copy()
        self.env["DISPLAY"] = self.name
        self.env.pop("WAYLAND_DISPLAY", None)
        self.env["LIBGL_ALWAYS_SOFTWARE"] = "1"
        self.env["WEBKIT_DISABLE_DMABUF_RENDERER"] = "1"
        self.env["GDK_BACKEND"] = "x11"
        self.runtime_directory = tempfile.TemporaryDirectory(prefix="observatory-x11-runtime-")
        os.chmod(self.runtime_directory.name,0o700)
        self.env["XDG_RUNTIME_DIR"] = self.runtime_directory.name
        self.x = X11(self.name)

    def close(self):
        self.x.close()
        self.process.terminate()
        self.process.wait(timeout=5)
        self.runtime_directory.cleanup()
        if self.socket_directory is not None:
            assert c.CDLL(None).umount(b"/tmp/.X11-unix") == 0
            self.socket_directory.cleanup()


class Attributes(c.Structure):
    _fields_ = [("x", c.c_int), ("y", c.c_int), ("width", c.c_int), ("height", c.c_int),
                ("border_width", c.c_int), ("depth", c.c_int), ("visual", c.c_void_p),
                ("root", c.c_ulong), ("window_class", c.c_int), ("bit_gravity", c.c_int),
                ("win_gravity", c.c_int), ("backing_store", c.c_int), ("backing_planes", c.c_ulong),
                ("backing_pixel", c.c_ulong), ("save_under", c.c_int), ("colormap", c.c_ulong),
                ("map_installed", c.c_int), ("map_state", c.c_int), ("all_event_masks", c.c_long),
                ("your_event_mask", c.c_long), ("do_not_propagate_mask", c.c_long),
                ("override_redirect", c.c_int), ("screen", c.c_void_p)]


class XImage(c.Structure):
    _fields_ = [("width", c.c_int), ("height", c.c_int), ("xoffset", c.c_int), ("format", c.c_int),
                ("data", c.c_void_p), ("byte_order", c.c_int), ("bitmap_unit", c.c_int),
                ("bitmap_bit_order", c.c_int), ("bitmap_pad", c.c_int), ("depth", c.c_int),
                ("bytes_per_line", c.c_int), ("bits_per_pixel", c.c_int)]


class X11:
    def __init__(self, display_name):
        self.x = x = c.CDLL("libX11.so.6")
        self.test = test = c.CDLL("libXtst.so.6")
        display, window = c.c_void_p, c.c_ulong
        signatures = {
            "XOpenDisplay": ([c.c_char_p], display),
            "XDefaultRootWindow": ([display], window),
            "XGetWindowAttributes": ([display, window, c.POINTER(Attributes)], c.c_int),
            "XFetchName": ([display, window, c.POINTER(c.c_char_p)], c.c_int),
            "XQueryTree": ([display, window, c.POINTER(window), c.POINTER(window), c.POINTER(c.POINTER(window)), c.POINTER(c.c_uint)], c.c_int),
            "XGetImage": ([display, window, c.c_int, c.c_int, c.c_uint, c.c_uint, c.c_ulong, c.c_int], c.POINTER(XImage)),
            "XDestroyImage": ([c.POINTER(XImage)], c.c_int),
            "XFree": ([c.c_void_p], c.c_int),
            "XRaiseWindow": ([display, window], c.c_int),
            "XSetInputFocus": ([display, window, c.c_int, c.c_ulong], c.c_int),
            "XTranslateCoordinates": ([display, window, window, c.c_int, c.c_int, c.POINTER(c.c_int), c.POINTER(c.c_int), c.POINTER(window)], c.c_int),
            "XSync": ([display, c.c_int], c.c_int),
            "XCloseDisplay": ([display], c.c_int),
            "XCreateSimpleWindow":([display,window,c.c_int,c.c_int,c.c_uint,c.c_uint,c.c_uint,c.c_ulong,c.c_ulong],window),
            "XMapWindow":([display,window],c.c_int),
            "XDestroyWindow":([display,window],c.c_int),
        }
        for function_name, (args, result) in signatures.items():
            function = getattr(x, function_name)
            function.argtypes, function.restype = args, result
        test.XTestFakeMotionEvent.argtypes = [display, c.c_int, c.c_int, c.c_int, c.c_ulong]
        test.XTestFakeButtonEvent.argtypes = [display, c.c_uint, c.c_int, c.c_ulong]
        self.display = x.XOpenDisplay(display_name.encode())
        if not self.display:
            raise RuntimeError("Could not open owned private Xvfb display")
        self.root = x.XDefaultRootWindow(self.display)
        callback_type = c.CFUNCTYPE(c.c_int, display, c.c_void_p)
        self.error_callback = callback_type(lambda *_: 0)
        x.XSetErrorHandler.argtypes = [callback_type]
        x.XSetErrorHandler(self.error_callback)

    def close(self):
        self.x.XCloseDisplay(self.display)

    def windows(self, parent=None, recursive=False):
        parent = self.root if parent is None else parent
        top, previous = c.c_ulong(), c.c_ulong()
        children, count = c.POINTER(c.c_ulong)(), c.c_uint()
        self.x.XQueryTree(self.display, parent, c.byref(top), c.byref(previous), c.byref(children), c.byref(count))
        values = [int(children[i]) for i in range(count.value)]
        if children:
            self.x.XFree(children)
        result = {}
        for child in values:
            attrs = self.attributes(child)
            name = c.c_char_p()
            title = ""
            if self.x.XFetchName(self.display, child, c.byref(name)) and name.value:
                title = name.value.decode(errors="replace")
                self.x.XFree(name)
            result[child] = {"title": title, "x": attrs.x, "y": attrs.y, "width": attrs.width, "height": attrs.height, "mapped": attrs.map_state == 2}
            if recursive:
                result.update(self.windows(child, True))
        return result

    def attributes(self, window):
        attrs = Attributes()
        self.x.XGetWindowAttributes(self.display, window, c.byref(attrs))
        return attrs

    def capture(self, window, region=None):
        attrs = self.attributes(window)
        px,py,width,height=region or (0,0,attrs.width,attrs.height)
        assert 0<=px<attrs.width and 0<=py<attrs.height and px+width<=attrs.width and py+height<=attrs.height
        data = self.x.XGetImage(self.display, window, px, py, width, height, c.c_ulong(-1).value, 2)
        assert data and data.contents.bits_per_pixel == 32
        image = data.contents
        raw = c.string_at(image.data, image.bytes_per_line * image.height)
        result = Image.frombytes("RGB", (image.width, image.height), raw, "raw", "BGRX", image.bytes_per_line)
        self.x.XDestroyImage(data)
        return result

    def move(self, window, px, py):
        ox, oy, child = c.c_int(), c.c_int(), c.c_ulong()
        self.x.XTranslateCoordinates(self.display, window, self.root, 0, 0, c.byref(ox), c.byref(oy), c.byref(child))
        self.test.XTestFakeMotionEvent(self.display, -1, ox.value + int(px), oy.value + int(py), 0)
        self.x.XSync(self.display, 0)

    def click(self, window, px, py):
        self.x.XRaiseWindow(self.display, window)
        self.x.XSetInputFocus(self.display, window, 2, 0)
        self.move(window, px, py)
        self.test.XTestFakeButtonEvent(self.display, 1, 1, 0)
        self.x.XSync(self.display, 0)
        time.sleep(0.01)
        self.test.XTestFakeButtonEvent(self.display, 1, 0, 0)
        self.x.XSync(self.display, 0)
