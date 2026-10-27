//! Opt-in real audio helper acceptance. The orchestrator supplies an isolated
//! PulseAudio null sink; this probe must never play through a desktop server.
use super::*;
use std::{fs, path::PathBuf, time::SystemTime};

fn wait(mut predicate: impl FnMut() -> bool, message: &str) {
    let started = Instant::now();
    while !predicate() {
        assert!(started.elapsed() < Duration::from_secs(15), "{message}");
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[ignore = "requires native/scripts/actual_audio_acceptance.py private PulseAudio server"]
fn real_helpers_private_pulse_output_queue_and_cancellation() {
    let server = std::env::var("PULSE_SERVER").unwrap();
    assert!(server.starts_with("unix:/tmp/observatory-audio-acceptance-"));
    let directory = PathBuf::from(std::env::var("OBSERVATORY_AUDIO_ACCEPTANCE").unwrap());
    let trace = directory.join("helper-events.jsonl");
    let helpers = || -> Vec<Value> {
        fs::read_to_string(&trace)
            .unwrap_or_default()
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .filter(|v| v["helper"] == "paplay")
            .collect()
    };
    let phase = |name: &str| {
        fs::write(directory.join("phase.txt"), name).unwrap();
        wait(
            || fs::read_to_string(directory.join("phase-ready.txt")).is_ok_and(|v| v == name),
            "private monitor did not start",
        );
    };
    let finished = |count: usize| {
        wait(
            || {
                let calls = helpers();
                calls.len() >= count
                    && calls.iter().all(|v| {
                        !PathBuf::from(format!("/proc/{}", v["pid"].as_u64().unwrap())).exists()
                    })
            },
            "real paplay did not finish",
        );
    };
    let mut settings = Settings {
        popup: false,
        speech: true,
        ..Settings::default()
    };
    settings.extra.insert(
        "audio_device".into(),
        serde_json::json!("observatory_qa_null"),
    );
    settings
        .extra
        .insert("AudioVolume".into(), serde_json::json!(0.25));
    let notifications = Notifications::new();
    notifications.set_settings(&settings);
    settings
        .extra
        .insert("ChimeEnabled".into(), serde_json::json!(true));
    for index in 0..6 {
        phase(&format!("chime-{index}"));
        settings
            .extra
            .insert("ChimeSelected".into(), serde_json::json!(index));
        notifications.lifecycle(serde_json::json!({"title":"Private audio acceptance", "detail":"Embedded chime", "popup":false}), settings.clone());
        finished(index + 1);
        assert!(notifications.take_errors().is_empty());
    }
    phase("ssml");
    settings
        .extra
        .insert("ChimeEnabled".into(), serde_json::json!(false));
    settings
        .extra
        .insert("speech_voice".into(), serde_json::json!("en-us"));
    settings
        .extra
        .insert("VoiceRate".into(), serde_json::json!(2));
    settings
        .extra
        .insert("VoiceVolume".into(), serde_json::json!(70));
    notifications.lifecycle(serde_json::json!({
        "title":"Private speech acceptance", "detail":"Real generated speech", "popup":false,
        "title_ssml":"<speak><voice name='Old'>Private speech acceptance</voice></speak>",
        "detail_ssml":"<speak><voice name='Old'>The private audio sink receives real speech.</voice></speak>"
    }), settings.clone());
    finished(7);
    assert!(notifications.take_errors().is_empty());

    phase("queued-instant");
    let a = directory.join("a.wav");
    let b = directory.join("b.wav");
    let instant = directory.join("instant.wav");
    notifications
        .play_file(
            &a,
            &serde_json::json!({"request_id":"a","DeleteAfterPlay":true}),
            &settings,
        )
        .unwrap();
    wait(|| helpers().len() == 8, "first queued audio did not start");
    assert!(notifications.take_audio_completions().is_empty());
    notifications
        .play_file(&b, &serde_json::json!({"request_id":"b"}), &settings)
        .unwrap();
    settings
        .extra
        .insert("AudioVolume".into(), serde_json::json!(0.5));
    notifications.set_settings(&settings);
    notifications
        .play_file(
            &instant,
            &serde_json::json!({"request_id":"instant","Instant":true}),
            &settings,
        )
        .unwrap();
    let mut completions = Vec::new();
    let mut completion_times = Vec::new();
    wait(
        || {
            for completion in notifications.take_audio_completions() {
                completion_times.push(serde_json::json!({"event":completion,"unix_seconds":SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64()}));
                completions.push(completion);
            }
            completions.len() == 3
        },
        "real audio completions were not delivered",
    );
    assert_eq!(
        completions
            .iter()
            .map(|v| v["request_id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["instant", "a", "b"]
    );
    assert!(completions
        .iter()
        .all(|v| v["ok"] == true && v["skipped"] == false && v["cancelled"] == false));
    assert!(!a.exists() && b.exists() && instant.exists());
    finished(10);
    assert!(notifications.take_errors().is_empty());
    let calls = helpers();
    assert!(calls[7]["args"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v == "--volume=16384"));
    assert!(calls[8..10].iter().all(|v| v["args"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v == "--volume=32768")));

    phase("missing-and-invalid");
    notifications
        .play_file(
            &directory.join("missing.wav"),
            &serde_json::json!({"request_id":"missing"}),
            &settings,
        )
        .unwrap();
    let missing = notifications.take_audio_completions();
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0]["skipped"], true);
    let invalid = directory.join("invalid.wav");
    fs::write(&invalid, b"invalid audio acceptance negative control").unwrap();
    notifications
        .play_file(
            &invalid,
            &serde_json::json!({"request_id":"invalid","DeleteAfterPlay":true}),
            &settings,
        )
        .unwrap();
    let mut failed = Vec::new();
    wait(
        || {
            failed.extend(notifications.take_audio_completions());
            !failed.is_empty()
        },
        "invalid audio did not complete",
    );
    assert_eq!(failed[0]["ok"], false);
    assert!(failed[0]["error"]
        .as_str()
        .unwrap()
        .contains("Audio command exited"));
    let errors = notifications.take_errors();
    assert_eq!(errors.len(), 1);
    assert!(invalid.exists());
    drop(notifications);

    phase("cancellation");
    let cancel = Notifications::new();
    let result = cancel.completions.clone();
    let long = directory.join("long.wav");
    cancel
        .play_file(
            &long,
            &serde_json::json!({"request_id":"cancel","DeleteAfterPlay":true}),
            &settings,
        )
        .unwrap();
    wait(|| helpers().len() == 12, "long real audio did not start");
    let pid = helpers().last().unwrap()["pid"].as_u64().unwrap();
    thread::sleep(Duration::from_millis(150));
    assert!(cancel.take_audio_completions().is_empty());
    let started = Instant::now();
    drop(cancel);
    let shutdown_ms = started.elapsed().as_secs_f64() * 1000.0;
    assert!(
        shutdown_ms < 1000.0,
        "audio cancellation took {shutdown_ms}ms"
    );
    assert!(!PathBuf::from(format!("/proc/{pid}")).exists());
    assert!(long.exists());
    let cancelled = result.lock().unwrap().drain(..).collect::<Vec<_>>();
    assert_eq!(cancelled.len(), 1);
    assert_eq!(cancelled[0]["cancelled"], true);
    assert_eq!(cancelled[0]["ok"], false);
    let receipt = serde_json::json!({
        "sink":"observatory_qa_null", "embedded_chimes":6, "ssml":true,
        "queued_and_instant":completions, "completion_times":completion_times,
        "missing":missing, "invalid":failed, "invalid_errors":errors,
        "cancelled":cancelled, "shutdown_ms":shutdown_ms, "helper_pid_reaped":true,
        "physical_audio_output_tested":false
    });
    fs::write(
        directory.join("probe.json"),
        serde_json::to_vec_pretty(&receipt).unwrap(),
    )
    .unwrap();
}
