#!/usr/bin/env python3
"""Exercise typed mutation echoes and actual idle CLR callbacks over JSONL."""
import json
import os
import pathlib
import select
import subprocess
import sys
import tempfile
import time


def main():
    if len(sys.argv) < 2:
        raise SystemExit("Usage: mutation_roundtrip.py <bridge executable> [arguments]")
    with tempfile.TemporaryDirectory(prefix="observatory-clr-mutation-") as directory:
        script = pathlib.Path(directory) / "criteria.lua"
        script.write_text("""---@Global
luanet.load_assembly('System.ComponentModel.TypeConverter')
Timer = luanet.import_type('System.Timers.Timer')
count = 0
---@End
---@Complex mutable
count = count + 1
if count == 1 then
  held = scan
  scan.BodyName = 'Mutated body'
  scan.Radius = 123
  scan.BodyID = 17
  scan.SystemAddress = 999
  scan.Parent = scan.Parent:Clear()
end
if count == 2 then
  timer = Timer(500)
  timer.AutoReset = false
  timer.Elapsed:Add(function(sender, args)
    held.Radius = 456
    notify('idle mutation', 'timer', held.BodyName)
  end)
  timer:Start()
end
return true, 'mutation', tostring(held:Equals(system[0])) .. '/' .. held.Radius .. '/' .. held.Parent.Count .. '/' .. held.Parents.Count
---@End
""", encoding="utf-8")
        process = subprocess.Popen(sys.argv[1:], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, text=True, env=dict(os.environ, TZ="UTC"))
        def request(value):
            process.stdin.write(json.dumps(value, sort_keys=True) + "\n")
            process.stdin.flush()
            assert select.select([process.stdout], [], [], 5)[0], "Bridge response timed out"
            line = process.stdout.readline()
            assert line, process.stderr.read()
            response = json.loads(line)
            assert response["ok"], response
            assert not response["diagnostics"], response
            return response

        try:
            request({"op": "init", "path": str(script)})
            first = dict(event="Scan", timestamp="2026-10-26T01:02:03Z", SystemAddress=42,
                         BodyID=1, BodyName="Original body", Radius=1, Parents=[{"Star": 0}])
            raw = json.dumps(first)
            result = request({"op": "process", "event": first, "raw_json": raw,
                              "system": [{"system_id": 42, "body_id": 1, "event": first, "raw_json": raw}]})
            update = result["history_updates"][0]
            assert (update["system_id"], update["body_id"], update["kind"]) == (42, 1, "scan"), update
            assert update["event"]["SystemAddress"] == 999 and update["event"]["BodyID"] == 17, update
            assert update["parent"] == [] and update["event"]["Parents"][0]["Star"] == 0, update
            assert result["event"]["BodyName"] == "Mutated body" and result["event_parent"] == [], result
            assert result["event_raw_json"] == raw and update["raw_json"] == raw, result
            second = dict(first, BodyID=2, BodyName="Second body")
            result = request({"op": "process", "event": second, "system": [update, second]})
            assert result["results"][0]["detail"] == "true/123.0/0/1", result
            assert result["history_updates"] == [], result
            time.sleep(0.7)
            idle = request({"op": "poll"})
            assert idle["notifications"][0]["title"] == "idle mutation", idle
            update = idle["history_updates"][0]
            assert (update["system_id"], update["body_id"], update["event"]["Radius"]) == (42, 1, 456), update
            assert request({"op": "poll"})["history_updates"] == [], "Unchanged mutation was emitted twice"
            third = dict(first, BodyID=3, BodyName="Third body")
            result = request({"op": "process", "event": third, "system": [update, second, third]})
            assert result["results"][0]["detail"] == "true/456.0/0/1", result
            assert result["history_updates"] == [], result
            request({"op": "retain_history", "keys": [{"kind": "scan", "system_id": 42, "body_id": 2}]})
            fourth = dict(first, BodyID=4, BodyName="Fourth body")
            result = request({"op": "process", "event": fourth, "system": [second, fourth]})
            assert result["results"][0]["detail"] == "false/456.0/0/1", result
            print("PASS: original dictionary keys, DTO/Parent mutation echoes, CLR identity, dirty-only updates, idle Timer poll, retained Lua references after pruning")
        finally:
            process.stdin.close()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)


if __name__ == "__main__":
    main()
