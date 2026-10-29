#!/usr/bin/env python3
"""Exercise the actual AppImage, managed install and isolated desktop payload."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile


def run(arguments, env, *, codes=(0,), timeout=90, cwd=None):
    command = [str(item) for item in arguments]
    process = subprocess.Popen(command, env=env, cwd=cwd, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, text=True, start_new_session=True)
    try:
        stdout, stderr = process.communicate(timeout=timeout)
        result = subprocess.CompletedProcess(command, process.returncode, stdout, stderr)
    finally:
        # Font discovery helpers can briefly outlive the test window. Stop only
        # this command's owned process group before removing its private cache.
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        process.wait()
    assert result.returncode in codes, (arguments, result.returncode, result.stdout[-4000:], result.stderr[-4000:])
    return result


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest() if hasattr(hashlib, 'file_digest') else hashlib.sha256(stream.read()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('image', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    image = args.image.resolve()
    scripts = Path(__file__).resolve().parent
    installer = scripts.parent / 'packaging/appimage/install.sh'
    update_info = run([image, '--appimage-updateinformation'], dict(os.environ)).stdout.strip()
    assert update_info == 'gh-releases-zsync|GoodrichDev|ObservatoryCore-Linux|continuous|Observatory_Core-x86_64.AppImage.zsync', update_info
    assert image.with_suffix(image.suffix + '.zsync').is_file()
    report = {'appimage_sha256': digest(image), 'update_information': update_info,
              'user_desktop_accessed': False, 'physical_audio_connected': False}
    with tempfile.TemporaryDirectory(prefix='observatory-appimage-verify-') as temporary:
        root = Path(temporary)
        home = root / "Home with spaces & 'quotes'"
        for relative in ['.local/bin', '.config/observatory-core', 'cache', 'tmp', 'journals']:
            (home / relative).mkdir(parents=True, exist_ok=True)
        sentinel = home / '.local/bin/observatory-core'
        sentinel.write_text('existing non-AppImage installation\n')
        preferences = home / '.config/observatory-core/settings.json'
        preferences.write_text('{"preserved":true}\n')
        env = dict(os.environ, HOME=str(home), XDG_DATA_HOME=str(home / '.local/share'),
                   XDG_CONFIG_HOME=str(home / '.config'), XDG_CACHE_HOME=str(home / 'cache'),
                   TMPDIR=str(home / 'tmp'), APPIMAGE_EXTRACT_AND_RUN='1', NO_CLEANUP='1',
                   # An unset bus permits auto-launching desktop portals, whose
                   # document FUSE mounts can outlive this isolated test window.
                   DBUS_SESSION_BUS_ADDRESS='unix:path=/nonexistent-observatory-appimage-bus',
                   PULSE_SERVER='unix:/nonexistent-observatory-appimage-audio')
        for key in ['APPIMAGE', 'APPDIR', 'OBSERVATORY_CLR_CRITERIA_HOST', 'DISPLAY',
                    'WAYLAND_DISPLAY', 'WAYLAND_SOCKET']:
            env.pop(key, None)
        version = run([image, '--version'], env).stdout.strip()
        assert version.startswith('observatory-core '), version
        extracted = list((home / 'tmp').glob('appimage_extracted_*'))
        assert len(extracted) == 1, extracted
        appdir = extracted[0]
        assert (appdir / 'usr/lib/observatory-core/clr/observatory-clr-criteria').is_file()
        updater = appdir / 'usr/lib/observatory-core/appimage-update'
        run([updater / 'usr/bin/appimageupdatetool', '--help'], dict(env, APPDIR=str(updater),
            LD_LIBRARY_PATH=str(updater / 'usr/lib') + ':' + str(updater / 'usr/lib/x86_64-linux-gnu')))
        report['version'] = version
        report['bundled_updater_runs'] = True

        criteria = home / 'typed criteria.lua'
        criteria.write_text("""---@Global
Math = luanet.import_type('System.Math')
---@End
---@Complex bundled-clr
return true, 'AppImage CLR ' .. scan:GetType().Name, tostring(Math.Sqrt(81))
---@End
""")
        journal = home / 'journals/Journal.2026-10-28T000000.01.log'
        events = [{'timestamp': '2026-10-28T00:00:00Z', 'event': 'Location', 'StarSystem': 'AppImage Probe', 'SystemAddress': 42},
                  {'timestamp': '2026-10-28T00:00:01Z', 'event': 'Scan', 'StarSystem': 'AppImage Probe', 'SystemAddress': 42,
                   'BodyID': 1, 'BodyName': 'AppImage Probe 1', 'Radius': 1000000}]
        journal.write_text(''.join(json.dumps(event) + '\n' for event in events))
        config = home / 'probe-settings.json'
        settings = {'journal_dir': str(journal.parent), 'auto_start': False, 'popup': False, 'speech': False,
                    'disabled_plugins': ['BioInsights', 'Evaluator', 'Hecate', 'StellarOverlay'],
                    'plugins': {'Explorer': {'EnableCustomCriteria': True, 'CustomCriteriaFile': str(criteria)}}}
        config.write_text(json.dumps(settings))
        replay = json.loads(run([image, '--headless', '--config', config], env).stdout)
        assert replay['events'] == 2 and not replay['errors'], (replay['events'], replay['errors'])
        assert any(row['title'] == 'AppImage CLR Scan' and float(row['detail']) == 9 for row in replay['findings']
                   if row['title'] == 'AppImage CLR Scan'), replay['findings']
        report['real_clr_criteria'] = True

        installed_image = home / '.local/share/observatory-core/appimage/Observatory_Core-x86_64.AppImage'
        launcher = home / '.local/bin/observatory-core-appimage'
        desktop = home / '.local/share/applications/observatory-core-appimage.desktop'
        run(['sh', installer, '--from', image], env, timeout=180)
        assert digest(installed_image) == digest(image)
        run(['desktop-file-validate', desktop], env)
        assert run([launcher, '--version'], env, timeout=120).stdout.strip() == version
        assert sentinel.read_text() == 'existing non-AppImage installation\n'
        assert preferences.read_text() == '{"preserved":true}\n'
        # A failed download/checksum must not replace the working installation.
        fakebin = root / 'fetch-fixture'
        fakebin.mkdir()
        curl = fakebin / 'curl'
        curl.write_text("#!/usr/bin/python3\nimport pathlib,shutil,sys,os\na=sys.argv\nout=pathlib.Path(a[a.index('--output')+1])\n"
                        "shutil.copyfile(os.environ['VERIFY_APPIMAGE_SOURCE'],out) if a[-1].endswith('.AppImage') else out.write_text('0'*64+'  Observatory_Core-x86_64.AppImage\\n')\n")
        curl.chmod(0o755)
        failed = run(['sh', installer], dict(env, PATH=str(fakebin) + ':' + env['PATH'],
                                           VERIFY_APPIMAGE_SOURCE=str(image)), codes=(1, 2))
        assert 'FAILED' in failed.stdout or 'checksum' in failed.stderr.lower(), failed
        assert digest(installed_image) == digest(image)
        report['per_user_install_spaces_and_quotes'] = True
        report['failed_checksum_preserves_installation'] = True

        # Exercise the real native window on an owned virtual display. Creating
        # this display is deliberately after every networking/update operation.
        from x11_qa import PrivateDisplay
        display = PrivateDisplay(width=1280, height=1024)
        try:
            gui_env = dict(env, **{key: value for key, value in display.env.items()
                                  if key in ['DISPLAY', 'XDG_RUNTIME_DIR', 'GDK_BACKEND', 'LIBGL_ALWAYS_SOFTWARE', 'WEBKIT_DISABLE_DMABUF_RENDERER']})
            png = root / 'settings.png'
            run([launcher, '--config', config, '--view', 'Settings', '--smoke-frames', '12',
                 '--screenshot', png], gui_env, timeout=45)
            from PIL import Image
            with Image.open(png) as capture:
                assert capture.width >= 492 and capture.height >= 735
                assert len(capture.convert('RGB').getcolors(capture.width * capture.height)) > 8
            report['real_native_window_no_fuse'] = True
        finally:
            display.close()
        run(['sh', installer, '--uninstall'], env)
        assert not launcher.exists() and not desktop.exists() and not installed_image.exists()
        assert sentinel.read_text() == 'existing non-AppImage installation\n'
        assert preferences.read_text() == '{"preserved":true}\n'
        report['uninstall_preserves_user_data_and_other_installs'] = True
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
