# Explorer CLR criteria runtime

The optional `observatory-clr-criteria` Linux process runs Explorer annotated Lua
criteria with the original `CustomCriteriaManager`, NLua 1.7.3, KeraLua 1.4.1,
and the complete Observatory Framework source types. The application, core,
journal reader, history ownership, built-in workers and UI remain Rust. This
process supplies actual CLR objects and reflection, including `LoadCLRPackage`,
`luanet.import_type`, `luanet.load_assembly`, constructors, properties, methods,
collections, delegates and events supported by NLua. It does not translate API
names into a whitelist of substitute functions.

The original files are linked by the projects; there is no copied or modified
criteria parser, helper library or DTO implementation. The dedicated Framework
project emits `ObservatoryFramework.dll` using the complete repository sources
without building the desktop application or the plugin's Windows deployment
targets. Reflection requires an untrimmed runtime. Lua native libraries and the
.NET runtime are included by self-contained publishing.

The runtime targets the original application's .NET 8 API surface and pins
the SDK's .NET 8.0.33 servicing runtime. Build on Linux with a .NET 8 or newer SDK:

```sh
bash native/observatory-clr-criteria/build.sh /path/to/runtime
python3 native/observatory-clr-criteria/tests/smoke.py /path/to/runtime/observatory-clr-criteria
python3 native/observatory-clr-criteria/tests/mutation_roundtrip.py /path/to/runtime/observatory-clr-criteria
```

Ship the complete output directory, including native `liblua54.so`, managed
assemblies, runtime configuration and .NET runtime. The executable takes no
arguments and uses persistent JSONL on stdin/stdout. `--version` prints a
human-readable version and exits. It inherits the application's working
directory, environment and local timezone, matching normal CLR/Lua file and
date behavior. User criteria and user assemblies execute with the application's
OS permissions. Windows-only assemblies and operating-system-specific APIs
retain their normal .NET Linux compatibility limits.

`DOTNET_COMMAND` selects an SDK outside `PATH`. The build uses the committed
NuGet lockfile and emits `source-provenance.json` with the SHA-256 of every
linked original source and a deterministic aggregate hash.

## Protocol version 1

Each request is one JSON object on one line. Each response is exactly one line.
The response schema is:

```json
{"ok":true,"error":null,"protocol":1,"runtime":".NET","nlua":"1.7.3","results":[{"title":"Criterion","detail":"Text","system_wide":false}],"notifications":[{"time":"2026-10-26 01:02:03","title":"Title","detail":"Text","extended_detail":"Longer text","body_id":null}],"diagnostics":[{"message":"Exception message","detail":"Original Explorer diagnostic","exception_type":"NLua.Exceptions.LuaScriptException"}]}
```

Operations:

| Request | Behavior |
| --- | --- |
| `{"op":"ping"}` | Return protocol/runtime metadata. |
| `{"op":"init","path":"/path/criteria.lua","read_all":false}` | Create the original manager; original global code executes immediately when the path exists. |
| `{"op":"set_path","path":"/path/criteria.lua"}` | Mutate the shared Explorer settings; the manager detects changes lazily as in the original application. |
| `{"op":"set_read_all","value":true}` | Apply the original read-all transition and forced reload. |
| `{"op":"reset"}` | Force the original reload path while preserving mode and previous notification event time. |
| `{"op":"clear_history"}` | Drop retained CLR history identities after native history is cleared. |
| `{"op":"retain_history","keys":[{"kind":"scan","system_id":42,"body_id":1}]}` | Trim cached scan/signal history to original native dictionary keys; Lua-held references remain valid. |
| `{"op":"poll"}` | Collect asynchronous callbacks and dirty CLR history updates without dispatching a journal or reloading criteria. |
| `{"op":"process","event":{},"event_type":"Scan","raw_json":"...","system":[],"signals":[]}` | Dispatch a typed journal through the original manager. |

`event_type` and `raw_json` are optional. `event_type` selects the CLR journal
type independently of the snapshot's `event` property. This is necessary for
synthetic barycentre Scans, whose `Event` is null in the original application.
`raw_json` sets the exact `JournalBase.Json` string; the typed properties are
deserialized from `event`. A history array entry accepts an ordinary journal
object or `{"event":{...},"raw_json":"..."}` to retain the corresponding
original journal bytes, including the synthetic barycentre's different source.
History wrappers also accept `system_id` and `body_id` for the original native
dictionary keys, independent of mutable `SystemAddress`/`BodyID` DTO properties.
Scan wrappers accept `parent`, an array of `{parent_type:"Star",body:0}` or null,
to retain the separately mutable, JsonIgnored `Scan.Parent` projection without
changing its original `Scan.Parents` representation. System entries are typed
Scans and their array order is preserved. The current
scan shares identity with its matching history entry. Unchanged history
snapshots preserve their CLR identities and Lua-held object references across
requests. No scan is added to history unless supplied in the native snapshot.
`retain_history` accepts optional `kinds:["scan"]` or `["signal"]` to prune only
the selected cache while preserving the other history, as the original Explorer
does when trimming scanned systems.

Responses include `history_updates`, containing only changed cached objects as
`{kind:"scan"|"signal",system_id,body_id,event,raw_json,parent}`. `parent` is
included for scans. The Rust owner applies these to its native history under the
original dictionary keys. Subsequent echoed typed snapshots reuse the same CLR
objects, including modifications performed by asynchronous callbacks between
requests. First FSS signal events share the history object; later duplicates
reach the handler as distinct objects while the first history object remains.
Process responses also include the mutated current journal in `event`, its exact
source in `event_raw_json`, and its parent projection in `event_parent`, for
source-compatible result rows and notification formatting. CLR non-finite float
values use the JSON strings `NaN`, `Infinity` and `-Infinity` on the wire.

The parent polls while idle so CLR events such as `System.Timers.Timer.Elapsed`
can deliver `notify` calls and history mutations without a subsequent journal.

The manager retains its original helpers, annotation parsing, global execution
order, file modification checks, read-all reload behavior, failed-handler
suppression and callbacks. Scan criteria results and immediate `notify` calls
are returned separately so Rust can preserve their original row and notification
semantics. `notify` event time intentionally follows the original manager, which
updates it on custom jump/discovery/signals/all-bodies calls and not on scans.

Protocol stdout uses a private duplicate of its file descriptor. Native Lua
`print`/`io.stdout` and managed `Console.WriteLine` go to stderr on Linux; they
cannot accidentally insert lines into JSON responses. Protocol stdin also uses
a private descriptor; ordinary Lua and CLR standard input return EOF, so script
reads cannot consume protocol requests. Requests are limited to 64 MiB. The parent should impose
a request deadline, response-size cap and process cleanup on failure. A killed
runtime must not silently replay arbitrary script side effects. This runtime
is a process boundary, not an operating-system sandbox.
