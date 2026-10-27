//! Original dispatcher/state and persistent notification methods execute in an
//! independent C# source harness. Downstream price/route/UI processors are inert;
//! their algorithms have separate direct source oracles.
use super::*;
fn oracle() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/evaluator-dispatch-oracle.json"
    ))
    .unwrap()
}
fn event(value: Value) -> JournalEvent {
    JournalEvent::parse(&value.to_string()).unwrap()
}
fn worker() -> Evaluator {
    let mut w = Evaluator::new(json!({"MinimumBodyValue":0}));
    w.fid = "F123".into();
    w.commander = "Source".into();
    w.on_event(&event(
        json!({"event":"Location","SystemAddress":42,"StarSystem":"Source"}),
    ));
    w.on_event(&event(json!({"event":"Scan","BodyID":1,"BodyName":"Source 1","PlanetClass":"Earthlike body","MassEM":1})));
    w.take_messages();
    w
}
#[test]
fn all_5376_original_dispatcher_state_captures_match() {
    let original = oracle();
    for row in original["journal"].as_array().unwrap() {
        let flags = row["Flags"].as_u64().unwrap();
        let name = row["Event"].as_str().unwrap();
        let mut w = worker();
        w.batch = flags & 2 != 0;
        w.preread = flags & 4 == 0;
        w.transport_mode = if flags & 8 != 0 { 1 } else { 0 };
        w.systems
            .get_mut(&w.key())
            .unwrap()
            .bodies
            .get_mut(&1)
            .unwrap()
            .mapped = flags & 64 != 0;
        w.on_event(&event(json!({"event":name,"SystemAddress":if flags&1!=0 {99}else{42},"BodyID":1,"BodyCount":5,"BodyName":"Source 1","StarSystem":"Source","FID":"F123","Commander":"Source","PlanetClass":"Earthlike body","MassEM":1,"ProbesUsed":2,"EfficiencyTarget":6,"JumpType":if flags&128!=0 {"Supercruise"}else{"Hyperspace"},"OnFoot":flags&16!=0,"Docked":flags&32!=0,"Odyssey":flags&16!=0,"InSRV":flags&8!=0,"SRV":flags&8!=0,"Signals":[{"Type":"$SAA_SignalType_Biological;","Count":3},{"Type":"$SAA_SignalType_Geological;","Count":2}]})));
        let sys = w.systems.get(&w.key()).unwrap();
        let body = sys.bodies.get(&1);
        assert_eq!(w.current, row["Current"].as_u64().unwrap(), "current {row}");
        assert_eq!(json!(w.transport_mode), row["Transport"], "transport {row}");
        assert_eq!(json!(w.odyssey), row["Odyssey"], "Odyssey {row}");
        assert_eq!(json!(sys.total), row["Total"], "body count {row}");
        assert_eq!(
            json!(body.is_some_and(|b| b.mapped)),
            row["Mapped"],
            "mapped {row}"
        );
        assert_eq!(
            json!(body.is_some_and(|b| b.efficient)),
            row["Efficient"],
            "mapping efficiency {row}"
        );
        assert_eq!(
            json!(sys.from.map_or(-1, |id| id as i64)),
            row["LastVisited"],
            "last visit {row}"
        );
        assert_eq!(
            json!(w.system_complete_or_mapped),
            row["Complete"],
            "completion {row}"
        );
        let delete = row["Actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a == "DeleteHecateData");
        assert_eq!(
            w.take_messages()
                .iter()
                .any(|m| m["command"] == "EvaluatorDelete"),
            delete,
            "sale/death {row}"
        );
    }
}
#[test]
fn all_7744_original_status_captures_preserve_target_and_popup_lifecycle() {
    let original = oracle();
    for row in original["status"].as_array().unwrap() {
        let flags = row["flags"].as_u64().unwrap();
        let mut w = worker();
        let previous = row["previous"].as_u64().unwrap();
        let focus = row["focus"].as_u64().unwrap();
        w.gui_focus = previous;
        w.settings["SetTargetedInFSSWorthMapping"] = json!(flags & 8 != 0);
        w.settings["Notifications"]["AlwaysPersistent"]["Popup"] = json!(flags & 2 != 0);
        w.settings["Notifications"]["DSSPersistent"]["Popup"] = json!(flags & 4 != 0);
        w.systems
            .get_mut(&w.key())
            .unwrap()
            .bodies
            .get_mut(&1)
            .unwrap()
            .star = flags & 16 != 0;
        if flags & 1 != 0 {
            w.request_overlay(false);
        }
        w.take_messages();
        w.on_status(&json!({"GuiFocus":focus,"Destination":{"System":if flags&32!=0 {99}else{42},"Body":1}}));
        assert_eq!(
            json!(w.systems[&w.key()].bodies[&1].override_mapping),
            row["Override"],
            "target {row}"
        );
        let actions = row["Actions"].as_array().unwrap();
        let canceled = actions.iter().any(|a| a == "CancelNotification");
        let requested = actions.iter().any(|a| {
            a.as_str().is_some_and(|a| {
                a.starts_with("Notify:Bodies still to map:") || a.starts_with("UpdateNotification:")
            })
        });
        assert_eq!(
            w.persistent_open,
            (flags & 1 != 0 || requested) && !canceled,
            "popup state {row}"
        );
        let messages = w.take_messages();
        assert_eq!(
            messages.iter().any(|m| m["payload"]["action"] == "cancel"),
            canceled,
            "cancel {row}"
        );
    }
}
#[test]
fn all_128_original_persistent_only_update_and_content_cases_match() {
    let original = oracle();
    for row in original["persistent"].as_array().unwrap() {
        let flags = row["flags"].as_u64().unwrap();
        let mut w = worker();
        w.batch = flags & 8 != 0;
        if flags & 1 != 0 {
            w.request_overlay(false);
        }
        if flags & 32 != 0 {
            w.systems.get_mut(&w.key()).unwrap().route.clear();
        }
        w.take_messages();
        w.request_overlay(flags & 64 != 0);
        let actions = row["Actions"].as_array().unwrap();
        assert_eq!(w.persistent_open, !actions.is_empty(), "onlyUpdate {row}");
        if let Some(action) = actions.first().and_then(Value::as_str) {
            let detail = action
                .strip_prefix("Notify:Bodies still to map:")
                .or_else(|| action.strip_prefix("UpdateNotification:"))
                .unwrap();
            assert_eq!(w.overlay_payload["detail"], detail, "detail {row}");
        }
    }
}

#[test]
fn all_320_original_journal_notification_delivery_predicates_match() {
    let original = oracle();
    for row in original["journalNotifications"].as_array().unwrap() {
        let flags = row["flags"].as_u64().unwrap();
        let name = row["Event"].as_str().unwrap();
        let mut w = worker();
        w.batch = flags & 8 != 0;
        w.preread = false;
        w.settings["Notifications"]["AlwaysPersistent"]["Popup"] = json!(flags & 1 != 0);
        w.settings["Notifications"]["DSSPersistent"]["Popup"] = json!(flags & 2 != 0);
        w.settings["Notifications"]["EfficientRoute"] =
            json!({"Popup":flags&4!=0,"Voice":false,"Plugin":false});
        if flags & 16 != 0 {
            w.request_overlay(false);
        }
        w.take_messages();
        let findings=w.on_event(&event(json!({"event":name,"SystemAddress":42,"BodyID":1,"BodyName":"Source 1","ProbesUsed":2,"EfficiencyTarget":6,"JumpType":if flags&32!=0 {"Supercruise"}else{"Hyperspace"}})));
        let actions = row["Actions"].as_array().unwrap();
        let expected = actions
            .iter()
            .filter_map(Value::as_str)
            .filter(|a| {
                a.starts_with("Notify:Mapping Complete:")
                    || a.starts_with("Notify:System Discovery Complete:")
            })
            .count();
        assert_eq!(findings.len(), expected, "transient {row}");
        let cancel = actions.iter().any(|a| a == "CancelNotification");
        let show = actions.iter().any(|a| {
            a.as_str()
                .is_some_and(|a| a.starts_with("Notify:Bodies still to map:"))
        });
        let update = actions.iter().any(|a| {
            a.as_str()
                .is_some_and(|a| a.starts_with("UpdateNotification:"))
        });
        let messages = w.take_messages();
        let notifications: Vec<_> = messages
            .iter()
            .filter(|m| m["command"] == "Notification")
            .collect();
        assert_eq!(
            notifications
                .iter()
                .any(|m| m["payload"]["action"] == "cancel"),
            cancel,
            "cancel {row}"
        );
        assert_eq!(
            notifications
                .iter()
                .any(|m| m["payload"]["action"] == "show"),
            show,
            "show {row}"
        );
        if !update {
            assert!(
                !notifications
                    .iter()
                    .any(|m| m["payload"]["action"] == "update"),
                "invented update {row}"
            );
        }
        assert_eq!(
            w.persistent_open,
            (!cancel && flags & 16 != 0) || show,
            "persistent {row}"
        );
    }
}

#[test]
fn repeated_address_headers_and_body_order_match_original_updateui() {
    let cases: Value = serde_json::from_str(include_str!(
        "../../../fixtures/evaluator-visit-view-oracle.json"
    ))
    .unwrap();
    for case in cases.as_array().unwrap() {
        let mut w = Evaluator::new(json!({"MinimumBodyValue":0}));
        w.filter = "All".into();
        w.fid = "F123".into();
        w.batch = true;
        for (index, id) in [42, 99, 42].into_iter().enumerate() {
            w.location(&event(
                json!({"event":"FSDJump","SystemAddress":id,"StarSystem":format!("Source{index}")}),
            ));
            for body in [9, 1, 5] {
                w.on_event(&event(json!({"event":"Scan","BodyID":body,"BodyName":format!("Source {body}"),"PlanetClass":"Earthlike body","MassEM":1})));
            }
        }
        w.current = case["current"].as_u64().unwrap();
        let summary = w.summary();
        let headers: Vec<Value> = summary["Systems"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| json!({"Name":s["System"],"Current":s["Current"]}))
            .collect();
        assert_eq!(json!(headers), case["headers"]);
        let rows: Vec<Value> = w.rows().iter().map(|r| r["BodyID"].clone()).collect();
        assert_eq!(json!(rows), case["rows"]);
    }
}
