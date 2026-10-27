# Native port verification and parity ledger

This ledger describes the native implementation and its acceptance evidence. The latest installed-source audits supersede the early rewrite descriptions: [Core](CORE_PARITY_AUDIT.md), [bundled workers](BUNDLED_WORKER_PARITY.md), [BioInsights feature inventory](observatory-plugins/src/bioinsights/FEATURE_PARITY.json), [BioInsights visual measurements](BIOINSIGHTS_VISUAL_QA.md), [Evaluator](EVALUATOR_PARITY_AUDIT.md), and [notification routing/focus](CORE_NOTIFICATION_AUDIT.md). A build or a screenshot alone does not establish complete parity.

## Bundled worker behavior

| Source contract | Native implementation | Verification |
| --- | --- | --- |
| `ObservatoryExplorer/DefaultCriteria.cs`: all 22 scan criteria, including orbital geometry, materials, stars and signal diversity | `observatory-plugins/src/explorer.rs` | The unchanged C# source generated 57 reference findings across the existing session and 59 independent cases. `tests/source_parity.rs` verifies exact body/title/detail/order, including float32 threshold and formatting behavior. Every scan criterion has positive source-oracle coverage. [Provenance and regeneration](fixtures/ORACLE.md). |
| `ObservatoryExplorer/Explorer.cs`: system/body and signal histories, repeat scan suppression, codex and barycentre processing | `observatory-plugins/src/explorer.rs` | Explorer tests plus a session with location, scans, signals, codex, unknown event and jump. |
| `ObservatoryExplorer/CustomCriteriaManager.cs`: user Lua criteria | Rust `criteria.rs`/`criteria_clr.rs` adapter and separate original-manager .NET 8/NLua runtime | All 448 public Framework types match the installed API metadata; 27 independent installed-Framework scenarios and 89 responses match through the host and Rust pipeline. Five integration tests and three Explorer state tests cover real CLR APIs, DTO/history mutation feedback, identity, idle callbacks and failure without replay. [CLR script parity](CLR_SCRIPT_PARITY.md). |
| `ObservatoryBotanist/Botanist.cs`: Odyssey signal totals, per-body species tracking, analysed mark and genus colony distances | `observatory-plugins/src/botanist.rs` | Unit tests for signal registration, repeated sample suppression, analysis without a preceding sample, Frontier's Clypeus identifier typo and unknown genera; shared fixture checks the final grid. |
| Botanist first and second sample distance overlay; close on analysis, jump, leaving the body or shutdown; suppress overlay during replay | `Botanist::sampler`, `on_status`, `set_batch` | 75 original whole-worker source traces compare every grid, sampler and notification transition, including repeated Status, first Analyse, sticky lifetime/update and body cleanup. Botanist preserves the source latitude cosine error and rounded radian constant; the separate StellarOverlay distance helper uses its correct source formula. |
| Botanist settings `OverlayEnabled`, `OverlayIsSticky` | Native settings and `SamplerStatus::sticky` | Consumed by the worker and exposed to the desktop. Desktop placement and timeout behavior require graphical acceptance. |
| `ObservatoryCommunicator/Communicator.cs`: sent/received messages, localized names/content, commander name, channel mapping and filters | `observatory-plugins/src/communicator.rs` | Unit tests and shared fixture verify all messages remain available after hiding/showing a channel. Unknown recipients are treated as private. |
| Communicator `ShowPrivate/Wing/Squadron/System/Local/NPC`, `NotifyPrivate/Wing/Squadron/System/Local/NPC`, `SuppressVoice` | Native settings and independent visibility/notification selectors | Defaults match `CommunicatorSettings.cs`: NPC hidden/silent, other channels shown/notified. Worker tests verify independent private visibility and notification settings, NPC restoration and source plugin-notification delivery during batch reads. SuppressVoice selects the source rendering mask5 instead of7; native channels are suppressed during batch reads. |

## Installed EDJP workers

BioInsights 1.3.444.1052, Evaluator 1.3.444.637, Hecate 1.3.444.761 and
StellarOverlay 1.3.228.387 now have native Rust implementations. Their
[versioned contracts and provenance](INSTALLED_PLUGINS.md) identify the exact
installed references and original resource authorship.

BioInsights includes all 117 installed species and 890 species/colour pairs,
region/environmental rules, sampling, first-footfall values, Codex Companion and
unsold data. Evaluator's calculation-only source oracle covers 49 K constants,
4,480 planet prices, 60 star prices, 320 body states, 72 mapping decisions,
5 completion bonuses and 15 orbital positions. Hecate retains the original
SQLite/Zstd schemas and serves real HTTP/WebSocket requests. StellarOverlay
uses source queue, timing, colors and radar geometry in native viewports.

The main window follows the original horizontal tabs, Core plugin list and
bottom controls. The original gray palette is covered by RGB/layout tests.
Evaluator's orange/custom theme and graphics come from its installed resources.
BioInsights' main view uses its original seven-button toolbar, commander banner,
spanning column headers, grouped body bands and species/variant symbols. The
original custom view disables Core Clear/Export, matching its source contract;
headless diagnostic export remains a native extension. Main, Unsold, About and
Codex Companion switch pages inside the existing BioInsights control. Only the
explicit Popout action detaches that control; Settings opens the Core settings
dialog and Zoom uses a cursor-anchored selector. Navigation/action tests use the
[installed-source contract](fixtures/bioinsights-navigation-contract.json). Hecate
and StellarOverlay have no result tabs, matching their source UI roles.

Integration checks exercise repeated replay, close/reopen, disabled Hecate,
configuration listener replacement and rollback, source shutdown ordering,
cached Codex flags during batch replay, and bounded native-message fan-out
without dropped vault mutations. Graphical smoke checks render the actual plugin
pages and overlay surfaces using synthetic data and disabled remote calls; they
assert that BioInsights content pages have exactly one native viewport.

## Reproducible fixtures

`fixtures/parity-journal.jsonl` is a synthetic 19-entry journal session containing a large 64-bit system address, discovery scans, ringed planet and moon history, organic scans, text messages, a codex entry, an unknown future event and shutdown. It does not contain private player data.

`fixtures/parity-status.jsonl` contains three independent Status snapshots. The replay integration test merges them by timestamp and dispatches them through `on_status`, preserving the game's distinction between journal events and ancillary files. The sample positions yield approximately 872.665 m per 0.2 degrees on a 250 km radius body.

Run from the repository root:

```sh
cargo test --workspace
cargo test -p observatory-plugins --test parity_replay
```

For a live monitor check, copy the journal fixture to a temporary directory using a real journal filename (`Journal.2026-01-01T000000.01.log`), then append complete and split JSON lines while monitoring. Replace `Status.json` atomically with each status snapshot. Validate truncation, replacement, new journal rotation, invalid JSON recovery, stop/start and cancellation without replaying already consumed lines. I/O unit tests automate the supported cases; desktop responsiveness needs a graphical session.

## Recorded native verification

The development-session Linux checks additionally exercised these paths. These results describe the tested build and machine, not every distribution or third-party plugin:

| Check | Observed result | Limit |
| --- | --- | --- |
| Read-only replay of locally present game journals through all seven native workers | 45,145 events across 53 files; 518 findings, 120 Botanist rows, 463 communication rows, 138 BioInsights rows and 1,054 Evaluator rows; zero errors in 3.415 seconds | Remote lookups were disabled. This measures this dataset during concurrent graphical checks; it is not a universal performance guarantee or a differential oracle for every worker. |
| Native graphical startup | Actual ELF launched with both X11 and Wayland display paths | Startup and rendering do not establish every interactive customization or game-overlay behavior. |
| Native desktop customization | Saved color palettes, natural sorting, column resizing, tab ordering, plugin popouts with independent filters/geometry, voice and audio-output selection | UI state tests plus real rendering of the custom-themed Settings page and detached Botanist window. Compositor policies still apply to placement. |
| Real live journal activity | Five-second read-only monitor initialized 1,010 context events, consumed three new events, loaded Status.json and reported no errors | This observed current client writes through Linux filesystem access; it does not replace a controlled Proton/on-foot acceptance run. |
| Freedesktop notification lifecycle | D-Bus notification protocol check observed three notification calls and a close operation | Protocol behavior is separate from a compositor's placement, fonts and game overlay presentation. |
| Runtime/controller integration | Automated tests cover the shared journal pipeline, current Status before organic samples, replay/stop behavior, export and bounded notification/message cycles | See the current test output for the final suite count; no fixed count is asserted here. |
| Native SDK | Documented API and executable sample cover lifecycle callbacks, namespaced storage/settings, table rows, messages, custom export, notification IDs/update/cancel, notifier overrides and batch acceptance | This provides a migration target rather than a CLR DLL ABI or arbitrary WinForms control host. |
| Shared HTTP service | Local TCP tests cover method/header/body transmission, non-2xx status/body preservation, binary response encoding, body cap, queued deadlines and response-body timeout; Lua tests cover scoped request IDs and callback rollback | Four HTTP workers share one client behind bounded queues. Host routing remains private to the originating active plugin generation. No external network endpoints are used by these tests. |
| Original Explorer CLR criteria | Real .NET 8.0.33, NLua 1.7.3 and original Framework types; typed mutations update native histories under original keys, and idle polling delivers timer callbacks | Independent 27-case/89-response oracle, five pipeline tests, three Explorer state tests and two host smoke scripts. Windows-only assemblies retain Linux .NET compatibility limits. |
| Private fullscreen/audio acceptance | Production audio helpers and application dispatch reach an isolated PulseAudio null sink; private X11/Wayland synthetic fullscreen clients retain input and pass lifecycle/negative-control checks | [Receipt](fixtures/private-fullscreen-audio-linux-qa.json). No user desktop, player files, actual game installation or physical audio output was used. |
| Existing settings import | Eight importer tests cover portable JSON, installed XML, GUID/name precedence, enable flags, sparse merge, unknown preferences, invalid input, UTF-16 and unavailable custom criteria | Windows friendly voice names and unsupported WinForms preferences are retained. Speech rate conversion is approximate and explicitly noted. |
| Release packaging | Native Linux ELF plus complete self-contained CLR criteria runtime packaged as portable archive and Debian package; staged spaced-prefix installation and automatic runtime discovery are verified | Runtime completeness/architecture checks reject partial publishes. System-wide installation into a clean distribution remains a separate acceptance step. |
| Packaging failure behavior | Portable packaging succeeds with a deliberately nonfunctional Debian tool; failed Debian generation preserves preexisting published artifacts | Exact artifact moves occur only after generation; publication of several separate files is not a filesystem-wide atomic transaction. |

The native extension contract and migration example are documented in [PLUGIN_API.md](PLUGIN_API.md) and `plugins/sample.lua`. [DIAGNOSTICS.md](DIAGNOSTICS.md) documents bounded live monitoring with `--watch-seconds` and isolated replay data directories.

## Compatibility boundaries and remaining acceptance

The bundled workers, four requested installed-plugin ports and native desktop have source-backed implementations and finite acceptance tests. These boundaries remain explicit when evaluating the application as a replacement for a particular existing installation:

- Explorer criteria can use CLR reflection, Framework methods and compatible user assemblies through the bundled Linux .NET/NLua process. Windows EOP workers/notifiers and arbitrary WinForms controls do not load into the Rust desktop plugin ABI. The four requested EDJP plugins run as native workers; other desktop plugins need their own port or the documented native Lua SDK. Windows-only libraries retain their normal Linux .NET limits.
- StellarOverlay implements native notification/radar presentation, monitor/placement preferences, colors, fonts, transparency and lifecycle. X11 supports foreground focus and monitor queries. Wayland compositor policy controls global focus, positioning and enforced stacking; successful rendering does not establish fullscreen overlay placement. Core native popups and StellarOverlay use passive native surfaces. Unsupported Wayland compositors report missing layer-shell capability rather than create a focus-taking xdg_toplevel popup.
- Native extension grids provide tabular rows, natural sorting, filtering and custom export. Arbitrary WinForms views and CLR comparer objects require a native UI adaptation. Source grouped-grid insertion/replacement/removal and sorting are covered in the Core API audit. Built-in workers use the implemented native views.
- Actual ELF startup, X11/Wayland rendering, notification behavior, staged installation, settings import and export have been exercised. Private fullscreen and PulseAudio null-sink checks cover real production presentation/audio dispatch without accessing the user's desktop or playing physical audio. A clean-distribution system package install and physical device/voice listening remain separate environment checks.
- Real journal replay and live event/status consumption have been exercised. No Linux/Proton Elite installation is available for controlled in-game organic sampling, fullscreen, suspend/resume or long-session acceptance. Automated fixtures cover file rotation, partial writes, replay cancellation and status movement; synthetic fullscreen-client receipts remain explicitly distinct from actual gameplay.

This ledger separates implemented native behavior, intentional platform changes and external acceptance work. It does not claim universal compatibility with every third-party plugin or compositor.
