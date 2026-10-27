# StellarOverlay 1.3.228.387 native port

Reference: installed `StellarOverlay.1.3.228.387.eop`, authored by MattG (EDJP).
The inspected DLL SHA256 is
`2d4fbc84be59060d8a4fdb781a24eaa65af51d43d873090c95f67b5d965d4665`.
Offline IL and decoded constants were used to inspect `OverlaySettings`,
`OverlayWorker`, `NotificationQueue`, `NotificationItem`, `Radar`, and
`BrushHandler`; plugin startup was not executed.

`stellar_overlay.rs` owns the native queue, lifecycle and original message
contracts. `observatory-desktop/src/stellar_ui.rs` renders transparent native
windows through the application's existing X11/Wayland event loop. It does not
host WinForms, CLR, a browser or a Windows overlay process.

## Original contracts preserved

- Worker/notifier role with no result tab, popup override, persistent IDs,
  replacement/cancellation, queue spacing, active limit and full queued lifetime.
- Original ARGB defaults, shape, padding, font selection/scaling, alignment,
  monitor, placement, opacity and 300 ms linear transitions. Open shapes have
  top/bottom borders; closed shapes use rounded borders and fill.
- Custom plugin color registration and response colors; bitmap/image-key
  notifications and structured native BioInsights/Evaluator alerts.
- Radar planet, genus, context, biological/vehicle/waypoint, colony-range,
  sample-count and force-hidden commands, including original payload names.
- Source radar distances: 1 km at 76% radius, 2 km at 88%, 10 km at the outer
  ring; contacts beyond 10 km clamp to the edge until hidden at 20 km. Surface
  positions use great-circle distance and heading-relative bearing.
- Source ring/point/crosshair palettes, sample range scaling and 300 ms movement
  smoothing, including shortest rotation across heading zero and the dateline.

Model and renderer tests exercise defaults, source command fields, TTL/queue
behavior, transitions, context changes, distances and color decoding. The plugin
smoke test captures actual notification and radar viewports with synthetic data.

## Linux presentation

X11 foreground detection and monitor discovery use X11/XRandR directly. Overlay
windows are transparent, always-on-top, undecorated, do not activate themselves,
and pass pointer input through. Fontconfig resolves the chosen family and uses
the system fallback if the original Windows font is absent.

Wayland does not provide a general application API for global foreground focus,
arbitrary positioning or enforced stacking. The application requests the same
native viewport properties; the compositor decides whether to honor them. A
controlled fullscreen Linux/Proton session is required to accept a particular
compositor/game combination. X11 and Wayland window smoke checks do not establish
fullscreen-game presentation on every desktop.

The Rust application distributes native updates. The original Windows `.eop`
installer/updater is not executed. No proprietary Windows font or DLL is bundled.
