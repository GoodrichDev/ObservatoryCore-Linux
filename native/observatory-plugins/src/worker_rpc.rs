//! Versioned process boundary for independently updated native Rust workers.
//!
//! The default host uses the same workers directly. An explicitly installed
//! native artifact runs through these typed adapters; Windows DLLs are never
//! loaded. Configuration and journal payloads travel over stdin, not argv.
mod client;
mod facade;
mod server;

pub use client::{inspect, Client};
pub use facade::{BioInsights, Evaluator, Hecate, StellarOverlay};
pub use server::serve;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const PROTOCOL: u32 = 1;
pub const MAX_FRAME_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WorkerKind {
    BioInsights,
    Evaluator,
    Hecate,
    StellarOverlay,
}
impl WorkerKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::BioInsights => "BioInsights",
            Self::Evaluator => "Evaluator",
            Self::Hecate => "Hecate",
            Self::StellarOverlay => "StellarOverlay",
        }
    }
    pub fn source_version(self) -> &'static str {
        match self {
            Self::BioInsights => crate::bioinsights::VERSION,
            Self::Evaluator => crate::evaluator::VERSION,
            Self::Hecate => crate::hecate::VERSION,
            Self::StellarOverlay => crate::stellar_overlay::SOURCE_VERSION,
        }
    }
}
impl std::str::FromStr for WorkerKind {
    type Err = anyhow::Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        [
            Self::BioInsights,
            Self::Evaluator,
            Self::Hecate,
            Self::StellarOverlay,
        ]
        .into_iter()
        .find(|kind| kind.name() == value)
        .ok_or_else(|| anyhow::anyhow!("Unknown native worker {value}"))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WorkerInfo {
    pub protocol: u32,
    pub plugin: WorkerKind,
    pub source_version: String,
    pub native_version: String,
    pub platform: String,
}
impl WorkerInfo {
    pub fn current(kind: WorkerKind) -> Self {
        Self {
            protocol: PROTOCOL,
            plugin: kind,
            source_version: kind.source_version().into(),
            native_version: env!("CARGO_PKG_VERSION").into(),
            platform: format!(
                "{}-unknown-linux-{}",
                std::env::consts::ARCH,
                if cfg!(target_env = "musl") {
                    "musl"
                } else {
                    "gnu"
                }
            ),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    protocol: u32,
    id: u64,
    now_ms: u64,
    method: String,
    #[serde(default)]
    args: Value,
}
#[derive(Debug, Serialize, Deserialize)]
struct Response {
    protocol: u32,
    id: u64,
    #[serde(default)]
    result: Value,
    #[serde(default)]
    error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn protocol_lifecycle_errors_are_recoverable_and_stellar_uses_host_clock() {
        let requests = [
            (999, "hello", Value::Null, 50_000_000),
            (PROTOCOL, "snapshot", Value::Null, 50_000_000),
            (PROTOCOL, "hello", Value::Null, 50_000_000),
            (
                PROTOCOL,
                "initialize",
                json!({"settings":{},"data_dir":"unused"}),
                50_000_000,
            ),
            (
                PROTOCOL,
                "on_notification",
                json!({"notification":{"Rendering":1,"Title":"Host clock","Detail":"Real alert","Guid":"00000000-0000-0000-0000-000000000123","XPos":20,"YPos":30,"Timeout":5000}}),
                50_000_000,
            ),
            (PROTOCOL, "snapshot", Value::Null, 50_000_001),
            (PROTOCOL, "initialize", json!({"settings":{}}), 50_000_001),
            (
                PROTOCOL,
                "handle_message",
                json!({"sender":"Core","command":"TestRadar","payload":{}}),
                50_000_000,
            ),
            (PROTOCOL, "snapshot", Value::Null, 50_000_001),
            (PROTOCOL, "snapshot", Value::Null, 50_006_001),
            (PROTOCOL, "shutdown", Value::Null, 50_006_001),
        ];
        let mut input = Vec::new();
        for (index, (protocol, method, args, now_ms)) in requests.into_iter().enumerate() {
            serde_json::to_writer(
                &mut input,
                &Request {
                    protocol,
                    id: index as u64 + 1,
                    now_ms,
                    method: method.into(),
                    args,
                },
            )
            .unwrap();
            input.push(b'\n');
        }
        let mut output = Vec::new();
        serve(
            WorkerKind::StellarOverlay,
            std::io::Cursor::new(input),
            &mut output,
        )
        .unwrap();
        let responses: Vec<Response> = output
            .split(|b| *b == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_slice(line).unwrap())
            .collect();
        assert_eq!(responses.len(), 11);
        for (index, response) in responses.iter().enumerate() {
            assert_eq!(response.id, index as u64 + 1);
            assert_eq!(response.protocol, PROTOCOL);
        }
        assert!(responses[0].error.as_ref().unwrap().contains("Unsupported"));
        assert!(responses[1]
            .error
            .as_ref()
            .unwrap()
            .contains("not initialized"));
        assert_eq!(responses[2].result["plugin"], "StellarOverlay");
        let notification = &responses[5].result["notifications"][0];
        assert_eq!(notification["created_ms"], 50_000_000u64);
        assert_eq!(notification["expires_ms"], 50_005_000u64);
        assert_eq!(responses[5].result["sampled_ms"], 50_000_001u64);
        assert!(responses[6]
            .error
            .as_ref()
            .unwrap()
            .contains("already initialized"));
        assert_eq!(responses[8].result["force_show"], true);
        assert_eq!(responses[9].result["force_show"], false);
    }
    #[test]
    fn truncated_frames_are_rejected_before_dispatch() {
        let mut output = Vec::new();
        let error = serve(
            WorkerKind::Hecate,
            std::io::Cursor::new(b"{\"protocol\":1}"),
            &mut output,
        )
        .unwrap_err();
        assert!(error.to_string().contains("truncated"));
        assert!(output.is_empty());
    }
}
