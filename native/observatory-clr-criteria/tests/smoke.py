#!/usr/bin/env python3
"""Exercise real CLR objects and protocol isolation with synthetic Lua only."""
import json
import os
import pathlib
import subprocess
import sys
import tempfile


def main():
    command = sys.argv[1:]
    if not command:
        raise SystemExit("Usage: smoke.py <bridge executable> [arguments]")
    with tempfile.TemporaryDirectory(prefix="observatory-clr-test-") as temporary:
        script = pathlib.Path(temporary) / "criteria.lua"
        script.write_text("""---@Global
print('lua-print-stdout'); io.stdout:write('io-stdout\\n')
local Console = luanet.import_type('System.Console'); Console.WriteLine('clr-stdout')
assert(io.read() == nil); assert(Console.ReadLine() == nil)
Math = luanet.import_type('System.Math')
Builder = luanet.import_type('System.Text.StringBuilder')
remembered = nil
---@End
---@Complex real-clr
local builder = Builder()
builder:Append(tostring(Math.Sqrt(81)))
builder:Append('/' .. scan.TimestampDateTime:ToString('yyyy-MM-dd'))
builder:Append('/' .. tostring(scan:Equals(system[system.Count-1])))
if remembered then builder:Append('/' .. tostring(remembered:Equals(system[0]))) end
remembered = scan
return scan:GetType().FullName, builder:ToString()
---@End
""", encoding="utf-8")
        first = {"timestamp": "2026-10-26T01:02:03Z", "event": "Scan", "SystemAddress": 1,
                 "BodyID": 1, "BodyName": "Test 1", "StarSystem": "Test"}
        second = dict(first, BodyID=2, BodyName="Test 2")
        raw = ' { "event" : "ScanBaryCentre", "SystemAddress":1,"BodyID":3 } '
        barycentre = dict(first, BodyID=3, BodyName="Test A-B", PlanetClass="Barycentre", event=None)
        requests = [
            {"op": "ping"},
            {"op": "init", "path": str(script)},
            {"op": "process", "event": first, "system": [first], "signals": []},
            {"op": "process", "event": second, "system": [first, second], "signals": []},
            {"op": "process", "event": barycentre, "event_type": "Scan", "raw_json": raw,
             "system": [first, second, {"event": barycentre, "raw_json": raw}], "signals": []},
        ]
        process = subprocess.run(command, input="".join(json.dumps(x) + "\n" for x in requests),
                                 capture_output=True, text=True, timeout=30, check=True, env=dict(os.environ, TZ="UTC"))
        frames = [json.loads(line) for line in process.stdout.splitlines()]
        assert len(frames) == len(requests), process.stdout
        assert all(frame["ok"] for frame in frames), frames
        assert not any(frame["diagnostics"] for frame in frames), frames
        details = [frame["results"][0]["detail"] for frame in frames[2:]]
        assert details == ["9.0/2026-10-26/true", "9.0/2026-10-26/true/true", "9.0/2026-10-26/true/false"], details
        for marker in ("lua-print-stdout", "io-stdout", "clr-stdout"):
            assert marker in process.stderr, process.stderr
            assert marker not in process.stdout, process.stdout
        print("PASS: real System.Math/StringBuilder/DateTime/DTO methods, object identity, synthetic barycentre, native/managed stdin/stdout isolation")


if __name__ == "__main__":
    main()
