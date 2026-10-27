//! Captures use the original installed Framework DLL (Windows ReadyToRun disabled)
//! and unchanged installed Explorer manager source, with real NLua 1.7.3.
//! These exercise the separately built Linux CLR host over its actual JSONL pipe.
#![cfg(target_os = "linux")]

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

fn host_path() -> PathBuf {
    if let Some(value) = std::env::var_os("OBSERVATORY_CLR_CRITERIA_HOST") {
        let path = PathBuf::from(value);
        assert!(
            path.is_file(),
            "OBSERVATORY_CLR_CRITERIA_HOST must point to the actual host executable"
        );
        return path;
    }
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let candidates = [
        Some(repo.join("native/observatory-clr-criteria/bin/Release/net8.0/linux-x64/publish/observatory-clr-criteria")),
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache/observatory-clr-criteria-net8/observatory-clr-criteria")),
    ];
    candidates.into_iter().flatten().find(|path| path.is_file())
        .expect("Build the CLR criteria host with native/scripts/build-clr-criteria.sh, or set OBSERVATORY_CLR_CRITERIA_HOST")
}

// Keep test time-zone settings local to the child process, never mutate the
// parallel Rust test runner's environment. The original source converts UTC to
// local time, so the oracle and its comparison process must use the same zone.
struct UtcHost(PathBuf);
impl UtcHost {
    fn new() -> Self {
        use std::os::unix::fs::PermissionsExt;
        let path =
            std::env::temp_dir().join(format!("clr-criteria-utc-host-{}.sh", std::process::id()));
        let executable = host_path().to_string_lossy().replace('\'', "'\\''");
        std::fs::write(&path, format!("#!/bin/sh\nTZ=UTC exec '{executable}'\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        Self(path)
    }
}
impl Drop for UtcHost {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

struct Host {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}

impl Host {
    fn start() -> Self {
        let mut child = Command::new(host_path())
            .env("TZ", "UTC")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("start actual Linux CLR host");
        let input = child.stdin.take().unwrap();
        let output = BufReader::new(child.stdout.take().unwrap());
        Self {
            child,
            input,
            output,
        }
    }

    fn send(&mut self, request: &Value) -> Value {
        serde_json::to_writer(&mut self.input, request).unwrap();
        self.input.write_all(b"\n").unwrap();
        self.input.flush().unwrap();
        let mut line = String::new();
        assert_ne!(
            self.output.read_line(&mut line).unwrap(),
            0,
            "CLR host terminated"
        );
        let response: Value =
            serde_json::from_str(&line).expect("unpolluted JSONL protocol response");
        // Extra protocol metadata is outside the original manager's API.
        json!({
            "ok": response["ok"],
            "results": response["results"].as_array().unwrap().iter().map(|row| json!({"title":row["title"],"detail":row["detail"]})).collect::<Vec<_>>(),
            "notifications": response["notifications"],
            "diagnostics": response["diagnostics"],
        })
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn upsert(history: &mut Vec<Value>, scan: Value, replace: bool) {
    let index = history.iter().position(|previous| {
        previous["BodyID"] == scan["BodyID"] && previous["SystemAddress"] == scan["SystemAddress"]
    });
    if let Some(index) = index {
        if replace {
            history[index] = scan;
        }
    } else {
        history.push(scan);
    }
}

fn rewrite(path: &std::path::Path, source: &str) {
    use std::time::{Duration, SystemTime};
    std::fs::write(path, source).unwrap();
    let file = std::fs::File::options().write(true).open(path).unwrap();
    file.set_times(
        std::fs::FileTimes::new().set_modified(SystemTime::now() + Duration::from_secs(2)),
    )
    .unwrap();
}

fn expected_protocol_response(expected: &Value) -> Value {
    let mut expected = expected.clone();
    for result in expected["results"].as_array_mut().unwrap() {
        result.as_object_mut().unwrap().remove("body");
    }
    expected
}

#[test]
fn original_installed_framework_oracle_matches_actual_linux_clr_host() {
    let inputs: Value =
        serde_json::from_str(include_str!("../../fixtures/clr-criteria-inputs.json")).unwrap();
    let oracle: Value =
        serde_json::from_str(include_str!("../../fixtures/clr-criteria-oracle.json")).unwrap();
    assert_eq!(inputs.as_array().unwrap().len(), 27);
    let mut host = Host::start();
    let mut responses = 0;
    for (case, expected_case) in inputs
        .as_array()
        .unwrap()
        .iter()
        .zip(oracle["cases"].as_array().unwrap())
    {
        assert_eq!(case["id"], expected_case["id"]);
        let path = std::env::temp_dir().join(format!(
            "clr-criteria-test-{}-{}.lua",
            std::process::id(),
            case["id"].as_str().unwrap()
        ));
        std::fs::write(&path, case["source"].as_str().unwrap()).unwrap();
        let actual = host.send(&json!({"op":"init","path":path,"read_all":false}));
        assert_eq!(
            actual, expected_case["responses"][0],
            "{} initialization",
            case["id"]
        );
        responses += 1;
        let mut history = Vec::new();
        let mut signals = Vec::new();
        let mut extra_paths = Vec::new();
        for (index, step) in case["steps"].as_array().unwrap().iter().enumerate() {
            let mut request = step.clone();
            if step["op"] == "process" {
                if let Some(snapshot) = step["system"].as_array() {
                    for body in snapshot {
                        upsert(&mut history, body.clone(), false);
                    }
                }
                if step["event"]["event"] == "Scan" {
                    upsert(&mut history, step["event"].clone(), true);
                }
                if step["event"]["event"] == "FSSBodySignals" {
                    upsert(&mut signals, step["event"].clone(), false);
                }
                request["system"] = json!(history);
                request["signals"] = json!(signals);
            }
            if step["op"] == "set_path" {
                let next_path = path.with_extension("next.lua");
                std::fs::write(&next_path, step["source"].as_str().unwrap()).unwrap();
                request = json!({"op":"set_path","path":next_path});
                extra_paths.push(next_path);
            }
            if step["op"] == "wait" {
                std::thread::sleep(std::time::Duration::from_millis(
                    step["milliseconds"].as_u64().unwrap(),
                ));
                request = json!({"op":"poll"});
            }
            let actual = if step["op"] == "rewrite" {
                rewrite(&path, step["source"].as_str().unwrap());
                json!({"ok":true,"results":[],"notifications":[],"diagnostics":[]})
            } else {
                host.send(&request)
            };
            assert_eq!(
                actual,
                expected_protocol_response(&expected_case["responses"][index + 1]),
                "{} step {}",
                case["id"],
                index + 1
            );
            responses += 1;
        }
        std::fs::remove_file(path).unwrap();
        for extra in extra_paths {
            std::fs::remove_file(extra).unwrap();
        }
    }
    assert!(
        responses >= 39,
        "exercise initialization, persistence, errors and mode transitions"
    );
}

#[test]
fn lua_and_clr_standard_output_do_not_corrupt_host_protocol() {
    let path = std::env::temp_dir().join(format!("clr-criteria-stdout-{}.lua", std::process::id()));
    std::fs::write(
        &path,
        r#"---@Global
luanet.load_assembly("System.Console")
Console = luanet.import_type("System.Console")
print("synthetic Lua stdout")
io.write("synthetic Lua io.stdout\n")
Console.WriteLine("synthetic CLR stdout")
---@End
---@Complex stdout
print("synthetic handler stdout")
return true, "stdout", scan:GetType().Name
---@End
"#,
    )
    .unwrap();
    let mut host = Host::start();
    let initialized = host.send(&json!({"op":"init","path":path}));
    assert_eq!(initialized["ok"], true);
    assert_eq!(initialized["diagnostics"], json!([]));
    let response = host.send(&json!({"op":"process","event":{"event":"Scan","BodyID":1,"SystemAddress":42,"timestamp":"2026-10-26T01:02:03Z"},"system":[]}));
    assert_eq!(
        response["results"],
        json!([{"title":"stdout","detail":"Scan"}])
    );
    assert_eq!(response["diagnostics"], json!([]));
    std::fs::remove_file(path).unwrap();
}

fn check_pipeline_response(
    criteria: &mut observatory_plugins::criteria::Criteria,
    actual: Vec<observatory_domain::Finding>,
    expected: &Value,
    event: Option<&observatory_domain::JournalEvent>,
    current_system: &str,
    label: &str,
) {
    let mut expected_findings = Vec::new();
    let timestamp = event.map(|event| event.timestamp.as_str()).unwrap_or("");
    let system = if event.is_some() {
        "CLR Oracle"
    } else {
        current_system
    };
    for note in expected["notifications"].as_array().unwrap() {
        expected_findings.push(json!({"plugin":"Explorer","timestamp":timestamp,"system":system,"body":note["title"],"title":note["detail"],"detail":note["extended_detail"]}));
    }
    for result in expected["results"].as_array().unwrap() {
        expected_findings.push(json!({"plugin":"Explorer","timestamp":timestamp,"system":system,"body":result["body"],"title":result["title"],"detail":result["detail"]}));
    }
    assert_eq!(
        serde_json::to_value(&actual).unwrap(),
        json!(expected_findings),
        "{label} findings"
    );
    for (finding, note) in actual
        .iter()
        .zip(expected["notifications"].as_array().unwrap())
    {
        let payload = criteria
            .take_notification_payload(finding)
            .expect("source callback notification payload survives pipeline");
        assert_eq!(
            payload,
            json!({"title":note["title"],"detail":note["detail"],"extended_details":note["extended_detail"],"coalescing_id":note["body_id"].as_i64().unwrap_or(-1),"sender":"Explorer","grid_time":note["time"]}),
            "{label} notification payload"
        );
    }
    let actual_diagnostics = criteria
        .take_diagnostics()
        .into_iter()
        .map(|mut diagnostic| {
            diagnostic.as_object_mut().unwrap().remove("Path");
            diagnostic
        })
        .collect::<Vec<_>>();
    let expected_diagnostics = expected["diagnostics"].as_array().unwrap().iter().map(|diagnostic| json!({"Level":"Error","Message":diagnostic["message"],"Detail":diagnostic["detail"],"ExceptionType":diagnostic["exception_type"]})).collect::<Vec<_>>();
    assert_eq!(
        actual_diagnostics, expected_diagnostics,
        "{label} diagnostics"
    );
}

#[test]
fn original_clr_oracle_matches_full_rust_criteria_pipeline() {
    use observatory_domain::JournalEvent;
    use observatory_plugins::{criteria::Criteria, explorer::Bodies};
    let inputs: Value =
        serde_json::from_str(include_str!("../../fixtures/clr-criteria-inputs.json")).unwrap();
    let oracle: Value =
        serde_json::from_str(include_str!("../../fixtures/clr-criteria-oracle.json")).unwrap();
    let host = UtcHost::new();
    for (case, expected_case) in inputs
        .as_array()
        .unwrap()
        .iter()
        .zip(oracle["cases"].as_array().unwrap())
    {
        let id = case["id"].as_str().unwrap();
        let path = std::env::temp_dir().join(format!(
            "clr-criteria-pipeline-{}-{id}.lua",
            std::process::id()
        ));
        std::fs::write(&path, case["source"].as_str().unwrap()).unwrap();
        let mut criteria = Criteria::with_managed_runtime(path.clone(), &host.0).unwrap();
        let startup = criteria.take_startup_findings();
        check_pipeline_response(
            &mut criteria,
            startup,
            &expected_case["responses"][0],
            None,
            "",
            id,
        );
        let mut history = Vec::new();
        let mut signal_history = Vec::new();
        let mut extra_paths = Vec::new();
        for (index, step) in case["steps"].as_array().unwrap().iter().enumerate() {
            let label = format!("{id} step {}", index + 1);
            let mut parsed_event = None;
            let actual = match step["op"].as_str().unwrap() {
                "process" => {
                    if let Some(snapshot) = step["system"].as_array() {
                        for body in snapshot {
                            upsert(&mut history, body.clone(), false);
                        }
                    }
                    let event = JournalEvent::parse(step["raw_json"].as_str().unwrap()).unwrap();
                    if event.event == "Scan" {
                        upsert(&mut history, event.data.clone(), true);
                    }
                    if event.event == "FSSBodySignals" {
                        upsert(&mut signal_history, event.data.clone(), false);
                    }
                    let bodies: Bodies = history
                        .iter()
                        .map(|body| (body["BodyID"].as_u64().unwrap(), body.clone()))
                        .collect();
                    let signals: Bodies = signal_history
                        .iter()
                        .map(|body| (body["BodyID"].as_u64().unwrap(), body.clone()))
                        .collect();
                    let findings = criteria.process(&event, &bodies, &signals, "CLR Oracle");
                    parsed_event = Some(event);
                    findings
                }
                "set_read_all" => {
                    criteria.set_read_all(step["value"].as_bool().unwrap());
                    criteria.take_startup_findings()
                }
                "set_path" => {
                    let next_path = path.with_extension("next.lua");
                    std::fs::write(&next_path, step["source"].as_str().unwrap()).unwrap();
                    criteria.set_path(next_path.clone());
                    extra_paths.push(next_path);
                    criteria.take_startup_findings()
                }
                "rewrite" => {
                    rewrite(&path, step["source"].as_str().unwrap());
                    Vec::new()
                }
                "wait" => {
                    std::thread::sleep(std::time::Duration::from_millis(
                        step["milliseconds"].as_u64().unwrap(),
                    ));
                    criteria.poll("CLR Oracle")
                }
                op => panic!("unhandled fixture operation {op}"),
            };
            check_pipeline_response(
                &mut criteria,
                actual,
                &expected_case["responses"][index + 1],
                parsed_event.as_ref(),
                if step["op"] == "wait" {
                    "CLR Oracle"
                } else {
                    ""
                },
                &label,
            );
        }
        std::fs::remove_file(path).unwrap();
        for extra in extra_paths {
            std::fs::remove_file(extra).unwrap();
        }
    }
}

#[test]
fn pending_global_notification_retains_payload_until_first_event() {
    use observatory_domain::JournalEvent;
    use observatory_plugins::{criteria::Criteria, explorer::Bodies};
    let path = std::env::temp_dir().join(format!(
        "clr-criteria-pending-global-{}.lua",
        std::process::id()
    ));
    std::fs::write(
        &path,
        "---@Global\nnotifyForBody('global-body','global-title','global-extended',7)\n---@End\n",
    )
    .unwrap();
    let mut criteria = Criteria::with_managed_runtime(path.clone(), &host_path()).unwrap();
    let scan = JournalEvent::parse(r#"{ "event":"Scan", "timestamp":"2026-10-26T01:02:03Z", "SystemAddress":42,"BodyID":7,"BodyName":"CLR Oracle 7" }"#).unwrap();
    let rows = criteria.process(&scan, &Bodies::new(), &Bodies::new(), "CLR Oracle");
    assert_eq!(rows.len(), 1);
    assert_eq!(
        criteria.take_notification_payload(&rows[0]),
        Some(
            json!({"title":"global-body","detail":"global-title","extended_details":"global-extended","coalescing_id":7,"sender":"Explorer","grid_time":""})
        )
    );
    std::fs::remove_file(path).unwrap();
}

#[test]
fn managed_runtime_exit_is_reported_once_without_replaying_script_effects() {
    use observatory_domain::JournalEvent;
    use observatory_plugins::{criteria::Criteria, explorer::Bodies};
    let directory =
        std::env::temp_dir().join(format!("clr-criteria-no-replay-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("criteria.lua");
    let receipt = directory.join("synthetic-effects.txt");
    let source = format!(
        r#"---@Global
luanet.load_assembly("System.Private.CoreLib")
luanet.load_assembly("System.Runtime")
File = luanet.import_type("System.IO.File")
Environment = luanet.import_type("System.Environment")
File.AppendAllText([[{}]], "global\n")
---@End
---@Complex child-exit
File.AppendAllText([[{}]], "handler\n")
Environment.Exit(23)
---@End
"#,
        receipt.display(),
        receipt.display()
    );
    std::fs::write(&path, source).unwrap();
    let mut criteria = Criteria::with_managed_runtime(path.clone(), &host_path()).unwrap();
    assert!(criteria.take_diagnostics().is_empty());
    let scan = JournalEvent::parse(
        r#"{"event":"Scan","timestamp":"2026-10-26T01:02:03Z","SystemAddress":42,"BodyID":1}"#,
    )
    .unwrap();
    assert!(criteria
        .process(&scan, &Bodies::new(), &Bodies::new(), "CLR Oracle")
        .is_empty());
    let first = criteria.take_diagnostics();
    assert_eq!(first.len(), 1, "surface the child runtime exit once");
    assert!(first[0]["Message"].as_str().unwrap().contains("CLR"));
    assert!(criteria
        .process(&scan, &Bodies::new(), &Bodies::new(), "CLR Oracle")
        .is_empty());
    assert!(
        criteria.take_diagnostics().is_empty(),
        "do not emit the same terminal failure repeatedly"
    );
    assert_eq!(
        std::fs::read_to_string(&receipt).unwrap(),
        "global\nhandler\n",
        "neither globals nor handler side effects are replayed after an uncertain child exit"
    );
    std::fs::remove_file(path).unwrap();
    std::fs::remove_file(receipt).unwrap();
    std::fs::remove_dir(directory).unwrap();
}

#[test]
fn failed_initial_globals_are_not_replayed_by_events_readall_or_reset() {
    const CHILD_DIRECTORY: &str = "OBSERVATORY_TEST_CLR_INITIAL_FAILURE_DIRECTORY";
    if let Some(directory) = std::env::var_os(CHILD_DIRECTORY) {
        run_initial_failure_probe(&PathBuf::from(directory));
        return;
    }
    // Set production runtime resolution only on an isolated child test process.
    // Parallel tests in this process never observe temporary environment changes.
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("clr-load-failure-{}-{nonce}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "failed_initial_globals_are_not_replayed_by_events_readall_or_reset",
            "--nocapture",
        ])
        .env(CHILD_DIRECTORY, &directory)
        .env("OBSERVATORY_CLR_CRITERIA_HOST", host_path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "isolated Criteria::load probe failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    for file in ["criteria.lua", "changed-path.lua", "synthetic-effects.txt"] {
        std::fs::remove_file(directory.join(file)).unwrap();
    }
    std::fs::remove_dir(directory).unwrap();
}

fn run_initial_failure_probe(directory: &std::path::Path) {
    use observatory_domain::JournalEvent;
    use observatory_plugins::{criteria::Criteria, explorer::Bodies};
    let path = directory.join("criteria.lua");
    let changed_path = directory.join("changed-path.lua");
    let receipt = directory.join("synthetic-effects.txt");
    let globals = |label: &str, crashes: bool| {
        format!(
            r#"---@Global
luanet.load_assembly("System.Private.CoreLib")
luanet.load_assembly("System.Runtime")
File = luanet.import_type("System.IO.File")
Environment = luanet.import_type("System.Environment")
File.AppendAllText([[{}]], "{}\n")
{}
---@End
---@Complex recovered
return true, "recovered", "ok"
---@End
"#,
            receipt.display(),
            label,
            if crashes { "Environment.Exit(23)" } else { "" }
        )
    };
    let event = |id| {
        JournalEvent::parse(&format!(r#"{{"event":"Scan","timestamp":"2026-10-26T01:02:03Z","SystemAddress":42,"BodyID":{id},"BodyName":"synthetic"}}"#)).unwrap()
    };
    let process = |criteria: &mut Criteria, id| {
        criteria.process(&event(id), &Bodies::new(), &Bodies::new(), "CLR Oracle")
    };
    let transitions = |criteria: &mut Criteria, id| {
        criteria.set_read_all(true);
        assert!(process(criteria, id).is_empty());
        criteria.set_read_all(false);
        criteria.reset();
        assert!(process(criteria, id + 1).is_empty());
    };
    std::fs::write(&path, globals("initial", true)).unwrap();
    let mut criteria = Criteria::load(path.clone()).unwrap();
    assert_eq!(criteria.take_diagnostics().len(), 1);
    for id in 1..=3 {
        assert!(process(&mut criteria, id).is_empty());
    }
    transitions(&mut criteria, 4);
    assert!(
        criteria.take_diagnostics().is_empty(),
        "one terminal failure diagnostic per attempted script revision"
    );
    assert_eq!(std::fs::read_to_string(&receipt).unwrap(), "initial\n");

    rewrite(&path, &globals("edited", true));
    assert!(process(&mut criteria, 6).is_empty());
    assert_eq!(criteria.take_diagnostics().len(), 1);
    for id in 7..=8 {
        assert!(process(&mut criteria, id).is_empty());
    }
    transitions(&mut criteria, 9);
    assert!(criteria.take_diagnostics().is_empty());
    assert_eq!(
        std::fs::read_to_string(&receipt).unwrap(),
        "initial\nedited\n",
        "a failing edited revision is attempted exactly once"
    );

    std::fs::write(&changed_path, globals("changed-path", true)).unwrap();
    criteria.set_path(changed_path.clone());
    assert!(process(&mut criteria, 11).is_empty());
    assert_eq!(criteria.take_diagnostics().len(), 1);
    assert!(process(&mut criteria, 12).is_empty());
    transitions(&mut criteria, 13);
    assert!(criteria.take_diagnostics().is_empty());
    assert_eq!(
        std::fs::read_to_string(&receipt).unwrap(),
        "initial\nedited\nchanged-path\n",
        "a failing new path is attempted exactly once"
    );

    rewrite(&changed_path, &globals("recovered", false));
    for id in [15, 16] {
        let findings = process(&mut criteria, id);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].title, "recovered");
    }
    assert!(criteria.take_diagnostics().is_empty());
    assert_eq!(
        std::fs::read_to_string(&receipt).unwrap(),
        "initial\nedited\nchanged-path\nrecovered\n",
        "editing the file permits recovery without repeated startup effects"
    );
}
