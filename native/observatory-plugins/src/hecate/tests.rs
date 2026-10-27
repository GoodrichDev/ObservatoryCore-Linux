use super::*;
fn event(mut value: Value) -> JournalEvent {
    if value.get("timestamp").is_none() {
        value["timestamp"] = json!("2026-01-01T00:00:00Z");
    }
    JournalEvent::parse(&value.to_string()).unwrap()
}
fn vault() -> Hecate {
    Hecate::new(json!({"RemoteLookups":false})).unwrap()
}
fn message(vault: &mut Hecate, command: &str, value: Value) -> Value {
    vault.handle_message(command, &value).unwrap().unwrap()["Payload"].clone()
}

#[test]
fn archive_whitelist_matches_all_source_dispatch_captures() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/hecate-archive-dispatch-oracle.json"
    ))
    .unwrap();
    let cases = fixture["Captures"].as_array().unwrap();
    assert_eq!(cases.len(), 282);
    for case in cases {
        let kind = case["Type"].as_str().unwrap();
        assert_eq!(
            archives_event(kind),
            case["Archive"].as_bool().unwrap(),
            "source capture {case}"
        );
    }
    let mut vault = vault();
    vault.current_id = 42;
    for kind in [
        "MarketFile",
        "CargoFile",
        "FutureJournalEvent",
        "LoadGame",
        "StartJump",
    ] {
        vault.on_event(&event(json!({"event":kind})));
    }
    assert!(vault.journal_text.is_empty());
    vault.current_id = 42;
    vault.on_event(&event(
        json!({"event":"CarrierJump","Docked":false,"OnFoot":false,"SystemAddress":84}),
    ));
    assert_eq!(vault.current_id, 42);
    assert!(vault.journal_text.contains("CarrierJump"));
    assert!(vault.errors().is_empty());
}

#[test]
fn archives_original_journal_json_bytes() {
    let mut vault = vault();
    vault.on_event(&event(
        json!({"event":"Location","SystemAddress":42,"StarSystem":"Original"}),
    ));
    let raw = r#"{ "timestamp":"2026-01-01T01:00:00Z", "event" : "Scan", "BodyName":"Original 1", "Future": 1e+3 }"#;
    vault.on_event(&JournalEvent::parse(raw).unwrap());
    assert!(vault.journal_text.contains(raw));
    assert!(!vault.journal_text.contains("\"Future\":1000"));
    assert!(vault.errors().is_empty());
}

#[test]
fn archives_system_visits_preserves_unknown_fields_and_unsigned_ids() {
    let mut vault = vault();
    vault.on_event(&event(
        json!({"event":"LoadGame","Commander":"Test Commander","FID":"F123"}),
    ));
    vault.on_event(&event(
        json!({"event":"Location","SystemAddress":u64::MAX,"StarSystem":"First","StarPos":[1,2,3]}),
    ));
    vault.on_event(&event(
        json!({"event":"Scan","NewField":{"future":[1,true,"unchanged"]}}),
    ));
    vault.on_event(&event(json!({"event":"FSDJump","SystemAddress":42,"StarSystem":"Second","timestamp":"2026-01-01T01:00:00Z"})));
    vault.on_event(&event(
        json!({"event":"Shutdown","timestamp":"2026-01-01T01:01:00Z"}),
    ));
    let loaded = message(
        &mut vault,
        "LoadSystem",
        json!({"ID64":u64::MAX,"DateTime":"2026-01-02T00:00:00Z"}),
    );
    assert_eq!(loaded["ID64"], json!(u64::MAX));
    assert_eq!(loaded["FIDs"], json!([123, 123]));
    assert_eq!(loaded["JournalEvents"][0]["SystemAddress"], json!(u64::MAX));
    assert_eq!(
        loaded["JournalEvents"][1]["NewField"]["future"],
        json!([1, true, "unchanged"])
    );
    assert_eq!(
        message(
            &mut vault,
            "GetSystemsVisitedFromList",
            json!({"ID64s":[42,55,u64::MAX]})
        )["ID64s"],
        json!([42, u64::MAX])
    );
    assert_eq!(
        message(&mut vault, "GetFID", json!({"Name":"Test Commander"}))["FID"],
        123
    );
    assert!(vault.errors().is_empty());
}

#[test]
fn evaluator_vault_upserts_and_keeps_commanders_separate() {
    let mut vault = vault();
    for (fid, body, value) in [(7, 1, 100), (7, 2, 200), (8, 1, 900), (7, 1, 500)] {
        vault.handle_message("EvaluatorAdd",&json!({"FID":fid,"ID64":42,"BodyID":body,"Value":value,"Name":"System A","DateTime":"2026-01-01T00:00:00Z"})).unwrap();
    }
    vault
        .handle_message(
            "EvaluatorSysAdd",
            &json!({"FID":7,"ID64":42,"Name":"System A","DateTime":"2026-01-01T00:00:00Z"}),
        )
        .unwrap();
    assert_eq!(vault.evaluator_total(7).unwrap(), 700);
    assert_eq!(vault.evaluator_total(8).unwrap(), 900);
    let result = message(
        &mut vault,
        "EvaluatorGetData",
        json!({"FID":7,"Response":"callback-token"}),
    );
    assert_eq!(result["Processed"], true);
    assert_eq!(
        result["Data"],
        json!([["System A", "2026-01-01T00:00:00Z", 700]])
    );
    assert_eq!(result["Response"], "callback-token");
    assert_eq!(
        message(
            &mut vault,
            "EvaluatorGetTotal",
            json!({"FID":7,"SubtractCurrent":true})
        )["SubtractCurrent"],
        true
    );
    vault
        .handle_message("EvaluatorDelete", &json!({"FID":7}))
        .unwrap();
    assert_eq!(vault.evaluator_total(7).unwrap(), 0);
    assert_eq!(vault.evaluator_total(8).unwrap(), 900);
}

#[test]
fn unsold_biology_defaults_to_first_discovery_and_persists_explicit_bonus() {
    let mut vault = vault();
    let row = json!({"FID":"F7","ID64":42,"BodyID":3,"BioName":"Stratum Tectonicas","DateTime":"2026-01-01T00:00:00Z","Species":12,"Value":19010800,"BodyName":"System A 1"});
    vault.handle_message("BioInsightsUnsoldAdd", &row).unwrap();
    assert_eq!(vault.bioinsights_total(7).unwrap(), 95_054_000);
    let id = vault.bioinsights_data(7).unwrap()[0][0].as_i64().unwrap();
    vault
        .handle_message("BioInsightsSetBonus", &json!({"ID":id,"Value":false}))
        .unwrap();
    assert_eq!(vault.bioinsights_total(7).unwrap(), 19_010_800);
    vault.handle_message("BioInsightsUnsoldAdd", &row).unwrap();
    assert_eq!(vault.bioinsights_total(7).unwrap(), 19_010_800);
    assert_eq!(vault.bioinsights_data(7).unwrap().len(), 1);
    vault
        .handle_message("BioInsightsUnsoldDelete", &json!({"FID":7}))
        .unwrap();
    assert_eq!(vault.bioinsights_total(7).unwrap(), 0);
}

#[test]
fn merit_totals_exclude_duplicates_and_apply_each_original_scope() {
    let mut vault = vault();
    for (cycle, session, system, gained, dupe) in [
        (1, 10, 42, 100, false),
        (1, 11, 42, 50, false),
        (2, 11, 84, 25, false),
        (1, 10, 42, 999, true),
    ] {
        vault.handle_message("PIAddMerits",&json!({"System":1,"TargetSystem":system,"FID":7,"Power":2,"Cycle":cycle,"Session":session,"Activity":3,"Gained":gained,"Total":1000,"DateTime":"2026-01-01T00:00:00Z","Dupe":dupe})).unwrap();
    }
    let totals = message(
        &mut vault,
        "GetCurrentMerits",
        json!({"FID":7,"Power":2,"Cycle":1,"Session":10,"Since":0,"System":42,"UpdateUI":true}),
    );
    assert_eq!(
        totals,
        json!({"TotalCycleMerits":150,"TotalDailyMerits":175,"TotalSessionMerits":100,"SystemCycleMerits":150,"SystemDailyMerits":150,"SystemSessionMerits":100,"UpdateUI":true})
    );
    assert_eq!(
        message(
            &mut vault,
            "PIGetMerits",
            json!({"FID":7,"Power":2,"Response":1})
        )["Data"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
}

#[test]
fn remote_cache_queues_native_http_and_never_performs_io_in_callback() {
    let mut vault = Hecate::new(default_settings()).unwrap();
    vault.set_batch(true);
    vault.on_event(&event(
        json!({"event":"Location","SystemAddress":42,"StarSystem":"System A"}),
    ));
    assert!(vault.take_http_requests().is_empty());
    vault.set_batch(false);
    assert!(vault
        .handle_message("SpanshSystem", &json!({"ID64":42,"Response":"opaque"}))
        .unwrap()
        .is_none());
    let requests = vault.take_http_requests();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0]["url"], "https://spansh.co.uk/api/dump/42");
    vault.handle_http_response(&json!({"request_id":requests[0]["request_id"],"ok":true,"body":"{\"system\":{\"name\":\"System A\",\"id64\":42}}"})).unwrap();
    let replies = vault.take_messages();
    assert_eq!(replies[0]["Payload"]["System"]["name"], "System A");
    assert_eq!(replies[0]["Payload"]["Response"], "opaque");
    assert_eq!(
        message(&mut vault, "SpanshSystem", json!({"ID64":42}))["System"]["id64"],
        42
    );
    assert!(vault.take_http_requests().is_empty());
}

#[test]
fn malformed_messages_do_not_mutate_the_vault() {
    let mut vault = vault();
    assert!(vault
        .handle_message("EvaluatorAdd", &json!({"FID":1,"ID64":42}))
        .is_err());
    assert!(vault
        .handle_message("PIAddMerits", &json!({"FID":1}))
        .is_err());
    assert!(vault
        .handle_message("EvaluatorDelete", &json!({"FID":"not an id"}))
        .is_err());
    assert_eq!(vault.evaluator_total(1).unwrap(), 0);
}

#[test]
fn census_records_npc_pilots_and_applies_original_two_hour_window() {
    let mut vault = vault();
    vault.on_event(&event(
        json!({"event":"Location","SystemAddress":42,"StarSystem":"NPC System"}),
    ));
    vault.on_event(&event(
        json!({"event":"ReceiveText","Channel":"player","From":"CMDR Player"}),
    ));
    for hour in [0, 1, 3] {
        vault.on_event(&event(json!({"event":"ReceiveText","Channel":"npc","From":"$npc_name_decorate:#name=Test Pilot;","From_Localised":"Test Pilot","timestamp":format!("2026-01-01T{hour:02}:00:00Z")})));
    }
    let (rows, count): (i64, i64) = vault
        .general
        .query_row("SELECT COUNT(*),SUM(count) FROM census", [], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .unwrap();
    assert_eq!((rows, count), (1, 2));
    assert!(vault.errors().is_empty());
}

#[test]
fn explicit_rebuild_is_idempotent_and_replaces_prior_commander_ledgers() {
    let mut vault = vault();
    let add = json!({"FID":7,"ID64":42,"BodyID":1,"Value":100,"Name":"System A","DateTime":"2026-01-01T00:00:00Z"});
    vault.handle_message("EvaluatorAdd", &add).unwrap();
    for _ in 0..2 {
        vault.begin_replay().unwrap();
        assert_eq!(vault.evaluator_total(7).unwrap(), 0);
        vault.on_event(&event(
            json!({"event":"LoadGame","Commander":"Commander","FID":"F7"}),
        ));
        vault.on_event(&event(
            json!({"event":"Location","SystemAddress":42,"StarSystem":"System A"}),
        ));
        vault.handle_message("EvaluatorAdd", &add).unwrap();
        vault.set_batch(false);
        assert_eq!(vault.evaluator_total(7).unwrap(), 100);
        assert_eq!(vault.journals.count().unwrap(), 1);
        assert!(vault.take_http_requests().is_empty());
    }
}

#[test]
fn explicit_spansh_consumers_share_an_inflight_prefetch_without_losing_replies() {
    let mut vault = Hecate::new(default_settings()).unwrap();
    vault.on_event(&event(
        json!({"event":"Location","SystemAddress":42,"StarSystem":"System A"}),
    ));
    let requests = vault.take_http_requests();
    assert_eq!(requests.len(), 2);
    for token in ["first", "second"] {
        assert!(vault
            .handle_message("SpanshSystem", &json!({"ID64":42,"Response":token}))
            .unwrap()
            .is_none());
    }
    assert!(vault.take_http_requests().is_empty());
    let request = requests
        .iter()
        .find(|request| request["url"] == "https://spansh.co.uk/api/dump/42")
        .unwrap();
    vault.handle_http_response(&json!({"request_id":request["request_id"],"ok":true,"body":"{\"system\":{\"id64\":42,\"name\":\"System A\",\"bodies\":[]}}"})).unwrap();
    let replies = vault.take_messages();
    assert_eq!(replies.len(), 2);
    assert_eq!(replies[0]["Payload"]["Response"], "first");
    assert_eq!(replies[1]["Payload"]["Response"], "second");
    assert_eq!(replies[0]["Payload"]["JournalEvents"], json!([]));
}

#[test]
fn opens_original_systemdb_schema_and_decodes_independent_zstandard_frame() {
    let root = std::env::temp_dir().join(format!(
        "hecate-schema-{}-{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    {
        let db = Connection::open(root.join("journals.db")).unwrap();
        db.execute_batch("CREATE TABLE journals(id INTEGER PRIMARY KEY AUTOINCREMENT,id64 INTEGER(8),fid INTEGER(4),data BLOB,inserted DATETIME);").unwrap();
        let text=b"{\"event\":\"Scan\",\"BodyID\":7,\"BodyName\":\"Imported 1\",\"UnknownVendorField\":{\"preserved\":true}}\r\n";
        // Independent standard Zstd frame: single segment, raw final block.
        // It is deliberately not produced by the port's encoder.
        let mut frame = vec![0x28, 0xb5, 0x2f, 0xfd, 0x20, text.len() as u8];
        let block = ((text.len() as u32) << 3) | 1;
        frame.extend_from_slice(&block.to_le_bytes()[..3]);
        frame.extend_from_slice(text);
        db.execute(
            "INSERT INTO journals(id64,fid,data,inserted) VALUES(?1,?2,?3,?4)",
            params![-2_i64, 123, frame, "2026-01-01 00:00:00.000"],
        )
        .unwrap();
    }
    {
        let mut vault = Hecate::open(json!({"RemoteLookups":false}), &root).unwrap();
        let response = message(
            &mut vault,
            "LoadSystem",
            json!({"ID64":u64::MAX-1,"DateTime":"2026-01-02T00:00:00Z"}),
        );
        assert_eq!(response["FIDs"], json!([123]));
        assert_eq!(response["JournalEvents"][0]["BodyName"], "Imported 1");
        assert_eq!(
            response["JournalEvents"][0]["UnknownVendorField"]["preserved"],
            true
        );
        assert_eq!(
            message(
                &mut vault,
                "GetSystemsVisitedFromList",
                json!({"ID64s":[u64::MAX-1]})
            )["ID64s"],
            json!([u64::MAX - 1])
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn compatible_zstd_sqlite_files_reopen_without_loading_vendor_code() {
    let root = std::env::temp_dir().join(format!(
        "hecate-port-test-{}-{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    {
        let mut vault = Hecate::open(json!({"RemoteLookups":false}), &root).unwrap();
        vault.on_event(&event(
            json!({"event":"LoadGame","FID":"F7","Commander":"Fixture"}),
        ));
        vault.on_event(&event(
            json!({"event":"Location","SystemAddress":42,"StarSystem":"Persistent"}),
        ));
        vault.on_event(&event(json!({"event":"Scan","BodyName":"Persistent 1"})));
        vault.flush().unwrap();
    }
    {
        let mut vault = Hecate::open(json!({"RemoteLookups":false}), &root).unwrap();
        let loaded = message(
            &mut vault,
            "LoadSystem",
            json!({"ID64":42,"DateTime":"2026-01-02T00:00:00Z"}),
        );
        assert_eq!(loaded["JournalEvents"].as_array().unwrap().len(), 2);
        assert_eq!(loaded["JournalEvents"][1]["BodyName"], "Persistent 1");
        let db = Connection::open(root.join("journals.db")).unwrap();
        let blob: Vec<u8> = db
            .query_row("SELECT data FROM journals LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert!(blob.starts_with(&[0x28, 0xb5, 0x2f, 0xfd]));
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn source_settings_changes_preserve_visit_and_restore_listener_on_bind_failure() {
    let mut vault = vault();
    vault.on_event(&event(
        json!({"event":"LoadGame","Commander":"Settings","FID":"F7"}),
    ));
    vault.on_event(&event(
        json!({"event":"Location","SystemAddress":42,"StarSystem":"Persistent visit"}),
    ));
    vault
        .register_plugin("BioInsights", json!("bio-guid"))
        .unwrap();
    let server = loop {
        let candidate = server::Server::start("", 0).unwrap();
        if candidate.port() < 59000 {
            break candidate;
        }
    };
    let port = server.port();
    vault.settings["ServerSettings"] =
        json!({"EnableWebServer":true,"ServerPort":port,"ServerLANIPAddress":""});
    server.register("BioInsights", json!("bio-guid")).unwrap();
    vault.server = Some(server);
    let journal = vault.journal_text.clone();
    let mut settings = vault.settings().clone();
    settings["VerboseLogging"] = json!(true);
    vault.try_set_settings(settings.clone()).unwrap();
    assert_eq!(vault.current_id, 42);
    assert_eq!(vault.current_fid, 7);
    assert_eq!(vault.journal_text, journal);
    assert_eq!(vault.server_address(), Some(("localhost".into(), port)));
    settings["ServerSettings"]["ServerLANIPAddress"] = json!("192.0.2.254");
    assert!(vault.try_set_settings(settings).is_err());
    assert_eq!(vault.settings()["ServerSettings"]["ServerLANIPAddress"], "");
    assert_eq!(vault.server_address(), Some(("localhost".into(), port)));
    assert_eq!(vault.journal_text, journal);
    assert!(vault.registered_plugins.contains_key("bioinsights"));
    assert!(std::net::TcpStream::connect(("127.0.0.1", port)).is_ok());
}

#[test]
fn source_reset_journal_database_requires_confirmation_and_preserves_other_vaults() {
    let mut vault = vault();
    vault.on_event(&event(
        json!({"event":"LoadGame","Commander":"Reset","FID":"F7"}),
    ));
    vault.on_event(&event(
        json!({"event":"Location","SystemAddress":42,"StarSystem":"A"}),
    ));
    vault.handle_message("EvaluatorAdd",&json!({"FID":7,"ID64":42,"BodyID":1,"Value":100,"Name":"A","DateTime":"2026-01-01T00:00:00Z"})).unwrap();
    vault
        .spansh
        .add(42, 0, "{\"system\":{}}", &now(), true)
        .unwrap();
    vault.canonn.add(42, 0, "[]", &now(), true).unwrap();
    assert!(vault
        .handle_message("ResetJournalDatabase", &json!({}))
        .is_err());
    vault.flush().unwrap();
    assert_eq!(vault.journals.visited(&[42]).unwrap(), vec![42]);
    vault.on_event(&event(
        json!({"event":"Scan","SystemAddress":42,"BodyID":1}),
    ));
    let current_journal = vault.journal_text.clone();
    let reply = vault
        .handle_message("ResetJournalDatabase", &json!({"Confirmed":true}))
        .unwrap()
        .unwrap();
    assert_eq!(reply["Payload"]["Success"], true);
    assert!(vault.journals.visited(&[42]).unwrap().is_empty());
    assert_eq!(vault.journal_text, current_journal);
    assert_eq!(vault.evaluator_total(7).unwrap(), 100);
    assert_eq!(vault.spansh.visited(&[42]).unwrap(), vec![42]);
    assert_eq!(vault.canonn.visited(&[42]).unwrap(), vec![42]);
    assert_eq!(vault.current_id, 42);
}

#[test]
fn original_remote_prefetch_timers_throttle_each_service_but_not_explicit_consumers() {
    let mut vault = Hecate::new(default_settings()).unwrap();
    vault.queue_cache("spansh", 42, None).unwrap();
    vault.queue_cache("canonn", 42, None).unwrap();
    assert_eq!(vault.take_http_requests().len(), 2);
    vault.queue_cache("spansh", 43, None).unwrap();
    vault.queue_cache("canonn", 43, None).unwrap();
    assert!(vault.take_http_requests().is_empty());
    vault
        .queue_cache("spansh", 43, Some(json!({"Response":"explicit"})))
        .unwrap();
    assert_eq!(vault.take_http_requests().len(), 1);
    vault.automatic_lookup_started.insert(
        "spansh",
        std::time::Instant::now() - std::time::Duration::from_millis(1001),
    );
    vault.queue_cache("spansh", 44, None).unwrap();
    assert_eq!(vault.take_http_requests().len(), 1);
}
