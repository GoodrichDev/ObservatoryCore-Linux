#!/usr/bin/env python3
"""Verify real bundled delta updates against a range-capable loopback feed."""
import argparse
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import threading
import time
from urllib.parse import urlsplit


def digest(path):
    result = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            result.update(block)
    return result.hexdigest()


def run(arguments, env, *, codes=(0,), timeout=180, cwd=None):
    result = subprocess.run([str(item) for item in arguments], env=env, cwd=cwd,
                            capture_output=True, text=True, timeout=timeout)
    if result.returncode not in codes:
        raise AssertionError(f'{arguments}: exit {result.returncode}\n'
                             f'{result.stdout[-6000:]}\n{result.stderr[-6000:]}')
    return result


def pinned_tool(name, cache, pins, override):
    expected = pins[name]
    candidates = [Path(override)] if override else [cache / f'{name}-{expected}']
    if not override:
        candidates += [cache / ('runtime-x86_64' if name == 'runtime' else f'{name}.AppImage')]
    for path in candidates:
        if path.is_file():
            assert digest(path) == expected, f'Tool SHA-256 mismatch: {path}'
            if name != 'runtime':
                assert os.access(path, os.X_OK), f'Tool is not executable: {path}'
            return path.resolve()
    raise AssertionError(f'Pinned {name} is absent from {cache}; run package-appimage.sh first')


class Feed(ThreadingHTTPServer):
    daemon_threads = True

    def __init__(self, directory):
        super().__init__(('127.0.0.1', 0), FeedHandler)
        self.directory = directory
        self.missing = False
        self.image_bytes = 0
        self.range_requests = 0
        self.multipart_requests = 0
        self.lock = threading.Lock()

    def counters(self):
        with self.lock:
            return {'image_bytes_served': self.image_bytes,
                    'range_get_requests': self.range_requests,
                    'multipart_get_requests': self.multipart_requests}


class FeedHandler(BaseHTTPRequestHandler):
    protocol_version = 'HTTP/1.1'

    def log_message(self, *_args):
        pass

    def do_HEAD(self):
        self.serve(head=True)

    def do_GET(self):
        self.serve(head=False)

    def serve(self, *, head):
        name = urlsplit(self.path).path.removeprefix('/')
        if name not in {'new.AppImage', 'new.AppImage.zsync'} or self.server.missing:
            self.send_error(404, 'Local update feed is unavailable')
            return
        path = self.server.directory / name
        if not path.is_file():
            self.send_error(404)
            return
        size = path.stat().st_size
        ranges = []
        requested = self.headers.get('Range')
        if requested:
            if not requested.startswith('bytes='):
                self.send_error(400, 'Unsupported range unit')
                return
            for item in requested[6:].split(','):
                match = re.fullmatch(r'\s*(\d*)-(\d*)\s*', item)
                if not match or not any(match.groups()):
                    self.send_error(400, 'Malformed byte range')
                    return
                first, last = match.groups()
                if first:
                    start, end = int(first), min(int(last) if last else size - 1, size - 1)
                else:
                    start, end = max(0, size - int(last)), size - 1
                if start <= end and start < size:
                    ranges.append((start, end))
            if not ranges:
                self.send_response(416)
                self.send_header('Content-Range', f'bytes */{size}')
                self.send_header('Content-Length', '0')
                self.end_headers()
                return
        boundary = 'observatory-local-delta-boundary'
        parts = []
        if len(ranges) > 1:
            for start, end in ranges:
                header = (f'--{boundary}\r\nContent-Type: application/octet-stream\r\n'
                          f'Content-Range: bytes {start}-{end}/{size}\r\n\r\n').encode()
                parts.append((header, start, end))
            closing = f'--{boundary}--\r\n'.encode()
            length = sum(len(header) + end - start + 1 + 2 for header, start, end in parts) + len(closing)
            content_type = f'multipart/byteranges; boundary={boundary}'
        else:
            start, end = ranges[0] if ranges else (0, size - 1)
            parts = [(b'', start, end)]
            closing = b''
            length = end - start + 1
            content_type = 'application/octet-stream'
        self.send_response(206 if ranges else 200)
        self.send_header('Accept-Ranges', 'bytes')
        self.send_header('Content-Type', content_type)
        self.send_header('Content-Length', str(length))
        if len(ranges) == 1:
            self.send_header('Content-Range', f'bytes {ranges[0][0]}-{ranges[0][1]}/{size}')
        self.end_headers()
        if head:
            return
        if name == 'new.AppImage' and ranges:
            with self.server.lock:
                self.server.range_requests += 1
                self.server.multipart_requests += int(len(ranges) > 1)
        with path.open('rb') as source:
            for header, start, end in parts:
                self.wfile.write(header)
                source.seek(start)
                remaining = end - start + 1
                while remaining:
                    block = source.read(min(128 * 1024, remaining))
                    assert block, 'Unexpected end of local feed image'
                    self.wfile.write(block)
                    remaining -= len(block)
                    if name == 'new.AppImage':
                        with self.server.lock:
                            self.server.image_bytes += len(block)
                if len(ranges) > 1:
                    self.wfile.write(b'\r\n')
            self.wfile.write(closing)


def isolated_env(root):
    env = dict(os.environ)
    for key in list(env):
        if key.lower().endswith('_proxy') or key in {
            'APPIMAGE', 'APPDIR', 'ARGV0', 'TARGET_APPIMAGE', 'LD_LIBRARY_PATH', 'LD_PRELOAD',
            'DISPLAY', 'WAYLAND_DISPLAY', 'WAYLAND_SOCKET', 'DBUS_SESSION_BUS_ADDRESS',
            'OBSERVATORY_APPIMAGE_ENV_SAVED', 'APPIMAGE_EXTRACT_AND_RUN', 'NO_CLEANUP',
        }:
            env.pop(key, None)
    for name in ['home', 'config', 'cache', 'data', 'tmp']:
        (root / name).mkdir()
    env.update(HOME=str(root / 'home'), XDG_CONFIG_HOME=str(root / 'config'),
               XDG_CACHE_HOME=str(root / 'cache'), XDG_DATA_HOME=str(root / 'data'),
               TMPDIR=str(root / 'tmp'), NO_PROXY='127.0.0.1,localhost',
               PULSE_SERVER='unix:/nonexistent-observatory-update-audio')
    return env


def clone_appdir(source, target):
    def link_or_copy(first, second):
        try:
            os.link(first, second)
        except OSError:
            shutil.copy2(first, second)
    shutil.copytree(source, target, symlinks=True, copy_function=link_or_copy)
    # appimagetool writes desktop version metadata. Detach every desktop file,
    # including symlinks, before building so the preserved source stays intact.
    detached = list(target.rglob('*.desktop')) + [
        target / 'AppRun', target / 'usr/lib/observatory-core/appimage-manage.sh']
    for path in detached:
        if path.is_file():
            private = path.with_name(path.name + '.delta-private')
            shutil.copy2(path, private)
            path.unlink()
            private.replace(path)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--appdir', type=Path)
    parser.add_argument('--image', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if not (args.appdir or args.image):
        parser.error('Pass --appdir or --image')
    assert os.uname().sysname == 'Linux' and os.uname().machine == 'x86_64'
    repository = Path(__file__).resolve().parents[2]
    lock = repository / 'native/packaging/appimage/tools.lock'
    pins = {fields[0]: fields[1] for line in lock.read_text().splitlines()
            if line and not line.startswith('#') for fields in [line.split('|')]}
    cache = Path(os.environ.get('OBSERVATORY_APPIMAGE_TOOLS_CACHE',
                               str(Path(os.environ.get('XDG_CACHE_HOME', str(Path.home() / '.cache')))
                                   / 'observatory-appimage-tools')))
    image_tool = pinned_tool('appimagetool', cache, pins, os.environ.get('APPIMAGETOOL_APPIMAGE'))
    runtime = pinned_tool('runtime', cache, pins, os.environ.get('APPIMAGE_RUNTIME'))
    started = time.monotonic()
    report = {'passed': False, 'user_desktop_accessed': False, 'physical_audio_connected': False,
              'feed_host': '127.0.0.1', 'tool_sha256': pins['appimagetool'],
              'runtime_sha256': pins['runtime']}
    with tempfile.TemporaryDirectory(prefix='observatory-appimage-delta-') as temporary:
        root = Path(temporary)
        env = isolated_env(root)
        tool_root = root / 'builder'
        tool_root.mkdir()
        run([image_tool, '--appimage-extract'], env, cwd=tool_root)
        tool = tool_root / 'squashfs-root/AppRun'
        env['PATH'] = f'{tool.parent / "usr/bin"}:{env["PATH"]}'
        source = args.appdir.resolve() if args.appdir else root / 'squashfs-root'
        if not args.appdir:
            run([args.image.resolve(), '--appimage-extract'], env, cwd=root)
        for relative in ['AppRun', 'usr/bin/observatory-core',
                         'usr/lib/observatory-core/clr/libcoreclr.so',
                         'usr/lib/observatory-core/appimage-update/AppRun']:
            assert (source / relative).is_file(), f'Incomplete real AppDir: {relative}'
        controls = {
            'AppRun': repository / 'native/packaging/appimage/AppRun',
            'usr/lib/observatory-core/appimage-manage.sh': repository / 'native/packaging/appimage/install.sh',
        }
        control_hashes = {relative: digest(path) for relative, path in controls.items()}
        for relative, expected in control_hashes.items():
            assert digest(source / relative) == expected, f'Stale packaged launcher: {relative}'
        desktop_hashes = {path: digest(path) for path in source.rglob('*.desktop') if path.is_file()}
        appdir = root / 'Complete Observatory.AppDir'
        clone_appdir(source, appdir)
        feed_dir = root / 'feed'
        feed_dir.mkdir()
        marker = appdir / 'usr/share/observatory-core-delta-test-version'
        if marker.exists() or marker.is_symlink():
            marker.unlink()
        with Feed(feed_dir) as feed:
            thread = threading.Thread(target=feed.serve_forever, daemon=True)
            thread.start()
            try:
                update_info = f'zsync|http://127.0.0.1:{feed.server_port}/new.AppImage.zsync'
                build_env = dict(env, ARCH='x86_64', VERSION='delta-test')
                build_env.pop('SOURCE_DATE_EPOCH', None)
                for name, content in [('old', 'old version\n'), ('new', 'new version\n')]:
                    print(f'Building full {name} AppImage for the local delta test...', file=sys.stderr, flush=True)
                    marker.write_text(content)
                    os.utime(marker, (1700000000, 1700000000))
                    run([tool, '--runtime-file', runtime, '--updateinformation', update_info,
                         '--file-url', 'new.AppImage', '--mksquashfs-opt', '-processors',
                         '--mksquashfs-opt', '2', '--mksquashfs-opt', '-all-time',
                         '--mksquashfs-opt', '1700000000', '--mksquashfs-opt', '-mkfs-time',
                         '--mksquashfs-opt', '1700000000', appdir, feed_dir / f'{name}.AppImage'],
                        build_env, cwd=feed_dir, timeout=240)
                old, new = feed_dir / 'old.AppImage', feed_dir / 'new.AppImage'
                assert new.with_suffix('.AppImage.zsync').is_file()
                old_hash, new_hash = digest(old), digest(new)
                assert old_hash != new_hash, 'Version marker did not change the actual image'
                target = root / 'Installed image with spaces.AppImage'
                current = root / 'Current image.AppImage'
                shutil.copy2(old, target)
                shutil.copy2(new, current)
                for image in [target, current]:
                    image.chmod(0o755)

                def action(image, option, codes):
                    return run([appdir / 'AppRun', option],
                               dict(env, APPIMAGE=str(image), APPDIR=str(appdir)), codes=codes)

                assert action(current, '--check-for-update', (0,)).returncode == 0
                assert action(target, '--check-for-update', (1,)).returncode == 1
                print('Applying real bundled delta update over loopback HTTP...', file=sys.stderr, flush=True)
                before = feed.counters()
                action(target, '--update', (0,))
                after = feed.counters()
                transferred = after['image_bytes_served'] - before['image_bytes_served']
                range_gets = after['range_get_requests'] - before['range_get_requests']
                assert digest(target) == new_hash, 'Updater did not install the exact new image'
                backup = Path(str(target) + '.zs-old')
                assert backup.is_file() and digest(backup) == old_hash, 'Old image rollback copy differs'
                assert action(target, '--check-for-update', (0,)).returncode == 0
                assert range_gets > 0, 'Updater made no real HTTP Range GET request'
                assert 0 < transferred < new.stat().st_size, (transferred, new.stat().st_size)
                failure_target = root / 'Missing feed image.AppImage'
                shutil.copy2(old, failure_target)
                feed.missing = True
                assert action(failure_target, '--check-for-update', (2,)).returncode == 2
                failure = action(failure_target, '--update', (1, 2))
                assert digest(failure_target) == old_hash, 'Failed update changed the old image'
                assert not Path(str(failure_target) + '.zs-old').exists()
                assert all(digest(path) == expected for path, expected in desktop_hashes.items()), 'Builder changed source desktop files'
                report.update(passed=True, old_sha256=old_hash, new_sha256=new_hash,
                              full_image_bytes=new.stat().st_size, delta_image_bytes=transferred,
                              range_get_requests=range_gets,
                              multipart_get_requests=after['multipart_get_requests'] - before['multipart_get_requests'],
                              downloaded_fraction=round(transferred / new.stat().st_size, 6),
                              status_codes={'current': 0, 'available': 1, 'missing_feed': 2,
                                            'missing_feed_update': failure.returncode},
                              target_matches_new=True, backup_matches_old=True,
                              failed_update_preserved_old=True, source_desktop_files_unchanged=True,
                              launcher_source_sha256=control_hashes)
            finally:
                feed.shutdown()
                thread.join(timeout=5)
    report['elapsed_seconds'] = round(time.monotonic() - started, 3)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    staged = args.output.with_name(args.output.name + '.tmp')
    staged.write_text(json.dumps(report, indent=2) + '\n')
    staged.replace(args.output)
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
