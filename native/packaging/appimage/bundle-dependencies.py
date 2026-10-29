#!/usr/bin/env python3
"""Close linuxdeploy's intentionally conservative exclusion list for this app.

GTK/WebKit, CLR and audio use dlopen and subprocesses. Inspect every staged ELF,
copy dependencies except the base C/C++ ABI, and give each ELF a relative RPATH.
Host C++ runtimes accompany the host graphics drivers, through bundled GLVND.
"""
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

appdir, patchelf = Path(sys.argv[1]).resolve(), sys.argv[2]
libdir = appdir / "usr/lib"
host_cpp = {"libstdc++.so.6", "libgcc_s.so.1"}
host_graphics = re.compile(r"^(libdrm(?:_[A-Za-z0-9_-]+)?|libgbm)\.so(?:\.\d+)*$")
base = re.compile(r"^(ld-linux[^/]*|lib(c|m|dl|pthread|rt|resolv|util|anl|nss_[^.]+)\.so(?:\..*)?|libstdc\+\+\.so\.6|libgcc_s\.so\.1)$")
origins = set()
host_dependencies = set()
optional_diagnostics = []


def elf(path):
    try:
        with path.open("rb") as stream:
            return stream.read(4) == b"\x7fELF"
    except OSError:
        return False


if patchelf == "--audit-updater":
    updater = appdir / "usr/lib/observatory-core/appimage-update"
    updater_files = sorted(p for p in updater.rglob("*") if p.is_file() and not p.is_symlink() and elf(p))
    updater_versions = set()
    environment = dict(os.environ, LD_LIBRARY_PATH=str(updater / "usr/lib") + ":" + str(updater / "usr/lib/x86_64-linux-gnu"))
    for path in updater_files:
        result = subprocess.run(["ldd", str(path)], capture_output=True, text=True, env=environment)
        if "not found" in result.stdout:
            raise SystemExit(f"Unresolved updater dependency for {path}:\n{result.stdout}")
        result = subprocess.run(["readelf", "--version-info", str(path)], capture_output=True, text=True, check=True)
        updater_versions.update(re.findall(r"\bGLIBC_(\d+\.\d+(?:\.\d+)?)\b", result.stdout))
    maximum = max(updater_versions, key=lambda value: tuple(map(int, value.split("."))), default="0.0")
    if tuple(map(int, maximum.split("."))) > (2, 35):
        raise SystemExit("Updater exceeds the supported GLIBC 2.35 ABI: " + maximum)
    receipt_path = appdir / "usr/share/doc/observatory-core/appimage-dependencies.json"
    receipt = json.loads(receipt_path.read_text())
    receipt["updater"] = {
        "elf_files": len(updater_files),
        "maximum_glibc_version": maximum,
        "elf_files_preserved_unmodified": True,
        "launcher_adjustments": ["Quoted upstream checkrt executable path to support installation paths containing spaces"],
    }
    receipt["application_elf_files"] = receipt.get("application_elf_files", receipt["bundled_elf_files"])
    receipt["bundled_elf_files"] = receipt["application_elf_files"] + len(updater_files)
    receipt_path.write_text(json.dumps(receipt, indent=2) + "\n")
    print(f"Verified {len(updater_files)} unmodified updater ELF files; maximum GLIBC requirement {maximum}")
    raise SystemExit(0)


# Do not make newer Mesa/LLVM vendor drivers load Jammy's older C++ runtime.
# Only remove the main payload copies; the updater keeps its isolated runtime.
for soname in host_cpp:
    (libdir / soname).unlink(missing_ok=True)
for path in libdir.iterdir():
    if host_graphics.fullmatch(path.name):
        path.unlink()

queue = sorted(p for p in appdir.rglob("*") if p.is_file() and not p.is_symlink() and elf(p))
names = {path.name for path in queue}
ldconfig = subprocess.run(["ldconfig", "-p"], capture_output=True, text=True, check=True)
for line in ldconfig.stdout.splitlines():
    match = re.match(r"\s*(\S+) .* => (/\S+)$", line)
    if match and match.group(1) in names:
        origins.add(match.group(2))
seen = set()
environment = dict(os.environ, LD_LIBRARY_PATH=str(libdir))
while queue:
    path = queue.pop(0)
    if path in seen:
        continue
    seen.add(path)
    result = subprocess.run(["ldd", str(path)], capture_output=True, text=True, env=environment)
    missing = re.findall(r"^\s*(\S+) => not found$", result.stdout, re.MULTILINE)
    # .NET includes its optional legacy LTTng trace provider even on systems
    # whose LTTng has a newer, incompatible SONAME. Script evaluation does not
    # load it. Preserve the exact published module without modifying its RPATH.
    if path == appdir / "usr/lib/observatory-core/clr/libcoreclrtraceptprovider.so" and missing == ["liblttng-ust.so.0"]:
        optional_diagnostics.append({"module": str(path.relative_to(appdir)), "missing": missing, "purpose": "Optional .NET LTTng diagnostics; not used by CLR criteria evaluation"})
    elif missing:
        raise SystemExit(f"Unresolved dependency for {path}:\n{result.stdout}")
    for line in result.stdout.splitlines():
        match = re.match(r"\s*(\S+) => (/\S+) \(", line)
        if not match:
            continue
        soname, source = match.group(1), Path(match.group(2))
        if base.match(soname) or host_graphics.fullmatch(soname):
            host_dependencies.add(soname)
            continue
        if source.is_relative_to(appdir):
            continue
        target = libdir / soname
        origins.add(str(source))
        if not target.exists():
            shutil.copy2(source.resolve(), target)
            queue.append(target)

versions = set()
for path in sorted(seen):
    result = subprocess.run(["readelf", "--version-info", str(path)], capture_output=True, text=True, check=True)
    versions.update(re.findall(r"\bGLIBC_(\d+\.\d+(?:\.\d+)?)\b", result.stdout))
    if any(item["module"] == str(path.relative_to(appdir)) for item in optional_diagnostics):
        continue
    relative = os.path.relpath(libdir, path.parent)
    paths = ["$ORIGIN", "$ORIGIN/" + relative]
    existing = subprocess.run([patchelf, "--print-rpath", str(path)], capture_output=True, text=True)
    if existing.returncode:
        # The AppImage updater may contain a static executable.
        continue
    paths += [part for part in existing.stdout.strip().split(":") if part.startswith("$ORIGIN")]
    subprocess.run([patchelf, "--set-rpath", ":".join(dict.fromkeys(paths)), str(path)], check=True)

maximum = max(versions, key=lambda value: tuple(map(int, value.split("."))), default="0.0")
if tuple(map(int, maximum.split("."))) > (2, 35):
    raise SystemExit("AppImage requires GLIBC " + maximum + "; build on Ubuntu 22.04 (GLIBC 2.35)")

# Retain distributor copyright/license notices alongside the existing app docs.
licenses = appdir / "usr/share/doc/observatory-core/dependencies"
licenses.mkdir(parents=True, exist_ok=True)
packages = set()
packages.update({"adwaita-icon-theme", "hicolor-icon-theme", "fonts-dejavu-core", "fonts-dejavu-extra", "fonts-noto-color-emoji", "fonts-ubuntu", "fontconfig", "fontconfig-config", "espeak-ng", "espeak-ng-data", "pulseaudio-utils", "shared-mime-info", "ca-certificates", "tzdata", "gstreamer1.0-plugins-base", "gstreamer1.0-plugins-good", "libgtk-3-0", "libwebkit2gtk-4.1-0"})
if origins:
    distribution_paths = set(origins)
    for source in origins:
        if source.startswith("/lib/"):
            distribution_paths.add("/usr" + source)
        elif source.startswith("/usr/lib/"):
            distribution_paths.add(source[4:])
    result = subprocess.run(["dpkg-query", "-S", *sorted(distribution_paths)], capture_output=True, text=True)
    for line in result.stdout.splitlines():
        if ": /" in line:
            packages.add(line.split(": /", 1)[0].split(":", 1)[0])
for package in sorted(packages):
    copyright_file = Path("/usr/share/doc") / package / "copyright"
    if copyright_file.is_file():
        shutil.copy2(copyright_file, licenses / (package + ".copyright"))

receipt = {
    "format": 1,
    "architecture": "x86_64",
    "maximum_glibc_version": maximum,
    "bundled_elf_files": len(seen),
    "host_glibc_sonames": sorted(name for name in host_dependencies - host_cpp if not host_graphics.fullmatch(name)),
    "host_cpp_sonames": sorted(host_dependencies & host_cpp),
    "host_cpp_runtime": "The main payload uses host libstdc++.so.6 and libgcc_s.so.1 to remain compatible with host graphics drivers.",
    "host_graphics_sonames": sorted(name for name in host_dependencies if host_graphics.fullmatch(name)),
    "additional_dependency_origins": sorted(origins),
    "distribution_packages": sorted(packages),
    "optional_diagnostics": optional_diagnostics,
    "optional_html_sandbox_helpers": {
        "host_paths": ["/usr/bin/bwrap", "/usr/bin/xdg-dbus-proxy"],
        "reason": "Distribution WebKit uses absolute sandbox helper paths; the optional HTML view needs these host executables. The sandbox remains enabled.",
    },
    "explicit_dlopen_graphics_dispatchers": ["libGL.so.1", "libEGL.so.1", "libGLESv2.so.2"],
    "graphics": "Bundled GLVND dispatchers use the host graphics driver, libdrm with its chipset libraries, libgbm and display server.",
}
(appdir / "usr/share/doc/observatory-core/appimage-dependencies.json").write_text(json.dumps(receipt, indent=2) + "\n")
print(f"Bundled {len(seen)} ELF files; maximum GLIBC requirement {maximum}")
