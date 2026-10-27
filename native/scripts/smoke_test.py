#!/usr/bin/env python3
"""Exercise a release ELF, its real window, and a local notification daemon.

Run on Linux inside `dbus-run-session -- xvfb-run -a python3 ...`.
The journals/settings/storage are disposable and contain no player data.
"""
import argparse
import ctypes as c
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time


READINESS_DRIVER = """return {
 id='native.smoke.readiness',name='Native Smoke Readiness',version='1',
 on_load=function() core.set_storage('ready',false) end,
 on_state_changed=function(s)
  if s.state=='Monitoring' and not s.batch then core.set_storage('ready',true) end
 end,
 on_status=function(s) core.set_storage('longitude',s.Longitude or -999) end,
 on_journal=function(e)
  if e.event=='ScanOrganic' and e.Species_Localised=='Smoke Tussock' then
   core.set_storage('scan_type',e.ScanType)
   core.set_storage('scan_was_batch',core.IsLogMonitorBatchReading)
  end
 end
}"""


def close_owned_x11_window(pid):
    """Request a normal close only for the mapped Core window owned by pid."""
    from core_parity_ui_test import X11
    connection = X11()
    try:
        atom = connection.x.XInternAtom(connection.display,b"_NET_WM_PID",False)
        owned = []
        for window,title in connection.windows().items():
            if not title.startswith("Elite Observatory Core"):
                continue
            kind,fmt,count,remaining = c.c_ulong(),c.c_int(),c.c_ulong(),c.c_ulong()
            data = c.POINTER(c.c_ubyte)()
            connection.x.XGetWindowProperty(connection.display,window,atom,0,1,False,0,
                c.byref(kind),c.byref(fmt),c.byref(count),c.byref(remaining),c.byref(data))
            try:
                if data and fmt.value==32 and count.value==1 and c.cast(data,c.POINTER(c.c_ulong))[0]==pid:
                    owned.append(window)
            finally:
                if data:connection.x.XFree(data)
        assert len(owned)==1, f"Expected one Core window owned by {pid}; found {owned}"
        event = connection.Client()
        event.type,event.display,event.window,event.format = 33,connection.display,owned[0],32
        event.message_type = connection.x.XInternAtom(connection.display,b"WM_PROTOCOLS",False)
        event.data[0] = connection.x.XInternAtom(connection.display,b"WM_DELETE_WINDOW",False)
        assert connection.x.XSendEvent(connection.display,owned[0],False,0,c.byref(event))
        connection.x.XSync(connection.display,0)
    finally:
        connection.x.XCloseDisplay.argtypes = [c.c_void_p]
        connection.x.XCloseDisplay(connection.display)


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
                                      "popup": True, "speech": False, "auto_start": True, "start_read_all": True,
                                      "notification_backend": "dbus",
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
        plugins = root / "data" / "observatory-core" / "plugins"
        plugins.mkdir(parents=True,exist_ok=True)
        (plugins / "smoke-readiness.lua").write_text(READINESS_DRIVER)
        state_file = root / "data" / "observatory-core" / "storage" / "native.smoke.readiness" / "state.json"
        monitor_log = (output / "notifications.txt").open("w")
        monitor = subprocess.Popen(["stdbuf", "-oL", "dbus-monitor", "--session", "interface='org.freedesktop.Notifications'"],
                                   env=env, stdout=monitor_log, stderr=subprocess.DEVNULL)
        daemon = subprocess.Popen(["dunst"], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        app = None
        app_log = (output / "window.log").open("w")

        def wait_for(predicate,description,seconds=20):
            deadline = time.monotonic()+seconds
            while time.monotonic()<deadline:
                for process,name in [(monitor,"D-Bus monitor"),(daemon,"notification daemon"),(app,"native application")]:
                    assert process is None or process.poll() is None, f"{name} exited before {description}"
                value = predicate()
                if value:return value
                time.sleep(.025)
            raise AssertionError(f"Timed out waiting for {description}; see {output}")

        def state():
            try:return json.loads(state_file.read_text())
            except (FileNotFoundError,json.JSONDecodeError):return {}

        def trace():
            return (output / "notifications.txt").read_text()

        try:
            wait_for(lambda:"NameAcquired" in trace(),"D-Bus monitor registration")
            def daemon_owner():
                result = subprocess.run(["dbus-send","--session","--print-reply","--reply-timeout=1000",
                    "--dest=org.freedesktop.DBus","/org/freedesktop/DBus","org.freedesktop.DBus.GetNameOwner",
                    "string:org.freedesktop.Notifications"],env=env,text=True,capture_output=True,timeout=2)
                return result.stdout if result.returncode==0 else None
            (output / "daemon-owner.txt").write_text(wait_for(daemon_owner,"notification daemon ownership"))
            app_env = env.copy()
            if args.wayland:
                app_env.pop("DISPLAY", None)  # Backend fallback must not hide a Wayland failure.
            # The harness closes this window only after readiness and the real
            # notification lifecycle; rendering speed cannot end the live probe.
            began = time.monotonic()
            app = subprocess.Popen([str(binary), "--config", str(config),
                                    "--screenshot", str(output / "window.png")],
                                   env=app_env, stdout=app_log, stderr=subprocess.STDOUT)
            wait_for(lambda:state().get("ready"),"native worker Monitoring state")
            ready_seconds = time.monotonic()-began
            wait_for(lambda:(output / "window.png").is_file() and (output / "window.png").stat().st_size>1000,
                     "real native window capture")

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
            wait_for(lambda:state().get("longitude")==0,"initial live status")
            organic = {"event": "ScanOrganic", "SystemAddress": 42, "Body": 2,
                       "Species_Localised": "Smoke Tussock", "Genus": "$Codex_Ent_Tussocks_Genus_Name;"}
            append({"event": "LoadGame", "Commander": "Smoke", "Odyssey": True},
                   {"event": "Location", "StarSystem": "Smoke", "SystemAddress": 42, "BodyID": 2, "Body": "Smoke 1"},
                   {"event": "Touchdown", "SystemAddress": 42, "BodyID": 2, "Body": "Smoke 1", "Latitude":0,"Longitude":0},
                   {"event": "SAASignalsFound", "SystemAddress": 42, "BodyID": 2, "BodyName": "Smoke 1",
                    "Signals": [{"Type": "$SAA_SignalType_Biological;", "Count": 1}]},
                   {"event": "ReceiveText", "Channel": "player", "From": "Test pilot", "Message": "Native notification check"},
                   dict(organic, ScanType="Log"))
            wait_for(lambda:state().get("scan_type")=="Log" and state().get("scan_was_batch") is False,
                     "live organic log dispatch")
            wait_for(lambda:trace().count('string "Smoke Tussock"')>=1,"sampler show notification")
            shown = trace().count('string "Smoke Tussock"')
            status(0.2)
            wait_for(lambda:state().get("longitude")==0.2,"updated live status")
            wait_for(lambda:trace().count('string "Smoke Tussock"')>shown,"sampler update notification")
            append(dict(organic, ScanType="Analyse"))
            wait_for(lambda:state().get("scan_type")=="Analyse" and state().get("scan_was_batch") is False,
                     "live organic analysis dispatch")
            wait_for(lambda:"member=CloseNotification" in trace(),"sampler cancellation")
            wait_for(lambda:trace().count("member=Notify")>=3,"all live notification calls")
            (output / "monitoring-state.json").write_text(json.dumps(state(),indent=2)+"\n")
            if args.wayland:
                # Generic Wayland clients cannot request another client's close.
                # All assertions above completed before this controlled stop.
                app.terminate()
            else:
                close_owned_x11_window(app.pid)
            app.wait(timeout=20)
            app_log.close()
            window_log = (output / "window.log").read_text()
            assert app.returncode == (-signal.SIGTERM if args.wayland else 0), window_log
            assert "desktop notification:" not in window_log, window_log
        finally:
            if app is not None and app.poll() is None:
                app.kill()
                app.wait(timeout=5)
            app_log.close()
            for process in [daemon, monitor]:
                process.terminate()
                process.wait(timeout=5)
            monitor_log.close()
        notification_trace = trace()
        assert notification_trace.count("member=Notify") >= 3, notification_trace
        assert notification_trace.count('string "Smoke Tussock"') >= 2, notification_trace
        assert "Sample 1 of 3" in notification_trace and "Colony distance: 200m" in notification_trace, notification_trace
        assert "member=CloseNotification" in notification_trace, notification_trace
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
                   "native_window": True, "notification_calls": notification_trace.count("member=Notify"),
                   "monitoring_ready_seconds":ready_seconds,"live_dispatch_outside_batch":True,
                   "controlled_window_stop":"SIGTERM" if args.wayland else "WM_DELETE_WINDOW",
                   "notification_cancel": True, "custom_theme_and_settings":True,
                   "detached_plugin_window":True,"restored_plugin_window":True,
                   "display_backend": "wayland" if args.wayland else "x11"}
        (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
        print(json.dumps(receipt))


if __name__ == "__main__":
    main()
