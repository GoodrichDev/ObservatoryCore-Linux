use notify_rust::{Notification, NotificationHandle};
use observatory_domain::Finding;
use observatory_io::Settings;
use serde_json::Value;
use std::{
    collections::{HashMap, VecDeque},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{sync_channel, RecvTimeoutError, SyncSender},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

type FilePlayer =
    Arc<dyn Fn(&std::path::Path, &Settings, &AtomicBool) -> Result<(), String> + Send + Sync>;
enum AudioWork {
    Notification(Box<(Finding, Settings)>),
    File(std::path::PathBuf, Value),
}
enum Operation {
    Lifecycle(Value, Settings),
}
pub struct Notifications {
    tx: SyncSender<Operation>,
    errors: Arc<Mutex<VecDeque<String>>>,
    cancelled: Arc<AtomicBool>,
    audio_handle: Option<thread::JoinHandle<()>>,
    audio_tx: SyncSender<AudioWork>,
    queued_titles: Arc<Mutex<HashMap<String, usize>>>,
    instant_handles: Mutex<Vec<thread::JoinHandle<()>>>,
    completions: Arc<Mutex<VecDeque<Value>>>,
    audio_settings: Arc<Mutex<Settings>>,
    file_player: FilePlayer,
}
impl Notifications {
    pub fn new() -> Self {
        Self::with_file_player(Arc::new(play_wave))
    }
    fn with_file_player(file_player: FilePlayer) -> Self {
        let queued_player = file_player.clone();
        let (tx, rx) = sync_channel::<Operation>(128);
        let errors = Arc::new(Mutex::new(VecDeque::new()));
        let output = errors.clone();
        let (audio_tx, audio_rx) = sync_channel::<AudioWork>(32);
        let audio_errors = errors.clone();
        let cancelled = Arc::new(AtomicBool::new(false));
        let audio_cancelled = cancelled.clone();
        let queued_titles = Arc::new(Mutex::new(HashMap::<String, usize>::new()));
        let playing_titles = queued_titles.clone();
        let owned_titles = queued_titles.clone();
        let enqueue_audio = audio_tx.clone();
        let completions = Arc::new(Mutex::new(VecDeque::new()));
        let audio_completions = completions.clone();
        let audio_settings = Arc::new(Mutex::new(Settings::default()));
        let playing_settings = audio_settings.clone();
        let audio_handle = thread::spawn(move || loop {
            if audio_cancelled.load(Ordering::Relaxed) {
                break;
            }
            let work = match audio_rx.recv_timeout(Duration::from_millis(50)) {
                Ok(message) => message,
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => break,
            };
            let current = playing_settings.lock().unwrap().clone();
            let title = match work {
                AudioWork::Notification(work) => {
                    let (finding, settings) = *work;
                    let key = finding.title.trim().to_lowercase();
                    if let Err(error) = play_audio(&finding, &settings, &current, &audio_cancelled)
                    {
                        push_error(&audio_errors, error);
                    }
                    key
                }
                AudioWork::File(path, options) => {
                    play_file_work(
                        &path,
                        &options,
                        &current,
                        &audio_cancelled,
                        &audio_errors,
                        &audio_completions,
                        &queued_player,
                    );
                    String::new()
                }
            };
            let mut titles = playing_titles.lock().unwrap();
            if let Some(count) = titles.get_mut(&title) {
                *count -= 1;
                if *count == 0 {
                    titles.remove(&title);
                }
            }
        });
        let popup_cancelled = cancelled.clone();
        thread::spawn(move || {
            let mut handles: HashMap<String, (NotificationHandle, Option<Instant>)> =
                HashMap::new();
            for operation in rx {
                if popup_cancelled.load(Ordering::Relaxed) {
                    break;
                }
                handles.retain(|_, (_, expires)| expires.is_none_or(|time| Instant::now() < time));
                let (finding, mut settings, lifecycle) = match operation {
                    Operation::Lifecycle(value, settings) => {
                        let id = value["id"].as_str().unwrap_or("").to_owned();
                        match value["action"].as_str().unwrap_or("show") {
                            "cancel" => {
                                if let Some((handle, _)) = handles.remove(&id) {
                                    handle.close();
                                }
                                continue;
                            }
                            "update" => {
                                if let Some((handle, expires)) = handles.get_mut(&id) {
                                    handle.timeout(
                                        expires
                                            .map(|e| {
                                                e.saturating_duration_since(Instant::now())
                                                    .as_millis()
                                                    .min(i32::MAX as u128)
                                                    as i32
                                            })
                                            .unwrap_or(0),
                                    );
                                    handle.summary(value["title"].as_str().unwrap_or(""));
                                    handle.body(&escape_markup(
                                        value["detail"].as_str().unwrap_or(""),
                                    ));
                                    if let Err(error) = handle.update() {
                                        push_error(
                                            &output,
                                            format!("update notification: {error}"),
                                        );
                                    }
                                }
                            }
                            _ => {}
                        }
                        let finding = Finding {
                            plugin: String::new(),
                            timestamp: String::new(),
                            system: String::new(),
                            body: String::new(),
                            title: value["title"].as_str().unwrap_or("").into(),
                            detail: value["detail"].as_str().unwrap_or("").into(),
                        };
                        (finding, settings, Some(value))
                    }
                };
                if let Some(value) = &lifecycle {
                    if value["popup"].as_bool() == Some(false) {
                        settings.popup = false;
                    }
                    if value["speech"].as_bool() == Some(false) {
                        settings.speech = false;
                    }
                }
                if lifecycle.as_ref().is_some_and(|v| v["action"] == "update") {
                    settings.popup = false;
                }
                if settings.popup {
                    let requested = lifecycle
                        .as_ref()
                        .and_then(|v| v["timeout"].as_i64())
                        .unwrap_or(-1);
                    let timeout = if requested == -1 {
                        settings
                            .extra
                            .get("NativeNotifyTimeout")
                            .or_else(|| {
                                settings
                                    .extra
                                    .get("legacy_settings")
                                    .and_then(|v| v.get("NativeNotifyTimeout"))
                            })
                            .and_then(Value::as_i64)
                            .or_else(|| {
                                settings
                                    .extra
                                    .get("notification_timeout_ms")
                                    .and_then(Value::as_i64)
                            })
                            .unwrap_or(8000)
                    } else {
                        requested
                    };
                    let mut notification = Notification::new();
                    notification
                        .appname("Observatory Core")
                        .icon("observatory-core")
                        .summary(&finding.title)
                        .body(&escape_markup(&finding.detail))
                        .timeout(timeout.clamp(-1, i32::MAX as i64) as i32);
                    if let Some(id) = lifecycle.as_ref().and_then(|v| v["id"].as_str()) {
                        if let Some((handle, _)) = handles.get(id) {
                            notification.id(handle.id());
                        }
                    }
                    match notification.show() {
                        Ok(handle) => {
                            if let Some(value) = &lifecycle {
                                if let Some(id) = value["id"].as_str() {
                                    if handles.len() >= 1024 {
                                        if let Some(key) = handles.keys().next().cloned() {
                                            if let Some((handle, _)) = handles.remove(&key) {
                                                handle.close();
                                            }
                                        }
                                    }
                                    handles.insert(id.into(), (handle, expiration(timeout)));
                                }
                            }
                        }
                        Err(error) => push_error(&output, format!("desktop notification: {error}")),
                    }
                }
                if let Some(value) = &lifecycle {
                    if value.get("title_ssml").is_some() || value.get("detail_ssml").is_some() {
                        settings.extra.insert("speech_markup".into(), value.clone());
                    }
                }
                if settings.speech {
                    let key = finding.title.trim().to_lowercase();
                    let mut titles = queued_titles.lock().unwrap();
                    let count = titles.entry(key.clone()).or_default();
                    settings
                        .extra
                        .insert("suppress_audio_title".into(), serde_json::json!(*count > 0));
                    *count += 1;
                    if enqueue_audio
                        .try_send(AudioWork::Notification(Box::new((finding, settings))))
                        .is_err()
                    {
                        *titles.get_mut(&key).unwrap() -= 1;
                        push_error(&output, "Audio notification queue is full".into());
                    }
                }
            }
            for (_, (handle, _)) in handles {
                handle.close();
            }
        });
        Self {
            tx,
            errors,
            cancelled,
            audio_handle: Some(audio_handle),
            audio_tx,
            queued_titles: owned_titles,
            instant_handles: Mutex::new(Vec::new()),
            completions,
            audio_settings,
            file_player,
        }
    }
    pub fn set_settings(&self, settings: &Settings) {
        *self.audio_settings.lock().unwrap() = settings.clone();
    }
    pub fn take_audio_completions(&self) -> Vec<Value> {
        self.completions.lock().unwrap().drain(..).collect()
    }
    pub fn play_file(
        &self,
        path: &std::path::Path,
        options: &Value,
        settings: &Settings,
    ) -> anyhow::Result<()> {
        self.set_settings(settings);
        if path.as_os_str().is_empty() || !path.is_file() {
            finish_file(options, Ok(()), false, true, &self.completions);
            return Ok(());
        }
        let instant = options
            .get("Instant")
            .or_else(|| options.get("instant"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if instant {
            let path = path.to_owned();
            let options = options.clone();
            let current = self.audio_settings.clone();
            let cancelled = self.cancelled.clone();
            let errors = self.errors.clone();
            let completions = self.completions.clone();
            let file_player = self.file_player.clone();
            let handle = thread::spawn(move || {
                let current = current.lock().unwrap().clone();
                play_file_work(
                    &path,
                    &options,
                    &current,
                    &cancelled,
                    &errors,
                    &completions,
                    &file_player,
                );
            });
            let mut jobs = self.instant_handles.lock().unwrap();
            let mut i = 0;
            while i < jobs.len() {
                if jobs[i].is_finished() {
                    let _ = jobs.swap_remove(i).join();
                } else {
                    i += 1;
                }
            }
            jobs.push(handle);
        } else {
            *self
                .queued_titles
                .lock()
                .unwrap()
                .entry(String::new())
                .or_default() += 1;
            if self
                .audio_tx
                .try_send(AudioWork::File(path.to_owned(), options.clone()))
                .is_err()
            {
                *self.queued_titles.lock().unwrap().get_mut("").unwrap() -= 1;
                anyhow::bail!("Audio notification queue is full");
            }
        }
        Ok(())
    }
    pub fn lifecycle(&self, value: Value, settings: Settings) {
        if self
            .tx
            .try_send(Operation::Lifecycle(value, settings))
            .is_err()
        {
            push_error(&self.errors, "Desktop notification queue is full".into());
        }
    }
    pub fn take_errors(&self) -> Vec<String> {
        self.errors.lock().unwrap().drain(..).collect()
    }
}
impl Drop for Notifications {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
        if let Some(handle) = self.audio_handle.take() {
            let _ = handle.join();
        }
        for handle in self.instant_handles.lock().unwrap().drain(..) {
            let _ = handle.join();
        }
    }
}
fn push_error(errors: &Arc<Mutex<VecDeque<String>>>, error: String) {
    let mut errors = errors.lock().unwrap();
    if errors.back() != Some(&error) {
        errors.push_back(error);
    }
    while errors.len() > 100 {
        errors.pop_front();
    }
}
fn expiration(timeout: i64) -> Option<Instant> {
    (timeout > 0).then(|| Instant::now() + Duration::from_millis(timeout as u64))
}
fn escape_markup(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
fn play_audio(
    finding: &Finding,
    settings: &Settings,
    playback_settings: &Settings,
    cancelled: &AtomicBool,
) -> Result<(), String> {
    use std::io::Write;
    if !settings.speech {
        return Ok(());
    }
    let value = |name: &str| {
        settings.extra.get(name).or_else(|| {
            settings
                .extra
                .get("legacy_settings")
                .and_then(|v| v.get(name))
        })
    };
    let mut wave = tempfile::Builder::new()
        .prefix("ObsCore_")
        .suffix(".wav")
        .tempfile()
        .map_err(|e| e.to_string())?;
    if value("ChimeEnabled")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        let bytes: &[u8] = match value("ChimeSelected").and_then(Value::as_u64).unwrap_or(0) {
            1 => include_bytes!("../../../ObservatoryCore/Assets/ObservatoryNotification1.wav"),
            2 => include_bytes!("../../../ObservatoryCore/Assets/ObservatoryNotification2.wav"),
            3 => include_bytes!("../../../ObservatoryCore/Assets/ObservatoryNotification3.wav"),
            4 => include_bytes!("../../../ObservatoryCore/Assets/ObservatoryNotification4.wav"),
            5 => include_bytes!("../../../ObservatoryCore/Assets/ObservatoryNotification5.wav"),
            _ => include_bytes!("../../../ObservatoryCore/Assets/ObservatoryNotification0.wav"),
        };
        wave.write_all(bytes).map_err(|e| e.to_string())?;
        wave.flush().map_err(|e| e.to_string())?;
    } else if let Some(file) = value("chime_file")
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
    {
        std::fs::copy(file, wave.path()).map_err(|e| e.to_string())?;
    } else {
        let mut command = speech_command(finding, settings, wave.path())?;
        run(&mut command, cancelled)?;
    }
    play_wave(wave.path(), playback_settings, cancelled)
}
fn speech_command(
    finding: &Finding,
    settings: &Settings,
    path: &std::path::Path,
) -> Result<Command, String> {
    let value = |name: &str| {
        settings.extra.get(name).or_else(|| {
            settings
                .extra
                .get("legacy_settings")
                .and_then(|v| v.get(name))
        })
    };
    let mut command = Command::new("espeak-ng");
    let rate = value("VoiceRate")
        .and_then(Value::as_i64)
        .map(|r| (175.0 * 2f64.powf(r.clamp(-10, 10) as f64 / 10.0)).round() as u64)
        .or_else(|| value("speech_rate").and_then(Value::as_u64))
        .unwrap_or(175)
        .clamp(80, 450)
        .to_string();
    let volume = value("VoiceVolume")
        .and_then(Value::as_u64)
        .or_else(|| value("speech_volume").and_then(Value::as_u64))
        .unwrap_or(75)
        .min(100)
        .to_string();
    command.args(["-s", &rate, "-a", &volume, "-w"]).arg(path);
    if let Some(voice) = value("speech_voice")
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
    {
        command.args(["-v", voice]);
    }
    let suppress = value("suppress_audio_title")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let text = if let Some(markup) = value("speech_markup") {
        command.arg("-m");
        let title = if suppress {
            String::new()
        } else {
            markup["title_ssml"]
                .as_str()
                .filter(|v| !v.is_empty())
                .map(str::to_owned)
                .unwrap_or_else(|| escape_markup(&finding.title))
        };
        let body = markup["detail_ssml"]
            .as_str()
            .filter(|v| !v.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| escape_markup(&finding.detail));
        let voice = value("speech_voice")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .unwrap_or("en");
        let title = if !suppress && markup["title_ssml"].as_str().is_some_and(|s| !s.is_empty()) {
            add_voice_to_ssml(&title, voice)?
        } else {
            title
        };
        let body = if markup["detail_ssml"]
            .as_str()
            .is_some_and(|s| !s.is_empty())
        {
            add_voice_to_ssml(&body, voice)?
        } else {
            body
        };
        format!("{title} {body}")
    } else if suppress {
        finding.detail.clone()
    } else {
        format!("{}. {}", finding.title, finding.detail)
    };
    command.arg("--").arg(text);
    Ok(command)
}
fn play_wave(
    path: &std::path::Path,
    settings: &Settings,
    cancelled: &AtomicBool,
) -> Result<(), String> {
    let value = |name: &str| {
        settings.extra.get(name).or_else(|| {
            settings
                .extra
                .get("legacy_settings")
                .and_then(|v| v.get(name))
        })
    };
    let mut playback = Command::new("paplay");
    if let Some(device) = value("audio_device")
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
    {
        playback.arg(format!("--device={device}"));
    }
    let volume = value("AudioVolume")
        .and_then(Value::as_f64)
        .unwrap_or_else(|| {
            value("chime_volume")
                .and_then(Value::as_f64)
                .unwrap_or(75.0)
                / 100.0
        })
        .clamp(0.0, 1.0);
    playback.arg(format!("--volume={}", (volume * 65_536.0).round() as u32));
    run(playback.arg("--").arg(path), cancelled)
}
fn add_voice_to_ssml(ssml: &str, voice: &str) -> Result<String, String> {
    let doc = roxmltree::Document::parse(ssml).map_err(|e| e.to_string())?;
    let root = doc.root_element();
    let node = (root.tag_name().name() == "speak")
        .then(|| {
            root.children().find(|n| {
                n.is_element()
                    && n.tag_name().name() == "voice"
                    && n.tag_name().namespace() == root.tag_name().namespace()
            })
        })
        .flatten();
    if let Some(attribute) = node.and_then(|n| n.attribute_node("name")) {
        let mut result = ssml.to_owned();
        let escaped = escape_markup(voice)
            .replace('"', "&quot;")
            .replace('\'', "&apos;");
        result.replace_range(attribute.range_value(), &escaped);
        Ok(result)
    } else {
        Ok(ssml.to_owned())
    }
}
fn finish_file(
    options: &Value,
    result: Result<(), String>,
    cancelled: bool,
    skipped: bool,
    completions: &Arc<Mutex<VecDeque<Value>>>,
) {
    let mut value = serde_json::json!({"request_id":options.get("request_id").or_else(||options.get("RequestID")).cloned().unwrap_or(Value::Null),"plugin":options.get("plugin").or_else(||options.get("Plugin")).cloned().unwrap_or(Value::Null),"ok":result.is_ok()&&!cancelled,"cancelled":cancelled,"skipped":skipped});
    if let Err(error) = result {
        value["error"] = serde_json::json!(error);
    }
    let mut queue = completions.lock().unwrap();
    queue.push_back(value);
}
fn play_file_work(
    path: &std::path::Path,
    options: &Value,
    settings: &Settings,
    cancelled: &AtomicBool,
    errors: &Arc<Mutex<VecDeque<String>>>,
    completions: &Arc<Mutex<VecDeque<Value>>>,
    player: &FilePlayer,
) {
    let result = player(path, settings, cancelled);
    if let Err(error) = &result {
        push_error(errors, error.clone());
    }
    if result.is_ok()
        && !cancelled.load(Ordering::Relaxed)
        && options
            .get("DeleteAfterPlay")
            .or_else(|| options.get("delete_after_play"))
            .and_then(Value::as_bool)
            .unwrap_or(false)
    {
        let _ = std::fs::remove_file(path);
    }
    finish_file(
        options,
        result,
        cancelled.load(Ordering::Relaxed),
        false,
        completions,
    );
}

fn run(command: &mut Command, cancelled: &AtomicBool) -> Result<(), String> {
    if cancelled.load(Ordering::Relaxed) {
        return Ok(());
    }
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("{}: {e}", command.get_program().to_string_lossy()))?;
    loop {
        if cancelled.load(Ordering::Relaxed) {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(());
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                return if status.success() {
                    Ok(())
                } else {
                    Err(format!("Audio command exited {status}"))
                }
            }
            Err(error) => return Err(error.to_string()),
            _ => {}
        }
        thread::sleep(Duration::from_millis(25));
    }
}

#[cfg(all(test, target_os = "linux"))]
#[path = "actual_audio_acceptance.rs"]
mod actual_audio_acceptance;

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    fn xml_semantics(text: &str) -> Vec<Value> {
        let doc = roxmltree::Document::parse(text).unwrap();
        doc.descendants().filter(|n|n.is_element()||n.is_text()).map(|n|{
            if n.is_element(){let mut attributes:Vec<_>=n.attributes().map(|a|(a.name(),a.value())).collect();attributes.sort();serde_json::json!({"tag":n.tag_name().name(),"namespace":n.tag_name().namespace(),"attributes":attributes})}else{serde_json::json!({"text":n.text()})}
        }).collect()
    }
    #[test]
    fn six_chime_assets_match_installed_core_resource_bytes() {
        let assets: [&[u8]; 6] = [
            include_bytes!("../../../ObservatoryCore/Assets/ObservatoryNotification0.wav"),
            include_bytes!("../../../ObservatoryCore/Assets/ObservatoryNotification1.wav"),
            include_bytes!("../../../ObservatoryCore/Assets/ObservatoryNotification2.wav"),
            include_bytes!("../../../ObservatoryCore/Assets/ObservatoryNotification3.wav"),
            include_bytes!("../../../ObservatoryCore/Assets/ObservatoryNotification4.wav"),
            include_bytes!("../../../ObservatoryCore/Assets/ObservatoryNotification5.wav"),
        ];
        let cases: Vec<Value> =
            serde_json::from_str(include_str!("../../fixtures/core-chime-assets.json")).unwrap();
        assert_eq!(cases.len(), 6);
        for case in cases {
            let index = case["Name"]
                .as_str()
                .unwrap()
                .trim_start_matches("ObservatoryNotification")
                .parse::<usize>()
                .unwrap();
            let data = assets[index];
            assert_eq!(data.len(), case["Bytes"].as_u64().unwrap() as usize);
            let sha = ring::digest::digest(&ring::digest::SHA256, data)
                .as_ref()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>();
            assert_eq!(sha, case["SHA256"].as_str().unwrap());
        }
    }
    #[test]
    fn ssml_voice_selection_matches_all_original_xml_method_cases() {
        let cases: Vec<Value> =
            serde_json::from_str(include_str!("../../fixtures/core-ssml-oracle.json")).unwrap();
        assert_eq!(cases.len(), 10);
        for case in cases {
            let actual = add_voice_to_ssml(
                case["Input"].as_str().unwrap(),
                case["Voice"].as_str().unwrap(),
            );
            if case["Error"] != "" {
                assert!(actual.is_err(), "{case}");
            } else {
                assert_eq!(
                    xml_semantics(&actual.unwrap()),
                    xml_semantics(case["Output"].as_str().unwrap()),
                    "{case}"
                );
            }
        }
    }
    #[test]
    fn queued_instant_delete_and_actual_completion_preserve_source_audio_contract() {
        use std::sync::mpsc::channel;
        use std::sync::Barrier;
        let folder = tempfile::tempdir().unwrap();
        for name in ["a.wav", "b.wav", "instant.wav", "fail.wav"] {
            std::fs::write(folder.path().join(name), b"synthetic audio").unwrap();
        }
        let release = Arc::new(Barrier::new(2));
        let blocked = release.clone();
        let (tx, rx) = channel();
        let player: FilePlayer = Arc::new(move |path, settings, _| {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            tx.send((name.clone(), settings.extra.get("AudioVolume").cloned()))
                .unwrap();
            if name == "a.wav" {
                blocked.wait();
            }
            if name == "fail.wav" {
                return Err("synthetic playback failure".into());
            }
            Ok(())
        });
        let notifications = Notifications::with_file_player(player);
        let mut settings = Settings::default();
        settings
            .extra
            .insert("AudioVolume".into(), serde_json::json!(0.5));
        notifications
            .play_file(
                &folder.path().join("a.wav"),
                &serde_json::json!({"request_id":"a","plugin":"Test","DeleteAfterPlay":true}),
                &settings,
            )
            .unwrap();
        assert_eq!(rx.recv_timeout(Duration::from_secs(2)).unwrap().0, "a.wav");
        assert!(notifications.take_audio_completions().is_empty());
        notifications
            .play_file(
                &folder.path().join("b.wav"),
                &serde_json::json!({"request_id":"b"}),
                &settings,
            )
            .unwrap();
        settings
            .extra
            .insert("AudioVolume".into(), serde_json::json!(0.9));
        notifications
            .play_file(
                &folder.path().join("instant.wav"),
                &serde_json::json!({"request_id":"instant","Instant":true}),
                &settings,
            )
            .unwrap();
        assert_eq!(
            rx.recv_timeout(Duration::from_secs(2)).unwrap().0,
            "instant.wav"
        );
        assert!(folder.path().join("a.wav").exists());
        release.wait();
        let queued = rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(queued.0, "b.wav");
        assert_eq!(queued.1, Some(serde_json::json!(0.9)));
        notifications
            .play_file(
                &folder.path().join("fail.wav"),
                &serde_json::json!({"request_id":"fail","DeleteAfterPlay":true}),
                &settings,
            )
            .unwrap();
        assert_eq!(
            rx.recv_timeout(Duration::from_secs(2)).unwrap().0,
            "fail.wav"
        );
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut completions = Vec::new();
        while completions.len() < 4 && Instant::now() < deadline {
            completions.extend(notifications.take_audio_completions());
            thread::yield_now();
        }
        assert_eq!(completions.len(), 4);
        assert!(
            completions
                .iter()
                .find(|v| v["request_id"] == "fail")
                .unwrap()["ok"]
                == false
        );
        assert!(completions
            .iter()
            .filter(|v| v["request_id"] != "fail")
            .all(|v| v["ok"] == true));
        assert!(!folder.path().join("a.wav").exists());
        assert!(folder.path().join("b.wav").exists());
        assert!(folder.path().join("fail.wav").exists());
    }
    #[test]
    fn ssml_synthesis_plan_honors_source_rate_volume_voice_and_title_suppression() {
        let finding = Finding {
            plugin: "Test".into(),
            timestamp: String::new(),
            system: String::new(),
            body: String::new(),
            title: "Title".into(),
            detail: "Detail".into(),
        };
        let mut settings = Settings::default();
        settings
            .extra
            .insert("VoiceRate".into(), serde_json::json!(10));
        settings
            .extra
            .insert("VoiceVolume".into(), serde_json::json!(60));
        settings
            .extra
            .insert("speech_voice".into(), serde_json::json!("en-us"));
        settings.extra.insert("speech_markup".into(),serde_json::json!({"title_ssml":"<speak><voice name='Old'>Title</voice></speak>","detail_ssml":"<speak>Detail</speak>"}));
        let command = speech_command(
            &finding,
            &settings,
            std::path::Path::new("/tmp/synthetic.wav"),
        )
        .unwrap();
        let args: Vec<_> = command
            .get_args()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        assert!(args.windows(2).any(|p| p == ["-s", "350"]));
        assert!(args.windows(2).any(|p| p == ["-a", "60"]));
        assert!(args.last().unwrap().contains("name='en-us'"));
        settings
            .extra
            .insert("suppress_audio_title".into(), serde_json::json!(true));
        let command = speech_command(
            &finding,
            &settings,
            std::path::Path::new("/tmp/synthetic.wav"),
        )
        .unwrap();
        assert!(!command
            .get_args()
            .last()
            .unwrap()
            .to_string_lossy()
            .contains("Title"));
        settings.extra["speech_markup"]["detail_ssml"] = serde_json::json!("not XML");
        assert!(speech_command(
            &finding,
            &settings,
            std::path::Path::new("/tmp/synthetic.wav")
        )
        .is_err());
    }
    #[test]
    fn shutdown_cancels_and_reaps_a_running_audio_helper() {
        let cancelled = Arc::new(AtomicBool::new(false));
        let token = cancelled.clone();
        let trigger = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            token.store(true, Ordering::Relaxed);
        });
        let started = Instant::now();
        run(Command::new("sleep").arg("30"), &cancelled).unwrap();
        trigger.join().unwrap();
        assert!(started.elapsed() < Duration::from_secs(3));
    }
}
