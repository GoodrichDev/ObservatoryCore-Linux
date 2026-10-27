# Native plugin process protocol, version 1

The normal installation calls the Rust workers directly. A verified independently
installed native artifact selects one worker process for that plugin on the next
Core start. Other workers and their disabled flags are unaffected. No Windows
assembly is loaded. The host updater owns download policy, compatibility checks,
SHA256/ELF validation and atomic staging under
`data_dir/native-plugins/<Plugin>/{worker,installed.json}`.

`observatory-native-worker --kind BioInsights|Evaluator|Hecate|StellarOverlay
--hello` prints metadata and exits without constructing a worker, opening a vault
or binding a listener. It returns `protocol`, `plugin`, `source_version`,
`native_version`, and `platform`. Future source versions are allowed when the
native protocol/platform remain compatible. Feed metadata must match the inspected
artifact, rather than an assumed version compiled into Core.

For a real session the host starts the same executable with `--kind <Plugin>`.
stdin/stdout carry UTF8 JSON followed by one LF. Each frame is limited to 64 MiB.
stdout is exclusively protocol output; diagnostics use stderr. The protocol has
no unsolicited output. Requests execute in order:

```json
{"protocol":1,"id":1,"now_ms":800000,"method":"hello","args":null}
{"protocol":1,"id":2,"now_ms":800000,"method":"initialize","args":{"settings":{},"data_dir":"/private/plugin-data"}}
```

Responses repeat the protocol and request ID. Exactly one of a meaningful
`result` or `error` is returned (unit results are JSON null):

```json
{"protocol":1,"id":1,"result":{"protocol":1,"plugin":"Hecate","source_version":"1.3.444.761","native_version":"2.0.0","platform":"x86_64-unknown-linux-gnu"},"error":null}
```

The method names and arguments in `worker_rpc/server.rs` expose the production
worker APIs directly, including their journal/status dispatch, peer messages,
HTTP results, projections, pending effects, notification rendering/payloads,
vault queries, route polling and preference changes. Typed adapters in
`worker_rpc/facade.rs` preserve those APIs for the host. `on_events` additionally
accepts an ordered journal array. Drain methods retain their consume-once
semantics. Settings are applied in place. `shutdown` flushes Hecate and exits;
ordinary EOF drops the worker, whose vault destructor also flushes.

`now_ms` is the host's monotonic clock. Every Stellar call uses it throughout
nested helpers so notification lifetimes, radar previews and force-display TTLs
do not acquire a new clock origin at process creation. JSON parsing enables
`float_roundtrip`; binary64 values and unsigned 64-bit system IDs survive the
process boundary without rounding.

The transport validates response protocol/ID and bounds stalled reads and writes.
A transport failure terminates that child. It does not silently construct an
empty replacement. Completed projections remain cached, including personal Codex
state; the host must drain `take_worker_errors` and avoid saving state from an
unhealthy worker. `worker_info`, `worker_process_id`, `is_external` and
`is_healthy` provide host diagnostics. Legitimate model errors, such as a failed
Hecate listener change, are returned without killing a healthy process.

Reproduce real subprocess equivalence and failure-isolation tests with:

```sh
cargo test -p observatory-plugins --test worker_process
```

Build a private QA feed, or generate release artifacts using an actual HTTPS
release directory, with:

```sh
python3 native/scripts/package_native_workers.py --out /tmp/native-workers \
  --base-url http://127.0.0.1:58081 --revision 2
```

This emits four independently named ELF artifacts, checksums and the compatible
`native-plugins.json` feed. It does not publish or configure an invented endpoint.
All four artifacts may share the same initial Rust binary; `--kind` fixes which
production model each selected process constructs. Installing a later BioInsights
artifact cannot replace the Evaluator, Hecate or Stellar worker already selected
by the host.

The Core updater integration has a separate executable regression:

```sh
python3 native/scripts/native_update_test.py /path/to/observatory-core \
  /path/to/observatory-native-worker --output /tmp/native-update-qa
```

It uses private XDG profiles, synthetic journals and a loopback release feed.
Checks include per-plugin automatic/manual/beta policy, preserving disabled
flags, staging while all four real children run, executable-inode retention until
restart, complete builtin/remote Core pipeline and SQLite-ledger comparison,
rejected artifacts retaining prior installs, and a killed Bio child while
Evaluator and Stellar continue. An independently compiled invalid-metadata ELF
is a protocol negative control, never a worker implementation. JSON outputs and
the final receipt remain under the chosen output directory; private profiles are
removed after the run.
