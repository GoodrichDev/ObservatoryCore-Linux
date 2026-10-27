#!/usr/bin/env python3
"""Exercise a release ELF, its real window, and a local notification daemon.

Run on Linux inside `dbus-run-session -- xvfb-run -a python3 ...`.
The journals/settings/storage are disposable and contain no player data.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("binary", type=Path)
    parser.add_argument("--output", type=Path, default=Path("dist/native-smoke"))
    parser.add_argument("--wayland", action="store_true", help="Require an existing Wayland display instead of X11")
    args = parser.parse_args()
    binary = args.binary.resolve()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    repo = Path(__file__).resolve().parents[2]
    env = os.environ.copy()
    if args.wayland:
        assert env.get("WAYLAND_DISPLAY"), "No Wayland display configured"
    else:
        env.pop("WAYLAND_DISPLAY", None)
    env["LIBGL_ALWAYS_SOFTWARE"] = "1"
    with tempfile.TemporaryDirectory(prefix="observatory-smoke-") as temporary:
        root = Path(temporary)
        journal_dir = root / "journals"
        journal_dir.mkdir()
        journal = journal_dir / "Journal.2026-01-01T000000.01.log"
        journal.write_bytes((repo / "native/fixtures/parity-journal.jsonl").read_bytes())
        config = root / "settings.json"
        config.write_text(json.dumps({"journal_dir": str(journal_dir), "poll_interval_ms": 50,
                                      "popup": True, "speech": False, "start_read_all": True,
                                      "disabled_plugins":["StellarOverlay"],"plugins":{"Hecate":{"RemoteLookups":False},"BioInsights":{"GalacticCodexRemote":False}}}))
        env["XDG_CONFIG_HOME"] = str(root / "config")
        env["XDG_DATA_HOME"] = str(root / "data")
        env["XDG_CACHE_HOME"] = str(root / "cache")
        replay = subprocess.run([str(binary), "--headless", "--config", str(config), "--export", str(output / "findings.csv")],
                                env=env, check=True, text=True, capture_output=True, timeout=20)
        data = json.loads(replay.stdout)
        assert data["events"] == 19, data
        assert data["findings"] and len(data["messages"]) == 2, data
        assert data["botanist"][0]["analysed"] is True, data
        assert data["errors"] == [], data["errors"]
        (output / "replay.json").write_text(replay.stdout)
        monitor_log = (output / "notifications.txt").open("w")
        monitor = subprocess.Popen(["dbus-monitor", "--session", "interface='org.freedesktop.Notifications'"],
                                   env=env, stdout=monitor_log, stderr=subprocess.DEVNULL)
        daemon = subprocess.Popen(["dunst"], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        app = None
        try:
            app_env = env.copy()
            if args.wayland:
                app_env.pop("DISPLAY", None)  # Backend fallback must not hide a Wayland failure.
            app = subprocess.Popen([str(binary), "--config", str(config), "--smoke-frames", "45",
                                    "--screenshot", str(output / "window.png")],
                                   env=app_env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            time.sleep(1)

            def append(*events):
                with journal.open("a") as stream:
                    for event in events:
                        event.setdefault("timestamp", "2026-10-25T20:00:00Z")
                        stream.write(json.dumps(event) + "\n")

            def status(longitude):
                temporary_status = journal_dir / "Status.new"
                temporary_status.write_text(json.dumps({"timestamp": "2026-10-25T20:00:00Z", "event": "Status",
                                                       "Latitude": 0, "Longitude": longitude, "PlanetRadius": 250000}))
                temporary_status.replace(journal_dir / "Status.json")

            status(0)
            organic = {"event": "ScanOrganic", "SystemAddress": 42, "Body": 2,
                       "Species_Localised": "Smoke Tussock", "Genus": "$Codex_Ent_Tussocks_Genus_Name;"}
            append({"event": "LoadGame", "Commander": "Smoke", "Odyssey": True},
                   {"event": "Location", "StarSystem": "Smoke", "SystemAddress": 42, "BodyID": 2, "Body": "Smoke 1"},
                   {"event": "Touchdown", "SystemAddress": 42, "BodyID": 2, "Body": "Smoke 1", "Latitude":0,"Longitude":0},
                   {"event": "ReceiveText", "Channel": "player", "From": "Test pilot", "Message": "Native notification check"},
                   dict(organic, ScanType="Log"))
            time.sleep(0.5)
            status(0.2)
            time.sleep(0.5)
            append(dict(organic, ScanType="Analyse"))
            stdout, stderr = app.communicate(timeout=20)
            (output / "window.log").write_text(stdout + stderr)
            assert app.returncode == 0, stderr
            assert (output / "window.png").stat().st_size > 1000
            assert "desktop notification:" not in stderr, stderr
        finally:
            if app is not None and app.poll() is None:
                app.kill()
                app.communicate(timeout=5)
            for process in [daemon, monitor]:
                process.terminate()
                process.wait(timeout=5)
            monitor_log.close()
        trace = (output / "notifications.txt").read_text()
        assert trace.count("member=Notify") >= 3, trace
        assert "member=CloseNotification" in trace, trace
        # Render the customized settings page and a real restored plugin window.
        settings=json.loads(config.read_text())
        settings["theme"]="Review Blue"
        settings["themes"]={"Review Blue":{"base":"dark","background":[16,24,32,255],
            "panel":[25,38,52,255],"text":[225,235,245,255],"accent":[105,200,255,255],
            "selection":[35,90,130,255],"widgets":[35,50,65,255],"link":[100,195,245,255]}}
        config.write_text(json.dumps(settings))
        for name,options in [("settings",["--view","Settings"]),
                             ("detached-botanist",["--view","Explorer","--detach","Botanist","--screenshot-viewport","Botanist"]),
                             ("restored-botanist",["--view","Explorer","--screenshot-viewport","Botanist"])]:
            image_path=output/(name+".png")
            rendered=subprocess.run([str(binary),"--config",str(config),"--smoke-frames","12",
                                     "--screenshot",str(image_path),*options],env=app_env,
                                    text=True,capture_output=True,timeout=20)
            (output/(name+".log")).write_text(rendered.stdout+rendered.stderr)
            assert rendered.returncode==0,rendered.stderr
            assert image_path.stat().st_size>1000,image_path
        receipt = {"events_replayed": data["events"], "findings": len(data["findings"]),
                   "native_window": True, "notification_calls": trace.count("member=Notify"),
                   "notification_cancel": True, "custom_theme_and_settings":True,
                   "detached_plugin_window":True,"restored_plugin_window":True,
                   "display_backend": "wayland" if args.wayland else "x11"}
        (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
        print(json.dumps(receipt))


if __name__ == "__main__":
    main()
