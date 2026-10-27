# Core 1.5.2 finite parity audit

This audit pins the installed reference, records the full public Core API and all saved preferences, and distinguishes independently captured source semantics from native GUI evidence. It is not a blanket claim of complete graphical or platform parity.

## Reference and executable source evidence

`fixtures/core-feature-contract.json` records the installed ObservatoryCore.dll and ObservatoryFramework.dll SHA-256 hashes, hashes of the 13 decompiled source files, all 38 public PluginCore declarations, all 36 saved Core properties, and 102 UI handler signatures. The installed Windows assemblies were read, not started or changed.

- `core-framework-metadata.cs/json`: PE metadata only; pinned Framework 1.10.26274.2239 contains 280 JournalBase-derived journal types.
- `core-sort-oracle.cs`, driver/stubs/JSON: original DefaultSorter source compiled unchanged with inert ListView stubs; 900 pairwise comparisons captured, including UTF-16, leading-zero, decimal and digit ordering cases.
- `core-grid-oracle.cs`, driver/stubs/JSON: original column-click method and DefaultSorter unchanged; seven insertion rotations each captured before/ascending/descending/ascending, including source group-head selection and reversed follower order.
- `core-export-oracle.cs`, driver/stubs/JSON: original ExportHandler with inert chooser/list controls; Windows line endings normalized explicitly for Linux execution. Captures exact TSV bytes and all five XLSX entries using synthetic strings, embedded tabs/newlines, Unicode, long IDs and formula-like text.
- `core-builtin-load-contract.json`: installed Explorer, Botanist and Communicator assembly hashes and their original Load implementations. Initial Basic grids are newly created and empty, rather than restored from a Core result database.

No oracle invokes a plugin entry point, vendor updater, network service or original user's data.

## Complete public API surface

Source locations are `Observatory.PluginManagement/PluginCore.cs`. Repeated rows are distinct source overloads.

| Source line | Declaration | Native mapping / evidence boundary |
|---:|---|---|
| 32 | `public string Version => Assembly.GetEntryAssembly()?.GetName().Version?.ToString() ?? "0";` | Query/property adapter; host version context exists before on_load |
| 34 | `public System.Net.Http.HttpClient HttpClient => Observatory.Utils.HttpClient.Client;` | `core_api.rs` query/property adapter; host contexts are installed before `on_load` |
| 36 | `public LogMonitorState CurrentLogMonitorState => LogMonitor.GetInstance.CurrentState;` | `core_api.rs` query/property adapter; host contexts are installed before `on_load` |
| 38 | `public bool IsLogMonitorBatchReading => LogMonitorStateChangedEventArgs.IsBatchRead(LogMonitor.GetInstance.CurrentState);` | Query/property adapter; all 16 flag combinations checked against source `(state & 6) != 0` |
| 40 | `public string PluginStorageFolder` | `core_api.rs` query/property adapter; host contexts are installed before `on_load` |
| 72 | `public string UpdatedPluginsFolder => PluginManager.PluginPath;` | `core_api.rs` query/property adapter; host contexts are installed before `on_load` |
| 74 | `public event EventHandler<NotificationArgs>? Notification;` | Ordered Core host command; GUI routing/integration tested separately from adapter output |
| 76 | `public event EventHandler<NotificationArgs>? UpdateNotificationEvent;` | Ordered Core host command; GUI routing/integration tested separately from adapter output |
| 78 | `public event EventHandler<Guid>? CancelNotificationEvent;` | Ordered Core host command; GUI routing/integration tested separately from adapter output |
| 89 | `public Action<Exception, string> GetPluginErrorLogger(IObservatoryPlugin plugin)` | Ordered Core host command; GUI routing/integration tested separately from adapter output |
| 97 | `public Status GetStatus()` | `core_api.rs` query/property adapter; host contexts are installed before `on_load` |
| 102 | `public CargoFile GetCargo()` | `core_api.rs` query/property adapter; host contexts are installed before `on_load` |
| 107 | `public Guid SendNotification(string title, string text)` | Core notification/audio host path (separate popup/audio audit and tests) |
| 116 | `public Guid SendNotification(NotificationArgs notificationArgs)` | Core notification/audio host path (separate popup/audio audit and tests) |
| 150 | `public void CancelNotification(Guid id)` | Core notification/audio host path (separate popup/audio audit and tests) |
| 159 | `public void UpdateNotification(Guid id, NotificationArgs notificationArgs)` | Core notification/audio host path (separate popup/audio audit and tests) |
| 165 | `public void UpdateNotification(NotificationArgs notificationArgs)` | Core notification/audio host path (separate popup/audio audit and tests) |
| 185 | `public void AddGridItem(IObservatoryWorker worker, object item)` | Core row adapter; shared hidden group per bulk call, replacement/clear retain order |
| 190 | `public void AddGridItems(IObservatoryWorker worker, IEnumerable<object> items, bool grouped = false)` | Core row adapter; shared hidden group per bulk call, replacement/clear retain order |
| 219 | `public void SetGridItems(IObservatoryWorker worker, IEnumerable<object> items)` | Core row adapter; shared hidden group per bulk call, replacement/clear retain order |
| 230 | `public void ClearGrid(IObservatoryWorker worker, object templateItem)` | Core row adapter; shared hidden group per bulk call, replacement/clear retain order |
| 235 | `public void ExecuteOnUIThread(Action action)` | Native adaptation: command producer runs on worker; ordered GUI operations are marshalled by host |
| 336 | `public Task PlayAudioFile(string filePath, AudioOptions? options = null)` | Core notification/audio host path (separate popup/audio audit and tests) |
| 341 | `public void SendPluginMessage(IObservatoryPlugin plugin, object message)` | Typed/legacy host envelopes; sender identity and targeted delivery are host responsibilities |
| 346 | `public void SendPluginMessage(IObservatoryPlugin plugin, string targetShortName, object message)` | Typed/legacy host envelopes; sender identity and targeted delivery are host responsibilities |
| 351 | `public void SendPluginMessage(IObservatoryPlugin plugin, PluginMessage message)` | Typed/legacy host envelopes; sender identity and targeted delivery are host responsibilities |
| 356 | `public void SendPluginMessage(IObservatoryPlugin plugin, Guid targetId, PluginMessage message)` | Typed/legacy host envelopes; sender identity and targeted delivery are host responsibilities |
| 361 | `public void RegisterControl(object control, Func<object, bool> applyTheme)` | Native control-descriptor adapter; callback gate/traversal/fallback/exclusion retained |
| 366 | `public void RegisterControl(object control)` | Native control-descriptor adapter; callback gate/traversal/fallback/exclusion retained |
| 371 | `public void UnregisterControl(object control)` | Native control-descriptor adapter; callback gate/traversal/fallback/exclusion retained |
| 376 | `public string GetCurrentThemeName()` | `core_api.rs` query/property adapter; host contexts are installed before `on_load` |
| 381 | `public Dictionary<string, Color> GetCurrentThemeDetails()` | `core_api.rs` query/property adapter; host contexts are installed before `on_load` |
| 386 | `public void SaveSettings(IObservatoryPlugin plugin)` | Extension settings output; host persists merged settings |
| 391 | `public void OpenSettings(IObservatoryPlugin plugin)` | Ordered Core host command; GUI routing/integration tested separately from adapter output |
| 399 | `public void OpenAbout(IObservatoryPlugin plugin)` | Ordered Core host command; GUI routing/integration tested separately from adapter output |
| 407 | `public JournalEventArgs DeserializeEvent(string json, bool replay = false)` | 280 pinned Framework event names; JSON retained; parse-only default, Replay=true dispatch command |
| 412 | `public void FocusPlugin(string pluginName)` | Ordered Core host command; GUI routing/integration tested separately from adapter output |
| 432 | `public void DisplayMessageBox(string title, string message)` | Ordered Core host command; GUI routing/integration tested separately from adapter output |

The native extension boundary carries JSON and control descriptors, rather than Windows widget pointers. `ExecuteOnUIThread` accepts a command producer whose GUI operations are queued to the real UI; extension Lua stays on its worker. Its `DeserializeEvent` preserves JSON and source event identity. Explorer criteria have a separate real .NET 8/NLua runtime with actual Framework DTO defaults, methods and object identities; that runtime is not the old Windows EOP/WinForms desktop ABI. Native extension callbacks retain their 4 MiB/10,000-output limits; trusted original criteria instead use the managed process's bounded protocol and request deadline.

## Complete persisted Core preferences

`legacy_settings.rs` imports every property below from portable Observatory.config or installed user.config, preserving unrecognized fields and serialized source forms. It maps geometry, tab order, generic popouts and column sizing to native structures, merges saved `.eot` themes through `core_theme`, and imports original plugin preferences. `legacy_data.rs` migrates four Hecate SQLite databases including live WAL state, Bio per-FID codex/notifier files, custom output colors and source custom popout geometry without changing the source files.

| Property | Source type | Source default |
|---|---|---|
| `JournalFolder` | `string` | `` |
| `NativeNotify` | `bool` | `True` |
| `NativeNotifyFont` | `string` | `` |
| `NativeNotifyColour` | `uint` | `4294944000` |
| `NativeNotifyCorner` | `int` | `0` |
| `NativeNotifyScreen` | `int` | `-1` |
| `TryPrimeSystemContextOnStartMonitor` | `bool` | `False` |
| `CoreVersion` | `string` | `` |
| `PluginSettings` | `string` | `` |
| `VoiceNotify` | `bool` | `False` |
| `VoiceSelected` | `string` | `` |
| `VoiceVolume` | `int` | `75` |
| `VoiceRate` | `int` | `0` |
| `MainWindowSize` | `Size` | `800, 500` |
| `MainWindowPosition` | `Point` | `100, 100` |
| `NativeNotifyScale` | `int` | `100` |
| `NativeNotifyTimeout` | `int` | `8000` |
| `StartMonitor` | `bool` | `False` |
| `ExportFolder` | `string` | `` |
| `StartReadAll` | `bool` | `False` |
| `Theme` | `string` | `Dark` |
| `ColumnSizing` | `string` | `` |
| `PluginsEnabled` | `string` | `` |
| `SavedThemes` | `string` | `` |
| `LastTabIndex` | `int` | `0` |
| `ExportFormat` | `int` | `0` |
| `AudioVolume` | `float` | `0.75` |
| `AudioDevice` | `string` | `` |
| `NativeNotifyTransparent` | `bool` | `True` |
| `NativeNotifyFontScale` | `int` | `100` |
| `PopoutLocation` | `string` | `` |
| `ChimeEnabled` | `bool` | `False` |
| `ChimeSelected` | `int` | `0` |
| `OpenPopouts` | `string` | `` |
| `TabOrder` | `string` | `` |
| `AltMonitor` | `bool` | `False` |

Import tests do not establish that every GUI control has been clicked. Native control wiring and runtime use are checked independently. Original `ExportFolder` is saved but ExportHandler initializes its chooser to Personal/Documents regardless of it. Original `TryPrimeSystemContextOnStartMonitor` has no reference in the inspected LogMonitor implementation; no behavior has been invented for that unused preference.

## Source behavior findings and required native checks

| Feature | Exact source contract | Verification / integration status |
|---|---|---|
| Batch state | Idle=0, Realtime=1, Batch=2, PreRead=4, BatchCancelled=8; IsBatchRead is `(state & 6) != 0` | Pure helper tests cover all flags 0..15; host must publish before callbacks |
| Notification identity | Send assigns a GUID before deciding batch suppression, so a suppressed send still returns a nonempty GUID | Source method checked; host canonical lifecycle covered by notification owner |
| Notification delivery | Plugin bit is invoked before batch check; native visual/audio are suppressed for Batch and PreRead; Update is wholly suppressed in batch; Cancel always fans out, including disabled notifiers | Source methods checked; native focus/compositor tests belong to popup owner |
| Legacy messages | Original handler and typed LegacyPluginMessage both receive payload; typed envelope is `{message: raw}`, source Guid.Empty | Core adapter and Evaluator unwrapping fixed; host dual delivery is an integration check |
| Typed messages | Exclude sender, disabled recipients, nonmatching GUID; clone payload per recipient; Guid.Empty broadcasts | Host routing audit supplied to runtime owner |
| DeserializeEvent | Default `replay=false` parses and advances lastEvent/totalEvents without invoking workers; true invokes; ancillary file dispatch remains outside batch | Seven production Engine integration tests verify parse/default/dispatch, pre-callback counters, nested ordering, all9 ancillary reads and Batch/PreRead suppression |
| New Basic grid | First object defines columns only; initial grid is empty, default sorter order=0 | Source Load and grid constructors checked; no restored historical Core rows |
| Append and sorting | Append order until first header click; first click ascending; same column toggles; natural comparer has no f64 shortcut | 900 source comparisons plus actual ascending/descending header clicks and exported visible order pass |
| Group sorting | Shared hidden GUID, no visual group header; sort group leaders and ungrouped rows, then append each group's followers in reverse source order | Group identity and all7 source group rotations across3 clicks pass in the production sorter used by the GUI |
| Clipboard | Full-row/multiple selection; Ctrl+C only with Ctrl and neither Alt nor Shift; selected cells each followed by TAB, each row CRLF; no headers | Actual XTest Ctrl multiselect/Ctrl+C captures exact selected trailing-TAB/CRLF rows; Ctrl+Shift+C preserves clipboard |
| Columns | CamelCase split by source's two Unicode regexes; attribute SuggestedWidth wins, otherwise header length ×10; saved sizing applies only matching name AND version | Source suggested pixel widths and matching-version restoration implemented; native restart/geometry interaction passes |
| Add/remove/reset | Add queue coalesces at 100 ms; Remove matches cell text; Reset clears queued rows and rebuilds | Source grid handlers checked; native runtime queues reviewed by host owner |
| Core plugin row menu | Three actions: Settings, About, Open <ShortName> Data Folder; dropdown is flat 25×25 “▾” | Source exact3 actions and native dropdown implemented; production UI gateway tests exercise settings/about/folder requests |
| Plugin tab menu | Four actions: Popout <ShortName>, Settings, About, Open <ShortName> Data Folder, in that order | Actual tab context opens generic popout; source menu ordering checked |
| Export | Original CSV option is raw TAB-separated, trailing TAB per cell and CRLF per row; XLSX cells are all inline strings | Original ExportHandler oracle plus actual repeated GTK saves verify initial/sorted/cleared TSV bytes and selected XLSX text cells |
| Core layout | Client defaults 800×500; minimum 592×269; source tabs and bottom actions anchor to client edges | Actual1200x800 client and persisted1000x650 restart geometry pass; no claim of Windows rasterization equality |
| Theme registration | Callback gates initial registration and is called again for initial apply; MenuStrip items precede strip properties; Type.Property then Default.Property; recurse Controls except child Name=ColourButton | Callback/order/fallback/unregister tests pass |
| UI persistence | Main position/size, LastTabIndex, tab order, column width with plugin version, generic Core popouts vs custom Bio/Eval chrome | 18 importer tests plus actual tab drag, generic popout move/resize/close/reopen and main restart geometry pass |
| Read All / monitor | Source396x86 progress form and cancellation; selected and auto-start behavior are separate | Actual ReadAll starts progress form and Cancel stops it; source total label N0 and error/status gating tested |
| About/settings/data folder | Source exact metadata and settings dialogs, actual folder path; no toolbar invents separate content windows | Source metadata and native forms implemented; actual Core Settings opens/closes, source page toolbars stay inline |

## Test receipts and limits

The [CLR criteria audit](CLR_SCRIPT_PARITY.md) verifies all 448 exported Framework types against installed API metadata and compares 27 real-NLua scenarios/89 responses against the installed Framework DLL. Six integration tests and three Explorer state tests cover DTO mutation feedback under original history keys, separate Parent projection, signal changes reaching native built-ins, idle timer callbacks and failure without script replay. This additional oracle loads only the original Framework data/API assembly with Windows ReadyToRun disabled; it does not start the original desktop.

`core_api.rs` contains 11 regression tests using production adapter/export/sort code. They cover initial contexts and all 16 state combinations, grouped Add/Set, UI/legacy/typed envelopes, theme callback traversal/MenuStrip ordering, all 900 original comparator captures, seven original group rotations with three successive column clicks, source storage concatenation, exact original TSV/XLSX cells, all 280 pinned Framework full type names, and original raw JournalBase.Json spacing/key order. Nine Engine tests independently cover typed delivery once with caller exclusion and cloned payloads, old-plus-typed legacy callback order per recipient, live UI command routing, parse-only/default replay, built-in theme names, manual ancillary replay without counter increments, nested dispatch counter/last-event order, and independent Bio/Evaluator page invalidation after unrelated worker output, and source peer-ready identity/version/disabled gates.

`legacy_settings.rs` and `legacy_data.rs` have 18 source-settings/storage tests, including four live-WAL source databases unchanged byte-for-byte, merge conflicts, idempotence, malformed-source all-or-nothing staging, Bio commander data and Eval-only custom theme/popout imports. No import of the real user's profile was run by this child agent.

`native/scripts/core_parity_ui_test.py` launches a private Xvfb display with a disposable HOME and XDG profile, synthetic journals and rows, and actual XTest input/GTK chooser. Its JSON receipt must name completed interactions and binary hash. A capture-only receipt is diagnostic, not interaction or parity proof. Native Wayland compositor behavior, original Windows font rasterization, speech/device output and game focus are not inferred from Xvfb screenshots.

## Completed graphical interaction receipt

`dist/core-parity-ui/clicks.json` records 12 actual interaction groups with its frozen binary SHA-256: requested client geometry; repeated GTK source CSV saves and initial append order; ascending/descending header clicks; Ctrl row multiselect and clipboard modifiers; generic popout context/move/resize/reopen; Core Settings; real CSV/XLSX selector and nine source text cells; Light RGB240 and Dark restoration;396×86 ReadAll/Cancel; Clear with retained columns; tab drag and persisted main/popout restart. `native/scripts/core_parity_ui_test.py` is the reproducible private Xvfb/XTest/GTK driver. All data is synthetic and HOME/XDG/GDK runtime isolated.

The former startup freeze was reproduced with a fresh debug binary and GDB: two egui Context data-lock closures called `viewport_id()` while holding that Context lock. Their keys are now computed before taking the data lock. After this correction the actual Linux window maps and completes the interactions above. The exact parent UI source was recovered from its pre-write Git snapshot and all 23 recorded UI-only transformations before either fix; the recovery did not replace the UI with an approximate rewrite.

Original source contains Windows-specific behaviors with native adaptations: WinForms controls, vendor Windows EOP packages and browser rasterization. Explorer's CLR DTO identity/mutation behavior is implemented in its separate managed runtime. The original ReadAll title has one Reticulating Splines... outcome among 20, selected once per window and verified across all 20 source outcomes. [Private fullscreen/audio acceptance](fixtures/private-fullscreen-audio-linux-qa.json) covers synthetic X11/Wayland clients and real production dispatch through a PulseAudio null sink; no user screen, actual gameplay or physical audio listening is inferred from it. Native updater checks have their own receipts.

The optional WebKit child-control focus bridge is also exercised with real X11 focus changes: it queries GetInputFocus and recognizes the owner or any descendant, preserving slider input. Only a transition outside that domain dispatches DOM blur; no focus/activation request is sent. Native and WebKit controls both pass50→600, Reset100, persistence, external-focus dismissal and one-window checks in `/home/loic/.cache/observatory-bio-visual-qa/controls-v2/receipt.json` (its frozen binary hash is recorded). The small browser-only DOM fixture additionally verifies gallery node/credit identity, selected URI, scroll preservation and source-menu blur cleanup.

## Verified graphical build

The completed 12-group Core interaction receipt pins immutable Linux release SHA-256 `985a16215aec6520120a8d8b97eaa2dc79bf7c7ced9a3ac00590e6b5594ed343`. The receipt and captures are in `dist/core-parity-ui/`; the same build also passed all five Evaluator window/navigation actions. The tests use isolated synthetic profiles and real XTest/GTK input.

The later CLR-integrated Core build has SHA-256 `9d8af27a8dfb9ed3c6719a89e800d9174722f31555ecc7e510bd8484302c2970`; its source passed 396 tests, zero failures, with 12 explicit/manual checks ignored, plus all-target clippy. This runtime integration did not change desktop UI source. The historical graphical receipt above retains its original binary identity; current artifact and separate acceptance receipts are indexed in [VALIDATION.md](VALIDATION.md).
