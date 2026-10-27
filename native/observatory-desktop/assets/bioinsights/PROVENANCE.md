# BioInsights main-view resources and reference cases

These resources come from the user's installed BioInsights 1.3.444.1052, by
MattG (EDJP), rather than from the previous native generic output grid.

`theme.json`, `theme-orange.json`, `style.css`, `buttons.css` and `symbols.css`
were extracted from the assembly's original embedded resources. The JSON files
have only their UTF-8 BOM removed and formatting normalized. `manifest.json`
records the original resource byte hashes. Original SVG content is retained next
to the PNGs produced through librsvg/Cairo. PNG content already embedded in the
theme/button CSS is copied without pixel changes. The original legacy WinForms
button PNGs are also retained as `*_btn_256.png`; the renderer uses the current
webview button assets identified as `Button-*` in `symbols.rs`.

## Source references

The reference C# project was decompiled and its constant strings decoded offline
without starting the installed assembly. The resulting source is inspected at:

- `UIHandler.UIButtons` (lines 366–410): the seven main toolbar keys/tooltips.
- `UIHandler.AddHTMLTableHeaders` (lines 7196–7359): commander banner, the two
  header rows, original labels/column spans, optional extra body fields and
  optional unsold banner.
- `UIHandler.GenerateHTMLBodyHeader` (lines 7369–7815): body/group heading,
  flags, planet label, signal count, radiant star and base-value range.
- `UIHandler.GenerateHTMLBodyDetails` (lines 8320–9084): genus/species rows,
  source/status symbols, merged variants cell, likelihood/mark/Codex symbols,
  compact base value and sampling distance.
- `UIHandler.FormatMillions` / `BioAnalysedString` (lines 9087–9158): compact
  millions and sample-status symbols.
- `UIHandler.AddHTMLTableFooter` (lines 7128–7162): blank orange banner when
  headers are enabled, followed by the dark footer row.
- `BaseUIHandler.GetSymbol` and `AddButtonsAndOpenContentDiv`: original HTML
  symbol span and toolbar generation. CSS/table layout uses the browser's auto
  table layout; the source does not prescribe fixed main-table column widths.

## Independent reference cases

The following calculation-only C# captures do not use the Rust renderer or Rust
view projection. They do not execute the original plugin assembly or contact a
network endpoint.

- `header-oracle.cs` / `.json`: eight combinations of ShowHeaders,
  ShowBodyFields and ShowTotal from the copied source header method. A
  synthesized ILSpy default-local declaration is hoisted to method entry to
  preserve the original IL initlocals behavior. Its three simple framework
  wrappers are copied unchanged.
- `body-oracle.cs` / `.json`: twelve combinations of current-body state, signal
  count and body flags from the copied source body-header method. Journal/body
  metadata are data-only stubs; original formatting, field order and column
  spans execute directly.
- `row-oracle.cs` / `.json`: seventy-five combinations of VariantType, BioSeen
  and BioAnalysed from the copied species-row and formatting methods. The input
  record properties are data-only stubs. Symbols are replaced by named tokens
  so the cases record the source symbol ordering independently of font or image
  rendering. URL enum IDs are intentionally dummy fixture values.

`main-layout-contract.json` extracts the default header labels/spans from those
source-generated cases and records the original CSS metrics/button order.
`header-reference.html` contains the captured source header output with the
original CSS and toolbar, for inspecting the source header structure; it is not
a complete journal session or proof of complete visual parity.

The original authorship and licensing of the embedded resources remain intact.
The native Rust renderer must still be checked visually against the supplied
original application screenshot; passing these fixtures alone is insufficient.


## In-place pages and source image resources

Main, Unsold Data, About, and Codex Companion navigate inside the same custom
output control, following `UIHandler.HandleURL` / `BaseUIHandler.HandleURL`.
Only explicit Popout delegates a native output viewport; Settings delegates the
original Core plugin settings action. `FZoomSelector` provides the small
cursor-anchored 505x54 selector: 50 through 600 percent, five-percent changes,
and Reset to 100. Ctrl+Popout bypasses saved geometry (`restore=false`).

`regionmap.png` and `species.Unknown.jpg` are copied byte-for-byte from the
installed assembly resources; their hashes are in `manifest.json`.
`region-coordinates.json` copies all 42 integer seed coordinates from
`UIHandler`'s constructor. The native region map runs the source flood-fill
algorithm with its original possible RGB(136,78,8), seen RGB(164,242,203),
400/720 coordinate conversion, and two legend swatches. It retains the original
400x400 map, 135x135 genus bioscans, and 480x270 contributed species images.
The unchanged original image manifest and bioscan URLs are used; download,
disk cache, image decode, and map generation run in bounded background workers.

Codex Companion uses the original genus/species navigation columns and species
detail blocks. Its Seen links update the existing region map; Mark switches
send the persistent variant command; BioForge opens the source external URL.
The old native auxiliary windows, flat catalogue grid, invented filters, and
additional Refresh button are removed. Unsold uses the original six columns,
body-group row colors, full species names, First Logged switches, and Total.
The 663 requirement blocks for all 117 species are independently captured in
`native/fixtures/bioinsights-companion-oracle.cs` and `.json`.
