# BioInsights native port provenance

The source version is EDJP's **BioInsights 1.3.444.1052**, supplied by the user
from their installed Elite Observatory `plugins` directory. The author's plugin
page is <https://edjp.colacube.net/observatory>.

The installed package and assemblies were inspected offline. No original plugin
assembly, Windows UI, CLR hosting layer, or original updater is part of the Rust
runtime. No original DLL is included in this source directory.

| Source | SHA-256 |
| --- | --- |
| `BioInsights.1.3.444.1052.eop` | `d3099a6a4133d09c59243234101c544c249692ba0bbc8e1554ed799c6c6d37f9` |
| `ObservatoryBioInsights.dll` | `50e61de9db4c7cdb3c42c01cec9135bd9841747014c2a95a8fa6bfd23912e2ca` |
| `CodexLib.dll` | `3bdf064f7771440985a52e20c9d6f3a2a06a5c203918eb98a546579f3c2d9455` |

## Derived resources

* `codex.json` is the complete installed `codex_json` resource, decoded from
  its base64/Brotli representation. It contains 21 genera, 117 species and 772
  Codex variant entry IDs, including exact values, sample distances, variant
  colours, regional masks, and all requirement alternatives. Its own metadata
  records `AssemblyVersion: 1.3.444.738` and build timestamp
  `2026-09-22T09:11:12.518587+01:00`; the containing CodexLib assembly reports
  version 1.3.444.739.
* `names.json`, `species-ids.json`, and `region-names.json` transcribe the
  installed enum/name tables, including all 42 galactic region names.
* `regions.bin` losslessly re-encodes the installed 2048 by 2048 RegionMap run
  length table as little endian row run counts followed by `(u16 length, u8
  region)` records. Coordinate offsets, scaling and the 20 light year border
  sampling are implemented in `rules.rs`.
* `spheres.csv` and `species-description.tsv` are installed CodexLib resources,
  containing nebula/planetary-nebula/tree/tuber spheres and species descriptions.
* `style.css`, `theme.json`, and `theme-orange.json` come from the installed
  BioInsights resources. Their colours, font declarations, table spacing and
  CSS rules are retained. The native web handler substitutes the original
  theme variables. Theme selection follows the installed default: Elite Orange.

These resources retain their original authorship. Their presence records the
user-provided material used for this port; it does not assert an upstream open
source licence or grant redistribution rights. The repository's licence for new
Rust code does not relicense these original data, description or theme assets.

## Behavioural evidence

Codex Companion now retains the original detail fields, variant star/material
associations, numeric genus/species routes, possible-region mask and personal
seen-region counts. `companion-requirements.json` contains all **663 requirement
blocks for 117 species**, produced by the original `UIHandler.RequirementAsString`
and `FormatReq` calculation-only methods against this installed catalogue.
`native/fixtures/bioinsights-companion-oracle.cs` preserves the standalone source
capture harness, and its JSON retains the original HTML strings. No plugin DLL
is loaded by this harness. Primitive data stubs replace the original data types;
ILSpy's synthetic local-default initializers are moved to method entry so
dispatcher jumps retain the original IL local values. The plain-text resource
only decodes the original HTML bullets, spacing and line breaks. A regression
test compares all 117 species against those captured original strings.

The Companion title and description use the original `GetLocalGenusSpeciesName`
rule: Odyssey joins genus and species; Horizons uses the species name. The
original `RadiantStarName` table supplies variant causes, including multiple
stars for one colour. Gallery and bioscan URLs retain their original sources.

The native tests verify catalogue completeness, FSS/DSS prediction changes,
strict/inclusive climate boundaries, installed giant-star classification,
spatial lookup, Codex sightings, first footfall values, sampling distances,
unsold sale accounting, batch persistence messages, native state reload,
and correlated Hecate HTTP/WebSocket responses.

`predicate-vectors.json` contains 84 synthetic volcanism vectors evaluated by a
separately compiled extraction of the installed CodexChecker predicate method.
Only constant-decoded method logic and standard .NET string/number wrappers
were compiled for this offline check; original assemblies were not executed.
The vectors cover empty, minor, regular and major volcanism, negated magnitude
rules and elemental composition, including the original `IRON_BASED` to
`metallic` mapping. Native tests compare every vector independently.

All desktop calculations use native Rust state. Hecate owns its existing unsold
ledger; BioInsights sends the original `BioInsightsUnsoldAdd`,
`BioInsightsUnsoldDelete`, `BioInsightsGetData`, and `BioInsightsSetBonus`
commands. User marks and personal Codex entries have a separate versioned native
snapshot. Web, radar and notification output is quiet during journal replay;
ledger mutations and Evaluator analysis-state messages remain enabled. Replay
tests verify that cached Galactic/personal Codex and user marks still produce
the original `SetBioInsightsFlags` contract without presentation output.

The original public Galactic Codex GET endpoint is also supported:
`https://edjp.colacube.net/api/v2/codex/galactic`. The host executes queued
requests asynchronously when monitoring. Zstandard and plain JSON v1.0 responses
are validated with a 16 MiB decompression limit; failed or incompatible responses
retain the last good cache. `GalacticCodexRemote` controls these lookups. The
versioned native snapshot preserves the feed and its upstream `Generated` date.
No commander identity or journal data is included in this GET. New-region and
never-seen flags update existing Evaluator bodies as well as BioInsights views.
Tests use synthetic local responses, including compressed data and updates that
set and clear an existing Evaluator flag. Native Codex Companion covers every
installed species and its 890 distinct species/colour combinations, with personal
and Galactic Codex views for each region.


The prediction differential harness compiles **the entire installed
CodexChecker source** with statically decoded constants and data-only Framework
DTOs. `fixtures/capture-bio-predictions.py` preserves the reproducible harness;
`bioinsights-predictions-oracle.json` records **4,026 synthetic cases**, including
a positive result for every one of the 117 species. Native comparisons include
genus/species, value, distance, variant type, every color/reason/likelihood and
ShowInUI. The original Bio worker's subsequent one-level star-check confidence
change is applied after CheckScan. The source oracle exposed and fixed omitted
binary filtering, hidden variants, canonical labels and confidence handling.

`fixtures/capture-bio-radiant.py` also compiles the original **BSystem** source,
with an inert logger. Its **241 synthetic orbital contexts** compare complete
resulting predictions, selected radiant body/type and every source-star distance.
They cover all five reachable enrichment modes, hints/missing scans, main/giant/
carbon/degenerate classes, moons/deep moons, binaries/nested barycentres and
scanned companions outside immediate parents. Fixes retain source apparent
visual magnitude selection, common-parent orbital arms, parent-planet
inheritance, source fallback selection and provisional hint confidence. No
installed assembly, Windows module initializer, game data or network call is
executed by these harnesses. Coverage is finite, not proof over every possible
journal sequence or orbital graph.

NativeOutput was rechecked against raw IL. UIHandler constructor IL0439-0457
passes `!NativeOutput.StartsWith("Disabled")` to BaseUIHandler.Initialise. The
true branch constructs PluginWebBrowser; false constructs UIBlankView whose
label is `Native Output Disabled`. Exact dispatcher counters are saved in
`fixtures/bioinsights-native-output-oracle.json`. OutputGrid classes are legacy
types, not evidence of an active Core basic-grid mode.

The original dependency explanation/Skip resource is retained as `hecate.html`.
Linux Hecate is bundled, so the explicit acquisition button is named **Enable
Hecate** and sends Core.InstallNativePlugin to the host's real enable operation.
The nag clears only after actual Ready or an explicit Skip, covered by a
regression test. No Windows package installation is reported. The finite
inventory and remaining host/live checks are in `FEATURE_PARITY.json`.
