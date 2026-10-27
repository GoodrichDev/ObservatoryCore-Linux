"""Synthetic criteria exercising real NLua CLR and original Framework objects."""
import json
from pathlib import Path

BASE = dict(timestamp="2026-10-26T01:02:03Z", event="Scan", SystemAddress=42,
            StarSystem="CLR Oracle", BodyID=2, BodyName="CLR Oracle 1 a",
            Parents=[{"Planet": 1}, {"Star": 0}], Radius=1234.5,
            Materials=[{"Name": "iron", "Percent": 12.5}, {"Name": "carbon", "Percent": 2}],
            Rings=[{"Name": "CLR Oracle 1 a A Ring", "RingClass": "eRingClass_Rocky", "MassMT": 3.5, "InnerRad": 2000, "OuterRad": 4000}])
PARENT = dict(timestamp=BASE["timestamp"], event="Scan", SystemAddress=42,
              BodyID=1, BodyName="CLR Oracle 1", Parents=[{"Star": 0}], Radius=9000)
STAR = dict(timestamp=BASE["timestamp"], event="Scan", SystemAddress=42,
            BodyID=0, BodyName="CLR Oracle", Parents=[], StarType="K")
CASES = []

def process(event=None, **fields):
    event = BASE if event is None else event
    return dict(op="process", event=event, raw_json=json.dumps(event, separators=(",", ":")), **fields)

def add(name, body, *, global_source="", steps=None, handler="Complex"):
    source = ("---@Global\n" + global_source + "\n---@End\n" if global_source else "")
    source += "---@" + handler + " " + name + "\n" + body + "\n---@End\n"
    CASES.append(dict(id=name, source=source, steps=steps or [process(system=[STAR, PARENT])]))

add("framework-types", '''local t = scan:GetType()
return true, "types", t.FullName .. ";" .. t.BaseType.FullName .. ";" .. scan.Parent[0]:GetType().Name .. ";" .. scan.Parent[0].Item1:ToString() .. ";" .. scan.Parent[0].Item2''')
add("framework-datetime", '''local dt = scan.TimestampDateTime
return true, "date", dt:AddDays(1):ToString("yyyy-MM-ddTHH:mm:ss") .. ";" .. dt:GetType().FullName .. ";" .. dt.DayOfWeek:ToString()''')
add("collection-methods", '''local list = scan.Materials
local slice = list:GetRange(0, 1)
local added = slice:Add(list[1])
return true, "collections", list:GetType().Name .. ";" .. list:IndexOf(list[1]) .. ";" .. tostring(list:Contains(list[0])) .. ";" .. slice.Count .. ";" .. added.Count .. ";" .. tostring(slice[0]:Equals(list[0])) .. ";" .. tostring(added[1]:Equals(list[1]))''')
add("collection-callback", '''local found = scan.Materials:Find(function(item) return item.Name == "carbon" end)
local exists = scan.Materials:Exists(function(item) return item.Percent > 10 end)
return true, "callbacks", found.Name .. ";" .. tostring(exists) .. ";" .. tostring(found:Equals(scan.Materials[1]))''')
add("clr-imports", '''local builder = StringBuilder()
builder:Append("CLR"):Append(":"):Append(Math.Sqrt(81))
return true, "imports", builder:ToString() .. ";" .. DateTime(2026, 10, 26):AddDays(2):ToString("yyyy-MM-dd") .. ";" .. tostring(Math.Round(2.5))''',
    global_source='''luanet.load_assembly("System.Runtime")
luanet.load_assembly("System.Private.CoreLib")
StringBuilder = luanet.import_type("System.Text.StringBuilder")
Math = luanet.import_type("System.Math")
DateTime = luanet.import_type("System.DateTime")''')
add("clr-collections-enum", '''local list = ArrayList()
list:Add(scan)
list:Add(scan.Parent[0].Item1)
local day = Enum.Parse(DayOfWeek.Monday:GetType(), "Monday")
return true, "enum", list.Count .. ";" .. tostring(list[0]:Equals(scan)) .. ";" .. list[1]:ToString() .. ";" .. day:ToString() .. ";" .. day:GetType().FullName''',
    global_source='''luanet.load_assembly("System.Collections.NonGeneric")
ArrayList = luanet.import_type("System.Collections.ArrayList")
Enum = luanet.import_type("System.Enum")
DayOfWeek = luanet.import_type("System.DayOfWeek")''')
add("framework-import", '''local other = Scan()
local ring = Ring()
return true, "framework-import", other:GetType().FullName .. ";" .. ring:GetType().FullName .. ";" .. tostring(other:Equals(scan))''',
    global_source='''luanet.load_assembly("ObservatoryFramework")
Scan = luanet.import_type("Observatory.Framework.Files.Journal.Scan")
Ring = luanet.import_type("Observatory.Framework.Files.ParameterTypes.Ring")''')
add("framework-own-methods", '''local reputation = Reputation()
reputation.Empire = 91
reputation.Federation = 35
reputation.Independent = -35
reputation.Alliance = -90
local message = PluginMessage("synthetic", nil, nil)
message.MessagePayload:Add("scan", scan)
local copy = message:Clone()
return true, "framework-methods", reputation:EmpireText() .. ";" .. reputation:FederationText() .. ";" .. reputation:IndependentText() .. ";" .. reputation:AllianceText() .. ";" .. copy.MessageType .. ";" .. tostring(copy:Equals(message)) .. ";" .. tostring(copy.MessagePayload:Equals(message.MessagePayload)) .. ";" .. tostring(copy.MessagePayload["scan"]:Equals(scan)) .. ";" .. tostring(copy.MessageId:Equals(message.MessageId)) .. ";" .. JournalUtilities.GetEventType(scan.Json)''',
    global_source='''luanet.load_assembly("ObservatoryFramework")
Reputation = luanet.import_type("Observatory.Framework.Files.Journal.Reputation")
PluginMessage = luanet.import_type("Observatory.Framework.PluginMessage")
JournalUtilities = luanet.import_type("Observatory.Framework.Files.JournalUtilities")''')
add("clr-ref-out-conversion", '''local good, parsed = Int32.TryParse("42")
local bad, rejected = Int32.TryParse("invalid")
return true, "ref-out", tostring(good) .. ";" .. parsed .. ";" .. tostring(bad) .. ";" .. rejected .. ";" .. Convert.ToString(scan.Radius)''',
    global_source='''luanet.load_assembly("System.Private.CoreLib")
Int32 = luanet.import_type("System.Int32")
Convert = luanet.import_type("System.Convert")''')
add("framework-assembly-identity", '''return true, "assembly", scan:GetType().Assembly.FullName''')
add("event-identity", '''local current = nil
for body in bodies(system) do if body.BodyID == scan.BodyID then current = body end end
local same = current:Equals(scan)
local parent = parents[0].Scan
local cachedSame = cachedParent == nil or cachedParent:Equals(parent)
cachedParent = parent
return true, "identity", tostring(same) .. ";" .. tostring(cachedSame) .. ";" .. parent.BodyName''',
    steps=[process(system=[STAR, PARENT]), process(BASE | {"BodyID": 3, "BodyName": "CLR Oracle 1 b"}, system=[STAR, PARENT, BASE])])
add("retained-callback-gc", '''if held == nil then
  held = scan.Materials:Find(function(item) return item.Name == "iron" end)
  predicate = function(item) return item:Equals(held) end
end
collectgarbage("collect")
return true, "retained", held.Name .. ";" .. tostring(scan.Materials:Exists(predicate))''',
    steps=[process(), process(BASE | {"BodyID": 3})])
add("clr-exception-disable", '''local bad = scan.Materials:GetRange(50, 1)
return true, "unreachable", "unreachable"''', steps=[process(), process()])
add("json-setter-exception", '''scan.Json = "replacement"
return true, "unreachable", "unreachable"''', steps=[process(), process()])
add("callback-exception-disable", '''scan.Materials:Exists(function(item) error("callback failure") end)
return true, "unreachable", "unreachable"''', steps=[process(), process()])
add("callback-notify", '''notify("CLR title", jump.StarPos:GetType().FullName, jump.TimestampDateTime:ToString("s"))
notifyForBody("Body title", "detail", "extended", 2)''', handler="Jump",
    steps=[process(dict(timestamp=BASE["timestamp"], event="FSDJump", SystemAddress=42,
                        StarSystem="CLR Oracle", StarPos=[1.25, -2.5, 3.75]))])
add("allbodies-dictionary", '''local list = parentsTable[2]
local matching = nil
for body in bodies(system) do if body.BodyID == 1 then matching = body end end
return notify("allbodies", tostring(parentsTable:ContainsKey(2)) .. ";" .. list[0].Scan:GetType().Name .. ";" .. tostring(list[0].Scan:Equals(matching)), "")''', handler="AllBodies",
    steps=[process(), process(dict(timestamp=BASE["timestamp"], event="FSSAllBodiesFound", SystemAddress=42), system=[STAR, PARENT])])
add("readall-recreates-clr-state", '''count = count + 1
builder:Append(count)
return true, "state", builder:ToString()''',
    global_source='''luanet.load_assembly("System.Private.CoreLib")
Builder = luanet.import_type("System.Text.StringBuilder")
builder = Builder()
count = 0''', steps=[process(), process(), dict(op="set_read_all", value=True), process(), dict(op="set_read_all", value=False), process()])
new_path_source = '''---@Global
notify("new-path", "global", "")
---@End
---@Complex changed
return true, "path", "new"
---@End
'''
add("path-readall-shared-settings", 'return true, "path", "old"',
    global_source='notify("old-path", "global", "")',
    steps=[process(), dict(op="set_path", source=new_path_source),
           dict(op="set_read_all", value=True), process(),
           dict(op="set_read_all", value=False), process()])
reload_source = '''---@Global
luanet.load_assembly("System.Private.CoreLib")
Builder = luanet.import_type("System.Text.StringBuilder")
builder = Builder()
notify("reload", "global", "")
---@End
---@Complex reload
builder:Append("x")
return true, "reloaded", builder:ToString()
---@End
'''
latest_source = '''---@Complex latest
return true, "reloaded", "latest"
---@End
'''
add("mtime-reload-recovers-failing-criteria", 'error("synthetic criterion failure")',
    steps=[process(), process(), dict(op="rewrite", source=reload_source), process(), process(),
           dict(op="set_read_all", value=True), dict(op="rewrite", source=latest_source), process(),
           dict(op="set_read_all", value=False), process()])
add("mutable-scan-parent-history", '''if saved == nil then
  saved = scan
  scan.BodyName = "mutated current body"
  scan.Radius = 4321
  parents[0].Scan.BodyName = "mutated parent body"
  parents[0].Scan.Radius = 99
  return true, "mutated", scan.BodyName .. ";" .. scan.Radius .. ";" .. parents[0].Scan.BodyName
end
local previous = nil
for body in bodies(system) do if body.BodyID == 2 then previous = body end end
return true, "retained-mutation", previous.BodyName .. ";" .. previous.Radius .. ";" .. tostring(saved:Equals(previous)) .. ";" .. parents[0].Scan.BodyName .. ";" .. parents[0].Scan.Radius''',
    steps=[process(system=[STAR, PARENT]), process(BASE | {"BodyID": 3}, system=[STAR, PARENT, BASE])])
add("mutated-id-preserves-original-history-key", '''if heldParent == nil then
  heldParent = parents[0].Scan
  parents[0].Scan.BodyID = 17
  scan.BodyID = 22
end
return true, "original-key", parents[0].Body .. ";" .. parents[0].Scan.BodyID .. ";" .. tostring(heldParent:Equals(parents[0].Scan))''',
    steps=[process(system=[STAR, PARENT]), process(BASE | {"BodyID": 3}, system=[STAR, PARENT, BASE])])
add("mutable-parent-projection", '''if savedProjection == nil then
  savedProjection = scan
  scan.Parent = parents[0].Scan.Parent
end
return true, "parent-projection", savedProjection.Parent[0].Item1:ToString() .. ";" .. tostring(savedProjection.Parents[0].Planet) .. ";" .. tostring(savedProjection.Parent:Equals(parents[0].Scan.Parent))''',
    steps=[process(system=[STAR, PARENT]), process(BASE | {"BodyID": 3}, system=[STAR, PARENT, BASE])])
signal_event = dict(timestamp=BASE["timestamp"], event="FSSBodySignals", SystemAddress=42, BodyID=2,
                    BodyName=BASE["BodyName"], Signals=[{"Type": "$SAA_SignalType_Biological;", "Count": 3}])
CASES.append(dict(id="mutable-signal-identity-original-key", source='''---@BodySignals mutation
if heldSignal == nil then
  heldSignal = bodySignals
  bodySignals.Signals[0].Count = 9
  bodySignals.BodyID = 999
else
  notify("signal-identity", tostring(heldSignal:Equals(bodySignals)), tostring(bodySignals.Signals[0].Count))
end
---@End
---@Complex signals
return true, "signal-mutation", tostring(biosignals) .. ";" .. heldSignal.BodyID
---@End
''', steps=[process(signal_event), process(), process(signal_event), process()]))
add("idle-timer-callback", '''timer = Timer(500)
timer.AutoReset = false
timer.Elapsed:Add(function(sender, args) notify("idle-timer", "callback", args:GetType().Name) end)
timer:Start()
return true, "timer-started", ""''',
    global_source='''luanet.load_assembly("System.ComponentModel.TypeConverter")
Timer = luanet.import_type("System.Timers.Timer")''',
    steps=[process(), dict(op="wait", milliseconds=1000)])
add("idle-timer-mutates-retained-scan", '''if savedTimerScan == nil then
  savedTimerScan = scan
  timer = Timer(500)
  timer.AutoReset = false
  timer.Elapsed:Add(function(sender, args)
    savedTimerScan.BodyName = "timer-mutated body"
    savedTimerScan.Radius = 888
    notify("idle-mutation", savedTimerScan.BodyName, tostring(savedTimerScan.Radius))
  end)
  timer:Start()
  return true, "timer-started", ""
end
return true, "after-timer", savedTimerScan.BodyName .. ";" .. savedTimerScan.Radius''',
    global_source='''luanet.load_assembly("System.ComponentModel.TypeConverter")
Timer = luanet.import_type("System.Timers.Timer")''',
    steps=[process(), dict(op="wait", milliseconds=1000), process(BASE | {"BodyID": 3})])
add("mutable-nonfinite-clr-numbers", '''if heldNumbers == nil then
  heldNumbers = scan
  scan.Radius = 0/0
  scan.SurfaceGravity = math.huge
  scan.DistanceFromArrivalLS = -math.huge
  scan.Materials[0].Percent = 0/0
end
return true, "nonfinite", tostring(heldNumbers.Radius) .. ";" .. tostring(heldNumbers.SurfaceGravity) .. ";" .. tostring(heldNumbers.DistanceFromArrivalLS) .. ";" .. tostring(heldNumbers.Materials[0].Percent)''',
    steps=[process(), process(BASE | {"BodyID": 3})])

if __name__ == "__main__":
    path = Path(__file__).with_name("clr-criteria-inputs.json")
    path.write_text(json.dumps(CASES, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote {len(CASES)} synthetic CLR scenarios to {path}")
