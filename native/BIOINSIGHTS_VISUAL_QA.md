# BioInsights graphical verification

The native renderer is checked against the installed plugin's source methods and source CSS. The optional WebKit renderer is a useful independent CSS/layout engine, but its HTML is only an original reference where the original builder has also been captured. An earlier manually reconstructed Codex/Unsold/About document was insufficient evidence and has been replaced.

## Original references

- `fixtures/bioinsights-codex-page-oracle.json`: 607 original Codex builder cases, the original About page and eight original Unsold cases. Production catalogue templates contain the original 139 base/genus/species documents; runtime state replaces explicit seams for marks, seen regions, gallery and map images.
- `fixtures/bioinsights-symbol-css-oracle.json`: original symbol/theme CSS across both bundled themes and custom colors.
- `observatory-desktop/assets/bioinsights/*-oracle.json`: original body/header/variant formatting. Absent-row italic classes are supplied by the original caller and are tested separately; the isolated variant formatter alone cannot prove that class.
- The source species table has independently measured original WebKit geometry for both themes, three species at three widths, plus credit-caption cases. Native row heights match exactly, and table spans, nested columns and production image/gallery/map rectangles differ by less than 1/16 CSS pixel in those twenty cases.

## Corrections made from those references

The renderer now uses the source toolbar order, SVG artwork and colors, 32-pixel buttons with 38-pixel steps, source body margin/content padding, intrinsic table widths, actual font roles and row spacing. Custom SVG colors are rendered with librsvg/Cairo. Absent predictions retain their source italic styling. Species links use their actual text hit area and source hover colors. The Original theme's omitted highlight flag defaults to false, as its constructor specifies.

About uses the original legend and credits order. Unsold has the bold Total row first, a blank row, the original six headings, alternating stripes by body, original dates and per-row bonus switches. Codex base/genus uses separate floated **vertical** tables; the production regression also verifies row alignment and shared top edges. Species details use the source four-column layout, criteria, gallery, Mark-before-Seen variant columns, region map and legend. The species page does not invent a permanent left navigation list.

Native text uses the installed system font selected through Fontconfig and cached Pango/Cairo rendering. The fractional advance helper preserves hinted glyph advances plus fractional kerning, matching the independently measured original description width. No proprietary Windows fonts are bundled. A private QA profile can explicitly include fonts already present in the user's Windows installation.

Color glyphs retain Cairo's actual premultiplied RGB channels; the native painter does not turn their palettes into a tint mask. Actual Segoe and privately extracted Noto Color Emoji regressions verify multiple real palette colors. A clean Linux profile also checks the case where absent Segoe is replaced by a color-only emoji face: ordinary text/digits use the independently measured normal Sans fallback, while actual emoji still use color-font cluster fallback. This avoids numeric emoji advances changing the table. `fixtures/css-font-linux-qa.json` retains the real WebKit canvas widths used by that regression. The [WebKit font implementation](https://github.com/WebKit/WebKit/blob/main/Source/WebCore/platform/graphics/freetype/FontCacheFreeType.cpp) separates requested-family selection from character-cluster fallback; the concrete fallback widths here come from the actual Linux browser probe, rather than an assumption about generic serif/sans defaults.

## Real application checks

`scripts/bioinsights_visual_parity_test.py` runs the release Core with private XDG profiles and synthetic journals on a private Xvfb server. It captures 1200x800, 3439x1400 and 3840x2160 without image normalization. It navigates within the actual application, checks that the plugin keeps one native window, and retains screenshots of the main, About, Unsold, Codex genus and species pages. Optional `--stress-bodies 200 --renderers rust --sizes 3840x2160` checks the actual large projection before measuring warmed rendering.

`scripts/bioinsights_controls_test.py` exercises both native and WebKit controls in the actual application: zoom endpoints 50/600, Reset 100, persisted settings, actual external X11 focus loss and the source `Native Output Disabled` view. Focused GTK descendants remain within the browser owner's focus domain; an actual transition outside that domain dispatches the source blur cleanup without activating any window.

Latency sampling uses a small pixel region, not a full 4K framebuffer readback. The receipt records capture overhead and the ten-millisecond mouse hold; reported times are observed upper bounds including those costs. Idle CPU includes the application and its children. Forced software rendering and WebKit's GTK pump are identified in the receipt, so their cost is not confused with ordinary native GPU operation.

The final immutable release receipt is `fixtures/bioinsights-visual-linux-qa.json` (Core SHA256 `985a16215aec6520120a8d8b97eaa2dc79bf7c7ced9a3ac00590e6b5594ed343`). All three Orange-theme sizes have equal native/source toolbar centers, main table bounds and row positions. Original-theme 1200/3439 captures and a private profile using the already installed Windows Segoe/Verdana fonts also pass those bounds checks. All seven toolbar images have mean absolute per-channel differences below 3/255 in the actual unhovered captures; the original Codex PNG is checked separately from the SVG rules.

The dedicated warmed 4K survey contains **200 bodies, 800 predicted species rows and 607 journal events**. The native release used **0.33% of one CPU core** while idle, with **42.8 ms median observed navigation response**, including the 10 ms input hold and 0.41 ms average pixel-region capture overhead. All eight navigation actions kept one native application window. These are actual application measurements on private Xvfb/software GL, rather than a renderer-only microbenchmark. Recreate the compact receipt with `scripts/summarize_bioinsights_visual_qa.py` after running the named graphical/control profiles.

## Limits of the evidence

Source builder and geometry tests establish the enumerated behavior, rather than universal pixel identity across DirectWrite, FreeType and every installed font set. Real captures retain small glyph rasterization differences. Synthetic surveys do not establish correctness for every historical journal or remote service response. Overlay non-activation has separate real X11, Sway scale-1/2 and missing-layer-shell acceptance in `fixtures/passive-notification-linux-qa.json`; ordinary viewport hints alone are not used as proof of game focus safety.
