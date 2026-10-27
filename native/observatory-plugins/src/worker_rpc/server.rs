use super::{Request, Response, WorkerInfo, WorkerKind, MAX_FRAME_BYTES, PROTOCOL};
use anyhow::{bail, Context, Result};
use observatory_domain::{Finding, JournalEvent};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{
    io::{BufRead, Read, Write},
    path::Path,
};

enum Worker {
    Bio(Box<crate::bioinsights::BioInsights>),
    Eval(Box<crate::evaluator::Evaluator>),
    Hecate(Box<crate::hecate::Hecate>),
    Stellar(Box<crate::stellar_overlay::StellarOverlay>),
}
fn arg<T: DeserializeOwned>(args: &Value, key: &str) -> Result<T> {
    serde_json::from_value(args.get(key).cloned().unwrap_or(Value::Null))
        .with_context(|| format!("Invalid RPC argument {key}"))
}
fn encode(value: impl serde::Serialize) -> Result<Value> {
    Ok(serde_json::to_value(value)?)
}
impl Worker {
    fn new(kind: WorkerKind, args: &Value) -> Result<Self> {
        let settings = arg(args, "settings")?;
        Ok(match kind {
            WorkerKind::BioInsights => {
                Self::Bio(Box::new(crate::bioinsights::BioInsights::new(settings)?))
            }
            WorkerKind::Evaluator => {
                Self::Eval(Box::new(crate::evaluator::Evaluator::new(settings)))
            }
            WorkerKind::Hecate => Self::Hecate(Box::new(crate::hecate::Hecate::open(
                settings,
                Path::new(&arg::<String>(args, "data_dir")?),
            )?)),
            WorkerKind::StellarOverlay => Self::Stellar(Box::new(
                crate::stellar_overlay::StellarOverlay::new(settings),
            )),
        })
    }
    fn call(&mut self, method: &str, args: &Value) -> Result<Value> {
        // The methods below call the actual production models. No parallel
        // implementation or transport-specific event interpretation lives here.
        match self {
            Self::Bio(w) => match method {
                "set_settings" => {
                    w.set_settings(arg(args, "settings")?)?;
                    Ok(Value::Null)
                }
                "reset" => {
                    w.reset();
                    Ok(Value::Null)
                }
                "set_batch" => {
                    w.set_batch(arg(args, "batch")?);
                    Ok(Value::Null)
                }
                "on_event" => encode(w.on_event(&arg::<JournalEvent>(args, "event")?)),
                "on_events" => encode(
                    arg::<Vec<JournalEvent>>(args, "events")?
                        .iter()
                        .flat_map(|e| w.on_event(e))
                        .collect::<Vec<_>>(),
                ),
                "on_status" => encode(w.on_status(&args["status"])),
                "on_message" => {
                    encode(w.on_message(&arg::<String>(args, "command")?, &args["payload"]))
                }
                "on_peer_message" => encode(w.on_peer_message(
                    &arg::<String>(args, "sender")?,
                    &arg::<String>(args, "command")?,
                    &args["payload"],
                )),
                "take_notification_rendering" => {
                    encode(w.take_notification_rendering(&arg::<Finding>(args, "finding")?))
                }
                "take_messages" => encode(w.take_messages()),
                "take_http_requests" => encode(w.take_http_requests()),
                "handle_http_response" => {
                    w.handle_http_response(&args["response"])?;
                    Ok(Value::Null)
                }
                "take_diagnostics" => encode(w.take_diagnostics()),
                "state" => encode(w.state()),
                "restore_state" => {
                    w.restore_state(&args["state"]);
                    Ok(Value::Null)
                }
                "restore_legacy_state" => encode(w.restore_legacy_state(&args["state"])?),
                "codex_companion" => encode(w.codex_companion()),
                "unsold_data" => encode(w.unsold_data()),
                "tools_summary" => encode(w.tools_summary()),
                "body_predictions" => encode(w.body_predictions(arg(args, "body")?)),
                "body_summary" => encode(w.body_summary(arg(args, "body")?)),
                "rows" => encode(w.rows()),
                "overlay" => encode(w.overlay()),
                "main_view" => encode(w.main_view()),
                "browser_route" => encode(w.browser_route()),
                "browser_html" => encode(w.browser_html()),
                _ => bail!("Unsupported BioInsights RPC method {method}"),
            },
            Self::Eval(w) => match method {
                "set_settings" => {
                    w.set_settings(arg(args, "settings")?)?;
                    Ok(Value::Null)
                }
                "reset" => {
                    w.reset();
                    Ok(Value::Null)
                }
                "set_batch" => {
                    w.set_batch(arg(args, "batch")?);
                    Ok(Value::Null)
                }
                "set_preread" => {
                    w.set_preread(arg(args, "preread")?);
                    Ok(Value::Null)
                }
                "on_event" => encode(w.on_event(&arg::<JournalEvent>(args, "event")?)),
                "on_events" => encode(
                    arg::<Vec<JournalEvent>>(args, "events")?
                        .iter()
                        .flat_map(|e| w.on_event(e))
                        .collect::<Vec<_>>(),
                ),
                "on_status" => {
                    w.on_status(&args["status"]);
                    Ok(Value::Null)
                }
                "on_message" => {
                    w.on_message(&arg::<String>(args, "command")?, &args["payload"]);
                    Ok(Value::Null)
                }
                "on_peer_message" => {
                    w.on_peer_message(
                        &arg::<String>(args, "sender")?,
                        &arg::<String>(args, "command")?,
                        &args["payload"],
                    );
                    Ok(Value::Null)
                }
                "set_bio_summary" => {
                    w.set_bio_summary(arg(args, "body_id")?, args["summary"].clone());
                    Ok(Value::Null)
                }
                "take_messages" => encode(w.take_messages()),
                "poll_routes" => encode(w.poll_routes()),
                "take_route_findings" => encode(w.take_route_findings()),
                "take_notification_rendering" => {
                    encode(w.take_notification_rendering(&arg::<Finding>(args, "finding")?))
                }
                "take_notification_payload" => {
                    encode(w.take_notification_payload(&arg::<Finding>(args, "finding")?))
                }
                "rows" => encode(w.rows()),
                "summary" => encode(w.summary()),
                "web_main_html" => encode(w.web_main_html()),
                "browser_html" => encode(w.browser_html(args["page"].as_str().unwrap_or("main"))),
                _ => bail!("Unsupported Evaluator RPC method {method}"),
            },
            Self::Hecate(w) => match method {
                "settings" => encode(w.settings()),
                "set_settings" => {
                    w.try_set_settings(arg(args, "settings")?)?;
                    encode(w.settings())
                }
                "register_plugin" => {
                    w.register_plugin(&arg::<String>(args, "name")?, args["target"].clone())?;
                    Ok(Value::Null)
                }
                "poll_bridge" => {
                    w.poll_bridge();
                    Ok(Value::Null)
                }
                "respond_web" => {
                    w.respond_web(&arg::<String>(args, "request")?, args["response"].clone())?;
                    Ok(Value::Null)
                }
                "server_address" => encode(w.server_address()),
                "reset" => {
                    w.reset();
                    Ok(Value::Null)
                }
                "begin_replay" => {
                    w.begin_replay()?;
                    Ok(Value::Null)
                }
                "set_batch" => {
                    w.set_batch(arg(args, "batch")?);
                    Ok(Value::Null)
                }
                "flush" => {
                    w.flush()?;
                    Ok(Value::Null)
                }
                "on_event" => encode(w.on_event(&arg::<JournalEvent>(args, "event")?)),
                "on_events" => encode(
                    arg::<Vec<JournalEvent>>(args, "events")?
                        .iter()
                        .flat_map(|e| w.on_event(e))
                        .collect::<Vec<_>>(),
                ),
                "errors" => encode(w.errors()),
                "rows" => encode(w.rows()),
                "take_http_requests" => encode(w.take_http_requests()),
                "take_messages" => encode(w.take_messages()),
                "handle_message" => {
                    encode(w.handle_message(&arg::<String>(args, "command")?, &args["payload"])?)
                }
                "handle_http_response" => {
                    w.handle_http_response(&args["response"])?;
                    Ok(Value::Null)
                }
                "evaluator_total" => encode(w.evaluator_total(arg(args, "fid")?)?),
                "evaluator_data" => encode(w.evaluator_data(arg(args, "fid")?)?),
                "bioinsights_total" => encode(w.bioinsights_total(arg(args, "fid")?)?),
                "bioinsights_data" => encode(w.bioinsights_data(arg(args, "fid")?)?),
                _ => bail!("Unsupported Hecate RPC method {method}"),
            },
            Self::Stellar(w) => match method {
                "settings" => encode(w.settings()),
                "set_settings" => {
                    w.set_settings(arg(args, "settings")?)?;
                    encode(w.settings())
                }
                "reset" => {
                    w.reset();
                    Ok(Value::Null)
                }
                "set_batch" => {
                    w.set_batch(arg(args, "batch")?);
                    Ok(Value::Null)
                }
                "handles_notifications" => encode(w.handles_notifications()),
                "on_event" => {
                    w.on_event(&arg::<JournalEvent>(args, "event")?);
                    Ok(Value::Null)
                }
                "on_events" => {
                    for event in arg::<Vec<JournalEvent>>(args, "events")? {
                        w.on_event(&event)
                    }
                    Ok(Value::Null)
                }
                "on_status" => {
                    w.on_status(&args["status"]);
                    Ok(Value::Null)
                }
                "on_notification" => {
                    w.on_notification(&args["notification"]);
                    Ok(Value::Null)
                }
                "on_notification_at" => {
                    w.on_notification_at(&args["notification"], arg(args, "now")?);
                    Ok(Value::Null)
                }
                "cancel" => {
                    w.cancel(&arg::<String>(args, "id")?);
                    Ok(Value::Null)
                }
                "tick" => {
                    w.tick();
                    Ok(Value::Null)
                }
                "tick_at" => {
                    w.tick_at(arg(args, "now")?);
                    Ok(Value::Null)
                }
                "snapshot" => encode(w.snapshot()),
                "handle_message" => encode(w.handle_message(
                    &arg::<String>(args, "sender")?,
                    &arg::<String>(args, "command")?,
                    &args["payload"],
                )?),
                "set_bio_panel" => {
                    w.set_bio_panel(args["panel"].clone());
                    Ok(Value::Null)
                }
                "set_evaluator_panel" => {
                    w.set_evaluator_panel(args["panel"].clone());
                    Ok(Value::Null)
                }
                _ => bail!("Unsupported StellarOverlay RPC method {method}"),
            },
        }
    }
}

/// Serve one worker for the lifetime of this process. The host explicitly
/// initializes it after validating hello; Hecate never binds a socket merely
/// because an updater is inspecting the executable's metadata.
pub fn serve(kind: WorkerKind, mut input: impl BufRead, mut output: impl Write) -> Result<()> {
    let mut worker = None;
    loop {
        let mut line = Vec::new();
        let bytes = input
            .by_ref()
            .take(MAX_FRAME_BYTES + 1)
            .read_until(b'\n', &mut line)?;
        if bytes == 0 {
            return Ok(());
        }
        anyhow::ensure!(
            bytes as u64 <= MAX_FRAME_BYTES && line.last() == Some(&b'\n'),
            "Native worker frame exceeds limit or is truncated"
        );
        let request: Request =
            serde_json::from_slice(&line).context("Invalid native worker request")?;
        let result =
            crate::stellar_overlay::with_host_clock(request.now_ms, || -> Result<Value> {
                anyhow::ensure!(
                    request.protocol == PROTOCOL,
                    "Unsupported native worker protocol {}",
                    request.protocol
                );
                match request.method.as_str() {
                    "hello" => encode(WorkerInfo::current(kind)),
                    "initialize" => {
                        anyhow::ensure!(worker.is_none(), "Native worker already initialized");
                        worker = Some(Worker::new(kind, &request.args)?);
                        Ok(Value::Null)
                    }
                    "shutdown" => {
                        if let Some(Worker::Hecate(w)) = worker.as_mut() {
                            w.flush()?;
                        }
                        Ok(Value::Null)
                    }
                    method => worker
                        .as_mut()
                        .context("Native worker not initialized")?
                        .call(method, &request.args),
                }
            });
        let response = match result {
            Ok(result) => Response {
                protocol: PROTOCOL,
                id: request.id,
                result,
                error: None,
            },
            Err(error) => Response {
                protocol: PROTOCOL,
                id: request.id,
                result: Value::Null,
                error: Some(format!("{error:#}")),
            },
        };
        let encoded = serde_json::to_vec(&response)?;
        anyhow::ensure!(
            (encoded.len() as u64) < MAX_FRAME_BYTES,
            "Native worker response exceeds frame limit"
        );
        output.write_all(&encoded)?;
        output.write_all(b"\n")?;
        output.flush()?;
        if request.method == "shutdown" {
            return Ok(());
        }
    }
}
