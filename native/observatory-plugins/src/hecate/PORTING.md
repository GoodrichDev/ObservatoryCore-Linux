# Hecate 1.3.444.761 native port evidence

The installed plugin identifies itself as **Hecate - Data Vault** and declares
`PluginUIType.None`. It is a background data and message service; a scan table
would add a UI that this version never had.

## Inspected inputs

- Installed archive: `C:\Program Files\Elite Observatory\plugins\Hecate.1.3.444.761.eop`.
  SHA-256: `C6BD71FF656D680739586BB8CB7F99D82D9349CE15B8C522828C8D6DF6F2CC28`.
- Extracted `Hecate.dll` SHA-256:
  `9779AE627496AA9B0A8ADF1C1A886A9251453E816584D0691558956EDADDF5D7`.
- Static IL, decompiled types, and decoded string arguments were inspected.
  No plugin constructor, journal callback, updater, or web server was executed.
  Embedded CSV files were read with `PEReader` and `MetadataReader`, without
  loading the assembly into an execution context.
- Types used: `HecateWorker`, `DBH`, `SystemDBH`, `JournalHelper`,
  `HecateServer2`, `HecateSettings`, `ServerSettings`, `JournalGenerator2`,
  `SpanshToJournal`, `Conversions`, and `JournalDateGenerator`.
- `identifiers.json` contains the factual ID/name rows used by that installed
  version: bodies, rings, SAA signals, genera, commodities, economy, government,
  security, and allegiance. Reverse maps preserve first-match CSV ordering,
  including the duplicate Clypeus spelling. No executable vendor code is shipped.
- Primary author page: <https://edjp.colacube.net/observatory>.
  Related upstream ID collection: <https://github.com/EDCD/FDevIDs>.

## Behavior and implementation

| Installed behavior | Rust implementation | Verification |
|---|---|---|
| Background plugin, no scan findings | `Hecate::on_event`, `HAS_UI=false` | Journal tests return no findings |
| Four SQLite databases | `open`, `schema`, `SystemStore` | Close/reopen test with real SQLite files |
| Zstandard UTF-8 journal documents | `SystemStore::add`, `read_document` | Zstd magic checked; recovered journal values compared |
| Journal visits and per-event commander association | `LoadSystem`, `GetSystemsVisitedFromList`, `GetFID` | Exact `u64::MAX` address, unknown event fields, parallel FIDs |
| Explicit full-history rebuild | `begin_replay`, `set_batch` | Repeated rebuilds retain one visit and one value row |
| Evaluator unsold exploration ledger | Evaluator add/delete/data/total messages | Upsert, aggregation, commander separation, opaque response token |
| BioInsights unsold biology and bonus overrides | BioInsights add/delete/data/total/bonus messages | Unknown bonus is fivefold; explicit false survives upsert |
| PowerInsights merit history and six scopes | `PIAddMerits`, `PIGetMerits`, `PIGetCycle`, `GetCurrentMerits` | Duplicate exclusion and cycle/day/session/system scopes |
| NPC census, two-hour/system change rule | Decorated NPC journal fields, SQL census upsert | Player messages excluded; one-hour repeat ignored |
| Spansh and Canonn daily document caches | Queued host HTTP requests, `handle_http_response` | No callback network I/O; pending requests shared among consumers |
| Spansh dump journal reconstruction | `from_spansh` | Star/planet units, body/genus IDs, barycentres, ring hotspots, discovery events |
| Experimental browser bridge, disabled by default | `server::Server` | Real loopback HTTP and WebSocket round trips |
| Plugin registration and targeted routing | `register_plugin`, `take_messages` | Registered target carried through HTTP and WS request |
| HTTP content/status/redirect and WS response/broadcast | `respond_web`, native message handlers | Text status response, WS response and broadcast, shutdown |

The original URLs are retained:

- `https://spansh.co.uk/api/dump/{id64}`
- `https://us-central1-canonn-api-236217.cloudfunctions.net/query/codex/bodies/{id64}`

Requests use the host's asynchronous HTTP service. Historical batch reading
does not trigger automatic remote requests. `RemoteLookups=false` is a native
setting allowing the cache and local vault to operate offline.

## Native host contract

- Construct persistent storage with `Hecate::open(settings, directory)`.
- Submit `take_http_requests()` entries `{request_id,url,method,timeout_ms}` to
  the host HTTP service; return `{request_id,ok,body}` to `handle_http_response`.
- Regularly drain `take_messages()`; it also polls the optional web bridge.
- Register plugin sender identity through `register_plugin(name,target)`.
- `WebRequest` payload is `{URI,RequestID,Response:{}}`. The original .NET
  response dictionary was mutable shared state. Rust returns its completed
  dictionary through `respond_web(RequestID,response)` or `WebResponse` with
  `{RequestID,Response}`. Fields: `Content`, `ContentType`, `StatusCode`,
  optional `Location`. `Status` is accepted for plugin versions using that key.
- `WebSocketRequest` uses `{URI,ClientID}`. `URI` is the client's text message,
  matching this installed version. Response and broadcast commands use
  `{ClientID,Json}` and `{Plugin,Json}` respectively.
- Call `flush()` at application shutdown; dropping the vault also flushes.

## Deliberate native differences and validation limits

`reset()` clears transient state and flushes durable data. Destructive Read All
reconstruction is explicit in `begin_replay()`. Unknown journal fields are
preserved. Documents, connection counts, message queues, and HTTP waits are
bounded; SQLite uses WAL. IDs retain their full 64-bit bit pattern in SQLite.

`InstallUpdates`, `InstallBetas` and `VerboseLogging` are exposed by the original
settings groups and handled by the shared native host. Web Server Settings is
a separate native form with source validation. Reset Journal Database truncates
only the persisted journal store and preserves current visit and other caches.
The bridge index follows the source HTML/CSS and `/status` lists live clients.
See `native/HECATE_STELLAR_AUDIT.md` and its machine-readable source inventory.

Tests cover the contracts above with local synthetic data and loopback sockets.
They do not establish exhaustive oracle equivalence for every vendor branch,
all historical Spansh variants, every original database file, or remote service
availability. No real journal data was sent to Spansh or Canonn for validation.
The optional bridge still requires the host to route requests to the enabled
native plugins and return their responses; the service does not invent plugin
content when a handler is absent.
