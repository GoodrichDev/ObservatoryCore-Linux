#!/usr/bin/env python3
"""Real native playback through a private PulseAudio null sink and monitor.

This deliberately makes no physical sound and accesses no desktop windows or
player files. The production audio helpers run unchanged. Transparent exec
wrappers preserve their PID/exit/cancellation behavior while copying rendered
WAVs and recording argv. The monitor receives actual server-rendered PCM, not
the input WAV or a mocked player. Run after cargo test --no-run --message-format=json.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time
import wave

import numpy as np

WRAPPER = r'''#!/usr/bin/python3
import json,os,pathlib,shutil,sys,time
folder=pathlib.Path(os.environ['OBSERVATORY_AUDIO_ACCEPTANCE'])
name=pathlib.Path(sys.argv[0]).name
event={'helper':name,'pid':os.getpid(),'unix_seconds':time.time(),'args':sys.argv[1:],'phase':(folder/'phase.txt').read_text()}
if name=='paplay':
 source=pathlib.Path(sys.argv[-1]); copied=folder/'rendered'/(str(time.time_ns())+'-'+str(os.getpid())+'.wav')
 shutil.copyfile(source,copied);event['rendered']=str(copied);event['source_name']=source.name
fd=os.open(str(folder/'helper-events.jsonl'),os.O_WRONLY|os.O_CREAT|os.O_APPEND,0o600)
os.write(fd,(json.dumps(event)+'\n').encode());os.close(fd)
os.execv('/usr/bin/'+name,[name,*sys.argv[1:]])
'''

APP_DRIVER = r'''return {id='acceptance.audio',name='Private Audio Acceptance',version='1',
 on_load=function() core.set_storage('ready',false) end,
 on_state_changed=function(s) if s.state=='Monitoring' then core.set_storage('ready',true) end end,
 on_status=function(s)
  if s.AudioPhase==1 then
   core.notify({id='app-speech',title='Private native application',detail='Journal dispatch reaches real audio playback',popup=false,
    title_ssml='<speak><voice name="Old">Private native application</voice></speak>',
    detail_ssml='<speak>Journal dispatch reaches real audio playback.</speak>'})
   core.play_audio_file('app.wav',{request_id='app-file',Instant=true})
  end
 end,
 on_audio_complete=function(e) core.set_storage('completion',e) end
}'''


def wait(predicate, label, seconds=15):
    started = time.monotonic()
    while time.monotonic() - started < seconds:
        result = predicate()
        if result:
            return result
        time.sleep(.01)
    raise AssertionError('Timed out: ' + label)


def tone(path, frequency, seconds):
    samples = np.arange(round(seconds * 48000)) / 48000
    values = (.16 * np.sin(2 * np.pi * frequency * samples) * 32767).astype('<i2')
    with wave.open(str(path), 'wb') as stream:
        stream.setnchannels(1)
        stream.setsampwidth(2)
        stream.setframerate(48000)
        stream.writeframes(values.tobytes())


def pcm_statistics(path, allow_empty=False):
    values = np.fromfile(path, dtype='<i2').reshape(-1, 2).astype(np.float64) / 32768
    assert allow_empty or values.shape[0] > 100, ('empty monitor capture', str(path))
    mono = values.mean(axis=1)
    return {'frames': len(values), 'seconds': len(values) / 48000,
            'nonzero_samples': int(np.count_nonzero(values)),
            'peak': float(np.abs(values).max()) if values.size else 0.0,
            'rms': float(np.sqrt(np.mean(values ** 2))) if values.size else 0.0}, mono


def main():
    parser = argparse.ArgumentParser()
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument('--test-binary', type=Path)
    group.add_argument('--build-log', type=Path)
    parser.add_argument('--app-binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert os.name == 'posix'
    for helper in ['pulseaudio', 'pactl', 'parec', 'paplay', 'espeak-ng']:
        assert shutil.which(helper), 'Missing real runtime helper: ' + helper
    if args.build_log:
        artifacts = [json.loads(s) for s in args.build_log.read_text().splitlines() if s.startswith('{')]
        args.test_binary = Path(next(x['executable'] for x in reversed(artifacts) if x.get('executable') and x.get('profile', {}).get('test')))
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    rendered = output / 'rendered'
    rendered.mkdir()
    for name, frequency, seconds in [('a', 523, 1.6), ('b', 659, .4), ('instant', 880, .35), ('long', 440, 20), ('app', 997, .6)]:
        tone(output / (name + '.wav'), frequency, seconds)
    children = []
    monitor = None
    monitor_file = None
    phases = {}
    observed_helpers = {}
    with tempfile.TemporaryDirectory(prefix='observatory-audio-acceptance-') as private:
        private = Path(private)
        private.chmod(0o700)
        (private / 'runtime').mkdir(mode=0o700)
        (private / 'bin').mkdir()
        for helper in ['paplay', 'espeak-ng']:
            (private / 'bin' / helper).write_text(WRAPPER)
            (private / 'bin' / helper).chmod(0o700)
        server = 'unix:' + str(private / 'pulse.sock')
        env = dict(os.environ, PULSE_SERVER=server, PULSE_SINK='observatory_qa_null',
                   PULSE_SOURCE='observatory_qa_null.monitor', PULSE_RUNTIME_PATH=str(private / 'runtime'),
                   OBSERVATORY_AUDIO_ACCEPTANCE=str(output), PATH=str(private / 'bin') + ':' + os.environ['PATH'])
        config = private / 'pulse.pa'
        config.write_text(f'load-module module-native-protocol-unix socket={private / "pulse.sock"} auth-anonymous=1\n'
                          'load-module module-null-sink sink_name=observatory_qa_null rate=48000 channels=2\n'
                          'set-default-sink observatory_qa_null\nset-default-source observatory_qa_null.monitor\n')
        pulse_log = (output / 'pulse.log').open('w')
        pulse = subprocess.Popen(['/usr/bin/pulseaudio', '-n', '--daemonize=no', '--exit-idle-time=-1', '--use-pid-file=no', '-F', str(config)], env=env, stdout=pulse_log, stderr=subprocess.STDOUT)
        children.append(pulse)
        try:
            def pulse_ready():
                return subprocess.run(['pactl', 'info'], env=env, capture_output=True).returncode == 0
            wait(pulse_ready, 'private PulseAudio socket')
            sinks = subprocess.check_output(['pactl', 'list', 'short', 'sinks'], env=env, text=True)
            modules = subprocess.check_output(['pactl', 'list', 'short', 'modules'], env=env, text=True)
            assert len(sinks.strip().splitlines()) == 1 and 'module-null-sink.c' in sinks, sinks
            assert 'module-alsa-' not in modules and 'module-rdp-' not in modules, modules
            (output / 'server.json').write_text(json.dumps({'info':subprocess.check_output(['pactl','info'],env=env,text=True),'sinks':sinks,'modules':modules,'physical_output_connected':False},indent=2)+'\n')

            def stop_monitor():
                nonlocal monitor, monitor_file
                if monitor is not None:
                    monitor.terminate()
                    monitor.wait(timeout=5)
                    monitor_file.close()
                    monitor = monitor_file = None

            def start_monitor(name):
                nonlocal monitor, monitor_file
                stop_monitor()
                raw = output / (name + '.raw')
                monitor_file = raw.open('wb')
                monitor = subprocess.Popen(['parec', '--device=observatory_qa_null.monitor', '--format=s16le', '--rate=48000', '--channels=2', '--latency-msec=20', '--raw'], env=env, stdout=monitor_file, stderr=subprocess.PIPE)
                children.append(monitor)
                time.sleep(.25)
                assert monitor.poll() is None, monitor.stderr.read().decode()
                phases[name] = {'started_unix_seconds':time.time(), 'raw':str(raw)}

            log = (output / 'probe.log').open('w')
            probe = subprocess.Popen([str(args.test_binary.resolve()), '--exact', 'notifications::actual_audio_acceptance::real_helpers_private_pulse_output_queue_and_cancellation', '--ignored', '--nocapture'], env=env, stdout=log, stderr=subprocess.STDOUT)
            children.append(probe)
            seen = None
            deadline = time.monotonic() + 110
            while probe.poll() is None and time.monotonic() < deadline:
                trace = output / 'helper-events.jsonl'
                if trace.exists():
                    for line in trace.read_text().splitlines():
                        helper = json.loads(line)
                        if helper['helper'] != 'paplay':
                            continue
                        pid = helper['pid']
                        executable = Path('/proc') / str(pid) / 'exe'
                        try:
                            if executable.samefile('/usr/bin/paplay'):
                                observed_helpers[str(pid)] = {'executable':str(executable.resolve()), 'unix_seconds':time.time()}
                        except FileNotFoundError:
                            pass
                marker = output / 'phase.txt'
                name = marker.read_text() if marker.exists() else None
                if name and name != seen:
                    start_monitor(name)
                    (output / 'phase-ready.txt').write_text(name)
                    seen = name
                    print(json.dumps({'phase':name,'monitor':'private null sink'}), flush=True)
                time.sleep(.01)
            assert probe.poll() is not None, 'real audio probe timed out'
            stop_monitor()
            log.close()
            assert probe.returncode == 0, (output / 'probe.log').read_text()

            # Full production journal -> Lua extension -> notification/audio path.
            profile = private / 'app'
            journals = profile / 'journals'
            journals.mkdir(parents=True)
            (journals / 'Journal.2026-10-26T000000.01.log').write_text(json.dumps({'timestamp':'2026-10-26T00:00:00Z','event':'Location','StarSystem':'Private audio fixture','SystemAddress':42})+'\n')
            status = journals / 'Status.json'
            status.write_text(json.dumps({'timestamp':'2026-10-26T00:00:00Z','event':'Status','AudioPhase':0}))
            appenv = dict(env)
            for kind in ['CONFIG','DATA','CACHE']:
                appenv['XDG_'+kind+'_HOME'] = str(profile / kind.lower())
            plugins = profile / 'data' / 'observatory-core' / 'plugins'
            plugins.mkdir(parents=True)
            (plugins / 'acceptance.lua').write_text(APP_DRIVER)
            storage = profile / 'data' / 'observatory-core' / 'storage' / 'acceptance.audio'
            storage.mkdir(parents=True)
            shutil.copyfile(output / 'app.wav', storage / 'app.wav')
            settings = profile / 'settings.json'
            settings.write_text(json.dumps({'journal_dir':str(journals),'popup':False,'speech':True,'auto_start':False,'poll_interval_ms':50,
                'audio_device':'observatory_qa_null','AudioVolume':.25,'speech_voice':'en-us','speech_rate':220,
                'disabled_plugins':['StellarOverlay','Hecate','BioInsights','Evaluator']}))
            (output / 'phase.txt').write_text('application-dispatch')
            start_monitor('application-dispatch')
            app_log = (output / 'application.log').open('w')
            app = subprocess.Popen([str(args.app_binary.resolve()),'--config',str(settings),'--headless','--watch-seconds','10'],env=appenv,stdout=app_log,stderr=subprocess.STDOUT)
            children.append(app)
            state = storage / 'state.json'
            wait(lambda:state.exists() and json.loads(state.read_text()).get('ready'), 'native application monitoring')
            temporary = status.with_suffix('.new')
            temporary.write_text(json.dumps({'timestamp':'2026-10-26T00:00:01Z','event':'Status','AudioPhase':1}))
            temporary.replace(status)
            app.wait(timeout=15)
            app_log.close()
            stop_monitor()
            assert app.returncode == 0, (output / 'application.log').read_text()
            appstate = json.loads(state.read_text())
            assert appstate['completion']['ok'] and not appstate['completion']['skipped'], appstate
            (output / 'application-state.json').write_text(json.dumps(appstate,indent=2)+'\n')
            appresult = json.loads((output / 'application.log').read_text())
            assert not appresult['errors'], appresult['errors']
            helpers = [json.loads(line) for line in (output/'helper-events.jsonl').read_text().splitlines()]
            playback = [v for v in helpers if v['helper']=='paplay']
            probe_result = json.loads((output/'probe.json').read_text())
            cancelled_helpers = [v for v in playback if v['phase']=='cancellation']
            assert len(cancelled_helpers)==1 and str(cancelled_helpers[0]['pid']) in observed_helpers, cancelled_helpers
            assert probe_result['helper_pid_reaped'] and probe_result['shutdown_ms'] < 1000, probe_result
            assert len(probe_result['cancelled'])==1 and probe_result['cancelled'][0]['cancelled'] and not probe_result['cancelled'][0]['ok'], probe_result
            wait(lambda:all(not (Path('/proc')/str(v['pid'])).exists() for v in playback), 'all playback helpers reaped')
            sink_inputs = subprocess.check_output(['pactl','list','short','sink-inputs'],env=env,text=True)
            assert not sink_inputs.strip(), ('playback did not become quiescent',sink_inputs)
            for phase_name, value in phases.items():
                stats, mono = pcm_statistics(Path(value['raw']), allow_empty=phase_name in ['missing-and-invalid','cancellation'])
                value.update(stats)
                if phase_name not in ['missing-and-invalid','cancellation']:
                    assert stats['nonzero_samples'] > 500 and stats['peak'] > .001, (phase_name, stats)
                if phase_name == 'cancellation':
                    value['cancel_before_first_monitor_buffer'] = stats['frames']==0
                    value['cancel_before_first_nonzero_pcm'] = stats['nonzero_samples']==0
                    value['observed_real_helper'] = observed_helpers[str(cancelled_helpers[0]['pid'])]
                if phase_name == 'queued-instant':
                    spectrum = np.abs(np.fft.rfft(mono))
                    frequencies = np.fft.rfftfreq(len(mono),1/48000)
                    bands = {str(hz):float(spectrum[np.abs(frequencies-hz)<3].max()) for hz in [523,659,880]}
                    assert all(v > 10 for v in bands.values()), bands
                    value['expected_tone_band_magnitudes'] = bands
                with wave.open(str(output/(phase_name+'-monitor.wav')),'wb') as capture:
                    capture.setnchannels(2);capture.setsampwidth(2);capture.setframerate(48000)
                    capture.writeframes(Path(value['raw']).read_bytes())
                Path(value['raw']).unlink()
                value.pop('raw')
            speech = [v for v in helpers if v['helper']=='espeak-ng']
            assert len(speech)==2, speech
            assert '-m' in speech[0]['args'] and "name='en-us'" in speech[0]['args'][-1], speech[0]
            assert len(playback)==14, len(playback)
            report = {'scope':'production native helpers plus full headless journal dispatch; private PulseAudio null sink',
                'physical_audio_output_tested':False,'actual_game_tested':False,'desktop_accessed_by_this_script':False,
                'test_binary_sha256':hashlib.sha256(args.test_binary.read_bytes()).hexdigest(),
                'app_binary_sha256':hashlib.sha256(args.app_binary.read_bytes()).hexdigest(),
                'helper_playbacks':len(playback),'speech_syntheses':len(speech),'monitor_phases':phases,
                'probe':probe_result,'application_completion':appstate['completion'],
                'playback_quiescent':True,'live_playback_helpers_after':0,
                'no_player_files_accessed':True}
            (output/'result.json').write_text(json.dumps(report,indent=2)+'\n')
            print(json.dumps(report),flush=True)
        finally:
            if monitor is not None:
                monitor.terminate();monitor.wait(timeout=5)
                monitor_file.close()
            for child in reversed(children):
                if child.poll() is None:
                    child.terminate()
                    try:child.wait(timeout=5)
                    except subprocess.TimeoutExpired:child.kill();child.wait()
            pulse_log.close()


if __name__ == '__main__':
    main()
