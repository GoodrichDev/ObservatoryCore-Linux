#!/usr/bin/env bash
# Build the fully bundled x86_64 AppImage from the existing, validated tarball.
set -euo pipefail
script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo=$(cd -- "$script_dir/../.." && pwd)
packaging=$repo/native/packaging/appimage
out=$repo/dist
bundle=
binary=
clr_runtime=${OBSERVATORY_CLR_RUNTIME:-}
version=
keep_appdir=
tools_cache=${OBSERVATORY_APPIMAGE_TOOLS_CACHE:-${XDG_CACHE_HOME:-$HOME/.cache}/observatory-appimage-tools}
linuxdeploy=${LINUXDEPLOY_APPIMAGE:-}
gtk_plugin=${LINUXDEPLOY_GTK_PLUGIN:-}
appimagetool=${APPIMAGETOOL_APPIMAGE:-}
runtime=${APPIMAGE_RUNTIME:-}
updater=${APPIMAGE_UPDATE_APPIMAGE:-}
update_info='gh-releases-zsync|GoodrichDev|ObservatoryCore-Linux|continuous|Observatory_Core-x86_64.AppImage.zsync'
while (($#)); do
    case "$1" in
        --bundle) bundle=$2; shift 2 ;;
        --binary) binary=$2; shift 2 ;;
        --clr-runtime) clr_runtime=$2; shift 2 ;;
        --version) version=$2; shift 2 ;;
        --out) out=$2; shift 2 ;;
        --keep-appdir) keep_appdir=$2; shift 2 ;;
        --update-information) update_info=$2; shift 2 ;;
        --help)
            printf '%s\n' 'Usage: bash native/scripts/package-appimage.sh [--bundle PACKAGE.tar.gz | --binary ELF --clr-runtime DIRECTORY] [--version VERSION] [--out DIRECTORY] [--keep-appdir NEW_DIRECTORY] [--update-information TRANSPORT]' \
                'Build on Ubuntu 22.04 x86_64. Tools are SHA-256 pinned in native/packaging/appimage/tools.lock.' \
                'Explicit verified tool paths: LINUXDEPLOY_APPIMAGE, LINUXDEPLOY_GTK_PLUGIN, APPIMAGETOOL_APPIMAGE, APPIMAGE_RUNTIME, APPIMAGE_UPDATE_APPIMAGE.'
            exit 0 ;;
        *) printf 'Unknown option: %s\n' "$1" >&2; exit 2 ;;
    esac
done
[[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || { echo 'AppImage packaging requires Linux x86_64.' >&2; exit 2; }
[[ $(getconf GNU_LIBC_VERSION) == 'glibc 2.35' ]] || { echo 'Build AppImages on Ubuntu 22.04 with GLIBC 2.35 to preserve the supported ABI.' >&2; exit 2; }
for command in python3 curl sha256sum tar file readelf ldd pkg-config glib-compile-schemas gio-querymodules; do
    command -v "$command" >/dev/null || { printf 'Missing build dependency: %s\n' "$command" >&2; exit 2; }
done
if [[ -z $version ]]; then
    version=$(sed -n '/\[workspace.package\]/,/^\[/s/^version = "\([^"]*\)"/\1/p' "$repo/Cargo.toml" | head -n1)
fi
[[ $version =~ ^[0-9A-Za-z.+:~_-]+$ ]] || { echo 'Invalid version.' >&2; exit 2; }
python3 - "$update_info" <<'PY'
import sys
from urllib.parse import urlsplit
value = sys.argv[1]
if value == 'gh-releases-zsync|GoodrichDev|ObservatoryCore-Linux|continuous|Observatory_Core-x86_64.AppImage.zsync':
    raise SystemExit(0)
if not value.startswith('zsync|') or value.count('|') != 1:
    raise SystemExit('Update override must use zsync|HTTPS_URL or a loopback HTTP test feed')
url = urlsplit(value.split('|', 1)[1])
if (url.scheme == 'https' or (url.scheme == 'http' and url.hostname in {'127.0.0.1', 'localhost', '::1'})) and url.hostname and not url.username and not url.password and url.path.endswith('.zsync') and not url.fragment:
    raise SystemExit(0)
raise SystemExit('Invalid update override: require HTTPS or loopback HTTP, without credentials, and a .zsync path')
PY
[[ -z $keep_appdir || ! -e $keep_appdir ]] || { echo '--keep-appdir target already exists.' >&2; exit 2; }
mkdir -p -- "$out" "$tools_cache"
out=$(cd -- "$out" && pwd)
tools_cache=$(cd -- "$tools_cache" && pwd)
stage=$(mktemp -d "${TMPDIR:-/tmp}/observatory-appimage.XXXXXXXX")
trap 'rm -rf -- "$stage"' EXIT

# Even explicit paths must match reviewed pins. Cached downloads avoid mutable
# latest/continuous lookups; a replaced upstream file fails closed on its SHA.
fetch_tool() {
    local name=$1 explicit=$2 checksum url destination
    IFS='|' read -r _ checksum url < <(awk -F'|' -v name="$name" '$1 == name {print; exit}' "$packaging/tools.lock")
    [[ $checksum =~ ^[0-9a-f]{64}$ && $url == https://* ]] || { echo "Invalid tool pin: $name" >&2; return 2; }
    destination=${explicit:-$tools_cache/$name-$checksum}
    if [[ ! -f $destination ]]; then
        [[ -z $explicit ]] || { echo "Tool path does not exist: $destination" >&2; return 2; }
        curl --fail --location --retry 3 --proto '=https' --tlsv1.2 -H 'Accept: application/octet-stream' "$url" -o "$stage/$name.download" >&2
        printf '%s  %s\n' "$checksum" "$stage/$name.download" | sha256sum --check --status
        mv -- "$stage/$name.download" "$destination"
    fi
    printf '%s  %s\n' "$checksum" "$destination" | sha256sum --check --status || { echo "Tool SHA-256 mismatch: $name" >&2; return 2; }
    chmod u+x -- "$destination"
    realpath -- "$destination"
}
linuxdeploy=$(fetch_tool linuxdeploy "$linuxdeploy")
gtk_plugin=$(fetch_tool gtk-plugin "$gtk_plugin")
appimagetool=$(fetch_tool appimagetool "$appimagetool")
# Reuse the runtime in the versioned, verified tool instead of depending on an
# upstream continuous asset that can disappear when its release is replaced.
IFS='|' read -r _ runtime_checksum runtime_source < <(awk -F'|' '$1 == "runtime" {print; exit}' "$packaging/tools.lock")
[[ $runtime_checksum =~ ^[0-9a-f]{64}$ && $runtime_source == derive:appimagetool-header ]] || { echo 'Invalid derived runtime pin.' >&2; exit 2; }
if [[ -z $runtime ]]; then
    runtime=$tools_cache/runtime-$runtime_checksum
    if [[ ! -f $runtime ]]; then
        runtime_offset=$("$appimagetool" --appimage-offset)
        [[ $runtime_offset =~ ^[0-9]+$ ]] || { echo 'Invalid AppImage runtime offset.' >&2; exit 2; }
        python3 - "$appimagetool" "$runtime_offset" "$stage/runtime.derived" <<'PY'
from pathlib import Path
import sys
source, offset, target = Path(sys.argv[1]), int(sys.argv[2]), Path(sys.argv[3])
with source.open('rb') as stream:
    runtime = stream.read(offset)
if len(runtime) != offset or runtime[8:11] != b'AI\x02':
    raise SystemExit('Invalid type 2 runtime prefix in pinned appimagetool')
target.write_bytes(runtime)
PY
        printf '%s  %s\n' "$runtime_checksum" "$stage/runtime.derived" | sha256sum --check --status
        mv "$stage/runtime.derived" "$runtime"
    fi
fi
printf '%s  %s\n' "$runtime_checksum" "$runtime" | sha256sum --check --status || { echo 'Derived runtime SHA-256 mismatch.' >&2; exit 2; }
runtime=$(realpath -- "$runtime")
updater=$(fetch_tool appimage-update "$updater")
mkdir -p "$stage/tools/linuxdeploy" "$stage/tools/appimagetool" "$stage/tools/bin"
(cd "$stage/tools/linuxdeploy" && "$linuxdeploy" --appimage-extract >/dev/null)
(cd "$stage/tools/appimagetool" && "$appimagetool" --appimage-extract >/dev/null)
deploy=$stage/tools/linuxdeploy/squashfs-root/AppRun
image_tool=$stage/tools/appimagetool/squashfs-root/AppRun
patchelf=$stage/tools/linuxdeploy/squashfs-root/usr/bin/patchelf
install -m755 "$gtk_plugin" "$stage/tools/bin/linuxdeploy-plugin-gtk.sh"
export PATH="$stage/tools/bin:$stage/tools/appimagetool/squashfs-root/usr/bin:$PATH"
export LINUXDEPLOY=$deploy DEPLOY_GTK_VERSION=3

if [[ -z $bundle ]]; then
    arguments=(--out "$stage/bundle" --version "$version" --tar-only)
    [[ -z $binary ]] || arguments+=(--binary "$binary")
    [[ -z $clr_runtime ]] || arguments+=(--clr-runtime "$clr_runtime")
    bash "$script_dir/package.sh" "${arguments[@]}"
    bundle=$stage/bundle/observatory-core-$version-x86_64-linux.tar.gz
fi
bundle=$(realpath -- "$bundle")
mkdir -p "$stage/unpacked"
tar -xzf "$bundle" -C "$stage/unpacked"
shopt -s nullglob
bundles=("$stage/unpacked"/observatory-core-*-x86_64-linux)
[[ ${#bundles[@]} == 1 && -x ${bundles[0]}/bin/observatory-core ]] || { echo 'Expected one native x86_64 release tarball.' >&2; exit 2; }
appdir=$stage/Observatory_Core.AppDir
mkdir -p "$appdir/usr"
cp -a "${bundles[0]}/." "$appdir/usr/"
rm -f "$appdir/usr/install.sh"
[[ -s $appdir/usr/lib/observatory-core/clr/libcoreclr.so ]] || { echo 'Tarball lacks the complete CLR runtime.' >&2; exit 2; }

libdir=/usr/lib/x86_64-linux-gnu
copy_directory() {
    local source=$1 target=$2
    [[ -d $source ]] || { echo "Missing runtime resource directory: $source" >&2; exit 2; }
    mkdir -p -- "$target"
    cp -a "$source/." "$target/"
}
library_path() {
    local result
    result=$(ldconfig -p | awk -v name="$1" '$1 == name && /x86-64/ && !found {print $NF; found=1}')
    [[ -n $result ]] || { echo "Missing runtime library: $1" >&2; return 2; }
    printf '%s\n' "$result"
}
libraries=()
# These are loaded by name, so ELF DT_NEEDED traversal alone cannot find them.
for name in libgtk-3.so.0 libwebkit2gtk-4.1.so.0 libpango-1.0.so.0 libpangocairo-1.0.so.0 libcairo.so.2 librsvg-2.so.2 libfontconfig.so.1 libfreetype.so.6 libGL.so.1 libEGL.so.1 libGLESv2.so.2 libxkbcommon-x11.so.0 libssl.so.3 libgssapi_krb5.so.2; do
    libraries+=(--library "$(library_path "$name")")
done
for name in libicuuc libicui18n; do
    path=$(ldconfig -p | awk -v name="$name" '$1 ~ ("^" name "\\.so\\.[0-9]+$") && /x86-64/ && !found {print $NF; found=1}')
    [[ -n $path ]] || { echo "Missing runtime library: $name" >&2; exit 2; }
    libraries+=(--library "$path")
done
for name in espeak-ng paplay fc-cache fc-list fc-match bwrap xdg-dbus-proxy; do
    path=$(command -v "$name") || { echo "Missing runtime executable: $name" >&2; exit 2; }
    install -m755 "$path" "$appdir/usr/bin/$name"
done
copy_directory "$libdir/webkit2gtk-4.1" "$appdir/usr/lib/webkit2gtk-4.1"
copy_directory "$libdir/gio/modules" "$appdir/usr/lib/gio/modules"
gio-querymodules "$appdir/usr/lib/gio/modules"
copy_directory "$libdir/gstreamer-1.0" "$appdir/usr/lib/gstreamer-1.0"
mkdir -p "$appdir/usr/libexec/gstreamer-1.0"
install -m755 "$libdir/gstreamer1.0/gstreamer-1.0/gst-plugin-scanner" "$appdir/usr/libexec/gstreamer-1.0/"
copy_directory "$libdir/espeak-ng-data" "$appdir/usr/share/espeak-ng-data"
copy_directory /usr/share/mime "$appdir/usr/share/mime"
copy_directory /usr/share/icons/Adwaita "$appdir/usr/share/icons/Adwaita"
copy_directory /usr/share/icons/hicolor "$appdir/usr/share/icons/hicolor"
copy_directory /usr/share/fonts/truetype/dejavu "$appdir/usr/share/fonts/truetype/dejavu"
if [[ -d /usr/share/fonts/truetype/ubuntu ]]; then copy_directory /usr/share/fonts/truetype/ubuntu "$appdir/usr/share/fonts/truetype/ubuntu"; fi
[[ -f /usr/share/fonts/truetype/noto/NotoColorEmoji.ttf ]] || { echo 'Missing build dependency: fonts-noto-color-emoji' >&2; exit 2; }
mkdir -p "$appdir/usr/share/fonts/truetype/noto"
install -m644 /usr/share/fonts/truetype/noto/NotoColorEmoji.ttf "$appdir/usr/share/fonts/truetype/noto/"
copy_directory /etc/fonts "$appdir/usr/etc/fonts"
# Dereference conf.d links: distro fontconfig uses absolute ../conf.avail links.
find "$appdir/usr/etc/fonts" -type l -print0 | while IFS= read -r -d '' link; do
    resolved=$(readlink -f "$link")
    [[ -f $resolved ]] && { cp -L "$link" "$stage/font.conf"; rm "$link"; cp "$stage/font.conf" "$link"; }
done
cat > "$appdir/usr/etc/fonts/fonts.conf" <<'EOF'
<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd">
<fontconfig>
  <!-- AppRun generates an absolute bundled-font path in the user's cache. -->
  <dir>/usr/share/fonts</dir>
  <dir>/usr/local/share/fonts</dir>
  <dir prefix="xdg">fonts</dir>
  <include ignore_missing="yes">conf.d</include>
  <cachedir prefix="xdg">fontconfig</cachedir>
</fontconfig>
EOF
mkdir -p "$appdir/usr/etc/ssl/certs"
install -m644 /etc/ssl/certs/ca-certificates.crt "$appdir/usr/etc/ssl/certs/"
[[ ! -d /usr/share/zoneinfo ]] || copy_directory /usr/share/zoneinfo "$appdir/usr/share/zoneinfo"

# The GTK plugin stages schemas, pixbuf loaders, input modules and their caches.
# This optional .NET LTTng module targets a legacy SONAME absent on Jammy. Keep
# its published bytes intact; the closure audit records the exact exception.
trace_provider=$appdir/usr/lib/observatory-core/clr/libcoreclrtraceptprovider.so
if [[ -f $trace_provider ]]; then mv "$trace_provider" "$stage/libcoreclrtraceptprovider.so"; fi
"$deploy" --appdir "$appdir" --executable "$appdir/usr/bin/observatory-core" \
    --desktop-file "$appdir/usr/share/applications/observatory-core.desktop" \
    --icon-file "$appdir/usr/share/icons/hicolor/128x128/apps/observatory-core.png" \
    "${libraries[@]}" --plugin gtk --deploy-deps-only "$appdir/usr/bin" \
    --deploy-deps-only "$appdir/usr/lib/webkit2gtk-4.1" \
    --deploy-deps-only "$appdir/usr/lib/gio/modules" \
    --deploy-deps-only "$appdir/usr/lib/gstreamer-1.0" \
    --deploy-deps-only "$appdir/usr/libexec/gstreamer-1.0"
if [[ -f $stage/libcoreclrtraceptprovider.so ]]; then mv "$stage/libcoreclrtraceptprovider.so" "$trace_provider"; fi
python3 "$packaging/bundle-dependencies.py" "$appdir" "$patchelf"
# Keep the updater's own launcher, libraries, licenses and checkrt hook intact.
mkdir -p "$stage/tools/updater" "$appdir/usr/lib/observatory-core/appimage-update"
(cd "$stage/tools/updater" && "$updater" --appimage-extract >/dev/null)
cp -a "$stage/tools/updater/squashfs-root/." "$appdir/usr/lib/observatory-core/appimage-update/"
# Upstream's checkrt hook leaves this command path unquoted. Correct just that
# shell invocation so the bundled updater also works below a home with spaces.
python3 - "$appdir/usr/lib/observatory-core/appimage-update/apprun-hooks/linuxdeploy-plugin-checkrt.sh" <<'PY'
from pathlib import Path
import sys
hook = Path(sys.argv[1])
source = hook.read_text()
original = 'CHECKRT_LIBS="$($APPDIR/checkrt/checkrt)"'
replacement = 'CHECKRT_LIBS="$("$APPDIR/checkrt/checkrt")"'
if source.count(original) != 1:
    raise SystemExit('Pinned updater checkrt hook changed; review its path quoting before packaging')
hook.write_text(source.replace(original, replacement))
PY
python3 "$packaging/bundle-dependencies.py" "$appdir" --audit-updater
rm -f "$appdir/AppRun"
install -m755 "$packaging/AppRun" "$appdir/AppRun"
install -m755 "$packaging/install.sh" "$appdir/usr/lib/observatory-core/appimage-manage.sh"
install -m644 "$packaging/tools.lock" "$appdir/usr/share/doc/observatory-core/appimage-tools.lock"
printf '%s\n' "$version" > "$appdir/usr/share/observatory-core-version"

filename=Observatory_Core-x86_64.AppImage
if [[ -n $keep_appdir ]]; then cp -a "$appdir" "$keep_appdir"; fi
# appimagetool writes the zsync file in its working directory, even when the
# AppImage destination is absolute. Keep both outputs in the private stage.
(cd "$stage" && ARCH=x86_64 VERSION="$version" "$image_tool" --runtime-file "$runtime" \
    --updateinformation "$update_info" --file-url "$filename" \
    --mksquashfs-opt -processors --mksquashfs-opt "${APPIMAGE_BUILD_PROCESSORS:-2}" \
    "$appdir" "$stage/$filename")
[[ -s $stage/$filename.zsync ]] || { echo 'AppImage builder did not generate the required zsync control file.' >&2; exit 2; }
chmod 755 "$stage/$filename"
(cd "$stage" && sha256sum "$filename" "$filename.zsync" > AppImage-SHA256SUMS)
mv "$stage/$filename" "$stage/$filename.zsync" "$stage/AppImage-SHA256SUMS" "$out/"
install -m755 "$packaging/install.sh" "$out/install.sh"
printf 'AppImage and zsync control file written to %s\n' "$out"
