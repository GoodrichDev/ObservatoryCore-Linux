The installed Core distribution's bundled workers were inspected directly from
`ObservatoryExplorer.eop`, `ObservatoryBotanist.eop` and
`ObservatoryCommunicator.eop`. Their assembly versions are respectively
`1.3.0.27422`, `1.1.3.27422` and `0.0.2.27422`. Original installation files were
only read. Original plugin entry points were not executed. The newer CLR oracle
loads the installed Framework data/API assembly with Windows ReadyToRun disabled,
and runs the unchanged installed criteria-manager source against real NLua;
it does not start the original desktop.

`BUNDLED_WORKER_PARITY.json` is the finite inventory: 27 Explorer settings/public
properties, two Botanist settings, 13 Communicator settings, journal events,
actions, state transitions, notification contracts, grid metadata, source hashes
and host/runtime verification. It is not a certificate of complete
desktop parity.

The fixtures exercise independently compiled original C# source with inert
Core/UI I/O. The captures include:

* 723 original DefaultCriteria cases, including positive close/colliding binary
  and ring/belt branches, disabled switches, Single thresholds, material sets,
  star classes, string casing and numeric formatting.
* 759 complete Explorer event scenarios comparing source grid rows and original
  notification payloads. This covers synthetic barycentre conversion, unusual
  ordinal boundaries, first-only signal caches, duplicate scans, Codex lookup,
  current-system clearing, batch clearing and pruning beyond 1,000 systems.
* 75 complete Botanist scenarios, with a grid and notification send/update/cancel
  comparison after every event/status/settings/batch step. These capture first
  organic events, known/unknown genera, species insertion order, repeated scans,
  sampler lifetime, all closing events, settings changes and geographic inputs.
  The original distance approximation is isolated from the correct shared
  distance helper used by StellarOverlay.
* 250 Communicator scenarios covering every source channel, casing, null/empty
  localized values and commander names, 128 settings combinations, incoming
  notifications during batch reads and the original timestamp parser.
* 38 ordinary Lua programs compiled by the original C# annotation parser with
  an inert NLua code recorder. Native tests execute both the recorded original
  Lua helpers/generated functions and the native compiler output, across six
  synthetic event calls. This verifies compiler/helper behavior; it does not
  establish CLR object-method compatibility.
* Public property schemas captured from 343 installed Framework data classes.
  Native DTO projection tests cover numeric defaults/Single precision, nil
  strings, source tuple members, unknown-only AdditionalProperties and StarPos.
* Real .NET 8/NLua criteria now execute the original parser/helpers and actual
  Framework types. All 448 exported public types match the installed API
  metadata. An independent installed-Framework oracle compares 27 scenarios and
  89 responses through both the Linux CLR process and Rust Criteria pipeline.
  Six integration tests cover imports, reflection, constructors, collections,
  actual methods/ref-out, lifecycle, callbacks and crash without side-effect
  replay. Three Explorer state tests verify original-key mutation feedback,
  signal changes reaching native built-ins and idle timer history updates.
  Separate host smoke tests cover stream isolation, mutation echoes, identity,
  Parent projection, dirty-only updates and pruning with retained Lua references.
  [CLR provenance, runtime and commands](CLR_SCRIPT_PARITY.md).

Time display captures use `TZ=UTC`; tests convert those captured display values
to the active local timezone before comparison. Production uses the source
Framework's local-time display policy. Invalid or noncanonical timestamp strings
retain the source DateTime.MinValue display.

Run `cargo test -p observatory-plugins --tests` and
`cargo clippy -p observatory-plugins --all-targets -- -D warnings` from the repository root.
The current whole-workspace run passed 396 tests with zero failures and 12
explicit/manual checks ignored, using the real CLR runtime override; all-target
workspace clippy passed. Historical owned-worker counts remain labeled as such
in the JSON inventory.
The `fixtures/capture-*` scripts show how source captures were produced; their
readable-source input paths are task caches, with hashes recorded in the ledger.

Current host evidence is explicit in the JSON inventory and `dist/final-validation.json`.
Core consumes the source notification/grid/startup/log APIs. Raw journal text is
retained in an optional shared-string event field, including cached Scan objects;
older RPC envelopes remain accepted. The twelve-group actual Core interaction
receipt verifies fixed pixel column widths, Basic-grid sorting/clipboard/export,
Communicator action controls, settings, progress cancellation and saved geometry.
Core notification routing additionally matches 768 original Send/Update/Cancel
source captures. Packaged Explorer criteria use the real self-contained
.NET 8.0.33/NLua 1.7.3 runtime; the main application and four requested workers
remain native Rust. Native fallback Lua limits are distinct from the managed
process's 64 MiB frame limit and bounded request timeout. Windows-only managed
libraries retain normal Linux .NET limitations; the bridge is not a Windows
EOP/WinForms desktop plugin host. Private fullscreen/null-sink audio evidence is
recorded separately in [its receipt](fixtures/private-fullscreen-audio-linux-qa.json).
