# Native extension API, version 1

Observatory loads UTF-8 `.lua` files from the data directory's `plugins/` folder.
On Linux the default is `~/.local/share/observatory-core/plugins/`; set
`XDG_DATA_HOME` to relocate the data directory. Copy `plugins/sample.lua` there
and use **Reload plugins** or restart the application to run the working example.
Plugins execute on the journal worker thread, in deterministic filename order.

This API supplies native desktop extensions. Existing Windows EOP assemblies
and custom WinForms controls do not load into this Rust plugin ABI. Port their
worker logic to Lua callbacks and their tabular views to `core.add_row`.

Explorer custom criteria are configured separately through Explorer's
`CustomCriteriaFile`. They run the original annotated criteria manager in the
bundled Linux .NET 8/NLua runtime, with real CLR imports, compatible user
assemblies, Framework objects, methods and callbacks. They are not extension
files and should not be copied into this folder. Criteria DTO mutations feed
back into native histories and asynchronous callbacks are polled while idle.
[Runtime, API evidence and limits](CLR_SCRIPT_PARITY.md). The restrictions and
JSON callback shapes documented below apply to this native extension SDK.

## Plugin declaration

Return one Lua table. `name` is required; a stable explicit `id` is strongly
recommended. IDs contain 1–128 ASCII letters, digits, dots, underscores or hyphens,
and start with a letter or digit. The host rejects duplicate names and IDs before
running their `on_load` callbacks. An omitted ID is derived from the name.

```lua
return {
    id = "net.example.my-plugin",
    name = "My Plugin",
    version = "1.0.0",
    description = "What the plugin does",
    author = "Your name",
    settings = { enabled = true, limit = 100 },
    on_journal = function(event)
        if plugin.settings.enabled and event.event == "FSDJump" then
            core.add_row({ Time=event.timestamp, System=event.StarSystem })
        end
    end,
}
```

`plugin` is the returned table. Saved settings are merged over default settings
**before** `on_load`; nested objects merge recursively. The Settings UI saves and
applies configuration by reloading the worker context. Within a callback use
`core.save_settings(plugin.settings)` to persist plugin changes through the host.
Direct table mutation without `save_settings` is temporary. `core.metadata`
contains the host's name, id, version, description and author; `core.version` is
the application version and `core.api_version` is `1`.

Optional boolean metadata follows notifier roles from the original framework:
`override_popup` suppresses native popups while this enabled plugin is active;
`override_audio` suppresses native speech and chimes; and
`accept_batch_notifications` allows `on_notification` during historical processing.
Built-in journal plugins and Lua extensions still receive historical journal
events. Use `on_state_changed` to suppress any plugin-specific batch effects.

## Callbacks

Callbacks are optional. Each receives a single value except `on_export`, which
receives the selected delimiter string and may return exported text or `nil`.

| Callback | Input |
| --- | --- |
| `on_load(context)` | `{ settings=... }`, after saved settings and storage load |
| `on_ready(context)` | Host ready lifecycle event |
| `on_journal(event)` | Original journal JSON object, preserving unknown fields |
| `on_status(status)` | Current valid `Status.json` object |
| `on_cargo(cargo)` | Current valid `Cargo.json` object |
| `on_auxiliary(update)` | `{ name="Market.json", data=... }` and other ancillary files |
| `on_state_changed(state)` | `{ state="...", batch=true/false }` |
| `on_message(message)` | `{ source={ name,id,version }, data=... }` |
| `on_notification(notification)` | Notification action and content from the host |
| `on_http_response(response)` | Completed request result, delivered only to its originating active plugin |
| `on_export(delimiter)` | Selected separator; return text for custom export |
| `on_shutdown(context)` | Shutdown lifecycle event |

`core.status` and `core.cargo` retain the latest valid snapshots. Historical replay
does not attach current ancillary snapshots to past events. Lua arrays use normal
one-based indexing. Unknown journal event types remain usable without a host
release. A missing journal field is `nil`; check before arithmetic.

## Output and persistent notifications

Use output/storage functions inside callbacks. Source declarations should only
construct and return the metadata/callback table.

| Function | Behavior |
| --- | --- |
| `core.add_row(value)` | Append a JSON-compatible row; named fields become table columns |
| `core.clear_rows()` | Clear existing host rows and rows queued earlier in the same callback |
| `core.add_finding(fields)` | Record a durable finding through the normal notification path |
| `core.notify(fields)` | Show a notification; return its plugin-scoped string ID |
| `core.update_notification(id, fields)` | Update an existing notification without adding a finding |
| `core.cancel_notification(id)` | Close an existing notification |
| `core.send_message(value)` | Broadcast JSON-compatible data with host-supplied source identity |
| `core.save_settings(object)` | Update effective settings and request host persistence |
| `core.log(text)` | Write a bounded diagnostic message |
| `core.http_request(request)` | Stage an HTTP request and return its plugin-scoped `request_id` immediately |
| `core.json_decode(text)` | Parse bounded UTF-8 JSON into a Lua value, preserving explicit null and empty arrays |
| `core.json_encode(value)` | Encode a JSON-compatible Lua value as bounded UTF-8 JSON text |

Finding fields are `timestamp`, `system`, `body`, `title` and `detail` (strings).
The host fills the originating plugin name. Notification fields additionally
accept `timeout` in milliseconds (`0` for persistent), `popup=false` and
`speech=false` to suppress those channels for this notification. `notify` also
records its initial content as a durable finding. Updates and cancellation do not
add findings. The desktop notification server ultimately controls placement,
appearance and timeout behavior.

```lua
local id = core.notify({title="Sampling", detail="Waiting for another sample", timeout=0})
core.update_notification(id, {detail="Sample complete", timeout=5000})
core.cancel_notification(id)
```

IDs are scoped to the plugin; another plugin's ID cannot cancel its notification.
You may supply an `id` field to `notify` for a stable local identifier. Keep the
returned ID when updating or cancelling. Custom exports are capped at 4 MiB;
return `nil` to use the normal host export. Request/reply messages are supported.
An originating dispatch is limited to eight nested handler levels and 256 host
dispatches, including notification round trips. Cycles are reported and stopped.

## Scoped storage

`core.get_storage(key, default)` reads a plugin value; absent keys return the
optional default (or `nil`). `core.set_storage(key, value)` stages a JSON-compatible
value, and `core.delete_storage(key)` removes it. Use `core.null` when an explicit
JSON null value is needed. Storage keys contain 1–128 bytes without control
characters. A plugin may store at most 1,024 keys and 1 MiB of JSON.

Storage lives at `<data directory>/storage/<plugin id>/state.json`. The host uses
atomic replacement and commits changes only after a successful callback. A
malformed existing storage file fails plugin load and is preserved for recovery.
The ID keeps storage stable when the plugin's display name changes. The exposed
`core.storage_path` is descriptive data; it does not grant direct filesystem I/O.

## Asynchronous HTTP

`core.http_request` validates and stages a request inside a callback. Network I/O
runs on the host's four-worker service after the callback succeeds, so it does
not block journal processing or consume the Lua callback's execution budget.
A failed callback discards its staged requests. The default sample plugin does
not issue network requests.

Request fields:

| Field | Meaning |
| --- | --- |
| `url` | Required `http://` or `https://` URL, at most 8,192 bytes |
| `method` | HTTP method string; defaults to `GET` |
| `headers` | Optional table mapping header names to strings or arrays of strings |
| `body` | Optional request-body string |
| `request_id` | Optional stable local ID; the returned ID is scoped to this plugin |
| `timeout_ms` | Positive timeout in milliseconds; capped at 10,000, default 10,000 |
| `include_headers` | Include response headers when `true`; defaults to `false` |

For example, place the desired URL in the plugin's settings and request it from
an event handler:

```lua
on_journal = function(event)
    if event.event == "FSDJump" and plugin.settings.url ~= "" then
        local id = core.http_request {
            url = plugin.settings.url,
            method = "GET",
            headers = { Accept = "application/json" },
            timeout_ms = 5000,
            include_headers = true
        }
        core.set_storage("pending_request", id)
    end
end,
on_http_response = function(response)
    if response.error ~= nil and response.error ~= core.null then
        core.log(response.error)
        return
    end
    core.add_row { request=response.request_id, status=response.status, ok=response.ok }
    if response.body ~= nil and response.body ~= core.null then
        local document = core.json_decode(response.body)
        -- Read the endpoint's fields from document here.
    end
end
```

Responses contain `request_id`, `ok`, `status`, `body`, `body_base64`, `error`,
`error_kind`, `truncated` and `elapsed_ms`. `ok` is true for 2xx HTTP status codes.
A non-2xx status still includes its body and has `error=null` when transport and
body reading succeeded. Transport failures and resource limits set `error` and
`error_kind`; `status` is null when no response headers arrived. UTF-8 bodies use
`body`; other bytes are returned losslessly in `body_base64`, with `body=null`.
Optional `headers` maps lowercase names to arrays, retaining repeated values;
non-UTF-8 header bytes use replacement characters when represented as strings.

The request queue holds at most 64 entries. The timeout includes queue waiting
and response-body reads. Requests are capped at 256 KiB including headers/body,
response bodies at 4 MiB, and returned headers at 64 KiB. An oversized body yields
`truncated=true` and no partial body. Redirects are limited to ten. Errors do not
include request headers or body content. Reload/shutdown discards pending
responses for the old plugin generation; a new instance cannot receive an old
instance's result. An in-flight request may continue until its deadline after
its plugin has been unloaded.

## JSON values

`core.json_decode(text)` and `core.json_encode(value)` perform no I/O or host
output and may also be used while constructing a plugin's declaration. Decode
accepts JSON objects, arrays and scalar values. Explicit JSON null becomes
`core.null`; compare with that value rather than relying on Lua truthiness.
Decoded empty objects and empty arrays retain their distinction when re-encoded.
A plain empty Lua table encodes as `{}`; use `core.json_decode("[]")` to construct
an empty JSON array.

```lua
local payload = core.json_encode {
    system = "Sol",
    optional = core.null,
    samples = { 1, 2, 3 }
}
local request_id = core.http_request {
    url = plugin.settings.url,
    method = "POST",
    headers = { ["Content-Type"] = "application/json" },
    body = payload
}
```

Input text and encoded output are limited to 4 MiB. Traversal is bounded to
262,144 values and nesting to 128 levels. Encoding streams into a bounded output
buffer and rejects recursive tables, excessive repeated-table expansion,
non-finite numbers, invalid UTF-8 strings and tables that mix array entries with
named object fields. Use `core.null` for array positions that must be retained;
`nil` removes Lua entries. Malformed JSON raises an error identifying line and
column, without including the input body. `pcall` can handle helper errors.

Lua represents numbers as signed 64-bit integers or doubles. Values outside that
integer range and decimal values may lose numeric precision; retain the original
HTTP `body` when exact numeric text is required. The separate 256 KiB HTTP request
and host-output limits still apply to encoded JSON submitted to those services.

## Failure and execution limits

Each plugin has its own Lua state and a 32 MiB Lua memory limit. Source files are
capped at 4 MiB. Every callback, source evaluation and export gets a 5,000,000
instruction budget and a 100 ms wall-clock check at instruction hooks, including
coroutines. Output is limited to 10,000 items and 4 MiB per undrained queue;
individual values are limited to 256 KiB. A callback may log at most 100 messages
of 4 KiB each. Keep callbacks short so journal processing remains responsive.

Shell, arbitrary filesystem access and module loading are unavailable:
`io`, `os`, `package`, `require`, `dofile`, `loadfile` and `debug` are removed.
Use host-provided storage and `core.http_request`. These are application resource limits, not operating
system process isolation.

A failed callback discards its notifications, findings, row changes, messages,
HTTP requests, settings and storage writes, then disables that instance. Output from earlier
successful callbacks is retained. Its Lua globals may have changed internally,
so recovery requires **Reload plugins**, which constructs a new state from source
and persisted settings/storage. Successful callbacks may already have committed
storage before a later callback fails.
