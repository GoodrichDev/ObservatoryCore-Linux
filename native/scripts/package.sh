#!/bin/sh
# Build a native Linux ELF and produce a portable tarball plus a Debian package.
set -eu
script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo=$(CDPATH= cd -- "$script_dir/../.." && pwd)
out=$repo/dist
binary=
clr_runtime=${OBSERVATORY_CLR_RUNTIME:-}
version=
tar_only=false
while [ "$#" -gt 0 ]; do
    case "$1" in
        --binary) binary=$2; shift 2 ;;
        --clr-runtime) clr_runtime=$2; shift 2 ;;
        --out) out=$2; shift 2 ;;
        --version) version=$2; shift 2 ;;
        --tar-only) tar_only=true; shift ;;
        --help) printf '%s\n' 'Usage: sh native/scripts/package.sh [--binary /path/to/ELF] [--clr-runtime /published/runtime] [--out /directory] [--version VERSION] [--tar-only]'; exit 0 ;;
        *) printf 'Unknown option: %s\n' "$1" >&2; exit 2 ;;
    esac
done
case "$(uname -s)" in Linux) ;; *) printf '%s\n' 'Packaging must run on Linux.' >&2; exit 2 ;; esac
cd "$repo"
if [ -z "$version" ]; then
    version=$(sed -n '/\[workspace.package\]/,/^\[/s/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)
fi
case "$version" in ''|*[!0-9A-Za-z.+:~_-]*) printf '%s\n' 'Invalid or missing version.' >&2; exit 2 ;; esac
case "$(uname -m)" in x86_64) arch=amd64; triple=x86_64-linux ;; aarch64) arch=arm64; triple=aarch64-linux ;; *) printf '%s\n' 'Unsupported package architecture.' >&2; exit 2 ;; esac
if [ -z "$binary" ]; then
    cargo build --release --locked -p observatory-desktop
    target=${CARGO_TARGET_DIR:-"$repo/target"}
    binary=$target/release/observatory-core
fi
test -f "$binary"
binary_type=$(file -b "$binary")
if ! printf '%s' "$binary_type" | grep -q 'ELF'; then
    printf '%s\n' 'The package binary must be a native Linux ELF.' >&2; exit 2
fi
case "$arch:$binary_type" in
    amd64:*x86-64*|arm64:*aarch64*) ;;
    *) printf '%s\n' 'Binary architecture does not match the packaging host.' >&2; exit 2 ;;
esac
case "$arch" in amd64) clr_rid=linux-x64 ;; arm64) clr_rid=linux-arm64 ;; esac
if [ -z "$clr_runtime" ]; then
    clr_runtime=${CARGO_TARGET_DIR:-"$repo/target"}/clr-criteria
    mkdir -p "$clr_runtime"
    clr_runtime=$(CDPATH= cd -- "$clr_runtime" && pwd)
    OBSERVATORY_CLR_RUNTIME_ID=$clr_rid bash "$repo/native/observatory-clr-criteria/build.sh" "$clr_runtime"
fi
clr_runtime=$(CDPATH= cd -- "$clr_runtime" && pwd)
# A framework-dependent apphost or a partial publish is not a release runtime.
for required in observatory-clr-criteria observatory-clr-criteria.dll observatory-clr-criteria.deps.json observatory-clr-criteria.runtimeconfig.json ObservatoryFramework.dll NLua.dll KeraLua.dll liblua54.so libcoreclr.so libhostfxr.so libhostpolicy.so System.Private.CoreLib.dll createdump source-provenance.json; do
    test -s "$clr_runtime/$required" || { printf 'Incomplete CLR runtime: missing %s\n' "$required" >&2; exit 2; }
done
test -x "$clr_runtime/observatory-clr-criteria" || { printf '%s\n' 'CLR runtime apphost must be executable.' >&2; exit 2; }
case "$arch:$(file -b "$clr_runtime/observatory-clr-criteria")" in
    amd64:*ELF*x86-64*|arm64:*ELF*aarch64*) ;;
    *) printf '%s\n' 'CLR runtime architecture does not match the package.' >&2; exit 2 ;;
esac
python3 - "$clr_runtime" "$clr_rid" <<'PY'
import json
from pathlib import Path
import sys
root, rid = Path(sys.argv[1]), sys.argv[2]
deps = json.loads((root / 'observatory-clr-criteria.deps.json').read_text())
target = deps['runtimeTarget']['name']
if not target.endswith('/' + rid):
    raise SystemExit('CLR dependencies do not target ' + rid)
options = json.loads((root / 'observatory-clr-criteria.runtimeconfig.json').read_text())['runtimeOptions']
if not options.get('includedFrameworks') or options.get('framework') or options.get('frameworks'):
    raise SystemExit('The CLR criteria runtime must be self-contained')
if options.get('tfm') != 'net8.0' or any(not item['version'].startswith('8.') for item in options['includedFrameworks']):
    raise SystemExit('The CLR criteria runtime must use the original .NET 8 API surface')
missing = []
for library in deps['targets'][target].values():
    for kind in ['runtime', 'native']:
        for asset in library.get(kind, {}):
            if not (root / Path(asset).name).is_file():
                missing.append(asset)
if missing:
    raise SystemExit('Incomplete CLR publish assets: ' + ', '.join(missing))
json.loads((root / 'source-provenance.json').read_text())
PY
"$clr_runtime/observatory-clr-criteria" --version
if [ "$tar_only" = false ]; then
    command -v dpkg-deb >/dev/null 2>&1 || { printf '%s\n' 'Install dpkg-dev to produce the Debian package, or use --tar-only.' >&2; exit 2; }
fi
mkdir -p "$out"
out=$(CDPATH= cd -- "$out" && pwd)
stage=$(mktemp -d "${TMPDIR:-/tmp}/observatory-package.XXXXXXXX")
# The temporary root is created above, never computed from user paths.
trap 'rm -rf -- "$stage"' EXIT HUP INT TERM
name=observatory-core-$version-$triple
bundle=$stage/$name
mkdir -p "$bundle/bin" "$bundle/share/applications" "$bundle/share/icons/hicolor/128x128/apps" "$bundle/share/metainfo" "$bundle/share/doc/observatory-core/plugins"
install -m 755 "$binary" "$bundle/bin/observatory-core"
install -d "$bundle/lib/observatory-core/clr"
cp -R -p "$clr_runtime/." "$bundle/lib/observatory-core/clr/"
chmod 755 "$bundle/lib/observatory-core/clr/observatory-clr-criteria" "$bundle/lib/observatory-core/clr/createdump"
install -m 755 "$script_dir/install.sh" "$bundle/install.sh"
install -m 644 "$repo/native/packaging/observatory-core.desktop" "$bundle/share/applications/observatory-core.desktop"
install -m 644 "$repo/native/packaging/observatory-core.png" "$bundle/share/icons/hicolor/128x128/apps/observatory-core.png"
install -m 644 "$repo/native/packaging/net.xjph.observatory.Core.metainfo.xml" "$bundle/share/metainfo/net.xjph.observatory.Core.metainfo.xml"
install -m 644 "$repo/LICENSE" "$bundle/share/doc/observatory-core/LICENSE"
install -m 644 "$repo/README.md" "$bundle/share/doc/observatory-core/README.md"
install -m 644 "$repo/native/PARITY.md" "$bundle/share/doc/observatory-core/PARITY.md"
install -m 644 "$repo/native/VALIDATION.md" "$bundle/share/doc/observatory-core/VALIDATION.md"
install -m 644 "$repo/native/ARCHITECTURE.md" "$bundle/share/doc/observatory-core/ARCHITECTURE.md"
install -m 644 "$repo/native/PLUGIN_API.md" "$bundle/share/doc/observatory-core/PLUGIN_API.md"
install -m 644 "$repo/native/DIAGNOSTICS.md" "$bundle/share/doc/observatory-core/DIAGNOSTICS.md"
install -m 644 "$repo/native/INSTALLED_PLUGINS.md" "$bundle/share/doc/observatory-core/INSTALLED_PLUGINS.md"
install -d "$bundle/share/doc/observatory-core/clr"
install -m 644 "$repo/native/observatory-clr-criteria/README.md" "$bundle/share/doc/observatory-core/clr/README.md"
install -m 644 "$clr_runtime/source-provenance.json" "$bundle/share/doc/observatory-core/clr/source-provenance.json"
for audit in CORE_PARITY_AUDIT.md BUNDLED_WORKER_PARITY.md BUNDLED_WORKER_PARITY.json BIOINSIGHTS_VISUAL_QA.md EVALUATOR_PARITY_AUDIT.md CORE_NOTIFICATION_AUDIT.md CLR_SCRIPT_PARITY.md; do
    install -m 644 "$repo/native/$audit" "$bundle/share/doc/observatory-core/$audit"
done
install -d "$bundle/share/doc/observatory-core/fixtures"
for receipt in bioinsights-visual-linux-qa.json passive-notification-linux-qa.json private-fullscreen-audio-linux-qa.json native-updater-e2e-linux-qa.json legacy-import-linux-qa.json clr-criteria-framework-api.json clr-criteria-worker-boundaries.json clr-criteria-oracle.json; do
    install -m 644 "$repo/native/fixtures/$receipt" "$bundle/share/doc/observatory-core/fixtures/$receipt"
done
for port in bioinsights evaluator hecate stellar_overlay; do
    docdir="$bundle/share/doc/observatory-core/observatory-plugins/src/$port"
    install -d "$docdir"
    if [ "$port" = bioinsights ]; then provenance=PROVENANCE.md; else provenance=PORTING.md; fi
    install -m 644 "$repo/native/observatory-plugins/src/$port/$provenance" "$docdir/$provenance"
    if [ "$port" = bioinsights ]; then
        install -m 644 "$repo/native/observatory-plugins/src/$port/FEATURE_PARITY.json" "$docdir/FEATURE_PARITY.json"
    fi
done
install -m 644 "$repo/native/plugins/sample.lua" "$bundle/share/doc/observatory-core/plugins/sample.lua"
tar -C "$stage" -czf "$stage/$name.tar.gz" "$name"
deb_name=observatory-core_${version}_${arch}.deb
if [ "$tar_only" = false ]; then
debian=$stage/debian
sh "$bundle/install.sh" --prefix /usr --destdir "$debian"
mkdir -p "$debian/DEBIAN"
size=$(du -sk "$debian/usr" | cut -f 1)
cat > "$debian/DEBIAN/control" <<EOF
Package: observatory-core
Version: $version
Architecture: $arch
Maintainer: Observatory Core Contributors
Section: games
Priority: optional
Installed-Size: $size
Depends: libc6 (>= 2.35), libgcc-s1, libstdc++6, zlib1g, libicu70 | libicu72 | libicu74 | libicu76 | libicu78, libssl3 | libssl3t64, libgssapi-krb5-2, ca-certificates, tzdata, libx11-6, libxcursor1, libxi6, libxrandr2, libgl1, libegl1, libwayland-client0, libxkbcommon0, libxkbcommon-x11-0, libxcb1, libgtk-3-0, libwebkit2gtk-4.1-0, libpango-1.0-0, libpangocairo-1.0-0, libcairo2, librsvg2-2, fontconfig, fonts-noto-color-emoji, espeak-ng, pulseaudio-utils
Description: Native Elite Dangerous journal observatory
 Rust desktop journal monitoring, exploration criteria, biological sampling
 guidance, BioInsights, Evaluator, Hecate, StellarOverlay and Lua extensions.
 Includes the self-contained .NET runtime for original Explorer CLR criteria.
EOF
dpkg-deb --root-owner-group --build "$debian" "$stage/$deb_name"
fi
# Nothing is published until every requested artifact has been generated.
(cd "$stage" && sha256sum "$name.tar.gz" > SHA256SUMS)
if [ "$tar_only" = false ]; then
    (cd "$stage" && sha256sum "$deb_name" >> SHA256SUMS)
fi
mv -- "$stage/$name.tar.gz" "$out/$name.tar.gz"
if [ "$tar_only" = false ]; then mv -- "$stage/$deb_name" "$out/$deb_name"; fi
mv -- "$stage/SHA256SUMS" "$out/SHA256SUMS"
printf 'Packages written to %s\n' "$out"
