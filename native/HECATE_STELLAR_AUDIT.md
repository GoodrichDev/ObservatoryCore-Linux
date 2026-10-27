# Hecate and StellarOverlay source parity audit

Audit date: 2026-10-25. Reference binaries are the installed Hecate 1.3.444.761 and StellarOverlay 1.3.228.387 identified by SHA-256 in each worker's `PORTING.md`.

`fixtures/hecate-stellar-source-inventory.json` is the finite source inventory. It records 23 Hecate and17 StellarOverlay original message names, every public/non-obfuscated private method in the reviewed worker, settings, forms, queue/drawing/storage/server classes (277), and144 properties with setting attributes. Method entries include payload keys and string constants, so the original actions, numeric validation labels, messages and returned fields can be located without guessing. File hashes identify the exact decoded source. Regenerate with `python3 native/scripts/inventory_vault_overlay.py --cache /home/loic/.cache/observatory-readable`. Dispatch presence is a structural check, not a claim that every branch is equivalent.

## Hecate

| Source behavior | Native implementation and verification |
| --- | --- |
| No output tab (`PluginUIType.None`), journal events and commander changes | `hecate.rs::on_event/process`; archive/FID/unknown-field/unsigned-ID, census and batch-rebuild tests |
| Location/FSDJump/CarrierJump; StartJump/CarrierJumpRequest prefetch; LoadGame; NPC ReceiveText/ShipTargeted/Bounty; scan/approach/supercruise/organic events | Worker source event-type list in inventory; native process preserves raw event documents and separates derived census/current-system behavior |
| Read All and pre-read lifecycle | `begin_replay`, `set_batch`, flush; repeated full rebuild test |
| Four databases and Zstandard stored journals | `schema.rs`, `storage.rs`; actual SQLite reopen, source SystemDB schema and independent Zstandard-frame tests |
| LoadSystem, GetSystemsVisitedFromList, GetFID | Complete original ID64/JournalEvents/FIDs response; unsigned address, preserved payload and commander tests |
| EvaluatorAdd/SysAdd/Delete/GetData/GetTotal | Native exploration ledger; upsert/aggregation/commander-separation/opaque-response tests |
| BioInsightsUnsoldAdd/Delete, SetBonus, GetData/GetTotal | Native biology ledger; default discovery bonus and explicit bonus survival tests |
| PIAddMerits/GetMerits/GetCycle, GetCurrentMerits | Native merit ledger; duplicates, cycle/day/session/system scopes tests |
| SpanshSystem and automatic Spansh/Canonn caches | Async host request queue, daily cache, in-flight coalescing; no network inside callback tests |
| Spansh scan reconstruction | `spansh.rs`; unit/body/genus/identifier/barycentre/ring/signal tests, independent source CSV resources |
| WebServerRegisterPlugin, WebSocketResponse/Broadcast | `server.rs`; real loopback HTTP and WebSocket request/reply/broadcast/shutdown tests |
| `/`, `/status`, `/favicon.ico`, plugin routes and `/ws/{plugin}` | Original absolute button links, title/CSS and actual connected-client status fields; live HTTP/WS assertions |
| Ready and legacy-message logging | Accepted without mutating game data; host handles logs |
| Reset Journal Database | Original confirmation and result; truncates only persisted journal database, retaining current visit buffer, general ledger and remote caches; regression test |
| Web Server Settings | Native child form preserving draft until Save, original enable/IP/port; invalid source port becomes58080; listener failures roll back and preserve current commander, journal and plugin registration; regression test |
| InstallUpdates, InstallBetas, VerboseLogging | Exposed original flags; native host update/logging implementation is shared with other plugins |

## StellarOverlay

| Source behavior | Native implementation and verification |
| --- | --- |
| Popup-only notifier override; add/update/cancel; persistent IDs; positive lifetime begins when released | `stellar_overlay.rs`, TTL/update/cancel/queue/first-tick tests |
| Queue active limit, source delay, fixed notifications, 300ms transitions | Worker + `stellar_ui.rs`; queue-lifetime and independent transition equations tests |
| Original shape/font/ARGB/opacity/padding/border/alignment/placement/monitor/FPS settings | Native source forms + renderer; source defaults, bounds/invalid-fallback, ARGB tests |
| Ready/LegacyPluginMessage | Accepted, no invented redispatch |
| RegisterPluginCustomColours/UnregisterPluginCustomColours | Partial brushes merge; defaults returned only on requested response; unregister disables but retains brush cache; existing active/pending colors refresh; regression test |
| ShowBitmapNotification/HideBitmapNotification/SetDisplayOverrideTTL | Persistent keyed alerts, filename-backed bitmap or native Bio/Evaluator draw data; parent host owns resource loading, actual bitmap projection and focus policy |
| RadarSetPlanet/Hide/ForceHidden | Planet/genus/context handling, altitude <=2000m visibility; force-show bypasses altitude only; source visibility oracle |
| RadarAdd/RemoveVehicle, Add/RemoveBiological, AddWaypoint, SetContextID, SetColonyRanges | Heading-relative projection, replacement/removal, genus ranges, source waypoint numbering; worker tests |
| Actual Radar.Draw | `stellar_radar.rs`; complete command trace compared against copied original method in284 cases:192 visibility combinations,32 sample/distance captions,60 point/context/radius cases |
| Radar visual details | Original palettes, inner-only translucent fill, three rings/bottom labels/ticks, vehicle corner markers, cross/plus/gap glyphs, genus collection areas, POSITION/CURRENT SAMPLE text, circle sample count and sorted two-column genus legend with seen checks. These replace authored approximations and omitted source metadata. |
| Status/heading/position; movement smoothing | Real status altitude/heading/lat/lon; shortest heading/dateline path test; source distance rings1km76%,2km88%,10km100% |
| Change Notification Settings / Change Radar Settings | Original distinct forms and Apply and Test/Apply/Close semantics, draft Close discard, exact numeric fallbacks and ARGB sanitizer; tests |
| Apply/Test notifications | Original test count/title/multiline detail; force display TTL; respects saved handler flag after direct preview; regression test |
| Apply radar | Source reinitialization, Visible=true five-second radar force-show then hides, six-second display override; regression test |
| Write Overlay State to Log / Spawn Debug Notifications | Actual snapshot reply for native log; five source-format debug notifications with original coordinate formulas; worker regression test |
| Ordinary preference changes | In-place settings retain active and queued notifications, bitmap panels, planet and sample/seen-genus state; regression test |
| Foreground game window / non-activation / click-through | Native host passive-window implementation owns X11 and Wayland backend acceptance; no claim that requested viewport hints alone establish game safety |

## Independent radar oracle

`fixtures/stellar-radar-draw-oracle.cs` contains the actual installed `Radar.Draw` method, its five trivial runtime wrappers and recording graphics stubs. It does not load a vendor assembly or initialize its workers. `stellar-radar-draw-oracle.json` captures284 source outputs using fixed fonts/measurement. The Rust comparison checks every recorded text, shape, position, color key, style and visibility result. Geometry uses a100px outer radius,12px font and fixed six-pixel UTF16 text measurement; live egui uses the actual selected font's measured text width. Raster font rendering across DirectWrite and Linux font engines is not covered by that command-level comparison.

## Acceptance boundaries

All original command names are dispatched. The inventory also includes internal getters, pure wrappers and source controls that are `[SettingIgnore]`; it is not a list of277 unimplemented features. The audit closes the concrete differences listed above, with executable proofs. It does not prove every possible remote Spansh document variant, every historical SQLite database, pixel identity across font engines, or fullscreen game focus on every compositor. Host update delivery, debug-log persistence, Core fallback popups, structured Bio/Evaluator bitmap projection and passive foreground behavior are integration responsibilities tested by the parent task; this document must not be used to claim they are finished solely from worker tests.

## Notification field audit

The installed Stellar notifier reads `Title`, `Detail`, `Sender`, `Timeout`, `Guid`, `Rendering`, `XPos` and `YPos`; the instant queue also reads the `ExtendedDetails` sentinel `CANCEL`. Its worker, queue, item and extended-item classes have no references to `CoalescingId`, `Suppression`, `TitleSsml` or `DetailSsml`; those are Core/audio contracts. They must not acquire invented Stellar meanings.

`stellar-notification-routing-oracle.cs` copies the installed `OnNotificationEvent` and `UpdateNotification` method bodies plus their pure Enum.HasFlag wrapper. Data, queue and logger stubs only record routing. The1,728 original-method cases cover no/empty/valid Guid; TTL-1,0,1,5000; all nine combinations of X/Y-1,0,50; Rendering0..7; handler enabled/disabled. Another32 cases verify updating requires an existing instant item, Popup bit and enabled handler. The native model now matches every case. In particular, BOTH X/Y must differ from-1 to bypass the stack, and TTL0 without a valid Guid becomes four times the configured TTL. Plain Gui­ds alone do not bypass the stack. Updates change positions and the CANCEL sentinel removes instant items.

The source font list is now a system-font dropdown populated asynchronously through Fontconfig; selected absent fonts remain visible. Colour swatches open the source ARGB channel selector on double-click, with OK/Cancel and draft semantics. The cached font-family parser has a regression test.

Radar placement follows the original integer-division formula (Radar.cs1094–1097), including subtraction of the radar's percentage anchor. For a1920×1080 monitor with the original20% size,2%X and50%Y, the circle is216px and its source origin is(34,400), not the previously authored(38.4,540). `radar_layout` now preserves circle diameter while sizing the native surface to measured source text/shape bounds, preventing high-font captions from clipping. The284 Draw traces remain independent of native window cropping.

## Original notification layout, draw and slot traces

`stellar-notification-layout-oracle.cs` now copies the installed `NotificationQueue.AddNotificationItem`, `InitialiseFontHeight`, `getNotificationHeight`, and `NotificationItem.Draw` methods with their pure wrappers. The two static three-character trim arrays are decoded by matching SHA256 of their UTF16 RVA bytes, not inferred from screenshots. Recording graphics/font/data stubs remain independent of native Rust code.

The native projection matches216 complete title/detail measurement cases,108 Draw command traces and108 slot-initialization cases (324 reference positions). Cases include empty text, long words, repeated spaces, explicit multiple lines, three width bounds, automatic font reduction, open/closed shapes, three alignments and borders0/3/10. The source uses measured width (minimum monitorWidth/12), normal font=max(trunc(monitorHeight*16/1440*FontScaling/100),4), bold normal+2, and stable preallocated reference slots. The native worker now retains each queued item's allocated slot after another notification expires; rendering uses the measured source layout and Draw command sequence. Instant notifications without explicit coordinates use source position(100,100). Empty Guids receive distinct generated IDs; anonymous TTL0 messages use TTL*4.

Fontconfig resolves both regular and bold families asynchronously. Native viewport bounds include half the border stroke outside the source content rectangle, avoiding clipping the source's centered outline. Geometry command comparison covers logical layout and selected color keys; font rasterization remains dependent on the installed Linux font engine.

## Linux passive surface acceptance (2026-10-26)

`passive-notification-linux-qa.json` records real compositor/window-manager tests with synthetic data only.

| Backend | Acceptance |
| --- | --- |
| X11 Stellar notification, radar, image, show/update/cancel/recreate/expire |5 mapped surfaces declare input=false and notification window type BEFORE mapping;0/827 foreground focus changes |
| X11 Core fallback with Stellar disabled |2 actual Core surfaces across lifecycle;0/834 focus changes |
| Pure Wayland, headless Sway, fullscreen game surrogate |0/209 focus violations;9 clicks pass through overlays and typedabcd reaches game;5 real native layers; notification/radar/Core screenshots contain actual text and file PNG renders |
| Pure Wayland at scale2 |0/208 focus violations;9 clicks+abcd; actual text/PNG captures, activated Elite title transitions both directions |
| Unsafe ordinary xdg toplevel negative control |210/210 and212/212 focus violations at scales1/2; no game clicks or typed text delivered |
| Compositor without layer-shell, headless Weston |Explicit capability error,0 overlay layers; protocol trace exactly1 xdg_toplevel (Coremain), so no focus-unsafe popup fallback |

The native Wayland implementation uses a dedicated Wayland connection and one owned EGL/egui-glow renderer shared by all passive layers. Reusing winit's wl_display with extra wl_output bindings was rejected after a real SCTK object-data mismatch; the owned connection avoids mixing framework proxies. Each fresh surface receives layer-shell Overlay role, keyboard interactivityNone, empty pointer/touch input region and exclusive zone-1 before its first buffer. Root GL context is restored after rendering. Root swaps are nonblocking so covered/minimized Core does not stall game overlays. A single ordered egui callback layer preserves incremental shared font atlas updates. CPU-decoded PNGs are cached; texture creation occurs in the context that renders them.

Read-only foreign-toplevel title/app-id/Activated state supplies Elite foreground visibility when the compositor advertises it. Logical output geometry supplies monitor placement/scaling. Core visibly reports missing layer-shell support; it never substitutes a focusable xdg popup. X11/XWayland remains the default when reachable. Compositors that provide neither XWayland nor layer-shell cannot supply this overlay role through the standard APIs; their capability failure is explicit.

Reproduce with `scripts/wayland_focus_test.py` against an isolated Sway session, `scripts/wayland_capability_test.py` against isolated Weston, and `scripts/notification_focus_test.py` under private Xvfb (including `--core-fallback`). Wayland probes are ignored native tests, built using `cargo test -p observatory-desktop --no-run --message-format=json`; pass that JSON log to the scripts' `--build-log` option. Use `--scale 2` for HiDPI. Screenshots and keyboard/pointer probes run only on the explicitly supplied isolated compositor.

## Independently installable native workers

`worker_rpc` and the real `observatory-native-worker` executable now expose all four production Rust models across a versioned JSON-lines process boundary. Typed adapters preserve every host API, including pending plugin messages, HTTP effects, route polling, UI/source HTML, notification payloads, persistent Codex and vault data. Builtin calls remain direct by default; an installed artifact selects only its own plugin on restart. Native artifact inspection does not initialize Hecate services. The host updater owns source InstallUpdates/InstallBetas policy, compatible feed delivery and atomic installation; the transport does not pretend that downloading a Windows EOP changes native code.

`fixtures/native-workers-linux-qa.json` records 178 passing plugin tests (166 unit, one shared replay, two source-parity tests and nine actual subprocess tests), clean all-targets Clippy, and four packaged/hash-verified ELF artifacts. Process comparisons cover actual production projections/effects and durable reopen. Real Hecate child HTTP/WebSocket requests cross the host RPC boundary with original correlation fields and UTF8 content. A real SIGKILL test proves a failed Bio child retains completed projections and personal state, reports one bounded error, and leaves a separately selected Evaluator child healthy. An unrelated Linux ELF fails the real worker handshake. Independent protocol tests prove lifecycle errors and a 50,000,000ms host clock, preventing child-origin TTL errors. Source binary64 projections are exact through JSON float-roundtrip parsing, and unsigned ID64 values retain all bits. See `NATIVE_WORKER_PROTOCOL.md` and `scripts/package_native_workers.py` for the release contract and reproducer.

## Actual release Core updater acceptance

`fixtures/native-updater-e2e-linux-qa.json` is the strict end-to-end run of the real release Core and real worker ELF, with frozen input copies and recorded SHA256 values. Private XDG profiles contain synthetic journals only; the update feed is loopback. Automatic and manual per-plugin installation, beta exclusion/opt-in and retained disabled flags pass. During a staged Bio revision, all four existing child PIDs continue running; `/proc/<pid>/exe` retains the old inode while the installed path receives a new one. The staged revision remains unactivated until the subsequent Core start, which launches four new children and exposes all four real Hello contracts.

Fresh builtin and all-external Core sessions produce identical plugin rows, Bio main view, Bio/Evaluator summaries, overlay data, findings and SQLite ledger values. Separate-session animation timing/opacity fields are normalized; the RPC clock tests verify their host-clock contract independently. The session retains a persistent notification and radar point, records 19,010,800 biology credits and 50,594 exploration credits, and produces the same journal-vault visit count in both modes. Corrupt SHA, Windows/non-ELF bytes, a compiled wrong-protocol negative control and mismatched source-version metadata all fail without replacing any existing installed worker or metadata bytes. A real Bio SIGKILL leaves its personal Codex checkpoint unchanged while a new Evaluator Earthlike scan and Stellar notification/radar continue through Core. The run also exposed and closed a background Hecate update-banner dispatch error and the startup activation/download race in the host integration.
