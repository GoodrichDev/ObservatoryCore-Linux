//! Persistent real NLua/CLR process. The Rust host owns journal ingestion and
//! presentation; managed code supplies the original custom-script object model.
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::{
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Receiver, SyncSender},
    time::Duration,
};

const MAX_FRAME: u64 = 64 * 1024 * 1024;

pub(super) fn history_entry(
    value: &Value,
    system: u64,
    body: u64,
    raw: Option<std::sync::Arc<str>>,
) -> Value {
    let mut event = value.clone();
    let parent = event
        .as_object_mut()
        .and_then(|object| object.remove("__clr_parent"));
    let system = event
        .as_object_mut()
        .and_then(|object| object.remove("__clr_system_key"))
        .unwrap_or_else(|| json!(system));
    let mut entry = json!({"event":event,"body_id":body,"system_id":system});
    if let Some(raw) = raw {
        entry["raw_json"] = json!(raw);
    }
    if let Some(parent) = parent {
        entry["parent"] = if let Some(parents) = parent.as_array() {
            json!(parents
                .iter()
                .filter_map(|parent| parent
                    .as_object()?
                    .iter()
                    .next()
                    .map(|(kind, body)| json!({"parent_type":kind,"body":body})))
                .collect::<Vec<_>>())
        } else {
            Value::Null
        };
    }
    entry
}

pub(super) fn project_parent(parent: &Value) -> Value {
    parent
        .as_array()
        .map(|parents| {
            json!(parents
                .iter()
                .map(|parent| {
                    let mut value = serde_json::Map::new();
                    if let Some(kind) = parent["parent_type"].as_str() {
                        value.insert(kind.into(), parent["body"].clone());
                    }
                    Value::Object(value)
                })
                .collect::<Vec<_>>())
        })
        .unwrap_or(Value::Null)
}

pub(super) struct Client {
    child: Child,
    input: SyncSender<Vec<u8>>,
    output: Receiver<Result<Value>>,
    path: PathBuf,
    failure: Option<String>,
    timeout: Duration,
}

pub(super) fn executable() -> Result<Option<PathBuf>> {
    if let Some(path) = std::env::var_os("OBSERVATORY_CLR_CRITERIA_HOST") {
        let path = PathBuf::from(path);
        anyhow::ensure!(
            path.is_file(),
            "CLR criteria runtime does not exist: {}",
            path.display()
        );
        return Ok(Some(path));
    }
    let current = std::env::current_exe()?;
    let directory = current
        .parent()
        .context("Executable has no parent directory")?;
    for candidate in [
        directory.join("../lib/observatory-core/clr/observatory-clr-criteria"),
        directory.join("observatory-clr-criteria/observatory-clr-criteria"),
    ] {
        if candidate.is_file() {
            return Ok(Some(candidate));
        }
    }
    Ok(None)
}

impl Client {
    pub(super) fn spawn(executable: &Path, path: &Path) -> Result<(Self, Value)> {
        let mut child = Command::new(executable)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("Start CLR criteria runtime {}", executable.display()))?;
        let startup = (|| -> Result<_> {
            let mut stdin = child.stdin.take().context("CLR stdin unavailable")?;
            let stdout = child.stdout.take().context("CLR stdout unavailable")?;
            let (responses, output) = mpsc::sync_channel(1);
            let writer_errors = responses.clone();
            let (input, frames) = mpsc::sync_channel::<Vec<u8>>(1);
            std::thread::Builder::new()
                .name("observatory-clr-input".into())
                .spawn(move || {
                    while let Ok(frame) = frames.recv() {
                        let written = stdin
                            .write_all(&frame)
                            .and_then(|_| stdin.write_all(b"\n"))
                            .and_then(|_| stdin.flush());
                        if let Err(error) = written {
                            let _ = writer_errors.send(Err(error.into()));
                            break;
                        }
                    }
                })?;
            std::thread::Builder::new()
                .name("observatory-clr-output".into())
                .spawn(move || {
                    let mut reader = BufReader::new(stdout);
                    loop {
                        let response = (|| -> Result<Value> {
                            let mut frame = Vec::new();
                            let count = reader
                                .by_ref()
                                .take(MAX_FRAME + 1)
                                .read_until(b'\n', &mut frame)?;
                            anyhow::ensure!(count != 0, "CLR criteria runtime closed its output");
                            anyhow::ensure!(
                                count as u64 <= MAX_FRAME && frame.last() == Some(&b'\n'),
                                "CLR response exceeds 64 MiB or is truncated"
                            );
                            serde_json::from_slice(&frame).context("Invalid CLR criteria response")
                        })();
                        let stop = response.is_err();
                        if responses.send(response).is_err() || stop {
                            break;
                        }
                    }
                })?;
            Ok((input, output))
        })();
        let (input, output) = match startup {
            Ok(pipes) => pipes,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error);
            }
        };
        let timeout = std::env::var("OBSERVATORY_CLR_TIMEOUT_MS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| (100..=3_600_000).contains(value))
            .unwrap_or(10_000);
        let mut client = Self {
            child,
            input,
            output,
            path: path.to_owned(),
            failure: None,
            timeout: Duration::from_millis(timeout),
        };
        let response = client.call(json!({"op":"init","path":path,"read_all":false}))?;
        Ok((client, response))
    }

    pub(super) fn call(&mut self, request: Value) -> Result<Value> {
        if let Some(failure) = &self.failure {
            anyhow::bail!("{failure}");
        }
        let response = self.request(request);
        if let Err(error) = &response {
            self.failure = Some(format!("CLR criteria runtime failed: {error:#}"));
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        response
    }

    fn request(&self, request: Value) -> Result<Value> {
        let bytes = serde_json::to_vec(&request)?;
        anyhow::ensure!(
            bytes.len() as u64 <= MAX_FRAME,
            "CLR request exceeds 64 MiB"
        );
        self.input
            .try_send(bytes)
            .context("CLR request pipe unavailable")?;
        let response = self
            .output
            .recv_timeout(self.timeout)
            .context("CLR criteria request timed out or runtime exited")??;
        anyhow::ensure!(
            response["protocol"] == 1
                && response["runtime"] == ".NET"
                && response["nlua"] == "1.7.3",
            "CLR criteria protocol/runtime version mismatch"
        );
        // A managed request error is returned with its diagnostic records; it
        // does not imply an I/O failure or permission to replay the script.
        Ok(response)
    }

    pub(super) fn set_path(&mut self, path: &Path) -> Result<Option<Value>> {
        if self.path == path {
            return Ok(None);
        }
        let response = self.call(json!({"op":"set_path","path":path}))?;
        self.path = path.to_owned();
        Ok(Some(response))
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl super::Criteria {
    /// Deliver managed callbacks and mutations even while the journal is idle.
    pub fn poll(&mut self, current_system: &str) -> Vec<observatory_domain::Finding> {
        if self.managed.is_none() {
            return Vec::new();
        }
        self.notification_payloads
            .retain(|(finding, _)| self.startup_findings.contains(finding));
        let mut rows = self.take_startup_findings();
        rows.extend(self.managed_request(json!({"op":"poll"}), None, current_system));
        rows
    }

    pub fn take_history_updates(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.managed_history_updates)
    }

    pub fn take_event_snapshot(&mut self) -> Option<Value> {
        self.managed_event_snapshot.take()
    }

    pub fn with_managed_runtime(path: PathBuf, executable: &Path) -> Result<Self> {
        // Do not run the script in native Lua first: globals may have external
        // effects and must run exactly once in the original managed interpreter.
        let mut criteria = Self::from_source(path.clone(), None, "")?;
        let (client, response) = Client::spawn(executable, &path)?;
        criteria.managed = Some(client);
        let findings = criteria.managed_response(response, None, "");
        criteria.startup_findings.extend(findings);
        Ok(criteria)
    }

    pub(super) fn managed_request(
        &mut self,
        request: Value,
        event: Option<&observatory_domain::JournalEvent>,
        current_system: &str,
    ) -> Vec<observatory_domain::Finding> {
        let response = self
            .managed
            .as_mut()
            .expect("managed runtime")
            .call(request);
        match response {
            Ok(response) => self.managed_response(response, event, current_system),
            Err(error) => {
                // Stop once. Do not respawn or re-run arbitrary user globals
                // after a timeout/crash whose side effects are unknown.
                if !self.managed_failed {
                    self.managed_failed = true;
                    self.diagnostics.push(
                        json!({"Level":"Error","Message":error.to_string(),"Path":self.path}),
                    );
                }
                Vec::new()
            }
        }
    }

    pub(super) fn managed_sync_path(&mut self) {
        let response = self
            .managed
            .as_mut()
            .expect("managed runtime")
            .set_path(&self.path);
        match response {
            Ok(Some(response)) => {
                let rows = self.managed_response(response, None, "");
                self.startup_findings.extend(rows);
            }
            Ok(None) => {}
            Err(error) if !self.managed_failed => {
                self.managed_failed = true;
                self.diagnostics
                    .push(json!({"Level":"Error","Message":error.to_string(),"Path":self.path}));
            }
            Err(_) => {}
        }
    }

    fn managed_response(
        &mut self,
        response: Value,
        event: Option<&observatory_domain::JournalEvent>,
        current: &str,
    ) -> Vec<observatory_domain::Finding> {
        let mut output = Vec::new();
        for update in response["history_updates"].as_array().into_iter().flatten() {
            if update["kind"] == "scan" {
                if let (Some(system), Some(body), Some(raw)) = (
                    update["system_id"].as_u64(),
                    update["body_id"].as_u64(),
                    update["raw_json"].as_str(),
                ) {
                    self.raw_scans
                        .insert((system, body), std::sync::Arc::from(raw));
                }
            }
            self.managed_history_updates.push(update.clone());
        }
        let snapshot = response.get("event").filter(|value| value.is_object());
        if let Some(snapshot) = snapshot {
            let mut snapshot = snapshot.clone();
            if event.is_some_and(|event| event.event == "Scan")
                && response.get("event_parent").is_some()
            {
                snapshot["__clr_parent"] = project_parent(&response["event_parent"]);
            }
            self.managed_event_snapshot = Some(snapshot);
        }
        for diagnostic in response["diagnostics"].as_array().into_iter().flatten() {
            self.diagnostics.push(json!({"Level":"Error","Message":diagnostic["message"],"Detail":diagnostic["detail"],"ExceptionType":diagnostic["exception_type"],"Path":self.path}));
        }
        if response["ok"] != true && response["diagnostics"].as_array().is_none_or(Vec::is_empty) {
            self.diagnostics
                .push(json!({"Level":"Error","Message":response["error"],"Path":self.path}));
        }
        // Direct notify calls are added to the grid by the source callback
        // before normal CheckInterest results are appended by Explorer.
        for notice in response["notifications"].as_array().into_iter().flatten() {
            let finding = observatory_domain::Finding {
                plugin: "Explorer".into(),
                timestamp: event.map(|e| e.timestamp.clone()).unwrap_or_default(),
                system: event
                    .map(|e| super::system_name(e, current))
                    .unwrap_or_else(|| current.to_owned()),
                body: notice["title"].as_str().unwrap_or("").into(),
                title: notice["detail"].as_str().unwrap_or("").into(),
                detail: notice["extended_detail"].as_str().unwrap_or("").into(),
            };
            self.notification_payloads.push((finding.clone(), json!({"title":notice["title"],"detail":notice["detail"],"extended_details":notice["extended_detail"],"coalescing_id":notice["body_id"].as_i64().unwrap_or(-1),"sender":"Explorer","grid_time":notice["time"]})));
            output.push(finding);
        }
        if let Some(event) = event {
            let typed = snapshot.unwrap_or(&event.data);
            for result in response["results"].as_array().into_iter().flatten() {
                output.push(observatory_domain::Finding {
                    plugin: "Explorer".into(),
                    timestamp: typed["timestamp"]
                        .as_str()
                        .unwrap_or(&event.timestamp)
                        .into(),
                    system: typed["StarSystem"].as_str().unwrap_or(current).into(),
                    body: if result["system_wide"] == true {
                        typed["StarSystem"].as_str().unwrap_or(current).into()
                    } else {
                        super::string(typed, "BodyName").into()
                    },
                    title: result["title"].as_str().unwrap_or("").into(),
                    detail: result["detail"].as_str().unwrap_or("").into(),
                });
            }
        }
        output
    }
}
