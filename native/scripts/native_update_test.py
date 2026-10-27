#!/usr/bin/env python3
"""Exercise the actual Core updater, real Rust workers and isolated journals.

No production profile, game journal, vendor DLL or public service is accessed.
The loopback feed serves four real native worker artifacts. Malformed metadata
ELFs are negative controls, never plugin implementations.
"""
import argparse
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import shutil
import signal
import sqlite3
import subprocess
import tempfile
import threading
import time

from plugin_smoke_test import fixture

PLUGINS = ("BioInsights", "Evaluator", "Hecate", "StellarOverlay")
DRIVER_ID = "native.update.qa"
DRIVER = r'''local batch=true
return {
 id='native.update.qa',name='Native Update QA',version='1.0',
 on_load=function() core.set_storage('ready',false) end,
 on_state_changed=function(s)
  batch=s.batch
  if s.state=='Monitoring' then core.set_storage('ready',true) end
 end,
 on_journal=function(e)
  if batch or e.event~='SendText' then return end
  if e.Message=='QA:Pipeline' then
   core.notify({id='00000000-0000-0000-0000-000000000123',title='Native update pipeline',detail='Real persistent notification',timeout=0,x=65,y=12})
   core.send_message({target='StellarOverlay',command='RadarSetPlanet',payload={ID64=42,BodyID=1,RadiusKM=250}})
   core.send_message({target='StellarOverlay',command='RadarSetContextID',payload={ContextID=7,Radius=500}})
   core.send_message({target='StellarOverlay',command='RadarAddBiological',payload={UseCurrentPosition=false,X=0.10,Y=0.10,ContextID=7,Radius=500,Label='Transport sample',MeasureDistance=true}})
  elseif string.sub(e.Message or '',1,11)=='QA:Install:' then
   core.send_message({target='Core',command='InstallUpdate',payload={Plugin=string.sub(e.Message,12)}})
  end
 end
}'''


def sha(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(65536), b""):
            digest.update(chunk)
    return digest.hexdigest()


def wait(predicate, description, timeout=15):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        result = predicate()
        if result:
            return result
        time.sleep(0.025)
    raise AssertionError(f"Timed out: {description}")


class Feed:
    def __init__(self):
        self.manifest = {}
        self.files = {}
        self.requests = []
        self.gates = {}
        owner = self

        class Handler(BaseHTTPRequestHandler):
            def do_GET(self):
                owner.requests.append(self.path)
                if self.path == "/native-plugins.json":
                    body = json.dumps(owner.manifest).encode()
                elif self.path in owner.files:
                    if self.path in owner.gates:
                        requested, release = owner.gates[self.path]
                        requested.set()
                        if not release.wait(20):
                            self.send_error(503)
                            return
                    body = owner.files[self.path]
                else:
                    self.send_error(404)
                    return
                size = body.stat().st_size if isinstance(body, Path) else len(body)
                self.send_response(200)
                self.send_header("Content-Length", str(size))
                self.end_headers()
                try:
                    if isinstance(body, Path):
                        with body.open("rb") as stream:
                            shutil.copyfileobj(stream, self.wfile, 65536)
                    else:
                        self.wfile.write(body)
                except (BrokenPipeError, ConnectionResetError):
                    pass

            def log_message(self, *_):
                pass

        self.server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        self.server.daemon_threads = True
        self.base = f"http://127.0.0.1:{self.server.server_port}"
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()

    def close(self):
        for _, release in self.gates.values():
            release.set()
        self.server.shutdown()
        self.server.server_close()
        self.thread.join()

    def artifact(self, plugin, binary, revision=2, channel="stable", version=None):
        info = json.loads(subprocess.check_output([str(binary), "--kind", plugin, "--hello"], timeout=10))
        name = f"/{plugin.lower()}-r{revision}-{channel}"
        self.files[name] = binary
        return {"plugin": plugin, "version": version or info["source_version"], "revision": revision,
                "channel": channel, "protocol": 1, "url": self.base + name,
                "sha256": sha(binary), "size": binary.stat().st_size}


class Profile:
    def __init__(self, root, name, settings=None):
        self.root = root / name
        self.root.mkdir()
        self.data = self.root / "data/observatory-core"
        self.plugins = self.data / "plugins"
        self.plugins.mkdir(parents=True)
        self.journals = self.root / "journals"
        self.journals.mkdir()
        self.journal = self.journals / "Journal.2026-10-25T000000.01.log"
        self.journal.write_text("".join(json.dumps(e) + "\n" for e in fixture()))
        (self.journals / "Status.json").write_text(json.dumps({"event": "Status", "timestamp": "2026-10-25T00:00:11Z", "Flags": 0, "Flags2": 0, "BodyName": "Native Smoke A 1", "Latitude": 0, "Longitude": 0, "Heading": 0, "Altitude": 0, "PlanetRadius": 250000}))
        (self.plugins / "update-qa.lua").write_text(DRIVER)
        self.config = self.root / "settings.json"
        self.settings = settings or {
            "journal_dir": str(self.journals), "poll_interval_ms": 50, "popup": True, "speech": False,
            "plugins": {"BioInsights": {"GalacticCodexRemote": False, "StellarOverlaySendFSSNotification": False,
                                         "StellarOverlaySendDSSNotification": False, "StellarOverlayUseRadar": False},
                        "Evaluator": {"MinimumBodyValue": 0}, "Hecate": {"RemoteLookups": False},
                        "StellarOverlay": {"DisplayOverride": True, "DelayBetweenNotificationsMS": 0}}}
        self.env = os.environ.copy()
        for variable, directory in [("XDG_CONFIG_HOME", "config"), ("XDG_DATA_HOME", "data"), ("XDG_CACHE_HOME", "cache")]:
            self.env[variable] = str(self.root / directory)
        self.save()

    def save(self):
        self.config.write_text(json.dumps(self.settings))

    def append(self, event):
        event = dict(event)
        event.setdefault("timestamp", "2026-10-26T01:00:00Z")
        with self.journal.open("a") as stream:
            stream.write(json.dumps(event) + "\n")
            stream.flush()

    def command(self, name):
        self.append({"event": "SendText", "Message": name, "To": "local"})

    def metadata(self, plugin):
        path = self.data / "native-plugins" / plugin / "installed.json"
        try:
            return json.loads(path.read_text())
        except FileNotFoundError:
            return None

    def revision(self, plugin):
        metadata = self.metadata(plugin)
        return metadata["artifact"]["revision"] if metadata else None

    def state_hashes(self):
        return {plugin: {name: sha(self.data / "native-plugins" / plugin / name)
                         for name in ("worker", "installed.json")} for plugin in PLUGINS}


class Run:
    def __init__(self, binary, profile, output, label, seconds=4):
        profile.save()
        ready = profile.data / "storage" / DRIVER_ID / "state.json"
        ready.unlink(missing_ok=True)
        self.profile, self.output, self.label = profile, output, label
        self.log = (output / f"{label}.stderr.log").open("w")
        self.process = subprocess.Popen([str(binary), "--config", str(profile.config), "--headless", "--watch-seconds", str(seconds)], env=profile.env, stdout=subprocess.PIPE, stderr=self.log, text=True)

    def ready(self):
        state = self.profile.data / "storage" / DRIVER_ID / "state.json"
        def check():
            assert self.process.poll() is None, f"Core exited before Monitoring ({self.label})"
            try:
                return json.loads(state.read_text()).get("ready")
            except FileNotFoundError:
                return False
        wait(check, f"Core Monitoring {self.label}")

    def finish(self, allow_errors=False):
        stdout, _ = self.process.communicate(timeout=30)
        self.log.close()
        assert self.process.returncode == 0, (self.label, (self.output / f"{self.label}.stderr.log").read_text())
        data = json.loads(stdout)
        (self.output / f"{self.label}.json").write_text(json.dumps(data, indent=2) + "\n")
        if not allow_errors:
            assert not data["errors"], (self.label, data["errors"])
        return data

    def kill(self):
        if self.process.poll() is None:
            self.process.kill()
        self.process.communicate()
        self.log.close()


def workers(core_pid):
    result = {}
    # Linux assigns children to the thread which called fork/posix_spawn. The
    # journal thread owns these workers, rather than the Core main thread.
    children = set()
    for task in Path(f"/proc/{core_pid}/task").iterdir():
        try:
            children.update((task / "children").read_text().split())
        except FileNotFoundError:
            pass
    for value in children:
        try:
            args = Path(f"/proc/{value}/cmdline").read_bytes().split(b"\0")
            if b"--kind" in args and b"--hello" not in args:
                plugin = args[args.index(b"--kind") + 1].decode()
                result[plugin] = int(value)
        except FileNotFoundError:
            pass
    return result


def clean_projection(value):
    if isinstance(value, dict):
        return {key: clean_projection(item) for key, item in value.items()
                if key not in ("sampled_ms", "created_ms", "expires_ms", "opacity")}
    if isinstance(value, list):
        return [clean_projection(item) for item in value]
    return value


def ledgers(profile):
    with sqlite3.connect(profile.data / "Hecate/general.db") as db:
        exploration = db.execute("SELECT SUM(value) FROM evaluator WHERE fid=4242").fetchone()[0]
        biology = db.execute("SELECT SUM(basevalue) FROM bioinsights_unsold WHERE fid=4242").fetchone()[0]
    with sqlite3.connect(profile.data / "Hecate/journals.db") as db:
        visits = db.execute("SELECT COUNT(*) FROM journals WHERE id64=42 AND fid=4242").fetchone()[0]
    return {"exploration": exploration, "biology": biology, "visits": visits}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("worker", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    binary, worker, output = args.binary.resolve(strict=True), args.worker.resolve(strict=True), args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    feed = Feed()
    running = []
    receipt = {"synthetic_data_only": True, "public_network": False, "core_sha256": sha(binary), "worker_sha256": sha(worker), "projection_normalization_ignored": ["sampled_ms", "created_ms", "expires_ms", "opacity"], "checks": {}}
    def start(profile, label, seconds=4):
        print(f"native update QA: {label}", flush=True)
        run = Run(binary, profile, output, label, seconds)
        running.append(run)
        run.ready()
        return run
    try:
        with tempfile.TemporaryDirectory(prefix="native-update-private-", dir=output) as temporary:
            root = Path(temporary)
            inputs = root / "inputs"
            inputs.mkdir()
            shutil.copy2(binary, inputs / "observatory-core")
            shutil.copy2(worker, inputs / "observatory-native-worker")
            binary, worker = inputs / "observatory-core", inputs / "observatory-native-worker"
            receipt["core_sha256"], receipt["worker_sha256"] = sha(binary), sha(worker)
            info = json.loads(subprocess.check_output([str(worker), "--kind", "Hecate", "--hello"]))
            artifacts = {plugin: feed.artifact(plugin, worker, 3 if plugin == "Evaluator" else 2, "beta" if plugin == "Evaluator" else "stable") for plugin in PLUGINS}
            feed.manifest = {"schema": 1, "platform": info["platform"], "core_min": "2.0.0", "plugins": list(artifacts.values())}
            profile = Profile(root, "updater")
            profile.settings["native_update_manifest_url"] = feed.base + "/native-plugins.json"
            profile.settings["disabled_plugins"] = ["Hecate"]
            for plugin in PLUGINS:
                profile.settings["plugins"][plugin]["InstallUpdates"] = plugin in ("Evaluator", "StellarOverlay")
                profile.settings["plugins"][plugin]["InstallBetas"] = False
            policy = start(profile, "01-policy-and-manual")
            wait(lambda: profile.revision("StellarOverlay") == 2, "Stellar automatic update")
            assert profile.revision("BioInsights") is None and profile.revision("Evaluator") is None and profile.revision("Hecate") is None
            assert "/evaluator-r3-beta" not in feed.requests
            profile.command("QA:Install:BioInsights")
            wait(lambda: profile.revision("BioInsights") == 2, "Bio explicit manual installation")
            first = policy.finish()
            assert profile.revision("Hecate") is None
            assert json.loads(profile.config.read_text())["disabled_plugins"] == ["Hecate"]
            receipt["checks"]["automatic_manual_and_disabled_policy"] = {"installed": [p for p in PLUGINS if profile.metadata(p)], "beta_downloaded": False, "disabled_flags_preserved": True}

            profile.settings["plugins"]["Evaluator"]["InstallBetas"] = True
            beta = start(profile, "02-beta-and-hecate-manual")
            profile.command("QA:Install:Hecate")
            wait(lambda: profile.revision("Evaluator") == 3 and profile.revision("Hecate") == 2, "beta opt-in and manual Hecate")
            beta.finish()
            assert json.loads(profile.config.read_text())["disabled_plugins"] == ["Hecate"]
            receipt["checks"]["beta_opt_in"] = {"Evaluator": profile.metadata("Evaluator")["artifact"]["channel"], "disabled_Hecate_preserved": True}

            # Gate the next artifact until all four actual children are running.
            profile.settings["disabled_plugins"] = []
            profile.settings["plugins"]["BioInsights"]["InstallUpdates"] = True
            newer = feed.artifact("BioInsights", worker, revision=3)
            feed.manifest["plugins"] = [newer if a["plugin"] == "BioInsights" else a for a in feed.manifest["plugins"]]
            requested, release = threading.Event(), threading.Event()
            feed.gates["/bioinsights-r3-stable"] = (requested, release)
            active = start(profile, "03-stage-with-active-workers", 5)
            pids = wait(lambda: (p if len(p := workers(active.process.pid)) == 4 else None), "all four real worker processes")
            wait(requested.is_set, "gated artifact download")
            previous = profile.state_hashes()
            old_inode = os.stat(f"/proc/{pids['BioInsights']}/exe").st_ino
            release.set()
            wait(lambda: profile.revision("BioInsights") == 3, "atomic native update replacement")
            assert profile.metadata("BioInsights")["activated"] is False, "Staged revision marked active before its executable loaded"
            assert workers(active.process.pid) == pids
            assert os.stat(f"/proc/{pids['BioInsights']}/exe").st_ino == old_inode
            assert (profile.data / "native-plugins/BioInsights/worker").stat().st_ino != old_inode
            after = profile.state_hashes()
            for plugin in PLUGINS[1:]:
                assert after[plugin] == previous[plugin], plugin
            staged = active.finish()
            assert all(staged["plugin_details"][p]["worker_healthy"] for p in PLUGINS)
            receipt["checks"]["atomic_stage_preserves_active_workers"] = {"pids": pids, "old_executable_inode_retained": True, "other_artifacts_unchanged": True, "activation_deferred_until_restart": True}

            profile.settings["native_update_manifest_url"] = ""
            restart = start(profile, "04-restart-activates")
            new_pids = wait(lambda: (p if len(p := workers(restart.process.pid)) == 4 else None), "restarted external workers")
            assert all(new_pids[p] != pids[p] for p in PLUGINS)
            assert os.stat(f"/proc/{new_pids['BioInsights']}/exe").st_ino == (profile.data / "native-plugins/BioInsights/worker").stat().st_ino
            restarted = restart.finish()
            assert all(profile.metadata(p)["activated"] for p in PLUGINS)
            assert all(restarted["plugin_details"][p]["worker"]["plugin"] == p for p in PLUGINS)
            receipt["checks"]["restart_activates_all_four"] = {"worker_metadata": {p: restarted["plugin_details"][p]["worker"] for p in PLUGINS}, "new_pids": new_pids}

            # Compare complete fresh-session projections and durable values.
            projections = {}
            for mode in ("builtin", "remote"):
                fresh = Profile(root, mode)
                if mode == "remote":
                    shutil.copytree(profile.data / "native-plugins", fresh.data / "native-plugins")
                run = start(fresh, f"05-pipeline-{mode}")
                fresh.command("QA:Pipeline")
                data = run.finish()
                assert all((data["plugin_details"][p]["worker"] is not None) == (mode == "remote") for p in PLUGINS)
                assert all(data["plugin_details"][p]["worker_healthy"] for p in PLUGINS)
                assert data["overlay"]["notifications"], "Real notification missing"
                notification = next(n for n in data["overlay"]["notifications"] if n["title"] == "Native update pipeline")
                assert notification["persistent"] and notification["expires_ms"] is None
                assert notification["created_ms"] <= data["overlay"]["sampled_ms"]
                assert data["overlay"]["radar"]["points"], "Real radar points missing"
                projections[mode] = {key: clean_projection(data[key]) for key in ("plugins", "plugin_views", "bio_summary", "eval_summary", "overlay", "findings")}
                projections[mode]["ledger"] = ledgers(fresh)
                assert projections[mode]["ledger"]["biology"] == 19_010_800
                assert projections[mode]["ledger"]["exploration"] > 0
            assert projections["builtin"] == projections["remote"], "Builtin/remote Core pipeline projections differ; inspect 05 JSON artifacts"
            receipt["checks"]["full_pipeline_equivalence"] = {"compared": list(projections["builtin"]), "ledger": projections["builtin"]["ledger"], "canonical_sha256": hashlib.sha256(json.dumps(projections["builtin"],sort_keys=True,separators=(",", ":")).encode()).hexdigest(), "rows": {p: len(projections["builtin"]["plugins"].get(p,[])) for p in PLUGINS}}

            # A compiled malformed metadata ELF is only a negative control.
            malformed = root / "wrong-protocol-worker"
            bad_info = dict(info, plugin="BioInsights", protocol=999)
            subprocess.run(["cc", "-x", "c", "-", "-o", str(malformed)], input="#include <stdio.h>\nint main(void){puts(" + json.dumps(json.dumps(bad_info)) + ");return 0;}\n", text=True, check=True)
            candidates = {"corrupt": worker, "bad-elf": root / "windows-dll", "wrong-protocol": malformed, "wrong-version": worker}
            candidates["bad-elf"].write_bytes(b"MZ-not-a-Linux-plugin" * 8)
            for name, candidate in candidates.items():
                before = profile.state_hashes()
                artifact = dict(newer, revision=4, url=feed.base + "/negative-" + name, size=candidate.stat().st_size, sha256="0" * 64 if name == "corrupt" else sha(candidate))
                if name == "wrong-version":
                    artifact["version"] = "1.3.999.999"
                feed.files["/negative-" + name] = candidate
                feed.manifest["plugins"] = [artifact]
                profile.settings["native_update_manifest_url"] = feed.base + "/native-plugins.json"
                run = start(profile, f"06-reject-{name}")
                data = run.finish(allow_errors=True)
                assert data["native_updates"]["plugins"]["BioInsights"]["state"] == "Error", data["native_updates"]
                assert data["errors"], name
                assert before == profile.state_hashes(), f"Rejected {name} replaced installed bytes"
                receipt["checks"][f"reject_{name}"] = {"previous_install_unchanged": True, "errors": data["errors"]}

            # Crash one selected child while live events continue through Core.
            failure = Profile(root, "failure-isolation")
            shutil.copytree(profile.data / "native-plugins", failure.data / "native-plugins")
            run = start(failure, "07-runtime-failure-isolation", 5)
            live = wait(lambda: (p if len(p := workers(run.process.pid)) == 4 else None), "failure test workers")
            failure.append({"event": "CodexEntry", "SystemAddress": 42, "BodyID": 1, "Name": "$Codex_Ent_Stratum_07_Emerald;", "EntryID": 1400711})
            state = failure.data / "BioInsights/state.json"
            wait(state.exists, "personal state checkpoint")
            checkpoint = sha(state)
            os.kill(live["BioInsights"], signal.SIGKILL)
            failure.append({"event": "Scan", "SystemAddress": 42, "BodyID": 9, "BodyName": "Native Smoke A 9", "PlanetClass": "Earthlike body", "MassEM": 1, "Radius": 6371000})
            failure.append({"event": "CodexEntry", "SystemAddress": 42, "BodyID": 1, "Name": "$Codex_Ent_Stratum_07_Emerald;", "EntryID": 1400711})
            failure.command("QA:Pipeline")
            data = run.finish(allow_errors=True)
            assert any("BioInsights" in str(error) for error in data["errors"]), data["errors"]
            assert data["plugin_details"]["BioInsights"]["worker_healthy"] is False
            assert all(data["plugin_details"][p]["worker_healthy"] for p in PLUGINS[1:])
            assert sha(state) == checkpoint, "Unhealthy worker overwrote personal Codex checkpoint"
            assert any(row.get("BodyName") == "Native Smoke A 9" for row in data["plugins"]["Evaluator"])
            assert data["overlay"]["notifications"] and data["overlay"]["radar"]["points"]
            receipt["checks"]["runtime_failure_isolation"] = {"personal_checkpoint_unchanged": True, "Evaluator_processed_new_body": True, "Stellar_remained_functional": True, "errors": data["errors"]}
            receipt["feed_requests"] = feed.requests
            (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
            print(json.dumps(receipt), flush=True)
    finally:
        for run in running:
            if run.process.poll() is None:
                run.kill()
        feed.close()


if __name__ == "__main__":
    main()
