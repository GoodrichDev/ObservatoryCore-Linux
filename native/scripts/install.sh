#!/bin/sh
# Install a built portable bundle without a package manager.
set -eu

prefix=${PREFIX:-"$HOME/.local"}
destdir=${DESTDIR:-}
while [ "$#" -gt 0 ]; do
    case "$1" in
        --prefix) prefix=$2; shift 2 ;;
        --destdir) destdir=$2; shift 2 ;;
        --help) printf '%s\n' 'Usage: sh install.sh [--prefix /absolute/prefix] [--destdir /staging/root]'; exit 0 ;;
        *) printf 'Unknown option: %s\n' "$1" >&2; exit 2 ;;
    esac
done
case "$prefix" in /*) ;; *) printf '%s\n' 'Installation prefix must be absolute.' >&2; exit 2 ;; esac
case "$destdir" in ''|/*) ;; *) printf '%s\n' 'DESTDIR must be empty or absolute.' >&2; exit 2 ;; esac
bundle=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
target=$destdir$prefix
test -f "$bundle/bin/observatory-core"
for required in observatory-clr-criteria observatory-clr-criteria.dll observatory-clr-criteria.deps.json observatory-clr-criteria.runtimeconfig.json ObservatoryFramework.dll NLua.dll KeraLua.dll liblua54.so libcoreclr.so libhostfxr.so libhostpolicy.so System.Private.CoreLib.dll createdump source-provenance.json; do
    test -s "$bundle/lib/observatory-core/clr/$required" || { printf 'Incomplete bundled CLR runtime: missing %s\n' "$required" >&2; exit 2; }
done
test -x "$bundle/lib/observatory-core/clr/observatory-clr-criteria"
install -d "$target/bin" "$target/share/applications" "$target/share/icons/hicolor/128x128/apps" "$target/share/metainfo" "$target/share/doc/observatory-core/plugins"
install -m 755 "$bundle/bin/observatory-core" "$target/bin/observatory-core"
install -d "$target/lib/observatory-core/clr"
cp -R -p "$bundle/lib/observatory-core/clr/." "$target/lib/observatory-core/clr/"
chmod 755 "$target/lib/observatory-core/clr/observatory-clr-criteria" "$target/lib/observatory-core/clr/createdump"
install -m 644 "$bundle/share/applications/observatory-core.desktop" "$target/share/applications/observatory-core.desktop"
# User prefixes are not necessarily in the desktop session's PATH. Desktop Exec
# has its own quoting rules, including doubled percent signs for literal paths.
if [ "$prefix" != /usr ]; then
    desktop_exec=$(printf '%s' "$prefix/bin/observatory-core" | sed 's/[\\"`$]/\\&/g; s/%/%%/g')
    desktop_file=$target/share/applications/observatory-core.desktop
    desktop_tmp=$desktop_file.tmp
    while IFS= read -r line; do
        case "$line" in Exec=*) printf 'Exec="%s"\n' "$desktop_exec" ;; *) printf '%s\n' "$line" ;; esac
    done < "$desktop_file" > "$desktop_tmp"
    mv -- "$desktop_tmp" "$desktop_file"
fi
install -m 644 "$bundle/share/icons/hicolor/128x128/apps/observatory-core.png" "$target/share/icons/hicolor/128x128/apps/observatory-core.png"
install -m 644 "$bundle/share/metainfo/net.xjph.observatory.Core.metainfo.xml" "$target/share/metainfo/net.xjph.observatory.Core.metainfo.xml"
for document in LICENSE README.md PARITY.md ARCHITECTURE.md PLUGIN_API.md DIAGNOSTICS.md VALIDATION.md INSTALLED_PLUGINS.md CORE_PARITY_AUDIT.md BUNDLED_WORKER_PARITY.md BUNDLED_WORKER_PARITY.json BIOINSIGHTS_VISUAL_QA.md EVALUATOR_PARITY_AUDIT.md CORE_NOTIFICATION_AUDIT.md CLR_SCRIPT_PARITY.md; do
    install -m 644 "$bundle/share/doc/observatory-core/$document" "$target/share/doc/observatory-core/$document"
done
install -d "$target/share/doc/observatory-core/fixtures"
for receipt in bioinsights-visual-linux-qa.json passive-notification-linux-qa.json private-fullscreen-audio-linux-qa.json native-updater-e2e-linux-qa.json legacy-import-linux-qa.json clr-criteria-framework-api.json clr-criteria-worker-boundaries.json clr-criteria-oracle.json; do
    install -m 644 "$bundle/share/doc/observatory-core/fixtures/$receipt" "$target/share/doc/observatory-core/fixtures/$receipt"
done
install -d "$target/share/doc/observatory-core/clr"
for document in README.md source-provenance.json; do
    install -m 644 "$bundle/share/doc/observatory-core/clr/$document" "$target/share/doc/observatory-core/clr/$document"
done
for port in bioinsights evaluator hecate stellar_overlay; do
    docdir=share/doc/observatory-core/observatory-plugins/src/$port
    install -d "$target/$docdir"
    if [ "$port" = bioinsights ]; then provenance=PROVENANCE.md; else provenance=PORTING.md; fi
    install -m 644 "$bundle/$docdir/$provenance" "$target/$docdir/$provenance"
    if [ "$port" = bioinsights ]; then
        install -m 644 "$bundle/$docdir/FEATURE_PARITY.json" "$target/$docdir/FEATURE_PARITY.json"
    fi
done
install -m 644 "$bundle/share/doc/observatory-core/plugins/sample.lua" "$target/share/doc/observatory-core/plugins/sample.lua"
# These caches belong to the actual installation, never a staged package.
if [ -z "$destdir" ]; then
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$target/share/applications" >/dev/null 2>&1 || :
    fi
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -f -t "$target/share/icons/hicolor" >/dev/null 2>&1 || :
    fi
fi
printf 'Installed Observatory Core into %s\n' "$target"
