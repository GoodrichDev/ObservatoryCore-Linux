# Evaluator installed-version feature audit

Reference: Evaluator 1.3.444.637. Installed EOP SHA256: `4de135ab48b8fbbc9ea350bfd1c8a5d40d52d94615cebf51c53a0f31a04a21e2`. Source hashes and the complete source symbol inventory are in `fixtures/evaluator-parity-contract.json`. Original installed files have not been modified.

This is an audit record, not a declaration of complete parity. `Source oracle` means output was captured by executing extracted original C# calculation code, independently of Rust. `Implemented` means native code exists and was reviewed against the cited source; it does not mean every original event ordering/UI pixel was independently verified. `Gap` is work still required.

## Acceptance evidence

| Surface | Evidence |
| --- | --- |
| Body valuation/mapping bonuses | Existing `evaluator-prices.json` original calculation captures and Rust tests |
| Display/MappingOnly/All filters and manual overrides | 3,600 original `Evaluator.UpdateUI` source captures, `evaluator-ui-oracle.*` |
| Orbital coordinates | 15 original Orrery coordinate captures in pricing fixture |
| Exact route order/cost | 12 original `Tsp3DSolver.SolveExact` source captures, 2 through 13 cities, `evaluator-route-oracle.*` |
| Larger route solver | 8 original full seeded chains match exact orders/costs; 8 greedy/MST/candidate-list/local-pass primitives; 128 fixed-iteration traces match every candidate route and move sequence; .NET Random covers signed seed boundaries; `evaluator-large-route-oracle.*` |
| Original HTML layout | 4 header states, 6 system/filter states and all 512 body-row states match independent original generators before themed symbol expansion; `evaluator-html-oracle.*` |
| Visits/vault totals | 16 visit-history cases and 63 optional SubtractCurrent boundary cases, original deletion retention; `evaluator-state-oracle.*` |
| Value symbol lifetime | 9 original threshold-change/update cases preserve sticky IsHighValue; `evaluator-value-flag-oracle.*` |
| Body notification labels/SSML | 32 original `Utils.NotificationBodyName` source captures, `evaluator-notification-oracle.*` |
| Journal dispatcher | All21 types ×256 flags =5,376 source state captures; downstream pure processors have separate oracles |
| Status and persistent notifications | 7,744 focus/target/lifecycle,128 content/onlyUpdate and320 delivery predicate source captures in `evaluator-dispatch-oracle.*` |
| Star labels | 53 original `UIHandler.StarTypeLookup` source captures, `evaluator-star-oracle.*` |
| Toolbar navigation | Actual egui pointer press/release of all six buttons; content/zoom stay root viewport; explicit popout/settings commands captured |
| Popout focus gates | All seven original focus mappings/default AlwaysShow tested |
| Settings save | Worker history/routes/commander/unsold/vault emission regression; invalid input leaves settings intact |
| Notification delivery | All eight Popup/Voice/Plugin combinations, body/coalescing/SSML checked; persistent Core show/update/cancel test |

## Verification boundaries and host integration

Native settings control declarations, dimensions, source labels, save/cancel and custom-theme Reset/Apply are implemented and reviewed against installed source. Linux toolkit font rasterization and WinForms rasterization are different platform stacks; no live Windows-versus-Linux settings screenshot identity is asserted.

Every recognized single-event dispatcher type and flag combination has independent source captures. Full unbounded cross-event histories cannot be established by that bounded harness: it executes unchanged original dispatcher/status/persistent methods with inert downstream endpoints. Downstream calculations, visit sequences, complete source solver chains, actual subprocess replay, notification host routing, transport and UI each have their separate evidence above and below. This distinction does not identify an omitted event handler.

Native update delivery is implemented and verified with actual compatible ELF workers in `fixtures/native-updater-e2e-linux-qa.json`: all four independent workers, automatic/manual/disabled/beta policies, SHA/platform/protocol/version rejection, atomic stage, continued old processes, restart activation and crash isolation. The receipt pins its binary hashes. Vendor Windows EOP packages cannot replace Rust executables; publishing a native release feed remains a distribution prerequisite, not a simulated successful vendor update.

Actual Hecate HTTP/WebSocket requests cross the subprocess/host boundary in `hecate_process_real_http_and_websocket_requests_bridge_through_host_rpc`. Original notification SSML/coalescing and AboutInfo metadata are retained. The initialized-peer Engine regression verifies source Ready versions, caller exclusion, disabled-peer suppression and unchanged default FID; a Ready message itself only records peer identity. `fixtures/passive-notification-linux-qa.json` records native Wayland scale1/2 and X11 no-activation input tests, including an intentionally unsafe negative control and unsupported-compositor handling. Those tests use a focused synthetic game surface and do not claim an actual Elite session.

## Settings (all public declarations, including helper properties)

| Original property | Source line | Native mapping / status |
| --- | --- | --- |
| `ThemeSettings` | EvaluatorSettings.cs:36 | Implemented embedded Default/Orange palettes, custom color mode, Reset/Apply/Save |
| `MinimumBodyValue` | EvaluatorSettings.cs:40 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `MinimumValueForWorthMapping` | EvaluatorSettings.cs:43 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `MaxDistanceELW` | EvaluatorSettings.cs:46 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `MaxDistanceWW` | EvaluatorSettings.cs:49 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `MaxDistanceAW` | EvaluatorSettings.cs:52 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `MaxDistanceAtmospheric` | EvaluatorSettings.cs:55 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `MaxDistanceOther` | EvaluatorSettings.cs:58 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `IncludeAtmosphericBio` | EvaluatorSettings.cs:61 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `SetTargetedInFSSWorthMapping` | EvaluatorSettings.cs:64 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `BIHighValueWorthMapping` | EvaluatorSettings.cs:68 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `AlwaysShowBIHighValue` | EvaluatorSettings.cs:71 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `BIPersonalCodexWorthMapping` | EvaluatorSettings.cs:74 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `AlwaysShowBIPersonalCodex` | EvaluatorSettings.cs:77 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `BIGalacticCodexWorthMapping` | EvaluatorSettings.cs:80 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `AlwaysShowBIGalacticCodex` | EvaluatorSettings.cs:83 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `BIMarkedWorthMapping` | EvaluatorSettings.cs:86 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `AlwaysShowBIMarked` | EvaluatorSettings.cs:89 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `ConfigureAlwaysShow` | EvaluatorSettings.cs:93 | Implemented explicit source settings form action |
| `ConfigureStellarOverlay` | EvaluatorSettings.cs:98 | Implemented explicit source settings form action |
| `NativeOutputDict` | EvaluatorSettings.cs:103 | Implemented equivalent: fixed source choices / computed button size, no separately persisted helper |
| `NativeOutput` | EvaluatorSettings.cs:123 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `ExpandOutput` | EvaluatorSettings.cs:136 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `ShowHeaders` | EvaluatorSettings.cs:139 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `ShowTotal` | EvaluatorSettings.cs:142 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `MaxSystems` | EvaluatorSettings.cs:145 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `ButtonSizeDict` | EvaluatorSettings.cs:150 | Implemented equivalent: fixed source choices / computed button size, no separately persisted helper |
| `ConfigureNotifications` | EvaluatorSettings.cs:183 | Implemented explicit source settings form action |
| `Notifications` | EvaluatorSettings.cs:186 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `ConfigureTheming` | EvaluatorSettings.cs:190 | Implemented explicit source settings form action |
| `ConfigurePopout` | EvaluatorSettings.cs:194 | Implemented explicit source settings form action |
| `ButtonSize` | EvaluatorSettings.cs:197 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `ButtonSizeValue` | EvaluatorSettings.cs:254 | Implemented equivalent: fixed source choices / computed button size, no separately persisted helper |
| `AlwaysShowMain` | EvaluatorSettings.cs:278 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `AlwaysShowELW` | EvaluatorSettings.cs:281 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `AlwaysShowWW` | EvaluatorSettings.cs:284 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `AlwaysShowAW` | EvaluatorSettings.cs:287 | Source behavior preserved: declared and editable, but original `EDSystem.ShowInInterface` never reads it (source oracle verifies) |
| `AlwaysShowTerraformables` | EvaluatorSettings.cs:290 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `AlwaysShowAtmospherics` | EvaluatorSettings.cs:293 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `AlwaysShowAtmosphericsBios` | EvaluatorSettings.cs:296 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `AlwaysShowNonAtmosphericsBios` | EvaluatorSettings.cs:299 | Implemented: `evaluator_settings.rs` control; `evaluator.rs`/model consumer |
| `PopoutAlwaysShow` | EvaluatorSettings.cs:302 | Implemented source OR focus gate and host popout opacity |
| `PopoutShowNoFocus` | EvaluatorSettings.cs:305 | Implemented source OR focus gate and host popout opacity |
| `PopoutShowLeftPanel` | EvaluatorSettings.cs:308 | Implemented source OR focus gate and host popout opacity |
| `PopoutShowRightPanel` | EvaluatorSettings.cs:311 | Implemented source OR focus gate and host popout opacity |
| `PopoutShowDSSMode` | EvaluatorSettings.cs:314 | Implemented source OR focus gate and host popout opacity |
| `PopoutShowRolePanel` | EvaluatorSettings.cs:317 | Implemented source OR focus gate and host popout opacity |
| `PopoutShowSysMap` | EvaluatorSettings.cs:320 | Implemented source OR focus gate and host popout opacity |
| `PopoutShowFSSMode` | EvaluatorSettings.cs:323 | Implemented source OR focus gate and host popout opacity |
| `PopoutOpacity` | EvaluatorSettings.cs:326 | Implemented source OR focus gate and host popout opacity |
| `InstallUpdates` | EvaluatorSettings.cs:330 | Source controls/defaults preserved; native updater distribution contract pending |
| `InstallBetas` | EvaluatorSettings.cs:333 | Source controls/defaults preserved; native updater distribution contract pending |
| `VerboseLogging` | EvaluatorSettings.cs:337 | Implemented Core Log diagnostics (event names and legacy messages) |
| `UseCustomNotificationColours` | EvaluatorSettings.cs:393 | Implemented color editor + Register/UnregisterPluginCustomColours handshake |
| `TitleColorHex` | EvaluatorSettings.cs:396 | Implemented color editor + Register/UnregisterPluginCustomColours handshake |
| `BackgroundColorHex` | EvaluatorSettings.cs:399 | Implemented color editor + Register/UnregisterPluginCustomColours handshake |
| `BorderColorHex` | EvaluatorSettings.cs:402 | Implemented color editor + Register/UnregisterPluginCustomColours handshake |
| `ColorHex` | EvaluatorSettings.cs:405 | Implemented color editor + Register/UnregisterPluginCustomColours handshake |

## Journal event dispatch (all original recognized types)

| Event | Native status |
| --- | --- |
| `ApproachBody` | Source oracle verifies256 single-event flag/state cases; cross-event boundary noted above |
| `CarrierJump` | Source oracle verifies256 single-event flag/state cases; cross-event boundary noted above |
| `Died` | Source oracle verifies256 single-event flag/state cases; cross-event boundary noted above |
| `Disembark` | Source oracle verifies256 single-event flag/state cases; cross-event boundary noted above |
| `Embark` | Source oracle verifies256 single-event flag/state cases; cross-event boundary noted above |
| `FSDJump` | Source oracle verifies256 single-event flag/state cases; cross-event boundary noted above |
| `FSSAllBodiesFound` | Source oracle verifies256 single-event flag/state cases; cross-event boundary noted above |
| `FSSBodySignals` | Source oracle verifies256 single-event flag/state cases; cross-event boundary noted above |
| `FSSDiscoveryScan` | Source oracle verifies256 single-event flag/state cases; cross-event boundary noted above |
| `LoadGame` | Source oracle verifies256 single-event flag/state cases; cross-event boundary noted above |
| `Location` | Source oracle verifies256 single-event flag/state cases; cross-event boundary noted above |
| `MultiSellExplorationData` | Source oracle verifies256 single-event flag/state cases; cross-event boundary noted above |
| `SAAScanComplete` | Source oracle verifies256 single-event flag/state cases; cross-event boundary noted above |
| `SAASignalsFound` | Source oracle verifies256 single-event flag/state cases; cross-event boundary noted above |
| `Scan` | Source oracle verifies256 single-event flag/state cases; cross-event boundary noted above |
| `ScanBaryCentre` | Source oracle verifies256 single-event flag/state cases; cross-event boundary noted above |
| `ScanOrganic` | Source oracle verifies256 single-event flag/state cases; cross-event boundary noted above |
| `SellExplorationData` | Source oracle verifies256 single-event flag/state cases; cross-event boundary noted above |
| `Shutdown` | Source oracle verifies256 single-event flag/state cases; cross-event boundary noted above |
| `StartJump` | Source oracle verifies256 single-event flag/state cases; cross-event boundary noted above |
| `SupercruiseExit` | Source oracle verifies256 single-event flag/state cases; cross-event boundary noted above |

## Original message dispatch (all command strings)

| Command | Native status |
| --- | --- |
| `EvaluatorSetTotal` | Implemented in `Evaluator::on_message` |
| `GetFIDResponse` | Implemented in `Evaluator::on_message` |
| `LegacyPluginMessage` | Implemented original diagnostic-only behavior; original does not execute a legacy command |
| `Ready` | Source peer handler records GUID/version only; native initialization lifecycle must be distinguished (host task) |
| `RegisterPluginCustomColoursResponse` | Implemented in `Evaluator::on_message` |
| `SetBioInsightsFlags` | Implemented in `Evaluator::on_message` |
| `SetBodyVisited` | Implemented in `Evaluator::on_message` |
| `SetServerAddress` | Implemented in `Evaluator::on_message` |
| `SetWorthVisiting` | Implemented in `Evaluator::on_message` |
| `WebRequest` | Implemented source HTML and dispatch/correlation; host live publish hook is tracked in Core integration audit |
| `WebSocketRequest` | Implemented source HTML and dispatch/correlation; host live publish hook is tracked in Core integration audit |
| `message` | Implemented in `Evaluator::on_message` |

## Original public methods (complete extracted inventory)

| Source method | Source location | Native equivalent / status |
| --- | --- | --- |
| `SetBodyExtraDetailsBI` | `Evaluator/Evaluator.cs:640` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `SetBodyExtraDetails` | `Evaluator/Evaluator.cs:704` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `ResetPluginUI` | `Evaluator/Evaluator.cs:2831` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `ProcessOrrery` | `Evaluator/Evaluator.cs:2836` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `GetOptimizedListNew` | `Evaluator/Evaluator.cs:3466` | Exact and larger branches verified with source captures; queued async completion and late route notifications |
| `SetCurrentSystem` | `Evaluator/Evaluator.cs:3757` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `Load` | `Evaluator/Evaluator.cs:3897` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `ObservatoryReady` | `Evaluator/Evaluator.cs:4216` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `StartedReadAll` | `Evaluator/Evaluator.cs:4246` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `FinishedReadAll` | `Evaluator/Evaluator.cs:4300` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `ForceWebClientsToRefresh` | `Evaluator/Evaluator.cs:4402` | Implemented source HTML layout, shell, symbols and request IDs; host live publish hook is tracked in Core integration audit |
| `StartedPreRead` | `Evaluator/Evaluator.cs:4443` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `FinishedPreRead` | `Evaluator/Evaluator.cs:4451` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `RequestUpdatedTotal` | `Evaluator/Evaluator.cs:4559` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `Test` | `Evaluator/Evaluator.cs:4619` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `UpdateUItmp` | `Evaluator/Evaluator.cs:4685` | Source oracle verified 3,600 filter/override cases |
| `UpdateUI` | `Evaluator/Evaluator.cs:4691` | Source oracle verified 3,600 filter/override cases |
| `StatusChange` | `Evaluator/Evaluator.cs:5045` | Source oracle:7,744 focus/target/lifecycle cases; ignores stellar FSS targets and preserves onlyUpdate semantics |
| `UpdatePersistentNotification` | `Evaluator/Evaluator.cs:5438` | Source oracle:128 content/onlyUpdate cases +320 actual event delivery predicates; Core rendering lifecycle |
| `HandlePluginMessage` | `Evaluator/Evaluator.cs:5876` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `HandleLegacyPluginMessage` | `Evaluator/Evaluator.cs:6436` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `HandleSetBioInsightsFlags` | `Evaluator/Evaluator.cs:6610` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `HandleSetWorthVisiting` | `Evaluator/Evaluator.cs:6707` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `HandleSetBodyVisited` | `Evaluator/Evaluator.cs:6759` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `HandleEvaluatorSetTotal` | `Evaluator/Evaluator.cs:6814` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `HandleGetFIDResponse` | `Evaluator/Evaluator.cs:6919` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `HandleWebRequest` | `Evaluator/Evaluator.cs:7020` | Implemented source HTML layout, shell, symbols and request IDs; host live publish hook is tracked in Core integration audit |
| `HandleWebSocketRequest` | `Evaluator/Evaluator.cs:7058` | Implemented source HTML layout, shell, symbols and request IDs; host live publish hook is tracked in Core integration audit |
| `HandleRegisterPluginCustomColoursResponse` | `Evaluator/Evaluator.cs:7120` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `HandleSetServerAddress` | `Evaluator/Evaluator.cs:7252` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `ProcessWebRequest` | `Evaluator/Evaluator.cs:7316` | Implemented source HTML layout, shell, symbols and request IDs; host live publish hook is tracked in Core integration audit |
| `HandleFilterChange` | `Evaluator/UIHandler.cs:302` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `GenerateCurrentPage` | `Evaluator/UIHandler.cs:352` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `HandleURL` | `Evaluator/UIHandler.cs:359` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `HandleEVToggle` | `Evaluator/UIHandler.cs:1110` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `SetCommander` | `Evaluator/UIHandler.cs:1255` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `OpenHTMLTable` | `Evaluator/UIHandler.cs:1263` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `CloseHTMLTable` | `Evaluator/UIHandler.cs:1274` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `AddHTMLTableFooter` | `Evaluator/UIHandler.cs:1280` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `AddHTMLTableHeaders` | `Evaluator/UIHandler.cs:1318` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `AddHTMLSystemDetails` | `Evaluator/UIHandler.cs:1459` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `AddHTMLBodyDetails` | `Evaluator/UIHandler.cs:1652` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `SaveSettings` | `Evaluator/UIHandler.cs:2187` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `GenerateAboutPage` | `Evaluator/UIHandler.cs:2217` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `GenerateUnsoldDataPage` | `Evaluator/UIHandler.cs:2389` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `GenerateUnsoldDataPage` | `Evaluator/UIHandler.cs:2526` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `HandleGuiStatusChange` | `Evaluator/UIHandler.cs:2738` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `StarTypeLookup` | `Evaluator/UIHandler.cs:2789` | Source oracle verified 53 labels |
| `AddBody` | `Evaluator/EDSystem.cs:408` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `GetBodyValue` | `Evaluator/EDSystem.cs:546` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `UpdateMainStarBonus` | `Evaluator/EDSystem.cs:627` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `GetMainStarId` | `Evaluator/EDSystem.cs:788` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `GetBodyIdFromVisitOrder` | `Evaluator/EDSystem.cs:912` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `ResetVisitOrder` | `Evaluator/EDSystem.cs:984` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `SetBioGeocount` | `Evaluator/EDSystem.cs:1025` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `ShowInInterface` | `Evaluator/EDSystem.cs:1096` | Source oracle verified 3,600 filter/override cases |
| `UpdateValues` | `Evaluator/Body.cs:782` | Implemented equivalent in native worker/model/UI; differential coverage limited as above |
| `KNearest` | `EDJP.Observatory.Solver/Tsp3DSolver.cs:224` | Exact and larger source algorithms verified by primitive/fixed-iteration/complete-chain captures; native work runs off dispatch with cancellation |
| `Run` | `EDJP.Observatory.Solver/Tsp3DSolver.cs:5044` | Exact and larger source algorithms verified by primitive/fixed-iteration/complete-chain captures; native work runs off dispatch with cancellation |
| `Solve` | `EDJP.Observatory.Solver/Tsp3DSolver.cs:5270` | Exact and larger source algorithms verified by primitive/fixed-iteration/complete-chain captures; native work runs off dispatch with cancellation |
| `SolveAsync` | `EDJP.Observatory.Solver/Tsp3DSolver.cs:5465` | Exact and larger source algorithms verified by primitive/fixed-iteration/complete-chain captures; native work runs off dispatch with cancellation |
| `Dispose` | `EDJP.Observatory/FZoomSelector.cs:105` | Native RAII lifecycle equivalent |
| `SetPanel` | `EDJP.Observatory/UIBlankView.cs:56` | Implemented NativeOutput Disabled restart latch and source Arial20pt text/alignment with native panel bounds after initialization; browser-only DOM operations have no output while disabled |
| `SetWidthHeight` | `EDJP.Observatory/UIBlankView.cs:67` | Implemented NativeOutput Disabled restart latch and source Arial20pt text/alignment with native panel bounds after initialization; browser-only DOM operations have no output while disabled |
| `ScrollToLine` | `EDJP.Observatory/UIBlankView.cs:77` | Implemented NativeOutput Disabled restart latch and source Arial20pt text/alignment with native panel bounds after initialization; browser-only DOM operations have no output while disabled |
| `SetVisible` | `EDJP.Observatory/UIBlankView.cs:87` | Implemented NativeOutput Disabled restart latch and source Arial20pt text/alignment with native panel bounds after initialization; browser-only DOM operations have no output while disabled |
| `SetNativeDocument` | `EDJP.Observatory/UIBlankView.cs:98` | Implemented NativeOutput Disabled restart latch and source Arial20pt text/alignment with native panel bounds after initialization; browser-only DOM operations have no output while disabled |
| `AddSchemeHandler` | `EDJP.Observatory/UIBlankView.cs:108` | Implemented NativeOutput Disabled restart latch and source Arial20pt text/alignment with native panel bounds after initialization; browser-only DOM operations have no output while disabled |
| `HideObjectByID` | `EDJP.Observatory/UIBlankView.cs:118` | Implemented NativeOutput Disabled restart latch and source Arial20pt text/alignment with native panel bounds after initialization; browser-only DOM operations have no output while disabled |
| `GetElementByIdGetAttribute` | `EDJP.Observatory/UIBlankView.cs:129` | Implemented NativeOutput Disabled restart latch and source Arial20pt text/alignment with native panel bounds after initialization; browser-only DOM operations have no output while disabled |
| `SetElementContent` | `EDJP.Observatory/UIBlankView.cs:140` | Implemented NativeOutput Disabled restart latch and source Arial20pt text/alignment with native panel bounds after initialization; browser-only DOM operations have no output while disabled |

## Source-only oracle normalization

Oracles compile copied original pure methods plus minimal framework stubs using .NET10 in a cache directory. No installed plugin assembly is started, and synthetic fixtures contain no commander data. The decompiler can place init-local assignments inside a goto target even though IL initializes them once at method entry: `Body.GetK` and `EDSystem.GetBodyValue` locals and notification formatter text2 were hoisted to method entry to match IL lifetime. No calculation conditions/constants were changed. The star-label method and exact solver method compile unchanged. Source fixtures and drivers are retained to make those boundaries reviewable.

## Larger-route oracle recovery and runtime limits

The complete original pure `Tsp3DSolver` was decompiled with `LocalFunctions=false` and `StackAllocInitializers=false`, recovering actual generated Find/Connect and local move helpers instead of inventing them. Compiler-generated identifiers were renamed consistently; encrypted exception strings came from the existing decoded constant map; scratch stack spans were replaced by equal-length heap arrays to resolve C# escape scope without changing operations. Six decompiler init-local/goto artifacts were normalized to method-entry initialization. Diagnostic trace calls record move inputs without changing algorithm decisions. Original source, driver and captures are retained.

Native uses the original 12-neighbor candidate search, degree-two greedy construction, cyclic shorter-complement 2-opt, 1..3-body Or-opt with exact rotation/requeue order, .NET seeded Random, source DoubleBridge, MST lower bound, 2% gap, 200-step stagnation and 2-second budget per restart. CPU-count parallel restarts use source seed spacing7919/+17 and select the shortest complete route. Clock-limited results remain nondeterministic across machines, as in the original. Exact fixed traces and complete non-time-limited captured chains are compared directly, without a loose cost-tolerance substitute.

A cancellation/revision guarded route worker keeps bounded optimization away from journal dispatch. Batch imports calculate routes once on completion. Mapping/discovery notifications wait for the final next target; old-commander completions are discarded. A 4,000-row GUI regression checks that only visible rows create painted shapes and that the history is indexed by visit instead of rescanning every body for every system each frame.


## Dispatcher and persistent lifecycle corrections

The source dispatcher harness covers all21 recognized journal types at256 flag combinations each. Additional StatusChange, UpdatePersistentNotification and ClosePersistentNotification source methods cover focus transitions, current/foreign destinations, star targets, target-override settings, existing notification IDs and onlyUpdate. No plugin assembly or private commander file executes in the harness. All original dispatcher methods compile unchanged; their48 journal transparent wrappers and additional status/persistent wrappers are copied verbatim. Synthetic Core/price/Orrery/UI stubs make the verification boundary explicit.

Captured differences corrected: Location changes displayed system only if its ID differs, while FSDJump always starts a visit; Location transport is InSRV then OnFoot then Ship; SRV/on-foot death does not delete exploration; ScanOrganic only updates Odyssey and Disembark only updates transport; ApproachBody records a body but optimizes only after completion/mapping and in ship mode; foreign FSSAll does not mutate current state but retains the source notification branch. ReadAll route notification suppression is distinct from Core batch suppression of body notifications.

Persistent notifications now follow original explicit request/update/cancel calls. AlwaysPersistent does not invent a notification on every scan/location/recalculation. Supercruise start may create it; hyperspace, SupercruiseExit and shutdown close it; DSS focus opens it and leaving DSS closes it unless AlwaysPersistent is enabled. Source onlyUpdate cannot create an absent notification. Unchanged rendered content is coalesced to avoid redundant GUI work. NativeOutput uses the original case-sensitive StartsWith("Disabled") restart latch. Source cached notification IDs are retained across batch transitions; no speculative batch-exit replay was added.

Legacy settings/data migration has11 settings and7 storage tests: all four active WAL SQLite snapshots leave original DB/WAL bytes unchanged; business-key merge preserves existing native records and is idempotent; Bio seen/marks/distance-cache import is per FID and preserves other native commanders; malformed source prevents all prepared file commits; source custom palettes and exact custom-popout files migrate; absence of adjacent original storage yields a settings-only receipt; destination symlinks into original storage are rejected before any commit. Imported files and previous snapshots have a durable receipt. These tests use synthetic files and have not migrated the user's live data.


The source About HTML generator and five Unsold datasets now have original C# captures. Native browser pages use that exact About template, source Unsold markup/row order, duplicate-name visits, positive-only displayed rows, inclusive totals, and Row2 first striping. Native Unsold mirrors those rows/spacing and uses intrinsic table width. The source UIAlert enum/resource is separately extracted with PE metadata reads: Evaluator has six alerts (no ReadAllRequired); every one is nonpersistent, unlike BioInsights. Native/browser banner IDs/classes/text, dismiss route and changed-system expiry are implemented and tested. Desktop tests exercise actual pointer input for all six toolbar buttons and banner dismissal, plus the exact Disabled prefix and restart latch.

Three original UpdateUI multi-visit captures verify ascending body-ID rows and the original address-based Current header behavior: all same-address visits remain marked Current during ReadAll. VisitID indexes prevent rows from separate visits merging; they do not change that original Current predicate. Browser row grouping is indexed once per visit to avoid repeated whole-history scans.

## Native CSS rendering and actual window checks

`fixtures/evaluator-css-layout.json` and `evaluator-ancillary-css-layout.json` are measured with real WebKit from the independently verified source page builders and original embedded style.css/buttons.css. `scripts/evaluator_dom_probe.py` records DOM rectangles, CSS fonts, intrinsic table widths and hyperlink bounds on a private display. Native geometry tests compare all 10 main columns in all four ShowHeaders/ShowTotal combinations, both About columns, all 3 Unsold columns and first header x/y/height and exact current-system row Y. Maximum column differences are below 1/32 CSS pixel on this Linux font configuration. A dedicated advance helper combines hinted glyph advances with fractional kerning, leaving only the browser layout-unit quantization; raster painting is unchanged. Unsold column widths match the captured values exactly.

Native uses actual Pango/Cairo platform font matching and librsvg/Cairo rendering of original SVG bytes, including source custom-color substitution, aspect ratios and original text-only symbols. It preserves the38px normal toolbar pitch, body/content padding, CSS fonts/sizes/weights, source row/border/blank/footer heights, proportional intrinsic-width expansion, source label contents (including HGG's displayed HG), hover titles, disabled-output 20pt Arial text and alignment (native panel bounds), the About checkbox example and original hyperlink destinations. Document scrolling includes the toolbar; source current-system anchoring replaces unconditional bottom sticking. Palette and intrinsic symbol dimension caches avoid reparsing thousands of repeated SVGs each frame and invalidate measured widths on projection/DPI changes.

`dist/evaluator-parity-ui/receipt.json` records five actual native XTest interaction groups: About, Show Current Data, Main, and Zoom remain in the existing window; only the explicit source Popout action creates one extra custom window. Captures are from a disposable synthetic profile, not the user's journals. The corresponding 14 nonignored native UI tests pass, plus the explicitly run release profile below.

`dist/evaluator-parity-ui/performance.json` measures production Rust layout, CSS glyph raster-cache use and egui tessellation with 4,000 source rows and 140 warmed samples per resolution. At 1200×800 mean 1.165 ms/p95 1.233 ms; at 3440×1440 mean 1.389 ms/p95 1.466 ms. First-frame initialization was 63.8 ms/43.3 ms respectively. This is a CPU renderer measurement on this host, not an asserted GPU-present or other-machine frame rate. The ignored opt-in test `profile_evaluator_native_paint` was explicitly executed in release mode and passed its 16 ms mean/33 ms p95 budget.

Evaluator and Bio native pages now have independent snapshot revisions. Unrelated message/finding/status outputs do not remeasure Evaluator's immutable rows or reset its current-system scroll; an Engine regression verifies both Arc identity and revision behavior for each page. Repeated-address visits retain the original Current flags, while native scrolling targets the first matching Current header, matching the source document's first duplicate currentsystem ID.

The native alert float follows the original35px single-line alert height (13px Verdana with15px normal line plus20px padding). Its separate sibling BR adds19px, and the floated22px close anchor does not inflate the line. `evaluator-alert-css-layout.json` independently captures this from WebKit; tests compare alert x/y/width/height and the exact following table-header Y. Strong spans and wrapped source descriptions retain their styles.

## Verified graphical build

All five actual Evaluator window/navigation action groups passed on immutable Linux release SHA-256 `985a16215aec6520120a8d8b97eaa2dc79bf7c7ced9a3ac00590e6b5594ed343`. `dist/evaluator-parity-ui/receipt.json` and its captures were refreshed from that build. Core's 12 interaction groups passed on the same binary. These receipts verify actual native window behavior, separately from the source geometry and calculation oracles.
