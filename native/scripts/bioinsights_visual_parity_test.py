#!/usr/bin/env python3
"""Compare the native Bio view against real WebKit/source CSS on private Xvfb.

Run as root under WSLg to use private mount/network namespaces for Xvfb; ordinary
Linux sessions can run as their own user when /tmp/.X11-unix is writable. Existing
Windows fonts may be explicitly added to this private QA Fontconfig profile.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import statistics
import subprocess
import tempfile
import time
import numpy as np
from PIL import ImageChops
from native_update_test import Profile
from x11_qa import PrivateDisplay


PALETTE=json.loads((Path(__file__).resolve().parents[1]/"observatory-plugins/src/bioinsights/theme-orange.json").read_text())
def rgb(key):
    color=PALETTE["Colors"].get(key,{})
    return [color.get(channel,0) for channel in ["R","G","B"]]

def wait(predicate, description, timeout=15):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if result := predicate():
            return result
        time.sleep(0.025)
    raise AssertionError(f"Timed out: {description}")


def runs(values):
    result = []
    for value in values:
        if not result or value != result[-1][-1] + 1:
            result.append([int(value)])
        else:
            result[-1].append(int(value))
    return [(r[0], r[-1] + 1) for r in result]


def geometry(image):
    pixels = np.asarray(image)
    orange = np.all(pixels == rgb("HeaderBackground"), axis=2)
    brown = np.all(pixels == rgb("Row1Background"), axis=2)
    toolbar_mask=np.all(pixels==rgb("ButtonFace"),axis=2)
    toolbar = None
    for y in range(min(220, image.height)):
        spans = [(a, b) for a, b in runs(np.flatnonzero(toolbar_mask[y])) if 20 <= b-a <= 70]
        if len(spans) == 7:
            toolbar = {"y": y, "spans": spans, "centers": [(a+b)/2 for a,b in spans]}
            break
    bands = []
    for start, end in runs(np.flatnonzero(orange.sum(axis=1) > image.width * 0.6)):
        mask = orange[start:end].sum(axis=0)
        x = np.flatnonzero(mask > (end-start)*0.5)
        bands.append({"x": int(x[0]), "y": start, "width": int(x[-1]-x[0]+1), "height": end-start})
    highlight=np.all(pixels==rgb("HighlightBackground"),axis=2) if PALETTE["Settings"].get("UseHighlightRow",False) else np.zeros(pixels.shape[:2],dtype=bool)
    highlighted=runs(np.flatnonzero(highlight.sum(axis=1)>image.width*.6))
    blue_centers=[]
    if highlighted:
        row_y=highlighted[0][1]+PALETTE["Settings"].get("RowSpacing",0)
        blue=np.all(pixels[row_y:row_y+27]==[0,120,215],axis=2)
        blue_centers=[(a+b)/2 for a,b in runs(np.flatnonzero(blue.sum(axis=0)>2))]
    elif len(bands)>1:
        row_y=bands[1]["y"]+bands[1]["height"]+PALETTE["Settings"].get("RowSpacing",0)
        blue=np.all(pixels[row_y:row_y+27]==[0,120,215],axis=2)
        blue_centers=[(a+b)/2 for a,b in runs(np.flatnonzero(blue.sum(axis=0)>2))]
    caption=None
    if bands:
        band=bands[0]
        area=pixels[band["y"]:band["y"]+band["height"],band["x"]:band["x"]+band["width"]]
        yy,xx=np.nonzero(np.all(area==rgb("HeaderText"),axis=2))
        if len(xx):caption=[int(xx.min()+band["x"]),int(yy.min()+band["y"]),int(xx.max()-xx.min()+1),int(yy.max()-yy.min()+1)]
    brown_rows=runs(np.flatnonzero(brown.sum(axis=1)>max(100,int(brown.sum(axis=1).max()*.5))))
    return {"toolbar": toolbar, "orange_bands": bands,"caption_text_box":caption,"first_species_blue_symbol_centers":blue_centers,"brown_row_bands":brown_rows, "orange_pixels": int(orange.sum()), "brown_pixels": int(brown.sum())}


def descendants(pid):
    result = {pid}
    todo = [pid]
    while todo:
        owner = todo.pop()
        try:
            tasks = list(Path(f"/proc/{owner}/task").iterdir())
        except FileNotFoundError:
            continue
        for task in tasks:
            try:
                children = (task / "children").read_text().split()
            except FileNotFoundError:
                continue
            for child in map(int, children):
                if child not in result:
                    result.add(child)
                    todo.append(child)
    return result


def ticks(pid):
    total = 0
    for child in descendants(pid):
        try:
            fields = Path(f"/proc/{child}/stat").read_text().rsplit(")",1)[1].split()
            total += int(fields[11]) + int(fields[12])
        except FileNotFoundError:
            pass
    return total


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--sizes", nargs="+", default=["1200x800", "3439x1400", "3840x2160"])
    parser.add_argument("--host-windows-fonts", action="store_true")
    parser.add_argument("--theme",choices=["orange","original"],default="orange")
    parser.add_argument("--capture-only", action="store_true")
    parser.add_argument("--resume", action="store_true", help="Append sizes to a receipt only when its immutable binary/theme/font profile match")
    parser.add_argument("--stress-bodies",type=int,default=0)
    parser.add_argument("--renderers",nargs="+",choices=["rust","webkit"],default=["rust","webkit"])
    args = parser.parse_args()
    global PALETTE
    PALETTE=json.loads((Path(__file__).resolve().parents[1]/("observatory-plugins/src/bioinsights/"+("theme.json" if args.theme=="original" else "theme-orange.json"))).read_text())
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    frozen = output / "observatory-core-tested"
    shutil.copy2(args.binary.resolve(), frozen)
    receipt = {"core_sha256": hashlib.sha256(frozen.read_bytes()).hexdigest(), "synthetic_only": True,
               "display": "owned private Xvfb; software GL", "host_windows_fonts": args.host_windows_fonts,"theme":args.theme,
               "network": "private network namespace; source offline gallery/bioscan paths are exercised",
               "normalization": "none; captures include actual native and embedded WebKit pixels", "cases": {}}
    if args.resume:
        previous=json.loads((output/"receipt.json").read_text())
        for key in ("core_sha256","host_windows_fonts","theme"):
            assert previous[key]==receipt[key],(key,previous[key],receipt[key])
        receipt["cases"]=previous["cases"]
    private = PrivateDisplay()
    try:
        with tempfile.TemporaryDirectory(prefix="bio-visual-private-",dir=output) as temporary:
            root = Path(temporary)
            fontconfig = None
            if args.host_windows_fonts:
                assert Path("/mnt/c/Windows/Fonts/seguiemj.ttf").is_file()
                fontconfig = root / "fonts.conf"
                fontconfig.write_text(f'<?xml version="1.0"?><!DOCTYPE fontconfig SYSTEM "fonts.dtd"><fontconfig><include>/etc/fonts/fonts.conf</include><dir>/mnt/c/Windows/Fonts</dir><cachedir>{root / "font-cache"}</cachedir></fontconfig>')
            for size in args.sizes:
                width, height = map(int,size.split("x"))
                for renderer in args.renderers:
                    label = f"{renderer}-{size}"
                    print(f"Bio visual QA: {label}",flush=True)
                    profile = Profile(root,label)
                    for path in profile.plugins.iterdir():
                        path.unlink()
                    profile.settings.update(auto_start=True,start_read_all=True,bioinsights_renderer=renderer)
                    # The source default (100,100) puts a full4K window partly
                    # outside a4K framebuffer; XGetImage correctly rejects it.
                    profile.settings["window_position"]=[0,0]
                    profile.settings["plugins"]["StellarOverlay"]["DisplayOverride"] = False
                    profile.settings["plugins"]["BioInsights"]["ThemeSettings"]={"ThemeIndex":0 if args.theme=="original" else 1,"ThemeMode":0}
                    if args.stress_bodies:
                        from plugin_smoke_test import fixture
                        base=fixture()
                        scan=next(event for event in base if event["event"]=="Scan" and event.get("BodyID")==1)
                        signals=next(event for event in base if event["event"]=="FSSBodySignals")
                        genera=next(event for event in base if event["event"]=="SAASignalsFound")
                        extra=[]
                        for body in range(2,args.stress_bodies+1):
                            for original in (scan,signals,genera):
                                event=dict(original);event.update(BodyID=body,BodyName=f"Native Smoke A {body}");extra.append(event)
                        base[2]["BodyCount"]=args.stress_bodies+1
                        position=next(index for index,event in enumerate(base) if event["event"]=="FSSAllBodiesFound")
                        base[position:position]=extra
                        base[position+len(extra)]["Count"]=args.stress_bodies+1
                        profile.journal.write_text("".join(json.dumps(event)+"\n" for event in base))
                    profile.save()
                    env = dict(private.env)
                    env.update({k:v for k,v in profile.env.items() if k.startswith("XDG_")})
                    if fontconfig:
                        env["FONTCONFIG_FILE"] = str(fontconfig)
                    stress=None
                    if args.stress_bodies:
                        result=subprocess.run([str(frozen),"--config",str(profile.config),"--headless"],env=env,capture_output=True,text=True,timeout=60,check=True)
                        data=json.loads(result.stdout)
                        groups=data["plugin_views"]["BioInsights"]["Groups"]
                        stress={"bodies":len(groups),"species_rows":sum(len(group["Rows"]) for group in groups),"journal_events":data["events"]}
                        assert stress["bodies"]>=args.stress_bodies and stress["species_rows"]>=args.stress_bodies*3,stress
                        (output/f"{label}-headless.json").write_text(result.stdout)
                    with (output / f"{label}.log").open("w") as log:
                        process = subprocess.Popen([str(frozen),"--config",str(profile.config),"--view","BioInsights","--window-size",size],env=env,stdout=log,stderr=log)
                        try:
                            def mapped():
                                assert process.poll() is None, f"Core exited: {label}"
                                return next((window for window, value in private.x.windows().items() if value["mapped"] and "Observatory" in value["title"]),None)
                            window = wait(mapped,f"mapped Core {label}")
                            time.sleep(2)
                            if args.stress_bodies:
                                wait(lambda: np.count_nonzero(np.all(np.asarray(private.x.capture(window,(26,93,500,35)))==[46,184,133],axis=2))>5000,f"{label} ReadAllcompleted",timeout=45)
                                time.sleep(.5)
                            private.x.move(window,width-25,height-100)
                            time.sleep(0.1)
                            initial = private.x.capture(window)
                            initial.save(output / f"{label}-main.png")
                            initial.save(output / f"{label}-initial-main.png")
                            case = geometry(initial)
                            case["window"] = [initial.width,initial.height]
                            if stress:case["stress_projection"]=stress
                            case["fontconfig"] = {font: subprocess.check_output(["fc-match","--format=%{family}|%{file}",font],env=env,text=True) for font in ("Segoe UI Emoji","Verdana","serif")}
                            assert case["toolbar"] is not None, f"Missing actual toolbar {label}"
                            assert len(case["orange_bands"]) >= 2, f"Missing source main bands {label}"
                            if not args.capture_only:
                                # Warmed idle CPU includes WebKit child processes
                                # and every native rendering/journal thread.
                                start_ticks, start = ticks(process.pid), time.monotonic()
                                time.sleep(3)
                                case["idle_cpu_percent_one_core"] = 100*(ticks(process.pid)-start_ticks)/os.sysconf("SC_CLK_TCK")/(time.monotonic()-start)
                                centers = case["toolbar"]["centers"]
                                toolbar_y = case["toolbar"]["y"] + 12
                                capture_start=time.monotonic()
                                for _ in range(10): private.x.capture(window,(30,145,500,160))
                                case["latency_sampling"]={"roi":[30,145,500,160],"capture_overhead_mean_ms":(time.monotonic()-capture_start)*100,"mouse_hold_ms":10,"description":"Observed response bound includes mouse hold and ROI capture/pixel comparison; no full-screen readback in timed interval"}
                                navigation = []
                                for name,index in [("About",2),("Main",0),("Unsold",1),("Main",0),("CodexCompanion",3),("Main",0)]:
                                    before = np.asarray(private.x.capture(window,(30,145,500,160))).copy()
                                    started = time.monotonic()
                                    private.x.click(window,centers[index],toolbar_y)
                                    def changed():
                                        after = np.asarray(private.x.capture(window,(30,145,500,160)))
                                        return np.count_nonzero(np.max(np.abs(after.astype(np.int16)-before),axis=2)>20)>1000
                                    wait(changed,f"same-page {name}")
                                    latency = (time.monotonic()-started)*1000
                                    time.sleep(0.1)
                                    assert len([v for v in private.x.windows().values() if v["mapped"] and ("Observatory" in v["title"] or "BioInsights" in v["title"])]) == 1
                                    private.x.capture(window).save(output / f"{label}-{name.lower()}.png")
                                    navigation.append({"page":name,"latency_ms":latency,"native_windows":1,"geometry":geometry(private.x.capture(window))})
                                    if name=="CodexCompanion" and args.theme=="orange":
                                        # Locate source table rows from an empty padding strip,
                                        # then click real genus/species anchors in the actual UI.
                                        shot=np.asarray(private.x.capture(window))
                                        row_fill=np.all(shot[:,53]==rgb("Row1Background"),axis=1)|np.all(shot[:,53]==rgb("Row2Background"),axis=1)
                                        bands=runs(np.flatnonzero(row_fill&(np.arange(height)>150)))
                                        assert len(bands)>3,(label,bands)
                                        click_y=sum(bands[2])/2
                                        for detail,click_x in [("GenusBacterium",131),("SpeciesAlcyoneum",None)]:
                                            if click_x is None:
                                                shot=np.asarray(private.x.capture(window))
                                                segments=[(a,b) for a,b in runs(np.flatnonzero(np.all(shot[bands[0][0]+1]==rgb("TitleBackground"),axis=1))) if b-a>70]
                                                assert len(segments)>=2,(label,segments)
                                                click_x=sum(segments[1])/2
                                            before=np.asarray(private.x.capture(window,(30,145,500,160))).copy()
                                            started=time.monotonic()
                                            private.x.click(window,click_x,click_y)
                                            wait(changed,f"same-page {detail}")
                                            latency=(time.monotonic()-started)*1000
                                            time.sleep(.15)
                                            private.x.move(window,width-25,height-100)
                                            time.sleep(.05)
                                            private.x.capture(window).save(output/f"{label}-{detail.lower()}.png")
                                            assert len([v for v in private.x.windows().values() if v["mapped"] and("Observatory" in v["title"] or "BioInsights" in v["title"])])==1
                                            navigation.append({"page":detail,"latency_ms":latency,"native_windows":1,"geometry":geometry(private.x.capture(window))})

                                case["navigation"] = navigation
                                case["navigation_median_ms"] = statistics.median(n["latency_ms"] for n in navigation)
                            receipt["cases"][label] = case
                            (output / "receipt.json").write_text(json.dumps(receipt,indent=2)+"\n")
                        finally:
                            process.terminate()
                            process.wait(timeout=10)
    finally:
        private.close()
    receipt["comparisons"]={}
    for size in sorted({name.split("-",1)[1] for name in receipt["cases"]}):
        if "rust-"+size not in receipt["cases"] or "webkit-"+size not in receipt["cases"]:continue
        native=receipt["cases"]["rust-"+size]
        source=receipt["cases"]["webkit-"+size]
        comparison={"toolbar_centers_equal":native["toolbar"]["centers"]==source["toolbar"]["centers"],"outer_bands_equal":native["orange_bands"]==source["orange_bands"],"brown_row_positions_equal":native["brown_row_bands"]==source["brown_row_bands"]}
        from PIL import Image
        native_image=np.asarray(Image.open(output/f"rust-{size}-initial-main.png").convert("RGB"))
        source_image=np.asarray(Image.open(output/f"webkit-{size}-initial-main.png").convert("RGB"))
        comparison["toolbar_icon_mean_absolute_channel_difference"]={}
        for button,center in zip(["main","unsold","about","codexcompanion","popout","zoom","settings"],native["toolbar"]["centers"]):
            y=native["toolbar"]["y"];x=int(center)-16
            difference=np.abs(native_image[y:y+32,x:x+32].astype(np.int16)-source_image[y:y+32,x:x+32])
            comparison["toolbar_icon_mean_absolute_channel_difference"][button]=float(difference.mean())
        if len(native["first_species_blue_symbol_centers"])==len(source["first_species_blue_symbol_centers"]):
            comparison["symbol_center_delta_px"]=[a-b for a,b in zip(native["first_species_blue_symbol_centers"],source["first_species_blue_symbol_centers"])]
        if native["caption_text_box"] and source["caption_text_box"]:
            comparison["caption_box_delta_px"]=[a-b for a,b in zip(native["caption_text_box"],source["caption_text_box"])]
        receipt["comparisons"][size]=comparison
    (output/"receipt.json").write_text(json.dumps(receipt,indent=2)+"\n")
    print(json.dumps(receipt),flush=True)


if __name__ == "__main__":
    main()
