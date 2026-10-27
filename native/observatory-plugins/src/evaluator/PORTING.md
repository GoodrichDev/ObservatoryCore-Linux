Evaluator native port provenance
================================

Reference: the user's installed `Evaluator.1.3.444.637.eop`, by Matt G,
SHA-256 `4DE135AB48B8FBBC9EA350BFD1C8A5D40D52D94615CEBF51C53A0F31A04A21E2`.
The port runs Rust directly; neither the Windows assembly nor its initialization
code runs in the Linux application.

`pricing.rs` preserves the installed `Body.cs` K values, operation order,
first-discovery/first-map bonuses, Odyssey additions, efficiency bonus,
fleet-carrier modifier, integer division for honk bonuses and `Math.Round`
ties-to-even behavior. Scan fields that the original framework deserializes as
`float` are narrowed to `f32` before calculation. `model.rs` implements the body
and system states, honk/main-star contribution, fully scanned/mapped bonuses,
distance limits, manual overrides and BioInsights flags. `routing.rs` propagates
the journal's orbital elements with the original Orrery coordinate conventions,
including barycentre parents, then computes a deterministic open nearest-neighbor
route with 2-opt improvement. This replaces the original randomized TSP solver;
equal-cost route ordering can differ.

The installed theme JSON, CSS, JavaScript and symbol resources are retained with
their original attribution. `native/scripts/import_evaluator_theme.py` extracts
the embedded SVG/PNG symbols and renders PNGs using librsvg/Cairo for the native
desktop view. SVG inputs are retained for inspection. They are build-time assets;
the native renderer needs no browser or SVG runtime. Both Default and Elite Orange
themes retain the original palette, symbols, 24px rows, spacing, command buttons,
system headers and body columns. Compact windows scroll horizontally to preserve
the original no-wrap header content.

The optional Hecate interface supports the original main/unsold/filter/toggle
routes over HTTP and WebSocket and uses the same model as the desktop. Unsold
requests wait for the Hecate response before rendering. Native messages retain
the installed `EvaluatorAdd`, `EvaluatorSysAdd`, `EvaluatorDelete`,
`EvaluatorGetTotal`, `EvaluatorGetData` and BioInsights payload keys. Persistent
DSS notifications use zero TTL, original title and proportional position, hide
when leaving DSS/hyperspace/shutdown, and are suppressed during journal batches.

Verification
------------

`native/scripts/check_evaluator_prices.ps1` compiles just the decompiled body,
system and orbital classes against minimal framework stubs. It never loads or
starts the plugin. It captures 49 type constants, 4,480 planet prices, 60 star
prices, 320 body states, 72 mapping-criteria cases, 5 system-bonus states and 15
original Orrery positions into `native/fixtures/evaluator-prices.json`. Rust
tests compare these captured results directly, in addition to journal, vault,
commander-isolation, restart/preread, web-correlation and overlay lifecycle tests.

Two ILSpy C# reconstruction artifacts are normalized only in the temporary
oracle build: `GetK`'s synthesized initlocals declarations are hoisted so a goto
cannot clear its decoded type name; the first `UpdatePositions` enumerator gets
a distinct lexical variable name to make legal C#. Formula and coordinate
operations are unchanged. Reference readable `Body.cs` SHA-256:
`419973CDA88AD16F2005510ADA9C64D6F8CB2B7FE3AE196A0165CA4A07B415A9`.

The plugin's Windows binary auto-updater is handled by the native application's
packaging/update mechanism. Downloading a Windows EOP cannot update native Rust
worker code.
