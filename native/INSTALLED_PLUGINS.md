# Installed EDJP plugin ports

These are native Rust workers integrated with the same journal pipeline, native
desktop, local message bus and bounded HTTP service as the bundled workers.
The application does not start a CLR, run the installed assemblies, or use Wine.
The Windows installation is unchanged.

| Plugin | Installed reference version | Native behavior |
| --- | --- | --- |
| BioInsights | 1.3.444.1052 | Original grouped main view/toolbar/symbols, Codex catalogue, environmental/region/variant predictions, FSS/DSS signals, observations, first footfall values, three-sample distances, unsold ledger, Codex Companion and persistent marked variants |
| Evaluator | 1.3.444.637 | Original pricing and bonuses, mapping/visit decisions, orbital positions and routes, commander ledgers, original custom themes/icons, unsold view and DSS notification |
| Hecate - Data Vault | 1.3.444.761 | Four schema-compatible SQLite/Zstd databases, system visits, exploration/biology ledgers, daily Spansh/Canonn caches, Spansh journal reconstruction and optional HTTP/WebSocket bridge |
| StellarOverlay | 1.3.228.387 | Native transparent/click-through notifications, queue/update/cancel/lifetimes, plugin colors and bitmap/structured alerts, heading-relative surface radar and sample collection areas |

The installed package named **Evaluator** is the plugin requested as
“Elvaulator.” Hecate and StellarOverlay have no result tab, matching their
original UI roles. All four appear in Core's plugin list and have settings.
The original GUIDs are recognized by `--import-settings` for both portable JSON
and installed XML configurations.

## Source evidence

- [BioInsights catalogue, rules and resources](observatory-plugins/src/bioinsights/PROVENANCE.md)
- [Evaluator pricing/model oracles and original assets](observatory-plugins/src/evaluator/PORTING.md)
- [Hecate schemas, message contracts and bridge tests](observatory-plugins/src/hecate/PORTING.md)
- [StellarOverlay queue, radar and compositor behavior](observatory-plugins/src/stellar_overlay/PORTING.md)

Reference assemblies were inspected offline, including decoded constant tables
and embedded resources. Their startup code was not executed. The oracle tests
execute extracted calculation-only C# code with journal/model stubs; no plugin
host or external service is started. Private player data is absent from fixtures.

## Verification

`cargo test --workspace --locked` exercises individual workers and the integrated
runtime, including cross-plugin biology/exploration ledgers, reopening the vault,
repeated full-history rebuilds, disabled plugins, settings rollback, HTTP and
WebSocket exchanges, and original formula fixtures.

```sh
dbus-run-session -- xvfb-run -a python3 native/scripts/plugin_smoke_test.py \
  target/release/observatory-core --output dist/native-plugin-smoke
```

The smoke script uses disposable synthetic journals and XDG directories. It
checks the real SQLite ledger contents and captures the Core, BioInsights,
Evaluator, notification and radar windows. Add `--wayland` in an existing
Wayland session to require that backend without X11 fallback.

Native updates are delivered with the Rust application; original Windows `.eop`
automatic replacement is not a native executable update mechanism. Platform
placement/focus restrictions, live remote data variants and a controlled
fullscreen Linux/Proton sampling session remain separate acceptance checks.
