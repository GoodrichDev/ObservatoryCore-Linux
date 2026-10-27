#!/usr/bin/env python3
"""Exercise the four installed-plugin ports with isolated synthetic Linux data.

Run under `dbus-run-session -- xvfb-run -a python3 ... BINARY`, or pass
--wayland inside a real Wayland session. No game journals or remote APIs are used.
The Lua fixture drives the public message bus; all four workers and all windows
are the real native implementation.
"""
import argparse
import json
import os
from pathlib import Path
import sqlite3
import struct
import subprocess
import tempfile
import zlib


def fixture():
    events = [
        {"event": "LoadGame", "FID": "F4242", "Commander": "Native Smoke", "Odyssey": True},
        {"event": "Location", "SystemAddress": 42, "StarSystem": "Native Smoke", "StarPos": [0, 0, 0]},
        {"event": "FSSDiscoveryScan", "SystemAddress": 42, "BodyCount": 2},
        {"event": "Scan", "SystemAddress": 42, "BodyID": 0, "BodyName": "Native Smoke A", "StarType": "F",
         "StellarMass": 1.2, "SurfaceTemperature": 6500, "Radius": 695700000,
         "DistanceFromArrivalLS": 0, "WasDiscovered": False},
        {"event": "Scan", "SystemAddress": 42, "BodyID": 1, "BodyName": "Native Smoke A 1",
         "PlanetClass": "High metal content body", "Atmosphere": "thin carbon dioxide atmosphere",
         "AtmosphereType": "CarbonDioxide", "AtmosphereComposition": [{"Name": "CarbonDioxide", "Percent": 100}],
         "SurfaceTemperature": 180, "SurfaceGravity": 1, "SurfacePressure": 100, "MassEM": 0.1,
         "Radius": 1000000, "SemiMajorAxis": 149597870700, "DistanceFromArrivalLS": 499,
         "Parents": [{"Star": 0}], "Volcanism": "", "Landable": True,
         "WasDiscovered": False, "WasMapped": False, "WasFootfalled": False},
        {"event": "FSSBodySignals", "SystemAddress": 42, "BodyID": 1, "BodyName": "Native Smoke A 1",
         "Signals": [{"Type": "$SAA_SignalType_Biological;", "Count": 2}]},
        {"event": "SAASignalsFound", "SystemAddress": 42, "BodyID": 1, "BodyName": "Native Smoke A 1",
         "Signals": [{"Type": "$SAA_SignalType_Biological;", "Count": 2}],
         "Genuses": [{"Genus": "$Codex_Ent_Stratum_Genus_Name;"}, {"Genus": "$Codex_Ent_Bacterial_Genus_Name;"}]},
        {"event": "FSSAllBodiesFound", "SystemAddress": 42, "SystemName": "Native Smoke", "Count": 2},
        {"event": "Disembark", "SystemAddress": 42, "BodyID": 1, "BodyName": "Native Smoke A 1", "OnPlanet": True},
        {"event": "ScanOrganic", "SystemAddress": 42, "BodyID": 1, "ScanType": "Analyse",
         "Genus": "$Codex_Ent_Stratum_Genus_Name;", "Species": "$Codex_Ent_Stratum_07_Name;",
         "Variant": "$Codex_Ent_Stratum_07_Emerald;"},
    ]
    for index, event in enumerate(events):
        event["timestamp"] = f"2026-10-25T00:00:{index:02}Z"
    return events


DRIVER = r"""local shown=false
return {
 id='native.plugin.smoke',name='Native Plugin Smoke Driver',version='1.0',
 on_state_changed=function(state)
  if state.batch or shown then return end
  shown=true
  core.send_message({target='StellarOverlay',command='RegisterPluginCustomColours',payload={TitleColor='FFFF7B00',TextColor='FFFF6300',BorderColor='FF2E8EFF',BackgroundColor='B0000000'}})
  core.notify({id='native-smoke-overlay',title='Native StellarOverlay',detail='Rust notification, original orange theme',timeout=0,x=65,y=12})
  core.send_message({target='StellarOverlay',command='RadarSetPlanet',payload={ID64=42,BodyID=1,RadiusKM=250}})
  core.send_message({target='StellarOverlay',command='RadarSetContextID',payload={ContextID=7,Radius=500}})
  core.send_message({target='StellarOverlay',command='RadarAddBiological',payload={UseCurrentPosition=false,X=0.10,Y=0.10,ContextID=7,Radius=500,Label='Stratum sample',MeasureDistance=true}})
  core.send_message({target='StellarOverlay',command='RadarAddVehicle',payload={UseCurrentPosition=false,X=-0.10,Y=0.10,ContextID=1,Label='Ship',MeasureDistance=true}})
  core.send_message({target='StellarOverlay',command='RadarAddWaypoint',payload={UseCurrentPosition=false,X=0.10,Y=-0.10,ContextID=2,Label='Waypoint',MeasureDistance=true}})
 end
}
"""


def visible_png(path, require_colour=False, require_bio_main=False):
    """Decode 8-bit RGB(A) PNG scanlines and reject blank native captures."""
    data = path.read_bytes()
    assert data[:8] == b"\x89PNG\r\n\x1a\n", path
    position, compressed = 8, bytearray()
    width = height = channels = None
    while position < len(data):
        length = struct.unpack_from(">I", data, position)[0]
        kind = data[position + 4:position + 8]
        chunk = data[position + 8:position + 8 + length]
        position += length + 12
        if kind == b"IHDR":
            width, height, depth, colour, _, _, interlace = struct.unpack(">IIBBBBB", chunk)
            assert depth == 8 and colour in (2, 6) and interlace == 0, (path, depth, colour, interlace)
            channels = 4 if colour == 6 else 3
        elif kind == b"IDAT":
            compressed.extend(chunk)
    assert width and height and channels and width > 50 and height > 50, path
    raw = zlib.decompress(compressed)
    stride, offset, chromatic, painted = width * channels, 0, 0, 0
    colours = set()
    bio_bands, brown_pixels = [], 0
    previous = bytearray(stride)
    for y in range(height):
        method = raw[offset]
        offset += 1
        row = bytearray(raw[offset:offset + stride])
        offset += stride
        for i in range(stride):
            left = row[i - channels] if i >= channels else 0
            above = previous[i]
            upper_left = previous[i - channels] if i >= channels else 0
            if method == 1:
                predictor = left
            elif method == 2:
                predictor = above
            elif method == 3:
                predictor = (left + above) // 2
            elif method == 4:
                estimate = left + above - upper_left
                distances = [abs(estimate - value) for value in (left, above, upper_left)]
                predictor = (left, above, upper_left)[distances.index(min(distances))]
            else:
                assert method == 0, (path, method)
                predictor = 0
            row[i] = (row[i] + predictor) & 255
        orange_pixels=0
        for i in range(0, stride, channels):
            red, green, blue = row[i:i + 3]
            if (red,green,blue)==(255,111,0):orange_pixels+=1
            if (red,green,blue)==(35,16,2):brown_pixels+=1
            if max(red, green, blue) > 60 and (channels == 3 or row[i + 3] > 80):
                painted += 1
            if len(colours) < 4096:
                colours.add((red, green, blue))
            if max(red, green, blue) - min(red, green, blue) > 35 and (channels == 3 or row[i + 3] > 80):
                chromatic += 1
        previous = row
        if orange_pixels>width*0.65:bio_bands.append(y)
    assert painted > 100 and len(colours) > 8, f"Blank or unpainted native viewport: {path}"
    if require_colour:
        assert chromatic > 30, f"Overlay colours were not painted: {path}"
    if require_bio_main:
        # The former incorrect generic grid passed a nonblank-image test. The
        # installed main view has full-width orange banner/body/footer bands
        # and #231002 species rows. Read the same factual palette as the source.
        bands=[]
        for y in bio_bands:
            if not bands or y>bands[-1][-1]+1:bands.append([y])
            else:bands[-1].append(y)
        assert sum(len(band)>=10 for band in bands)>=2, f"BioInsights custom main bands missing: {path}"
        assert brown_pixels>width*40, f"BioInsights original row backgrounds missing: {path}"
    return {"width": width, "height": height, "painted_pixels": painted, "chromatic_pixels": chromatic}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("binary", type=Path)
    parser.add_argument("--output", type=Path, default=Path("dist/native-plugin-smoke"))
    parser.add_argument("--wayland", action="store_true", help="Require an existing Wayland display")
    parser.add_argument("--window-size",default="800x500",help="Native main-window client size")
    args = parser.parse_args()
    binary, output = args.binary.resolve(), args.output.resolve()
    assert binary.is_file(), binary
    output.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    if args.wayland:
        assert env.get("WAYLAND_DISPLAY"), "No Wayland display configured"
    else:
        env.pop("WAYLAND_DISPLAY", None)
    env["LIBGL_ALWAYS_SOFTWARE"] = "1"
    captures = {}
    with tempfile.TemporaryDirectory(prefix="observatory-plugin-smoke-") as temporary:
        root = Path(temporary)
        journals = root / "journals"
        journals.mkdir()
        events = fixture()
        (journals / "Journal.2026-10-25T000000.01.log").write_text("".join(json.dumps(event) + "\n" for event in events))
        (journals / "Status.json").write_text(json.dumps({"timestamp": "2026-10-25T00:00:11Z", "event": "Status",
                                                         "BodyName": "Native Smoke A 1", "Latitude": 0,
                                                         "Longitude": 0, "Heading": 0, "PlanetRadius": 250000}))
        for variable, directory in [("XDG_CONFIG_HOME", "config"), ("XDG_DATA_HOME", "data"), ("XDG_CACHE_HOME", "cache")]:
            env[variable] = str(root / directory)
        plugins = root / "data" / "observatory-core" / "plugins"
        plugins.mkdir(parents=True)
        (plugins / "native-smoke.lua").write_text(DRIVER)
        settings = {"journal_dir": str(journals), "poll_interval_ms": 50, "popup": True, "speech": False,
                    "auto_start": True, "start_read_all": True, "theme": "dark",
                    "plugins": {"Hecate": {"RemoteLookups": False},
                                "BioInsights": {"ShowUnlikely": True, "GalacticCodexRemote": False,
                                                "StellarOverlaySendFSSNotification":False,
                                                "StellarOverlaySendDSSNotification":False,"StellarOverlayUseRadar":False},
                                "Evaluator": {"MinimumBodyValue": 0},
                                "StellarOverlay": {"DisplayOverride": True, "TTL": 0,
                                                   "HandleObservatoryNotifications": True,
                                                   "RadarSettings": {"Enabled": True, "PositionXPct": 5,
                                                                     "PositionYPct": 35, "SizePct": 28}}}}
        config = root / "settings.json"
        config.write_text(json.dumps(settings))
        replay = subprocess.run([str(binary), "--headless", "--config", str(config)], env=env,
                                check=True, text=True, capture_output=True, timeout=45)
        data = json.loads(replay.stdout)
        assert data["events"] == len(events), data
        assert data["errors"] == [], data["errors"]
        bio = data["plugins"].get("BioInsights", [])
        evaluator = data["plugins"].get("Evaluator", [])
        assert any(row.get("Species") == "Tectonicas" and row.get("Analysed") for row in bio), bio
        assert any(row.get("BodyName") == "Native Smoke A 1" for row in evaluator), evaluator
        (output / "replay.json").write_text(replay.stdout)
        vault_dir = root / "data" / "observatory-core" / "Hecate"
        with sqlite3.connect(vault_dir / "general.db") as database:
            exploration = database.execute("SELECT SUM(value) FROM evaluator WHERE fid=4242").fetchone()[0]
            biological = database.execute("SELECT SUM(basevalue) FROM bioinsights_unsold WHERE fid=4242").fetchone()[0]
        with sqlite3.connect(vault_dir / "journals.db") as database:
            visits = database.execute("SELECT COUNT(*) FROM journals WHERE id64=42 AND fid=4242").fetchone()[0]
        assert exploration and exploration > 0, exploration
        assert biological == 19_010_800, biological
        assert visits == 1, visits
        app_env = env.copy()
        if args.wayland:
            app_env.pop("DISPLAY", None)
        views = [("core", ["--view", "Core"]), ("bioinsights", ["--view", "BioInsights"]),
                 ("evaluator", ["--view", "Evaluator"]),
                 ("codex-companion", ["--view", "Codex Companion"]),
                 ("unsold-biology", ["--view", "Unsold BioInsights"]),
                 ("bioinsights-about", ["--view", "BioInsights - About"]),
                 ("stellar-notification", ["--screenshot-viewport", "StellarOverlay"]),
                 ("stellar-radar", ["--screenshot-viewport", "StellarOverlay Radar"])]
        for name, options in views:
            # Analysis screenshots show only the installed native ports. The
            # fixture driver is needed only to create the overlay surfaces.
            driver_path=plugins / "native-smoke.lua"
            if name.startswith("stellar-"):
                driver_path.write_text(DRIVER)
            elif driver_path.exists():
                driver_path.unlink()
            # Keep separate startup runs deterministic even if UI window state is saved.
            config.write_text(json.dumps(settings))
            image = output / f"{name}.png"
            rendered = subprocess.run([str(binary), "--config", str(config), "--smoke-frames", "60",
                                       "--window-size",args.window_size,"--screenshot-delay-ms","1000",
                                       "--screenshot", str(image), *options], env=app_env,
                                      text=True, capture_output=True, timeout=45)
            (output / f"{name}.log").write_text(rendered.stdout + rendered.stderr)
            assert rendered.returncode == 0, (name, rendered.stderr)
            assert image.is_file() and image.stat().st_size > 1000, image
            captures[name] = visible_png(image, require_colour=name.startswith("stellar-"),require_bio_main=name=="bioinsights")
            if name in ("bioinsights", "codex-companion", "unsold-biology", "bioinsights-about"):
                reports=[json.loads(line.removeprefix("observatory-smoke ")) for line in rendered.stderr.splitlines()
                         if line.startswith("observatory-smoke ")]
                assert len(reports)==1, (name,rendered.stderr)
                assert reports[0]["native_viewport_count"]==1, (name,reports)
                assert reports[0]["selected_tab"]=="BioInsights", (name,reports)
                captures[name]["native_viewport_count"]=reports[0]["native_viewport_count"]
        receipt = {"events_replayed": data["events"], "bioinsights_rows": len(bio),
                   "evaluator_rows": len(evaluator), "hecate_exploration_value": exploration,
                   "hecate_biology_basevalue": biological, "hecate_journal_visits": visits,
                   "native_plugin_views": captures, "bioinsights_content_viewport":"root",
                   "display_backend": "wayland" if args.wayland else "x11",
                   "synthetic_data_only": True, "remote_lookups": False}
        (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
        print(json.dumps(receipt))


if __name__ == "__main__":
    main()
