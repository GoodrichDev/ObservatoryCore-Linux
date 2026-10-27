#!/bin/sh
# Check both native package formats without installing into the real filesystem.
set -eu
script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo=$(CDPATH= cd -- "$script_dir/../.." && pwd)
out=$repo/dist
clr_runtime=${OBSERVATORY_CLR_RUNTIME:-}
while [ "$#" -gt 0 ]; do
    case "$1" in
        --clr-runtime) clr_runtime=$2; shift 2 ;;
        --out) out=$2; shift 2 ;;
        --help) printf '%s\n' 'Usage: sh native/scripts/verify-package.sh [output-directory] [--clr-runtime /published/runtime]'; exit 0 ;;
        --*) printf 'Unknown option: %s\n' "$1" >&2; exit 2 ;;
        *) out=$1; shift ;;
    esac
done
out=$(CDPATH= cd -- "$out" && pwd)
version=$(sed -n '/\[workspace.package\]/,/^\[/s/^version = "\([^"]*\)"/\1/p' "$repo/Cargo.toml" | head -n 1)
case "$(uname -m)" in x86_64) triple=x86_64-linux; arch=amd64 ;; aarch64) triple=aarch64-linux; arch=arm64 ;; *) exit 2 ;; esac
validation=$(mktemp -d "${TMPDIR:-/tmp}/observatory-package-check.XXXXXXXX")
trap 'rm -rf -- "$validation"' EXIT HUP INT TERM
tar -xzf "$out/observatory-core-$version-$triple.tar.gz" -C "$validation"
bundle=$validation/observatory-core-$version-$triple
sh "$bundle/install.sh" --prefix "/opt/Observatory Core" --destdir "$validation/staged"
installed=$validation/staged/opt/Observatory\ Core
if [ -z "$clr_runtime" ]; then clr_runtime=$bundle/lib/observatory-core/clr; fi
"$installed/bin/observatory-core" --version
cmp "$bundle/bin/observatory-core" "$installed/bin/observatory-core"
diff -r "$bundle/lib/observatory-core/clr" "$installed/lib/observatory-core/clr"
test -x "$installed/lib/observatory-core/clr/observatory-clr-criteria"
test -x "$installed/lib/observatory-core/clr/createdump"
"$installed/lib/observatory-core/clr/observatory-clr-criteria" --version
cmp "$bundle/lib/observatory-core/clr/source-provenance.json" "$installed/share/doc/observatory-core/clr/source-provenance.json"
cmp "$repo/native/plugins/sample.lua" "$installed/share/doc/observatory-core/plugins/sample.lua"
cmp "$repo/native/INSTALLED_PLUGINS.md" "$installed/share/doc/observatory-core/INSTALLED_PLUGINS.md"
cmp "$repo/native/CORE_PARITY_AUDIT.md" "$installed/share/doc/observatory-core/CORE_PARITY_AUDIT.md"
cmp "$repo/native/CLR_SCRIPT_PARITY.md" "$installed/share/doc/observatory-core/CLR_SCRIPT_PARITY.md"
for receipt in clr-criteria-framework-api.json clr-criteria-worker-boundaries.json clr-criteria-oracle.json; do
    cmp "$repo/native/fixtures/$receipt" "$installed/share/doc/observatory-core/fixtures/$receipt"
done
cmp "$repo/native/fixtures/bioinsights-visual-linux-qa.json" "$installed/share/doc/observatory-core/fixtures/bioinsights-visual-linux-qa.json"
cmp "$repo/native/fixtures/private-fullscreen-audio-linux-qa.json" "$installed/share/doc/observatory-core/fixtures/private-fullscreen-audio-linux-qa.json"
cmp "$repo/native/observatory-plugins/src/bioinsights/FEATURE_PARITY.json" "$installed/share/doc/observatory-core/observatory-plugins/src/bioinsights/FEATURE_PARITY.json"
cmp "$repo/native/observatory-plugins/src/bioinsights/PROVENANCE.md" "$installed/share/doc/observatory-core/observatory-plugins/src/bioinsights/PROVENANCE.md"
desktop-file-validate "$installed/share/applications/observatory-core.desktop"
grep '^Exec=' "$installed/share/applications/observatory-core.desktop"
dpkg-deb --info "$out/observatory-core_${version}_${arch}.deb"
dpkg-deb --field "$out/observatory-core_${version}_${arch}.deb" Depends
dpkg-deb --extract "$out/observatory-core_${version}_${arch}.deb" "$validation/debian"
cmp "$bundle/bin/observatory-core" "$validation/debian/usr/bin/observatory-core"
diff -r "$bundle/lib/observatory-core/clr" "$validation/debian/usr/lib/observatory-core/clr"
test -x "$validation/debian/usr/lib/observatory-core/clr/observatory-clr-criteria"
test -x "$validation/debian/usr/lib/observatory-core/clr/createdump"
cmp "$repo/native/observatory-plugins/src/evaluator/PORTING.md" "$validation/debian/usr/share/doc/observatory-core/observatory-plugins/src/evaluator/PORTING.md"
(cd "$out" && sha256sum -c SHA256SUMS)
# Exercise automatic runtime discovery from the installed path containing spaces.
# No developer host override, system .NET installation, desktop or player data.
python3 - "$installed" "$validation/CLR criteria profile with spaces" <<'PY'
import json
import os
from pathlib import Path
import subprocess
import sys
installed, profile = map(Path, sys.argv[1:])
journals = profile / 'journals'
journals.mkdir(parents=True)
criteria = profile / 'package criteria.lua'
criteria.write_text('''---@Global
Math = luanet.import_type('System.Math')
---@End
---@Complex package-runtime
return true, 'Bundled CLR ' .. scan:GetType().Name, tostring(Math.Sqrt(81))
---@End
''')
events = [
    {'timestamp':'2026-10-26T00:00:00Z','event':'Location','StarSystem':'Package Probe','SystemAddress':42},
    {'timestamp':'2026-10-26T00:00:01Z','event':'Scan','StarSystem':'Package Probe','SystemAddress':42,'BodyID':1,'BodyName':'Package Probe 1','Radius':1000000},
]
(journals / 'Journal.2026-10-26T000000.01.log').write_text(''.join(json.dumps(e)+'\n' for e in events))
settings = profile / 'settings.json'
settings.write_text(json.dumps({'journal_dir':str(journals),'popup':False,'speech':False,
    'disabled_plugins':['BioInsights','Evaluator','Hecate','StellarOverlay'],
    'plugins':{'Explorer':{'EnableCustomCriteria':True,'CustomCriteriaFile':str(criteria)}}}))
env = dict(os.environ)
for name in ['OBSERVATORY_CLR_CRITERIA_HOST','DOTNET_ROOT','DOTNET_ROOT_X64','DOTNET_ROOT_ARM64','DISPLAY','WAYLAND_DISPLAY','WAYLAND_SOCKET']:
    env.pop(name, None)
for kind in ['CONFIG','DATA','CACHE']:
    env['XDG_'+kind+'_HOME'] = str(profile / kind.lower())
env.update(PULSE_SERVER='unix:/nonexistent-package-audio',DOTNET_MULTILEVEL_LOOKUP='0')
process = subprocess.run([str(installed/'bin'/'observatory-core'),'--config',str(settings),'--headless'],env=env,capture_output=True,text=True,timeout=30,check=True)
result = json.loads(process.stdout)
assert not result['errors'], result['errors']
matches = [f for f in result['findings'] if f['title']=='Bundled CLR Scan' and float(f['detail'])==9]
assert len(matches)==1, result
print('PASS: installed Core automatically discovered bundled real CLR; System.Math and typed Scan.GetType().Name produced the expected finding')
PY
# Exercise portable installation without a functioning Debian toolchain, and
# ensure a failed Debian build cannot replace previously published artifacts.
mkdir -p "$validation/no-debian" "$validation/failed-output"
printf '#!/bin/sh\nexit 95\n' > "$validation/no-debian/dpkg-deb"
chmod 755 "$validation/no-debian/dpkg-deb"
PATH="$validation/no-debian:$PATH" sh "$script_dir/package.sh" --tar-only --binary "$bundle/bin/observatory-core" --clr-runtime "$clr_runtime" --out "$validation/tar-only"
test ! -e "$validation/tar-only/observatory-core_${version}_${arch}.deb"
(cd "$validation/tar-only" && sha256sum -c SHA256SUMS)
printf '%s\n' 'existing artifact must survive' > "$validation/failed-output/observatory-core-$version-$triple.tar.gz"
cp "$validation/failed-output/observatory-core-$version-$triple.tar.gz" "$validation/sentinel"
if PATH="$validation/no-debian:$PATH" sh "$script_dir/package.sh" --binary "$bundle/bin/observatory-core" --clr-runtime "$clr_runtime" --out "$validation/failed-output"; then
    printf '%s\n' 'Expected Debian build failure did not occur.' >&2; exit 1
fi
cmp "$validation/sentinel" "$validation/failed-output/observatory-core-$version-$triple.tar.gz"
test ! -e "$validation/failed-output/SHA256SUMS"
mkdir "$validation/incomplete-clr"
cp -R -p "$clr_runtime/." "$validation/incomplete-clr/"
rm -- "$validation/incomplete-clr/System.Linq.dll"
if sh "$script_dir/package.sh" --tar-only --binary "$bundle/bin/observatory-core" --clr-runtime "$validation/incomplete-clr" --out "$validation/failed-output"; then
    printf '%s\n' 'Expected incomplete CLR publish rejection did not occur.' >&2; exit 1
fi
cmp "$validation/sentinel" "$validation/failed-output/observatory-core-$version-$triple.tar.gz"
test ! -e "$validation/failed-output/SHA256SUMS"
printf '%s\n' 'Package extraction, staged installation, automatic bundled CLR criteria, runtime completeness, launcher, binary identity and checksums passed.'
