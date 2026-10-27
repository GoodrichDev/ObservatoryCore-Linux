use super::{Request, Response, WorkerInfo, WorkerKind, MAX_FRAME_BYTES, PROTOCOL};
use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::{
    io::{BufRead, BufReader, Read, Write},
    path::Path,
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Receiver, SyncSender},
    time::{Duration, Instant},
};

/// Inspect a staged artifact before committing its installation. This starts
/// only its metadata CLI, not a model or network service. The staging writer
/// must be closed first: Linux will not execute an ELF still open for writing.
pub fn inspect(executable: &Path, kind: WorkerKind) -> Result<WorkerInfo> {
    let mut child = Command::new(executable)
        .args(["--kind", kind.name(), "--hello"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("Inspect native worker artifact")?;
    let stdout = child
        .stdout
        .take()
        .context("Native worker metadata stdout unavailable")?;
    let (tx, rx) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = stdout.take(65537).read_to_end(&mut bytes).map(|_| bytes);
        let _ = tx.send(result);
    });
    let deadline = Instant::now() + Duration::from_secs(10);
    let result = (|| -> Result<WorkerInfo> {
        loop {
            if let Some(status) = child.try_wait()? {
                anyhow::ensure!(status.success(), "Native worker metadata command failed");
                break;
            }
            anyhow::ensure!(
                Instant::now() < deadline,
                "Native worker metadata command timed out"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        let bytes = rx
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .context("Native worker metadata output timed out")??;
        anyhow::ensure!(
            bytes.len() <= 65536,
            "Native worker metadata output exceeds limit"
        );
        let info: WorkerInfo =
            serde_json::from_slice(&bytes).context("Invalid native worker metadata")?;
        anyhow::ensure!(
            info.protocol == PROTOCOL && info.plugin == kind,
            "Native worker metadata protocol/plugin mismatch"
        );
        anyhow::ensure!(
            info.platform == WorkerInfo::current(kind).platform,
            "Native worker metadata platform mismatch"
        );
        Ok(info)
    })();
    let _ = child.kill();
    let _ = child.wait();
    result
}

/// One sequential worker actor. stdout is exclusively bounded JSON frames;
/// stderr remains diagnostic output. A crashed or stalled module is isolated
/// from the host and never silently replaced with a fresh builtin model.
pub struct Client {
    child: Child,
    input: SyncSender<Vec<u8>>,
    output: Receiver<Result<Response>>,
    next_id: u64,
    failed: Option<String>,
    info: WorkerInfo,
}
impl Client {
    pub fn spawn(
        executable: &Path,
        kind: WorkerKind,
        settings: Value,
        data_dir: &Path,
    ) -> Result<Self> {
        let mut child = Command::new(executable)
            .arg("--kind")
            .arg(kind.name())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| {
                format!(
                    "Start {} native worker {}",
                    kind.name(),
                    executable.display()
                )
            })?;
        let mut stdin = child
            .stdin
            .take()
            .context("Native worker stdin unavailable")?;
        let stdout = child
            .stdout
            .take()
            .context("Native worker stdout unavailable")?;
        let (tx, output) = mpsc::sync_channel(1);
        let writer_tx = tx.clone();
        let (input, frames) = mpsc::sync_channel::<Vec<u8>>(1);
        std::thread::Builder::new()
            .name(format!("{}-rpc-input", kind.name()))
            .spawn(move || {
                while let Ok(frame) = frames.recv() {
                    let result = (|| -> Result<()> {
                        stdin.write_all(&frame)?;
                        stdin.write_all(b"\n")?;
                        stdin.flush()?;
                        Ok(())
                    })();
                    if let Err(error) = result {
                        let _ = writer_tx.send(Err(error));
                        break;
                    }
                }
            })?;
        std::thread::Builder::new()
            .name(format!("{}-rpc-output", kind.name()))
            .spawn(move || {
                let mut reader = BufReader::new(stdout);
                loop {
                    let result = (|| -> Result<Response> {
                        let mut line = Vec::new();
                        let bytes = reader
                            .by_ref()
                            .take(MAX_FRAME_BYTES + 1)
                            .read_until(b'\n', &mut line)?;
                        anyhow::ensure!(bytes != 0, "Native worker exited or closed stdout");
                        anyhow::ensure!(
                            bytes as u64 <= MAX_FRAME_BYTES && line.last() == Some(&b'\n'),
                            "Native worker response exceeds limit or is truncated"
                        );
                        serde_json::from_slice(&line).context("Invalid native worker response")
                    })();
                    let stop = result.is_err();
                    if tx.send(result).is_err() || stop {
                        break;
                    }
                }
            })?;
        let mut client = Self {
            child,
            input,
            output,
            next_id: 1,
            failed: None,
            info: WorkerInfo::current(kind),
        };
        let info: WorkerInfo = client.call("hello", Value::Null)?;
        anyhow::ensure!(
            info.protocol == PROTOCOL && info.plugin == kind,
            "Native worker protocol/plugin mismatch"
        );
        anyhow::ensure!(
            info.platform == WorkerInfo::current(kind).platform,
            "Native worker platform mismatch: {}",
            info.platform
        );
        client.info = info;
        client.call::<()>(
            "initialize",
            json!({"settings":settings,"data_dir":data_dir}),
        )?;
        Ok(client)
    }
    pub fn info(&self) -> &WorkerInfo {
        &self.info
    }
    pub fn is_healthy(&self) -> bool {
        self.failed.is_none()
    }
    pub fn process_id(&self) -> u32 {
        self.child.id()
    }
    pub fn call<T: DeserializeOwned>(&mut self, method: &str, args: Value) -> Result<T> {
        if let Some(error) = &self.failed {
            anyhow::bail!("{error}");
        }
        let result = self.request(method, args);
        // A legitimate model error (e.g. listener conflict) is recoverable.
        // Protocol/I/O/timeout failures leave an uncertain model and are fatal.
        match result {
            Ok(response) => {
                if let Some(error) = response.error {
                    anyhow::bail!("{}: {error}", self.info.plugin.name());
                }
                serde_json::from_value(response.result)
                    .context("Native worker result type mismatch")
            }
            Err(error) => {
                let error = format!(
                    "{} native worker failed: {error:#}",
                    self.info.plugin.name()
                );
                self.failed = Some(error.clone());
                let _ = self.child.kill();
                anyhow::bail!("{error}")
            }
        }
    }
    fn request(&mut self, method: &str, args: Value) -> Result<Response> {
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .context("Native worker request sequence overflow")?;
        let request = Request {
            protocol: PROTOCOL,
            id,
            now_ms: crate::stellar_overlay::now_ms(),
            method: method.into(),
            args,
        };
        let frame = serde_json::to_vec(&request)?;
        anyhow::ensure!(
            (frame.len() as u64) < MAX_FRAME_BYTES,
            "Native worker request exceeds frame limit"
        );
        // Writing a large frame to a stuck child's pipe must not block the
        // journal thread beyond the same bounded response timeout.
        self.input
            .try_send(frame)
            .context("Native worker input queue unavailable")?;
        let response = self
            .output
            .recv_timeout(Duration::from_secs(10))
            .context("Native worker response timed out/disconnected")??;
        anyhow::ensure!(
            response.protocol == PROTOCOL && response.id == id,
            "Native worker response protocol/sequence mismatch"
        );
        Ok(response)
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        if self.failed.is_none() {
            // Flush Hecate on a normal shutdown. The reader timeout bounds a
            // broken module; terminate and reap it in every case.
            let _ = self.call::<()>("shutdown", Value::Null);
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
