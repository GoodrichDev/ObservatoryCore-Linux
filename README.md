# Observatory Core for native Linux

Rust desktop application for reading and monitoring Elite Dangerous journals. The native application runs directly on Linux using X11 or Wayland; Elite Dangerous itself may run through Steam/Proton or Wine.

The workspace contains the native desktop, lossless journal domain model, journal ingestion, Explorer, Botanist and Communicator, and native ports of the installed BioInsights, Evaluator, Hecate and StellarOverlay plugins. It includes Lua extensions, journal replay, SQLite storage, CSV export, overlays and speech integration. Original Explorer custom criteria run through a bundled self-contained .NET 8 process with real NLua and Observatory Framework objects; the desktop and main workers remain native Rust.

The original C# projects remain as the Core source reference. The main window preserves the original horizontal plugin tabs, gray theme, plugin list and bottom controls. BioInsights uses its original grouped main view and seven-button toolbar. Evaluator retains its original custom orange view and graphics. Hecate and StellarOverlay keep their original background roles. [Installed plugin versions and implementation evidence](native/INSTALLED_PLUGINS.md) and the [parity ledger](native/PARITY.md) describe the supported contracts and platform acceptance boundaries. Other CLR DLLs and arbitrary WinForms controls require their own native port.

The [native plugin API](native/PLUGIN_API.md) and [sample plugin](native/plugins/sample.lua) provide the supported extension contract and a migration starting point.

## Build and run

Source builds need a current stable Rust toolchain, a C compiler, Linux development libraries and a .NET 8 or newer SDK for the Explorer criteria runtime. Release packages include the complete runtime; users do not need to install an SDK or .NET separately. On Ubuntu 22.04 or later:

```sh
sudo apt-get update
sudo apt-get install build-essential pkg-config cmake python3 libx11-dev libxcursor-dev libxi-dev libxrandr-dev libxkbcommon-dev libxkbcommon-x11-0 libwayland-dev libgl1-mesa-dev libegl1-mesa-dev
sudo apt-get install libgtk-3-0 libwebkit2gtk-4.1-0 libpango-1.0-0 libpangocairo-1.0-0 libcairo2 librsvg2-2 fontconfig fonts-noto-color-emoji
sudo apt-get install espeak-ng pulseaudio-utils libstdc++6 zlib1g libicu70 libssl3 libgssapi-krb5-2 ca-certificates tzdata
cargo build --release --locked -p observatory-desktop
mkdir -p target/release/observatory-clr-criteria
bash native/observatory-clr-criteria/build.sh "$PWD/target/release/observatory-clr-criteria"
./target/release/observatory-core
```

The default `make` target builds the native release. `make run`, `make test`, `make check`, `make package` and `make install PREFIX="$HOME/.local"` provide the corresponding workflows. Packaging builds the self-contained CLR criteria runtime automatically. For development, the runtime is discovered beside the Core executable as `observatory-clr-criteria/observatory-clr-criteria`, or through `OBSERVATORY_CLR_CRITERIA_HOST`. `DOTNET_COMMAND` selects an SDK executable outside `PATH`. The old .NET/Wine desktop build is retained in `Makefile.wine` and is available only through explicit `make legacy` or `make legacy-winesetup` targets.

The main window provides monitoring, replay, worker results, a biological sampler panel, communications, settings and export. Journals are discovered in standard Steam/Proton, Wine and Saved Games locations. Select a custom directory in settings or pass it explicitly:

```sh
./target/release/observatory-core --discover
./target/release/observatory-core --journal-dir "/path/to/Frontier Developments/Elite Dangerous"
./target/release/observatory-core --view Settings --detach Botanist
```

`ELITE_JOURNAL_DIR` also sets the journal path. Settings and application data use XDG locations. `--config /path/settings.json` selects an alternate settings file. Run `--help` for available options.

Replay without a window and export results:

```sh
./target/release/observatory-core --headless --journal-dir /path/to/journals --export /path/to/findings.csv
```

StellarOverlay provides transparent, click-through native notification and radar windows with the original colors, placement preferences and lifetimes. Disable its notification handling to use the desktop session's freedesktop notification service. X11 supports foreground detection and monitor selection; Wayland placement, stacking and global focus are controlled by the compositor. Speech uses `espeak-ng`; custom chimes use `paplay`. Font selection uses `fontconfig` when available.

## Import existing settings

Import the original portable/Proton `Observatory.config` JSON file or installed application's `user.config` XML file into native settings:

```sh
./target/release/observatory-core --import-settings /path/to/Observatory.config
./target/release/observatory-core --config /path/to/native-settings.json --import-settings /path/to/user.config
```

The command merges imported preferences with the selected native settings file, saves it atomically and exits. It does not modify the source. Core monitoring/notification preferences, built-in plugin settings and plugin enable flags migrate; GUID-keyed settings take precedence over older name-keyed entries. All original preferences remain available under `legacy_settings`, including WinForms-specific values without a native equivalent.

The importer translates identifiable Wine paths. Unresolvable Windows journal paths require selecting their Linux location; unavailable custom criteria remain retained and are disabled until their Lua file is available. Windows voice names remain retained rather than being passed to eSpeak. SAPI speech speed is translated approximately to words per minute, and the command prints notes about these adjustments.

For a bounded live-monitor check, use `--headless --watch-seconds 30 --journal-dir /path/to/journals`. [Diagnostics](native/DIAGNOSTICS.md) describes isolated data directories and replay/monitor checks.

## Install and package

Build a portable tarball and Debian package on Linux:

```sh
sudo apt-get install dpkg-dev file python3
sh native/scripts/package.sh
```

The output directory is `dist/`, with checksums in `SHA256SUMS`. To package an already built release binary, pass `--binary /absolute/path/observatory-core`. The default package build publishes the complete self-contained CLR runtime to `${CARGO_TARGET_DIR:-target}/clr-criteria`. Pass `--clr-runtime /absolute/published/runtime` or set `OBSERVATORY_CLR_RUNTIME` to reuse an existing publish without an SDK. Partial or framework-dependent runtime directories are rejected. `CARGO_TARGET_DIR` is honored when the script builds the binary. `--tar-only` produces just the portable archive without requiring Debian packaging tools; `make install` uses this mode.

[Native Linux CI](https://github.com/GoodrichDev/ObservatoryCore-Linux/actions/workflows/native-linux.yml) builds and packages Linux x86_64 on every push and pull request, and can also be started with **Run workflow**. Open a successful run and download the `observatory-core-linux-x86_64` artifact for the `.deb`, portable `.tar.gz`, `SHA256SUMS` and package verification report. Packages include the self-contained CLR criteria runtime. Artifacts are retained for 30 days; diagnostic captures are available separately for 14 days. The build runs formatting, Clippy, workspace tests, native plugin UI checks, isolated audio playback and fullscreen focus checks before validating and uploading the packages.

Install the Debian package with the system package manager:

```sh
sudo apt install ./dist/observatory-core_*_amd64.deb
```

Or extract the portable archive and install it into your user prefix:

```sh
tar -xzf dist/observatory-core-*-x86_64-linux.tar.gz
cd observatory-core-*-x86_64-linux
sh install.sh
```

The portable installer defaults to `$HOME/.local`. It accepts `--prefix /absolute/path` and `--destdir /staging/root`, or `PREFIX` and `DESTDIR`. User-prefix launchers use the installed binary's absolute path. The CLR runtime installs under `lib/observatory-core/clr` and is discovered automatically relative to the Core executable, including prefixes containing spaces. Its [source provenance and runtime contract](native/observatory-clr-criteria/README.md) ship with both packages.

The portable binary needs the standard X11/Wayland/OpenGL libraries; X11 also requires `libxkbcommon-x11-0` and `libxcb1` on Ubuntu. Install the GTK/WebKit and Pango/Cairo/librsvg runtime packages listed above for native dialogs, original CSS text metrics and theme-colored SVGs. An emoji font supplies the original text symbols; `fonts-noto-color-emoji` is the open Linux fallback. Existing local Windows fonts may also be used through Fontconfig. Headless journal replay does not load these GUI libraries. The Debian package declares speech, playback and [.NET Linux system dependencies](https://learn.microsoft.com/en-us/dotnet/core/install/linux-ubuntu-decision#dependencies); portable installations need those libraries from the distribution as well. The self-contained publish includes .NET itself.

## Verification

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

The Linux CI workflow additionally builds the release ELF and self-contained CLR runtime, opens the actual desktop window under Xvfb, packages them, and validates a staged installation. The installed headless smoke check removes the developer runtime override and verifies real `System.Math` and typed `Scan.GetType().Name` calls from an install prefix containing spaces. Synthetic fixtures and the coverage ledger are in [`native/fixtures`](native/fixtures) and [`native/PARITY.md`](native/PARITY.md). [Architecture and runtime invariants](native/ARCHITECTURE.md) describe ownership, persistence, replay and extension boundaries.

[Release validation results](native/VALIDATION.md) record the automated tests, installed-plugin formula comparisons, X11/Wayland rendering and local journal checks.

## Original project

Elite Observatory was created by Jonathan Miller (Vithigar); the retained Core source and original native code use the [MIT license](LICENSE). BioInsights, Evaluator, Hecate and StellarOverlay were created by MattG (EDJP). Their extracted catalogues, themes and graphics retain their original authorship and licensing; they are identified in each port's provenance document. Original Core documentation is at [observatory.xjph.net](https://observatory.xjph.net/).
