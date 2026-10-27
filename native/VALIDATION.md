# Native Linux validation

Validated on 2026-10-26 with Ubuntu 22.04 under WSL2, x86-64 and rustc 1.99.1 (58cf8f9c3 2026-10-13). Core and all four requested workers execute as native Rust Linux ELF code. Original Explorer custom criteria run in a separate bundled Linux .NET 8/NLua process; the application is not a Wine or managed-desktop wrapper. Most source oracles use inert platform stubs. The CLR oracle additionally executes the installed Framework DLL's DTO methods with Windows ReadyToRun disabled, without starting the original application.

## Build and automated checks

The current source passed **396 tests**, zero failures, with **12 explicit/manual tests ignored**, using the real CLR runtime override; all-target workspace clippy also passed. `dist/final-validation.json` records the executable and complete commands/results. CLR coverage includes six integration tests and three Explorer state tests, described in [CLR script parity](CLR_SCRIPT_PARITY.md). The hashes below identify the current native executables; graphical receipts retain their own explicitly tested hashes. Ignored graphical, environment-dependent and profiling checks are separate from ordinary suite counts and are identified by their actual receipts.

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked -q`
- `git diff --check`
- Native release Core and native-worker builds with `--release --locked`.

Current Core SHA256: `9d8af27a8dfb9ed3c6719a89e800d9174722f31555ecc7e510bd8484302c2970`.
Current native worker SHA256: `4d57a957305acbcef107ffae15df63d8ae845c58ccb177e913c2aac0b2fbab0b`.
The complete CLR runtime contains 193 files, about 75 MB/72 MiB unpacked; its
content-manifest SHA256 is `cd8691af4e235d1944b0019851d77459c0bc56f682351f42067fee611f9ce755`.

Earlier graphical Core SHA256: `985a16215aec6520120a8d8b97eaa2dc79bf7c7ced9a3ac00590e6b5594ed343`.
Earlier native worker SHA256: `e7ca372e66c5bbce2ae2b0a35ca298527031556d38b0c3f714728b22d0bf6f27`.

These identities refer to immutable local artifacts; individual graphical and updater receipts carry their own tested hashes. A newer binary is not assumed to inherit a graphical result without a rerun.

## Independent source behavior comparisons

The finite inventories and provenance are [Core](CORE_PARITY_AUDIT.md), [bundled workers](BUNDLED_WORKER_PARITY.md), [BioInsights](observatory-plugins/src/bioinsights/FEATURE_PARITY.json), [Evaluator](EVALUATOR_PARITY_AUDIT.md), and [notification delivery](CORE_NOTIFICATION_AUDIT.md).

- Explorer: 723 pure criterion cases and 759 whole-worker scenarios. Botanist: 75 complete source event/status/settings sequences. Communicator: 250 source scenarios. Ordinary Explorer Lua: 38 original annotation/compiler programs, six synthetic calls each, and 343 Framework DTO property schemas.
- Real Explorer CLR criteria: all 448 exported Framework types match the installed reference's public API metadata. An independent installed-Framework/NLua oracle matches 27 scenarios and 89 responses through the actual Linux CLR host and Rust Criteria pipeline. Tests cover constructors, reflection, collections, ref/out, actual DTO methods, retained object identity, scan/signal/Parent mutations feeding native history, non-finite floats and idle timer callbacks. Crash/exit handling reports failure once and does not replay script side effects. [Provenance and commands](CLR_SCRIPT_PARITY.md).
- BioInsights: 4,026 complete prediction cases with positive coverage for all 117 species, 241 full source stellar-context cases, the 55-value settings constructor, 64 original bitmap drawing traces, and source notification/radar/visibility combinations. The catalogue alone is not treated as prediction proof.
- Original BioInsights builders: 607 Codex cases covering all 139 routes, complete About markup and eight Unsold cases. Twenty independent WebKit species-layout states cover both themes, three species at three widths and the gallery-credit branch. Native row heights match exactly; spans, nested columns and actual image/gallery/map rectangles differ by less than 1/16 CSS pixel in those cases.
- Evaluator: independently captured prices, type constants, mapping/completion/orbit decisions, complete source event scenarios and original HTML formatter states. Native main-table geometry, About and Unsold are measured against original WebKit CSS; the report records each bounded comparison.
- Core notification routing: 768 original Send/Update/Cancel cases; native popup geometry: 480 source vectors; six byte-identical chime assets and ten original SSML vectors. Queue/Instant audio, deletion/completion/error/cancellation behavior use the production playback seam.

A finite oracle establishes the enumerated inputs and outputs. It is not an exhaustive proof over every journal, arbitrary orbit graph, plugin or remote service response.

## Actual application interactions and responsiveness

`dist/core-parity-ui/clicks.json` records twelve real interaction groups using private Xvfb/XTest/GTK: geometry, repeated source TSV saves, sorting, multirow clipboard/modifiers, generic popout persistence, settings, XLSX, both Core colors, Read All cancellation, Clear, tab drag and restart. `dist/evaluator-parity-ui/receipt.json` records actual inline Main/About/Unsold/Zoom actions and explicit Popout.

[BioInsights graphical verification](BIOINSIGHTS_VISUAL_QA.md) links the compact [immutable-release receipt](fixtures/bioinsights-visual-linux-qa.json), screenshots and reproduction scripts. It covers Orange at 1200×800, 3439×1400 and 3840×2160, Original theme, existing Windows fonts through a private Fontconfig profile, actual genus/species navigation and both renderers' zoom/persistence/focus-loss/disabled controls. Page actions remain inside one application window; only explicit Popout detaches. GTK/WebKit same-page updates preserve the selected gallery, credits and scroll; a separate actual X11 test finds popout owners below reparenting frames.

The dedicated actual-application stress profile uses 200 bodies and 800 predicted species at 4K. Its receipt measures idle CPU and navigation response with a small pixel region and records capture overhead and the 10 ms mouse hold. The values in that final receipt supersede older renderer-only timings and full-screen readback measurements. They describe this machine and software-GL test environment, not every Linux desktop.

The startup freeze was independently reproduced and fixed: two egui Context-lock closures recursively called `viewport_id`. Per-plugin immutable snapshots, independent Bio/Evaluator revisions, cached font/image/layout work and offscreen row culling avoid rebuilding unrelated large views on each event.

## Passive notifications, native updates and migration

[Actual passive-surface evidence](fixtures/passive-notification-linux-qa.json) covers X11 with an aggressive focus policy and supported Wayland layer-shell at scales 1 and 2. A focused **synthetic game surface** retains keyboard/pointer input and fullscreen presentation; a deliberately focus-taking negative control fails as expected. Unsupported compositors report a missing capability and never create a focus-taking toplevel fallback. This is not a claim that an actual Elite Dangerous gameplay session was tested.

[Private fullscreen/audio acceptance](fixtures/private-fullscreen-audio-linux-qa.json) additionally records real production audio-helper and application dispatch through an isolated PulseAudio null sink, six chimes, SSML, queued/instant completion and cancellation/error handling. Private X11/Sway/Weston tests retain synthetic fullscreen-client input and exercise the negative control and unsupported-compositor path. These checks did not access the user's desktop or player files and did not play through physical hardware.

[Actual native updater evidence](fixtures/native-updater-e2e-linux-qa.json) uses a local HTTP server and real compatible Linux ELF artifacts. It covers automatic/manual/disabled/beta policy, hash/ELF/metadata rejection, atomic staging while active worker inodes remain live, all four restart activations, complete builtin/remote projection equivalence and worker SIGKILL isolation. The original Windows EOP payload is never installed as a Linux update. No public vendor Linux feed was found; publishers must provide a compatible manifest.

Eighteen migration tests cover source schemas, FID-scoped Bio marks/seen/sampling, palettes/popouts, idempotent merges, malformed-source rollback and four live-WAL SQLite databases unchanged byte-for-byte. A separate [actual read-only installed-profile import](fixtures/legacy-import-linux-qa.json) verified37 source files plus settings unchanged, six native files and3,311 records imported; a second import added zero records. Bio JSON and every logical table of all four Hecate databases matched. That receipt names its own debug executable hash, and contains no private journal bodies. Audio synthesis was checked silently; audible hardware playback remains an environment check.

## Packaging and acceptance boundaries

`native/scripts/package.sh` packages Core as a portable tarball and Debian package. It includes the complete self-contained .NET 8.0.33/NLua runtime for Explorer criteria (about 72 MiB unpacked), current audits/compact receipts and native dependencies, including audio helpers, Fontconfig, Pango/Cairo/librsvg and an emoji font. The runtime installs under `lib/observatory-core/clr`; no system .NET installation is required. `native/scripts/verify-package.sh` checks checksums, executable identity in both archives, a staged prefix containing spaces, installed-Core automatic CLR discovery, partial-runtime rejection, portable packaging without Debian tools and preservation of prior published artifacts after a deliberately failed package build. Artifact checksums are in `dist/SHA256SUMS`; no real system-wide package installation is implied by staging.

Explorer criteria use real `LoadCLRPackage`, CLR imports and .NET/Framework object methods through the bundled runtime. Windows-only libraries retain normal Linux .NET limitations. Loading a user assembly from criteria does not provide the old Windows EOP/WinForms desktop plugin ABI; other binary plugins and custom Windows controls need native ports. Linux speech engines/device inventories, font rasterization and compositor policies remain platform adaptations.

Controlled actual Elite/Proton fullscreen organic sampling and long-session/suspend acceptance could not run because no Linux/Proton game installation is available. Clean-distribution system installation and physical device/voice listening have not been performed. Private compositor and null-sink audio checks above establish their stated paths without using the user's screen or physical audio. [Compatibility ledger](PARITY.md).
