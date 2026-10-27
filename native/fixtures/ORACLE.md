# Original Explorer source oracle

`oracle-cases.jsonl` contains 59 independent synthetic systems and 120 journal events. `oracle-expected.json` contains the 40 findings emitted by the original C# `DefaultCriteria.CheckInterest`: body, description (`title`), detail, and their original order. `oracle-session-expected.json` captures another 17 scan findings from the existing `parity-journal.jsonl` session. The two Rust integration tests read these files and compare all 57 findings exactly, including case and order.

The capture was generated on 2026-10-25 with PowerShell 7.6.5's bundled Roslyn on .NET 10.0.11. The oracle compiles the repository's unchanged `DefaultCriteria.cs`, `ExplorerSettings.cs`, original journal/parameter classes and material converter in memory. The original C# projects are never built or changed. No SDK, NuGet packages, Wine, compiled captures, user settings, private paths or real commander data are included.

The helper supplies the projects' implicit using directives and extracts the original `ParentType` enum without its unrelated JSON-converter annotations. The original `Scan` parser constructs that enum itself. Numeric input types and rule implementation come from the original source. Formatting uses `en-US`, matching the captured comma grouping and decimal separator.

The LF-normalized SHA256 of the complete 16-file original source set is:

```
ED8F437ACFA5563E959C0CAD836553DDDD5C9DC8D785E3E962586C9EE4CF3A65
```

The regeneration script lists that source set. To reproduce its digest, sort those relative paths, append each forward-slash path, a newline, its UTF-8 source with CRLF converted to LF, and another newline, then hash the concatenation. The raw Windows source hashes for the main rule/type inputs at capture time were:

| Original source | SHA256 |
| --- | --- |
| `ObservatoryExplorer/DefaultCriteria.cs` | `885824E8EDD2DC7434E104FA6A0B70203115D144451DF39A9424CAF020836BD0` |
| `ObservatoryFramework/Files/Journal/Exploration/Scan.cs` | `9ADA6723DE4F102FF87182B196FC4506F30BE890283E3AE1CBA5137E20E555FD` |
| `ObservatoryFramework/Files/Journal/Exploration/ScanBaryCentre.cs` | `503322EB9B16D69005F9B0852E7E93758740B84DAF8F79F72D8C3E81B2D35B41` |
| `ObservatoryFramework/Files/ParameterTypes/Ring.cs` | `BE16CBC4AB321750A0AE71184C52CA7651426A7751BE262BFC4709BFF85BA094` |

Raw hashes can change with checkout line endings; the complete normalized source digest is portable.

All 22 scan criteria have positive coverage across these two source captures, including both binary outcomes, nested moons, biological signal counts, green/gold material sets, and high-value discovery states. Boundary cases exercise strict comparisons for gravity, radius, rotation/orbit periods, eccentricity, close orbits, shepherd moons, ring width/proximity, secondary star distance and binary separation. The original framework stores most physical fields as `float`; the fixture catches differences caused by using `double` for those computations. Examples include radius `18000000.1`, period `28799.9999`, displayed period `12060.0001`, and binary ratio `1000000f / 2500000f > 0.4`.

Codex is a separate Explorer handler and is covered by the plugin/runtime fixtures rather than `DefaultCriteria`. Custom Lua, plugin lifecycle, desktop rendering and third-party compatibility are also outside this particular oracle.

An additional comparison against `parity-journal.jsonl` produced 17 exact matching scan findings. Its primary-star `Parents: []` also exposed an original `ArgumentOutOfRangeException`: the source reads `Parent?[0]` without checking for an empty list. The native worker safely handles that input. The 59-case oracle avoids that original crash and generated no source exceptions.

Run the durable Rust acceptance check on Linux:

```sh
cargo test --locked -p observatory-plugins --test source_parity
```

Regenerate the C# results with PowerShell 7.3 or newer, which includes Roslyn and the required runtime reference assemblies:

```powershell
pwsh -NoProfile -File native/scripts/check_source_parity.ps1
pwsh -NoProfile -File native/scripts/check_source_parity.ps1 -RepositoryRoot /path/to/checkout
```

Verify the existing session capture, explicitly reporting and skipping its known original empty-parent crash:

```powershell
pwsh -NoProfile -File native/scripts/check_source_parity.ps1 -Cases native/fixtures/parity-journal.jsonl -Expected native/fixtures/oracle-session-expected.json -AllowSourceErrors
```

The default command verifies the checked-in expected results without changing them. After reviewing an intentional source change, replace the expected capture explicitly and rerun the Rust check:

```powershell
pwsh -NoProfile -File native/scripts/check_source_parity.ps1 -UpdateExpected
```

Only a uniquely named directory in the OS temporary directory is created for compiler helpers; it is checked and removed after each run. Input and expected file paths can be overridden with `-Cases` and `-Expected`.
