#!/bin/sh
# Also shipped inside the image as appimage-manage.sh. Never needs root.
set -eu

name=Observatory_Core-x86_64.AppImage
release=https://github.com/GoodrichDev/ObservatoryCore-Linux/releases/download/continuous
action=install
source_image=${APPIMAGE:-}
while [ "$#" -gt 0 ]; do
    case "$1" in
        --install) action=install; shift ;;
        --from) source_image=${2:?--from requires an AppImage path}; shift 2 ;;
        --update) action=update; shift ;;
        --check-for-update) action=check; shift ;;
        --uninstall) action=uninstall; shift ;;
        --help) printf '%s\n' 'Usage: sh install.sh [--from /path/to/AppImage] [--install|--update|--check-for-update|--uninstall]'; exit 0 ;;
        *) printf 'Unknown option: %s\n' "$1" >&2; exit 2 ;;
    esac
done
case "$(uname -s):$(uname -m)" in Linux:x86_64) ;; *) printf '%s\n' 'This AppImage is for Linux x86_64.' >&2; exit 2 ;; esac
data=${XDG_DATA_HOME:-"$HOME/.local/share"}
cache=${XDG_CACHE_HOME:-"$HOME/.cache"}
bin=$HOME/.local/bin
for directory in "$data" "$cache" "$bin"; do
    case "$directory" in /*) ;; *) printf '%s\n' 'HOME and XDG directories must be absolute.' >&2; exit 2 ;; esac
done
managed=$data/observatory-core/appimage
installed=$managed/$name
launcher=$bin/observatory-core-appimage
desktop=$data/applications/observatory-core-appimage.desktop
icon=$data/icons/hicolor/128x128/apps/observatory-core-appimage.png

if [ "$action" = uninstall ]; then
    # Only remove our exact managed files; journals, settings, plugins and caches
    # are retained. No recursive deletion or package-manager installation changes.
    rm -f -- "$launcher" "$desktop" "$icon" "$installed" "$installed.zs-old"
    rmdir -- "$managed" 2>/dev/null || :
    printf '%s\n' 'AppImage removed. Your settings, plugins and journals are retained.'
    exit 0
fi

if [ "$action" = update ] || [ "$action" = check ]; then
    source_image=${source_image:-$installed}
    test -f "$source_image" || { printf '%s\n' 'Install the AppImage first.' >&2; exit 2; }
    test -n "${APPDIR:-}" && test -x "$APPDIR/usr/lib/observatory-core/appimage-update/AppRun" || {
        # Standalone installer updates through the installed image itself.
        test -x "$launcher" || { printf '%s\n' 'Run the installed AppImage to update it.' >&2; exit 2; }
        if [ "$action" = check ]; then exec "$launcher" --check-for-update; else exec "$launcher" --update; fi
    }
    updater=$APPDIR/usr/lib/observatory-core/appimage-update
    SSL_CERT_FILE=${SSL_CERT_FILE:-$APPDIR/usr/etc/ssl/certs/ca-certificates.crt}
    export SSL_CERT_FILE
    # The upstream checkrt hook has an unquoted APPDIR command and LD_PRELOAD
    # cannot represent spaces. A private alias preserves the unmodified updater
    # while supporting application paths and HOME directories containing spaces.
    updater_stage=$(mktemp -d /tmp/observatory-appimage-updater.XXXXXXXX)
    trap 'rm -rf -- "$updater_stage"' EXIT HUP INT TERM
    ln -s "$updater" "$updater_stage/AppDir"
    run_updater() { env APPDIR="$updater_stage/AppDir" "$updater_stage/AppDir/AppRun" "$@" "$source_image"; }
    if [ "$action" = check ]; then
        set +e
        run_updater --check-for-update
        result=$?
        set -e
        exit "$result"
    fi
    test -w "$source_image" && test -w "$(dirname -- "$source_image")" || {
        printf '%s\n' 'The AppImage location is read-only. Install it for this user before updating.' >&2; exit 2;
    }
    # Upstream retains a .zs-old rollback copy and checks the downloaded data
    # before replacing the image. Verify installation too, not only download.
    run_updater --overwrite
    test -s "$source_image" && test -x "$source_image"
    set +e
    run_updater --check-for-update
    result=$?
    set -e
    if [ "$result" -ne 0 ]; then
        printf '%s\n' 'The update could not be confirmed. The previous image is retained as .zs-old.' >&2
        exit 2
    fi
    printf '%s\n' 'Update complete. Restart Observatory Core to use the updated version.'
    exit 0
fi

mkdir -p -- "$managed" "$bin" "$(dirname -- "$desktop")" "$(dirname -- "$icon")" "$cache/observatory-core"
stage=$(mktemp -d "$managed/.install.XXXXXXXX")
trap 'rm -rf -- "$stage"' EXIT HUP INT TERM
fetch() {
    if command -v curl >/dev/null 2>&1; then
        curl --fail --location --proto '=https' --proto-redir '=https' --connect-timeout 20 --max-time 600 --retry 3 --output "$2" "$1"
    elif command -v wget >/dev/null 2>&1; then
        wget --https-only --timeout=60 --tries=3 -O "$2" "$1"
    else printf '%s\n' 'Install curl or wget to download the AppImage.' >&2; exit 2; fi
}
if [ -z "$source_image" ]; then
    fetch "$release/$name" "$stage/$name"
    fetch "$release/SHA256SUMS" "$stage/SHA256SUMS"
    expected=$(awk -v name="$name" '$2==name || $2=="*"name {print $1}' "$stage/SHA256SUMS")
    case "$expected" in ''|*[!0-9a-fA-F]*) printf '%s\n' 'Release checksum is missing or malformed.' >&2; exit 2 ;; esac
    [ "${#expected}" -eq 64 ] || { printf '%s\n' 'Invalid release checksum.' >&2; exit 2; }
    printf '%s  %s\n' "$expected" "$name" > "$stage/verify.sha256"
    (cd "$stage" && sha256sum -c verify.sha256)
    source_image=$stage/$name
fi
test -s "$source_image" || { printf '%s\n' 'AppImage file not found.' >&2; exit 2; }
# Require the AppImage type-2 signature before making the copied file executable.
signature=$(od -An -tx1 -j8 -N3 "$source_image" | tr -d ' \n')
[ "$signature" = 414902 ] || { printf '%s\n' 'The selected file is not a type-2 AppImage.' >&2; exit 2; }
install -m 755 "$source_image" "$stage/image"
"$stage/image" --appimage-version >/dev/null

# Always use the runtime's retained extraction cache: no FUSE package or root
# setup, and unchanged files are reused on later launches.
quote_sh() { printf "'"; printf '%s' "$1" | sed "s/'/'\\\\''/g"; printf "'"; }
{
    printf '#!/bin/sh\nset -eu\nimage='; quote_sh "$installed"; printf '\n'
    printf 'cache=${XDG_CACHE_HOME:-"$HOME/.cache"}/observatory-core/appimage-runtime\n'
    printf 'mkdir -p -- "$cache"\nchmod 700 "$cache"\n'
    printf 'export TMPDIR="$cache" APPIMAGE_EXTRACT_AND_RUN=1 NO_CLEANUP=1\n'
    printf 'exec "$image" "$@"\n'
} > "$stage/launcher"
desktop_exec=$(printf '%s' "$launcher" | sed 's/[\\"`$]/\\&/g; s/%/%%/g')
cat > "$stage/desktop" <<EOF
[Desktop Entry]
Type=Application
Version=1.0
Name=Observatory Core (AppImage)
Comment=Monitor Elite Dangerous journals and exploration discoveries
Exec="$desktop_exec"
Icon=observatory-core-appimage
Terminal=false
Categories=Game;
Keywords=Elite;Dangerous;Exploration;Journal;
StartupWMClass=observatory-core
EOF
if [ -n "${APPDIR:-}" ] && [ -f "$APPDIR/usr/share/icons/hicolor/128x128/apps/observatory-core.png" ]; then
    install -m 644 "$APPDIR/usr/share/icons/hicolor/128x128/apps/observatory-core.png" "$stage/icon.png"
else
    # The installed image contains the trusted icon; extract only that resource
    # into our own staging directory rather than running the application.
    (cd "$stage" && "$stage/image" --appimage-extract 'usr/share/icons/hicolor/128x128/apps/observatory-core.png' >/dev/null)
    install -m 644 "$stage/squashfs-root/usr/share/icons/hicolor/128x128/apps/observatory-core.png" "$stage/icon.png"
fi
# Validate every resource before replacing an existing working installation.
test -s "$stage/icon.png"
mv -f -- "$stage/image" "$installed"
install -m 755 "$stage/launcher" "$launcher"
install -m 644 "$stage/desktop" "$desktop"
install -m 644 "$stage/icon.png" "$icon"
if command -v update-desktop-database >/dev/null 2>&1; then update-desktop-database "$data/applications" >/dev/null 2>&1 || :; fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then gtk-update-icon-cache -f -t "$data/icons/hicolor" >/dev/null 2>&1 || :; fi
printf '%s\n' 'Installed Observatory Core. Launch it from your application menu.' 'Install and update controls are available in About.'
if [ ! -x /usr/bin/bwrap ] || [ ! -x /usr/bin/xdg-dbus-proxy ]; then
    printf '%s\n' 'Optional sandboxed HTML views require the distribution packages bubblewrap and xdg-dbus-proxy.'
fi
