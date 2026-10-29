#!/usr/bin/env bash
# Publish only artifacts from a successful, trusted default-branch build.
set -euo pipefail
out=${1:?Usage: publish-linux.sh /directory/with/verified/artifacts}
: "${GH_REPO:?GH_REPO must identify the release repository}"
: "${GITHUB_SHA:?GITHUB_SHA must identify the tested commit}"
[[ $GITHUB_SHA =~ ^[0-9a-f]{40}$ ]] || { echo 'Invalid commit SHA.' >&2; exit 2; }
cd "$out"
for asset in Observatory_Core-x86_64.AppImage Observatory_Core-x86_64.AppImage.zsync install.sh SHA256SUMS; do
    test -s "$asset"
done
sha256sum -c SHA256SUMS
notes=$(mktemp)
trap 'rm -f -- "$notes"' EXIT
cat > "$notes" <<EOF
Automatically built and verified native Linux packages from commit $GITHUB_SHA.

Recommended: download Observatory_Core-x86_64.AppImage. Use its About window to install it for your user and check for updates. No SDK is needed.

Alternatively download install.sh and run sh install.sh. This installs the AppImage and its application-menu shortcut without root. Updates use the bundled AppImageUpdate tool and the matching zsync file; the previous image is retained as a rollback copy.

Linux x86_64 with glibc 2.35 or newer is required. The installed launcher works without FUSE. Sandboxed optional HTML views use the distribution's bubblewrap and xdg-dbus-proxy helpers.

This rolling prerelease is replaced only after the default-branch build and package verification pass. Settings, plugins and journals are preserved across application updates. SHA256SUMS covers the downloadable packages and installer.
EOF
if gh release view continuous --repo "$GH_REPO" >/dev/null 2>&1; then
    fresh=false
else
    gh release create continuous --repo "$GH_REPO" --target "$GITHUB_SHA" --draft --prerelease \
        --title 'Continuous Linux build' --notes-file "$notes"
    fresh=true
fi
# Upload the image before advertising its new zsync control data. Clients verify
# downloaded data and keep their old image if a publication is still in flight.
gh release upload continuous --repo "$GH_REPO" Observatory_Core-x86_64.AppImage --clobber
gh release upload continuous --repo "$GH_REPO" Observatory_Core-x86_64.AppImage.zsync --clobber
assets=(install.sh SHA256SUMS ./*.deb ./*.tar.gz ./*verification.json)
for asset in "${assets[@]}"; do
    [[ -f $asset ]] || continue
    gh release upload continuous --repo "$GH_REPO" "$asset" --clobber
done
if [[ $fresh == false ]]; then
    gh api --method PATCH "repos/$GH_REPO/git/refs/tags/continuous" -f sha="$GITHUB_SHA" -F force=true >/dev/null
fi
gh release edit continuous --repo "$GH_REPO" --draft=false --prerelease --latest=false \
    --title 'Continuous Linux build' --notes-file "$notes"
