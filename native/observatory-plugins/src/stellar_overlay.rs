//! Native state/queue/radar port of installed StellarOverlay 1.3.228.387.
//! Reference DLL SHA256: 2d4fbc84be59060d8a4fdb781a24eaa65af51d43d873090c95f67b5d965d4665.
use crate::botanist::great_circle_distance;
use observatory_domain::JournalEvent;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::OnceLock,
    time::Instant,
};

pub const SOURCE_VERSION: &str = "1.3.228.387";
pub const PLUGIN_ID: &str = "4d617474-4703-5354-454c-4c41524f5645";
/// Installed Radar.cs constants, normalized to the outer 10 km ring.
pub const RADAR_INNER_FRACTION: f64 = 0.76;
pub const RADAR_MIDDLE_FRACTION: f64 = 0.88;
pub const NOTIFICATION_TRANSITION_MS: u64 = 300;
pub fn now_ms() -> u64 {
    if let Some(now) = RPC_CLOCK.with(std::cell::Cell::get) {
        return now;
    }
    static ORIGIN: OnceLock<Instant> = OnceLock::new();
    ORIGIN.get_or_init(Instant::now).elapsed().as_millis() as u64
}
thread_local! {
    static RPC_CLOCK: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
}
/// RPC workers use the host monotonic clock, including nested notification and
/// radar helpers. A child process's own Instant origin is not interchangeable.
pub(crate) fn with_host_clock<T>(now: u64, run: impl FnOnce() -> T) -> T {
    struct Restore(Option<u64>);
    impl Drop for Restore {
        fn drop(&mut self) {
            RPC_CLOCK.with(|clock| clock.set(self.0));
        }
    }
    let _restore = Restore(RPC_CLOCK.with(|clock| clock.replace(Some(now))));
    run()
}
/// Conditions executed by the installed Radar.Draw method. Its `show`
/// argument does not alter these conditions; the host handles game focus.
pub fn radar_visible(
    visible: bool,
    force_hidden: bool,
    altitude_m: f64,
    force_show: bool,
    initialised: bool,
    updating: bool,
) -> bool {
    initialised && !updating && !force_hidden && visible && (force_show || altitude_m <= 2000.0)
}
pub fn default_settings() -> Value {
    json!({
        "HandleObservatoryNotifications":true,"DisplayOverride":false,"SelectedMonitor":"0 (Default)",
        "InstallUpdates":true,"InstallBetas":false,"VerboseLogging":false,
        "NotificationShape":"Open","SelectedFont":"Arial","FontScaling":100.0,"TextAlignment":"Left",
        "TitleColorHex":"FFFF7B00","ColorHex":"FFFF6300","BorderColorHex":"FF2E8EFF","BackgroundColorHex":"B0000000",
        "BorderWidth":3,"CornerRounding":20,"XPadding":10,"YPadding":5,"EdgeIndentation":20,"VerticalSpacing":15,
        "MaxNotifications":3,"TTL":5000,"DelayBetweenNotificationsMS":200,"MaxWidth":25,"FPS":120,"FromRight":true,"VerticalPositionPercent":97,
        "RadarSettings":{"Enabled":true,"PositionXPct":2.0,"PositionYPct":50.0,"SizePct":20.0,"FontScaling":100.0}
    })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OverlayNotification {
    pub id: String,
    pub sender: String,
    pub title: String,
    pub detail: String,
    pub created_ms: u64,
    pub expires_ms: Option<u64>,
    pub x: f32,
    pub y: f32,
    pub opacity: f32,
    pub colors: Value,
    pub persistent: bool,
    #[serde(default)]
    pub instant: bool,
    #[serde(default)]
    pub queue_slot: Option<usize>,
    #[serde(default)]
    pub source_guid: bool,
    pub entry_transition: String,
    pub exit_transition: String,
    pub transition_ms: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RadarPoint {
    pub kind: String,
    pub context_id: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_m: f64,
    pub label: String,
    pub measure: bool,
    pub distance_m: f64,
    pub x: f32,
    pub y: f32,
    pub visible: bool,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RadarSnapshot {
    pub visible: bool,
    pub system_address: u64,
    pub body_id: u64,
    pub radius_m: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub heading: f64,
    pub altitude_m: f64,
    pub context_id: i64,
    pub sample_count: u64,
    pub minimum_distance_m: f64,
    pub sample_distance_m: f64,
    pub colony_ranges: BTreeMap<i64, f64>,
    pub genuses: BTreeMap<i64, String>,
    pub seen_genuses: BTreeSet<i64>,
    pub points: Vec<RadarPoint>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OverlaySnapshot {
    pub notifications: Vec<OverlayNotification>,
    pub alerts: Vec<Value>,
    pub radar: RadarSnapshot,
    pub settings: Value,
    pub game_running: bool,
    pub force_show: bool,
    pub sampled_ms: u64,
}

pub struct StellarOverlay {
    settings: Value,
    pending: VecDeque<OverlayNotification>,
    active: VecDeque<OverlayNotification>,
    alerts: BTreeMap<String, Value>,
    radar: RadarSnapshot,
    colors: BTreeMap<String, Value>,
    disabled_custom_colors: BTreeSet<String>,
    batch: bool,
    game_running: bool,
    last_status: Value,
    last_release_ms: Option<u64>,
    force_until: u64,
    radar_force_until: u64,
    force_hidden: bool,
    next_id: u64,
    waypoint_count: u64,
}
impl StellarOverlay {
    pub fn new(settings: Value) -> Self {
        let mut merged = default_settings();
        if let Some(values) = settings.as_object() {
            for (k, v) in values {
                merged[k] = v.clone();
            }
        }
        if let Some(shape) = merged["NotificationShape"].as_u64() {
            merged["NotificationShape"] = json!(if shape == 0 { "Closed" } else { "Open" });
        }
        if let Some(align) = merged["TextAlignment"].as_u64() {
            merged["TextAlignment"] = json!(match align {
                1 => "Center",
                2 => "Right",
                _ => "Left",
            });
        }
        if let Some(radar) = merged["RadarSettings"].as_object_mut() {
            for (key, value) in default_settings()["RadarSettings"].as_object().unwrap() {
                radar.entry(key).or_insert_with(|| value.clone());
            }
        }
        Self {
            settings: merged,
            pending: VecDeque::new(),
            active: VecDeque::new(),
            alerts: BTreeMap::new(),
            radar: RadarSnapshot::default(),
            colors: BTreeMap::new(),
            disabled_custom_colors: BTreeSet::new(),
            batch: false,
            game_running: false,
            last_status: Value::Null,
            last_release_ms: None,
            force_until: 0,
            radar_force_until: 0,
            force_hidden: false,
            next_id: 1,
            waypoint_count: 0,
        }
    }
    pub fn settings(&self) -> &Value {
        &self.settings
    }
    /// The original settings object changes presentation without reconstructing
    /// its notification queue, persistent alerts, radar or journal context.
    pub fn set_settings(&mut self, settings: Value) -> anyhow::Result<()> {
        anyhow::ensure!(
            settings.is_object(),
            "StellarOverlay settings must be an object"
        );
        self.settings = Self::new(settings).settings;
        Ok(())
    }
    pub fn set_batch(&mut self, batch: bool) {
        self.batch = batch;
        if batch {
            self.pending.clear();
        }
    }
    pub fn reset(&mut self) {
        self.pending.clear();
        self.active.clear();
        self.alerts.clear();
        self.radar = RadarSnapshot::default();
        self.waypoint_count = 0;
        self.last_release_ms = None;
        self.radar_force_until = 0;
    }
    pub fn handles_notifications(&self) -> bool {
        self.settings["HandleObservatoryNotifications"]
            .as_bool()
            .unwrap_or(true)
    }
    pub fn on_event(&mut self, event: &JournalEvent) {
        match event.event.as_str() {
            "LoadGame" | "Location" => self.game_running = true,
            "FSDJump" | "CarrierJump" => {
                self.game_running = true;
                self.radar.visible = false;
                self.radar.points.clear();
                self.radar.system_address = event.id("SystemAddress").unwrap_or(0);
            }
            "LeaveBody" => self.radar.visible = false,
            "Shutdown" => {
                self.game_running = false;
                self.reset();
            }
            _ => {}
        }
    }
    pub fn on_status(&mut self, status: &Value) {
        self.last_status = status.clone();
        let Some(lat) = status["Latitude"].as_f64() else {
            return;
        };
        let Some(lon) = status["Longitude"].as_f64() else {
            return;
        };
        if !lat.is_finite() || !lon.is_finite() || lat.abs() > 90.0 || lon.abs() > 180.0 {
            return;
        }
        self.radar.latitude = lat;
        self.radar.longitude = lon;
        self.radar.heading = status["Heading"].as_f64().unwrap_or(0.0);
        self.radar.altitude_m = status["Altitude"].as_f64().unwrap_or(0.0);
        if let Some(radius) = status["PlanetRadius"]
            .as_f64()
            .filter(|r| r.is_finite() && *r > 0.0)
        {
            self.radar.radius_m = radius;
        }
        self.update_radar_positions();
    }
    pub fn on_notification(&mut self, value: &Value) {
        self.on_notification_at(value, now_ms());
    }
    pub fn on_notification_at(&mut self, value: &Value, now: u64) {
        let action = text(value, "action", "Action");
        let mut id = text(value, "id", "Guid").to_owned();
        let has_guid = !id.is_empty() && id != "00000000-0000-0000-0000-000000000000";
        if !self.handles_notifications() {
            return;
        }
        if action == "cancel" {
            self.cancel(&id);
            return;
        }
        if self.batch {
            return;
        }
        if value["popup"].as_bool() == Some(false) {
            return;
        }
        if let Some(render) = value
            .get("rendering")
            .or_else(|| value.get("Rendering"))
            .and_then(Value::as_u64)
        {
            if render & 1 == 0 {
                return;
            }
        }
        if !has_guid {
            id = format!("stellar:{}", self.next_id);
            self.next_id += 1;
        }
        if action == "update" {
            for item in self
                .active
                .iter_mut()
                .filter(|n| n.id == id && (n.instant || n.source_guid))
            {
                if text(value, "extended_details", "ExtendedDetails") == "CANCEL" {
                    item.expires_ms = Some(now);
                    continue;
                }
                if value.get("title").or_else(|| value.get("Title")).is_some() {
                    item.title = text(value, "title", "Title").into();
                }
                if value
                    .get("detail")
                    .or_else(|| value.get("Detail"))
                    .is_some()
                {
                    item.detail = text(value, "detail", "Detail").into();
                }
                if let Some(ttl) = value
                    .get("timeout")
                    .or_else(|| value.get("Timeout"))
                    .and_then(Value::as_i64)
                {
                    let ttl = if ttl < 0 {
                        self.settings["TTL"].as_i64().unwrap_or(5000)
                    } else {
                        ttl
                    };
                    item.expires_ms = (ttl > 0).then_some(now.saturating_add(ttl.max(0) as u64));
                    item.persistent = ttl == 0;
                }
                if let Some(x) = value
                    .get("x")
                    .or_else(|| value.get("XPos"))
                    .and_then(Value::as_f64)
                {
                    item.x = x as f32;
                }
                if let Some(y) = value
                    .get("y")
                    .or_else(|| value.get("YPos"))
                    .and_then(Value::as_f64)
                {
                    item.y = y as f32;
                }
                item.created_ms = now;
                if value.get("EntryTransition").is_some() {
                    item.entry_transition =
                        transition_name(value.get("EntryTransition"), &item.entry_transition);
                }
                if value.get("ExitTransition").is_some() {
                    item.exit_transition =
                        transition_name(value.get("ExitTransition"), &item.exit_transition);
                }
            }
            return;
        }
        let mut ttl = value
            .get("timeout")
            .or_else(|| value.get("Timeout"))
            .and_then(Value::as_i64)
            .filter(|t| *t >= 0)
            .unwrap_or_else(|| self.settings["TTL"].as_i64().unwrap_or(5000));
        let sender = text(value, "plugin", "Sender").to_owned();
        let fixed = value
            .get("x")
            .or_else(|| value.get("XPos"))
            .and_then(Value::as_f64)
            .is_some_and(|x| x != -1.0)
            && value
                .get("y")
                .or_else(|| value.get("YPos"))
                .and_then(Value::as_f64)
                .is_some_and(|y| y != -1.0)
            || ttl == 0;
        if ttl == 0 && !has_guid {
            ttl = self.settings["TTL"]
                .as_i64()
                .unwrap_or(5000)
                .saturating_mul(4);
        }
        if fixed && text(value, "extended_details", "ExtendedDetails") == "CANCEL" {
            self.cancel(&id);
            return;
        }
        let from_right = self.settings["FromRight"].as_bool().unwrap_or(true);
        let notification = OverlayNotification {
            id: id.clone(),
            sender: sender.clone(),
            title: text(value, "title", "Title").into(),
            detail: text(value, "detail", "Detail").into(),
            created_ms: now,
            expires_ms: (ttl > 0).then_some(now.saturating_add(ttl.max(0) as u64)),
            persistent: ttl == 0,
            instant: fixed,
            queue_slot: None,
            source_guid: has_guid,
            x: value
                .get("x")
                .or_else(|| value.get("XPos"))
                .and_then(Value::as_f64)
                .unwrap_or(-1.0) as f32,
            y: value
                .get("y")
                .or_else(|| value.get("YPos"))
                .and_then(Value::as_f64)
                .unwrap_or(-1.0) as f32,
            opacity: value["Opacity"].as_f64().unwrap_or(100.0).clamp(0.0, 100.0) as f32,
            colors: if self.disabled_custom_colors.contains(&sender) {
                Value::Null
            } else {
                self.colors.get(&sender).cloned().unwrap_or(Value::Null)
            },
            entry_transition: transition_name(
                value.get("EntryTransition"),
                if fixed {
                    "None"
                } else if from_right {
                    "Left"
                } else {
                    "Right"
                },
            ),
            exit_transition: transition_name(
                value.get("ExitTransition"),
                if fixed {
                    "None"
                } else if from_right {
                    "Right"
                } else {
                    "Left"
                },
            ),
            transition_ms: NOTIFICATION_TRANSITION_MS,
        };
        self.cancel(&id);
        if fixed {
            self.active.push_back(notification);
        } else {
            if self.pending.len() >= 1024 {
                self.pending.pop_front();
            }
            self.pending.push_back(notification);
        }
        self.tick_at(now);
    }
    pub fn cancel(&mut self, id: &str) {
        self.active.retain(|n| n.id != id);
        self.pending.retain(|n| n.id != id);
        self.alerts.remove(id);
    }
    pub fn tick(&mut self) {
        self.tick_at(now_ms());
    }
    pub fn tick_at(&mut self, now: u64) {
        if self.radar_force_until > 0 && now >= self.radar_force_until {
            self.radar_force_until = 0;
            self.radar.visible = false;
        }
        self.active
            .retain(|n| n.expires_ms.is_none_or(|time| time > now));
        self.alerts
            .retain(|_, v| v["expires_ms"].as_u64().is_none_or(|time| time > now));
        let limit = self.settings["MaxNotifications"]
            .as_u64()
            .unwrap_or(3)
            .clamp(1, 64) as usize;
        let delay = self.settings["DelayBetweenNotificationsMS"]
            .as_u64()
            .unwrap_or(200);
        if self.active.iter().filter(|n| !n.instant).count() < limit
            && self
                .last_release_ms
                .is_none_or(|previous| now.saturating_sub(previous) >= delay)
        {
            if let Some(mut item) = self.pending.pop_front() {
                item.queue_slot = (0..limit)
                    .find(|slot| !self.active.iter().any(|n| n.queue_slot == Some(*slot)));
                let ttl = item.expires_ms.map(|v| v.saturating_sub(item.created_ms));
                item.created_ms = now;
                item.expires_ms = ttl.map(|v| now.saturating_add(v));
                self.active.push_back(item);
                self.last_release_ms = Some(now);
            }
        }
    }
    pub fn snapshot(&self) -> OverlaySnapshot {
        let mut radar = self.radar.clone();
        radar.visible = radar_visible(
            radar.visible || now_ms() < self.radar_force_until,
            self.force_hidden,
            radar.altitude_m,
            now_ms() < self.radar_force_until,
            true,
            false,
        );
        OverlaySnapshot {
            notifications: self.active.iter().cloned().collect(),
            alerts: self.alerts.values().cloned().collect(),
            radar,
            settings: self.settings.clone(),
            game_running: self.game_running,
            force_show: self.settings["DisplayOverride"].as_bool().unwrap_or(false)
                || self.force_until > now_ms(),
            sampled_ms: now_ms(),
        }
    }
    pub fn handle_message(
        &mut self,
        sender: &str,
        command: &str,
        args: &Value,
    ) -> anyhow::Result<Option<Value>> {
        match command {
            "Ready" | "LegacyPluginMessage" => {}
            "TestNotifications" => {
                self.apply_test_settings(args);
                self.reset();
                let count = self.settings["MaxNotifications"]
                    .as_u64()
                    .unwrap_or(3)
                    .clamp(1, 5);
                self.force_until =
                    now_ms().saturating_add(self.settings["TTL"].as_u64().unwrap_or(5000) + 1000);
                let handles = self.settings["HandleObservatoryNotifications"].clone();
                self.settings["HandleObservatoryNotifications"] = json!(true);
                for index in 1..=count {
                    self.on_notification(&json!({"Sender":"StellarOverlay","Rendering":1,"Title":format!("A Test Notification {index}"),"Detail":format!("A sample notification ({index} of {count}) to test Overlay settings\nA multi-line test.")}));
                }
                self.settings["HandleObservatoryNotifications"] = handles;
            }
            "TestRadar" => {
                self.apply_test_settings(args);
                self.reset();
                self.radar.visible = true;
                self.radar_force_until = now_ms().saturating_add(5000);
                self.force_until = now_ms().saturating_add(6000);
            }
            "DebugNotifications" => {
                self.force_until = now_ms().saturating_add(8000);
                for index in 0..5 {
                    let instant = chrono::Utc::now();
                    let micro = instant.timestamp_subsec_micros() % 1000;
                    let millis = instant.timestamp_subsec_millis();
                    let second = instant.timestamp().rem_euclid(60) as u32;
                    self.on_notification(&json!({"Sender":"StellarOverlay","Rendering":1,"Title":"Debug Notification","Detail":format!("A test {micro}{second}"),"Timeout":6000,"XPos":(micro+51)*(index+1)%101,"YPos":(micro*37+second*23+millis*17*(index*index*727))%101}));
                }
            }
            "DebugState" => {
                return Ok(Some(
                    json!({"Command":"OverlayState","Payload":self.snapshot()}),
                ))
            }
            "SetDisplayOverrideTTL" => {
                self.force_until = now_ms().saturating_add(integer(args, "TTL", 0).max(0) as u64)
            }
            "RegisterPluginCustomColours" => {
                let mut colors = self
                    .colors
                    .get(sender)
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                for (source, target) in [
                    ("TitleColor", "TitleColorHex"),
                    ("TextColor", "ColorHex"),
                    ("BackgroundColor", "BackgroundColorHex"),
                    ("BorderColor", "BorderColorHex"),
                ] {
                    if let Some(value) = args.get(source) {
                        colors[target] = value.clone();
                    }
                }
                self.colors.insert(sender.into(), colors.clone());
                self.disabled_custom_colors.remove(sender);
                for item in self
                    .active
                    .iter_mut()
                    .chain(self.pending.iter_mut())
                    .filter(|item| item.sender == sender)
                {
                    item.colors = colors.clone();
                }
                if args["GetDefaultColors"].as_bool() == Some(true) {
                    return Ok(Some(
                        json!({"Command":"RegisterPluginCustomColoursResponse","Payload":{"TitleColor":self.settings["TitleColorHex"],"TextColor":self.settings["ColorHex"],"BackgroundColor":self.settings["BackgroundColorHex"],"BorderColor":self.settings["BorderColorHex"]}}),
                    ));
                }
            }
            "UnregisterPluginCustomColours" => {
                // The source disables custom-color use, retaining registered
                // brushes so a later partial registration can reuse them.
                self.disabled_custom_colors.insert(sender.into());
                for item in self
                    .active
                    .iter_mut()
                    .chain(self.pending.iter_mut())
                    .filter(|item| item.sender == sender)
                {
                    item.colors = Value::Null;
                }
            }
            "ShowBitmapNotification" => {
                let key = args
                    .get("ImageKey")
                    .or_else(|| args.get("Key"))
                    .or_else(|| args.get("key"))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                anyhow::ensure!(!key.is_empty(), "ShowBitmapNotification requires ImageKey");
                let mut value = args.clone();
                let ttl = integer(args, "TTLms", integer(args, "TTL", 3000));
                let ttl = if ttl < 0 {
                    self.settings["TTL"].as_i64().unwrap_or(5000)
                } else {
                    ttl
                };
                let created = now_ms();
                value["Key"] = json!(key);
                value["ImageKey"] = json!(key);
                value["created_ms"] = json!(created);
                value["TTLms"] = json!(ttl);
                value["EntryTransition"] =
                    json!(transition_name(args.get("EntryTransition"), "Fade"));
                value["ExitTransition"] =
                    json!(transition_name(args.get("ExitTransition"), "Fade"));
                value["TransitionMS"] =
                    json!(
                        integer(args, "TransactionTTLms", NOTIFICATION_TRANSITION_MS as i64).max(0)
                    );
                if integer(args, "TransactionTTLms", -1) < 0 {
                    value["TransitionMS"] = json!(NOTIFICATION_TRANSITION_MS);
                }
                value["Opacity"] =
                    json!(args["Opacity"].as_f64().unwrap_or(100.0).clamp(0.0, 100.0));
                value["ForceShow"] = json!(args["ForceShow"].as_bool().unwrap_or(false));
                value["XPos"] = json!(args["XPos"].as_f64().unwrap_or(0.0));
                value["YPos"] = json!(args["YPos"].as_f64().unwrap_or(0.0));
                value["expires_ms"] = if ttl > 0 {
                    json!(created.saturating_add(ttl as u64))
                } else {
                    Value::Null
                };
                self.alerts.insert(key.into(), value);
            }
            "HideBitmapNotification" => {
                let key = args
                    .get("ImageKey")
                    .or_else(|| args.get("Key"))
                    .or_else(|| args.get("key"))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                if let Some(alert) = self.alerts.get_mut(key) {
                    let expiry = now_ms().saturating_add(
                        alert["TransitionMS"]
                            .as_u64()
                            .unwrap_or(NOTIFICATION_TRANSITION_MS),
                    );
                    alert["expires_ms"] =
                        json!(alert["expires_ms"].as_u64().unwrap_or(expiry).min(expiry));
                }
            }
            "RadarSetPlanet" => {
                let address = args["ID64"].as_u64().unwrap_or(0);
                let body = args["BodyID"].as_u64().unwrap_or(0);
                if (address, body) != (self.radar.system_address, self.radar.body_id) {
                    self.radar.points.clear();
                    self.radar.genuses.clear();
                    self.radar.seen_genuses.clear();
                    self.radar.context_id = -1;
                    self.radar.sample_distance_m = 0.0;
                    self.waypoint_count = 0;
                    self.force_hidden = false;
                }
                self.radar.system_address = address;
                self.radar.body_id = body;
                self.radar.radius_m = args["RadiusKM"].as_f64().unwrap_or(0.0) * 1000.0;
                self.radar.visible = true;
                if let Some(genuses) = args["Genuses"].as_object() {
                    self.radar.genuses.extend(
                        genuses
                            .iter()
                            .filter_map(|(k, v)| Some((k.parse().ok()?, v.as_str()?.to_owned()))),
                    );
                }
                self.update_radar_positions();
            }
            "RadarHide" => self.radar.visible = false,
            "RadarForceHidden" => {
                self.force_hidden = args["Value"]
                    .as_bool()
                    .or_else(|| {
                        args["ForceHidden"]
                            .as_bool()
                            .or_else(|| args["Hidden"].as_bool())
                    })
                    .unwrap_or(true)
            }
            "RadarAddBiological" | "RadarAddVehicle" | "RadarAddWaypoint" => {
                let kind = if command == "RadarAddBiological" {
                    "Biological"
                } else if command == "RadarAddVehicle" {
                    "Vehicle"
                } else {
                    "Waypoint"
                };
                let context = integer(args, "ContextID", 0);
                let current = args["UseCurrentPosition"]
                    .as_bool()
                    .unwrap_or(command == "RadarAddWaypoint");
                // Installed HandleRadarAddPoint logs missing keys at Verbose
                // and returns; it does not throw into the sending plugin.
                if kind != "Waypoint" && args.get("ContextID").is_none()
                    || !current && !(args["X"].is_number() && args["Y"].is_number())
                {
                    return Ok(None);
                }
                let latitude = if current || args["X"].as_f64() == Some(1000.0) {
                    self.radar.latitude
                } else {
                    args["X"].as_f64().unwrap_or(self.radar.latitude)
                };
                let longitude = if current || args["Y"].as_f64() == Some(1000.0) {
                    self.radar.longitude
                } else {
                    args["Y"].as_f64().unwrap_or(self.radar.longitude)
                };
                anyhow::ensure!(
                    latitude.is_finite()
                        && longitude.is_finite()
                        && latitude.abs() <= 90.0
                        && longitude.abs() <= 180.0,
                    "invalid radar coordinates"
                );
                self.radar.points.retain(|p| {
                    !(p.kind == kind
                        && p.context_id == context
                        && (kind == "Vehicle"
                            || (kind == "Biological"
                                && (p.distance_m < 15.0
                                    || (p.distance_m > 15.0 && p.radius_m == 0.0)))
                            || (kind != "Waypoint"
                                && p.latitude == latitude
                                && p.longitude == longitude)))
                });
                let mut label = text(args, "Label", "label").to_owned();
                if kind == "Waypoint" && label.is_empty() {
                    self.waypoint_count += 1;
                    label = format!("WP{}", self.waypoint_count);
                }
                self.radar.points.push(RadarPoint {
                    kind: kind.into(),
                    context_id: context,
                    latitude,
                    longitude,
                    radius_m: args["Radius"].as_f64().unwrap_or(0.0),
                    label,
                    measure: args["MeasureDistance"].as_bool().unwrap_or(false),
                    distance_m: 0.0,
                    x: 0.0,
                    y: 0.0,
                    visible: true,
                });
                self.update_radar_positions();
            }
            "RadarRemoveBiological" | "RadarRemoveVehicle" => {
                let kind = if command == "RadarRemoveBiological" {
                    "Biological"
                } else {
                    "Vehicle"
                };
                let id = integer(args, "ContextID", 0);
                if kind == "Biological" && self.radar.genuses.contains_key(&id) {
                    self.radar.seen_genuses.insert(id);
                }
                self.radar
                    .points
                    .retain(|p| p.kind != kind || p.context_id != id);
                if let Some(context) = args.get("NewContextID") {
                    self.radar.context_id = context.as_i64().unwrap_or(-1);
                }
                if let Some(count) = args["SampleCount"].as_u64() {
                    self.radar.sample_count = count;
                }
                self.update_radar_positions();
            }
            "RadarSetContextID" => {
                self.radar.context_id = integer(args, "ContextID", 0);
                if let Some(radius) = args
                    .get("MinDistance")
                    .or_else(|| args.get("Radius"))
                    .and_then(Value::as_f64)
                {
                    self.radar.minimum_distance_m = radius;
                    self.radar
                        .colony_ranges
                        .insert(self.radar.context_id, radius);
                }
                if let Some(count) = args["SampleCount"].as_u64() {
                    self.radar.sample_count = count;
                }
                self.update_radar_positions();
            }
            "RadarSetColonyRanges" | "SetColonyRanges" => {
                let ranges = args
                    .get("Payload")
                    .or_else(|| args.get("Ranges"))
                    .unwrap_or(args);
                if let Some(values) = ranges.as_object() {
                    self.radar.colony_ranges = values
                        .iter()
                        .filter_map(|(k, v)| Some((k.parse().ok()?, v.as_f64()?)))
                        .collect();
                }
            }
            "RadarSetSampleCount" => self.radar.sample_count = args["Count"].as_u64().unwrap_or(0),
            _ => return Ok(None),
        }
        Ok(None)
    }
    fn apply_test_settings(&mut self, args: &Value) {
        if args["Settings"].is_object() {
            let _ = self.set_settings(args["Settings"].clone());
        }
    }
    pub fn set_bio_panel(&mut self, panel: Value) {
        if panel.is_null() {
            self.alerts.remove("BioInsights");
        } else {
            self.alerts.insert("BioInsights".into(), panel);
        }
    }
    pub fn set_evaluator_panel(&mut self, panel: Value) {
        if panel.is_null() {
            self.alerts.remove("Evaluator");
        } else {
            self.alerts.insert("Evaluator".into(), panel);
        }
    }
    fn update_radar_positions(&mut self) {
        for point in &mut self.radar.points {
            let (distance, x, y) = relative_position(
                self.radar.latitude,
                self.radar.longitude,
                self.radar.heading,
                point.latitude,
                point.longitude,
                self.radar.radius_m,
            );
            point.distance_m = distance;
            point.x = x;
            point.y = y;
            point.visible = distance < 20_000.0;
        }
        self.radar.sample_distance_m = self
            .radar
            .points
            .iter()
            .filter(|point| {
                point.kind == "Biological"
                    && point.context_id == self.radar.context_id
                    && point.measure
            })
            .map(|point| point.distance_m)
            .min_by(f64::total_cmp)
            .unwrap_or(0.0);
    }
}
/// Original enum values: 0 unspecified, 1 none, 2 fade, 3 left, 4 right,
/// 5 up, 6 down. Native callers may also provide the enum name.
fn transition_name(value: Option<&Value>, default: &str) -> String {
    const NAMES: [&str; 7] = [
        "NotSpecified",
        "None",
        "Fade",
        "Left",
        "Right",
        "Up",
        "Down",
    ];
    value
        .and_then(|value| {
            value
                .as_u64()
                .and_then(|index| NAMES.get(index as usize).copied())
                .or_else(|| {
                    value.as_str().and_then(|text| {
                        NAMES
                            .iter()
                            .copied()
                            .find(|name| name.eq_ignore_ascii_case(text))
                    })
                })
        })
        .unwrap_or(default)
        .to_owned()
}
/// Sample-radius scaling from Radar.Draw: near circles shrink between the
/// first two rings and keep the middle-ring scale up to the 10 km boundary.
pub fn radar_radius_fraction(distance_m: f64, radius_m: f64) -> f32 {
    if !distance_m.is_finite() || !radius_m.is_finite() || radius_m <= 0.0 {
        return 0.0;
    }
    let near = RADAR_INNER_FRACTION * radius_m / 1000.0;
    let far = (RADAR_MIDDLE_FRACTION - RADAR_INNER_FRACTION) * radius_m / 2000.0;
    if distance_m <= 1000.0 {
        near as f32
    } else if distance_m < 2000.0 {
        (far + (near - far) * (2000.0 - distance_m) / 1000.0) as f32
    } else {
        far as f32
    }
}
fn text<'a>(value: &'a Value, key: &str, other: &str) -> &'a str {
    value
        .get(key)
        .or_else(|| value.get(other))
        .and_then(Value::as_str)
        .unwrap_or("")
}
fn integer(value: &Value, key: &str, default: i64) -> i64 {
    value[key]
        .as_i64()
        .or_else(|| value[key].as_u64().and_then(|v| v.try_into().ok()))
        .unwrap_or(default)
}
pub fn relative_position(
    lat: f64,
    lon: f64,
    heading: f64,
    target_lat: f64,
    target_lon: f64,
    radius_m: f64,
) -> (f64, f32, f32) {
    let distance = great_circle_distance((lat, lon), (target_lat, target_lon), radius_m.max(0.0));
    let lat = lat.to_radians();
    let other = target_lat.to_radians();
    let delta = (target_lon - lon).to_radians();
    let bearing = (delta.sin() * other.cos())
        .atan2(lat.cos() * other.sin() - lat.sin() * other.cos() * delta.cos())
        - heading.to_radians();
    let km = distance / 1000.0;
    let scale = if km <= 1.0 {
        km * RADAR_INNER_FRACTION
    } else if km <= 2.0 {
        RADAR_INNER_FRACTION + (km - 1.0) * (RADAR_MIDDLE_FRACTION - RADAR_INNER_FRACTION)
    } else if km <= 10.0 {
        RADAR_MIDDLE_FRACTION + (km - 2.0) / 8.0 * (1.0 - RADAR_MIDDLE_FRACTION)
    } else {
        1.0
    };
    (
        distance,
        (bearing.sin() * scale) as f32,
        (-bearing.cos() * scale) as f32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_defaults_are_preserved() {
        let settings = default_settings();
        assert_eq!(settings["TTL"], 5000);
        assert_eq!(settings["ColorHex"], "FFFF6300");
        assert_eq!(settings["MaxNotifications"], 3);
        assert_eq!(settings["RadarSettings"]["Enabled"], true);
    }
    #[test]
    fn notification_update_cancel_and_persistent_lifetime() {
        let mut overlay = StellarOverlay::new(json!({}));
        overlay.on_notification_at(&json!({"id":"one","title":"First","timeout":0}), 10);
        overlay.on_notification_at(&json!({"id":"one","action":"update","detail":"Second"}), 20);
        overlay.tick_at(999_999);
        assert_eq!(overlay.snapshot().notifications[0].detail, "Second");
        overlay.on_notification_at(&json!({"id":"one","action":"cancel"}), 1_000_000);
        assert!(overlay.snapshot().notifications.is_empty());
    }
    #[test]
    fn queued_notifications_receive_full_lifetime_when_released() {
        let mut overlay = StellarOverlay::new(json!({"MaxNotifications":1}));
        overlay.on_notification_at(&json!({"id":"a","title":"A","timeout":100}), 1);
        overlay.on_notification_at(&json!({"id":"b","title":"B","timeout":100}), 2);
        overlay.tick_at(300);
        assert_eq!(overlay.snapshot().notifications[0].id, "b");
        overlay.tick_at(399);
        assert_eq!(overlay.snapshot().notifications.len(), 1);
        overlay.tick_at(400);
        assert!(overlay.snapshot().notifications.is_empty());
    }
    #[test]
    fn radar_is_heading_relative_and_handles_dateline() {
        let (d, x, y) = relative_position(0.0, 179.9, 90.0, 0.0, -179.9, 250000.0);
        assert!((d - 872.6646).abs() < 0.1);
        assert!(x.abs() < 0.00001);
        assert!(y < 0.0);
    }
    #[test]
    fn radar_context_planet_changes_and_batch_suppression() {
        let mut o = StellarOverlay::new(json!({}));
        o.handle_message(
            "BioInsights",
            "RadarSetPlanet",
            &json!({"ID64":u64::MAX,"BodyID":2,"RadiusKM":250.0}),
        )
        .unwrap();
        o.on_status(&json!({"Latitude":0.0,"Longitude":0.0,"Heading":0.0}));
        o.handle_message(
            "BioInsights",
            "RadarAddBiological",
            &json!({"UseCurrentPosition":true,"ContextID":7,"Radius":500.0}),
        )
        .unwrap();
        assert_eq!(o.snapshot().radar.points.len(), 1);
        o.handle_message(
            "BioInsights",
            "RadarSetPlanet",
            &json!({"ID64":42,"BodyID":3,"RadiusKM":300.0}),
        )
        .unwrap();
        assert!(o.snapshot().radar.points.is_empty());
        o.set_batch(true);
        o.on_notification(&json!({"title":"Past"}));
        assert!(o.snapshot().notifications.is_empty());
    }
    #[test]
    fn installed_ring_projection_uses_seventy_six_and_eighty_eight_percent() {
        // Direct oracle: Radar.Initialise sets radii 76%,88%,100%;
        // UpdateRelativePosition interpolates 0–1,1–2,2–10 kilometres.
        for (distance, fraction) in [
            (0.0, 0.0),
            (500.0, 0.38),
            (1000.0, 0.76),
            (1500.0, 0.82),
            (2000.0, 0.88),
            (6000.0, 0.94),
            (10000.0, 1.0),
            (15000.0, 1.0),
        ] {
            let (latitude, radius) = (
                distance / 1_000_000.0 * 180.0 / std::f64::consts::PI,
                1_000_000.0,
            );
            let (actual, x, y) = relative_position(0.0, 0.0, 0.0, latitude, 0.0, radius);
            assert!((actual - distance).abs() < 1e-7);
            assert!(x.abs() < 1e-7);
            assert!((f64::from(-y) - fraction).abs() < 1e-6, "{distance} m");
        }
        assert!((radar_radius_fraction(500.0, 500.0) - 0.38).abs() < 1e-6);
        assert!((radar_radius_fraction(1500.0, 500.0) - 0.205).abs() < 1e-6);
        assert!((radar_radius_fraction(2000.0, 500.0) - 0.03).abs() < 1e-6);
        assert!((radar_radius_fraction(6000.0, 500.0) - 0.03).abs() < 1e-6);
    }
    #[test]
    fn original_radar_payload_fields_and_planet_context_are_preserved() {
        let mut overlay = StellarOverlay::new(json!({}));
        overlay
            .handle_message(
                "BioInsights",
                "RadarSetPlanet",
                &json!({"ID64":42,"BodyID":1,"RadiusKM":250,"Genuses":{"7":"Stratum"}}),
            )
            .unwrap();
        assert_eq!(overlay.snapshot().radar.context_id, -1);
        overlay
            .handle_message(
                "BioInsights",
                "SetColonyRanges",
                &json!({"Payload":{"7":500,"8":1000}}),
            )
            .unwrap();
        overlay
            .handle_message(
                "BioInsights",
                "RadarSetContextID",
                &json!({"ContextID":7,"MinDistance":500,"SampleCount":2}),
            )
            .unwrap();
        let radar = overlay.snapshot().radar;
        assert_eq!(radar.minimum_distance_m, 500.0);
        assert_eq!(radar.sample_count, 2);
        assert_eq!(radar.colony_ranges[&8], 1000.0);
        overlay
            .handle_message("BioInsights", "RadarForceHidden", &json!({"Value":true}))
            .unwrap();
        assert!(!overlay.snapshot().radar.visible);
        overlay
            .handle_message(
                "BioInsights",
                "RadarSetPlanet",
                &json!({"ID64":42,"BodyID":1,"RadiusKM":250,"Genuses":{"8":"Electricae"}}),
            )
            .unwrap();
        assert!(!overlay.snapshot().radar.visible);
        assert_eq!(overlay.snapshot().radar.genuses.len(), 2);
        overlay
            .handle_message("BioInsights", "RadarForceHidden", &json!({"Value":false}))
            .unwrap();
        assert!(overlay.snapshot().radar.visible);
        overlay
            .handle_message(
                "BioInsights",
                "RadarRemoveBiological",
                &json!({"ContextID":7,"NewContextID":-1,"SampleCount":0}),
            )
            .unwrap();
        assert_eq!(overlay.snapshot().radar.context_id, -1);
        assert_eq!(overlay.snapshot().radar.sample_count, 0);
        overlay
            .handle_message(
                "BioInsights",
                "RadarSetPlanet",
                &json!({"ID64":42,"BodyID":2,"RadiusKM":300}),
            )
            .unwrap();
        assert!(overlay.snapshot().radar.genuses.is_empty());
    }
    #[test]
    fn radar_vehicle_replacement_biology_proximity_and_waypoint_numbering_match_source() {
        let mut overlay = StellarOverlay::new(json!({}));
        overlay
            .handle_message(
                "BioInsights",
                "RadarSetPlanet",
                &json!({"ID64":42,"BodyID":1,"RadiusKM":250}),
            )
            .unwrap();
        overlay.on_status(&json!({"Latitude":0.0,"Longitude":0.0}));
        overlay
            .handle_message(
                "BioInsights",
                "RadarSetContextID",
                &json!({"ContextID":7,"MinDistance":500}),
            )
            .unwrap();
        overlay.handle_message("BioInsights","RadarAddBiological",&json!({"UseCurrentPosition":true,"ContextID":7,"Radius":500,"MeasureDistance":true})).unwrap();
        let latitude = 100.0 / 250_000.0 * 180.0 / std::f64::consts::PI;
        overlay.on_status(&json!({"Latitude":latitude,"Longitude":0.0}));
        assert!((overlay.snapshot().radar.sample_distance_m - 100.0).abs() < 1e-6);
        overlay
            .handle_message(
                "BioInsights",
                "RadarAddBiological",
                &json!({"UseCurrentPosition":true,"ContextID":7,"Radius":500}),
            )
            .unwrap();
        let latitude = 105.0 / 250_000.0 * 180.0 / std::f64::consts::PI;
        overlay.on_status(&json!({"Latitude":latitude,"Longitude":0.0}));
        overlay
            .handle_message(
                "BioInsights",
                "RadarAddBiological",
                &json!({"UseCurrentPosition":true,"ContextID":7,"Radius":500}),
            )
            .unwrap();
        assert_eq!(overlay.snapshot().radar.points.len(), 2);
        for x in [1.0, 2.0] {
            overlay
                .handle_message(
                    "BioInsights",
                    "RadarAddVehicle",
                    &json!({"X":x,"Y":0.0,"ContextID":0}),
                )
                .unwrap();
        }
        assert_eq!(
            overlay
                .snapshot()
                .radar
                .points
                .iter()
                .filter(|point| point.kind == "Vehicle")
                .count(),
            1
        );
        for _ in 0..2 {
            overlay
                .handle_message("BioInsights", "RadarAddWaypoint", &json!({}))
                .unwrap();
        }
        assert_eq!(
            overlay
                .snapshot()
                .radar
                .points
                .iter()
                .filter(|point| point.kind == "Waypoint")
                .map(|point| point.label.as_str())
                .collect::<Vec<_>>(),
            vec!["WP1", "WP2"]
        );
        let before = overlay.snapshot().radar.points.len();
        assert!(overlay
            .handle_message(
                "BioInsights",
                "RadarAddBiological",
                &json!({"ContextID":7,"UseCurrentPosition":false})
            )
            .unwrap()
            .is_none());
        assert_eq!(overlay.snapshot().radar.points.len(), before);
    }
    #[test]
    fn source_notification_transition_defaults_and_bitmap_aliases_are_exposed() {
        let mut overlay = StellarOverlay::new(json!({}));
        overlay.on_notification_at(&json!({"id":"queued","title":"Queued","timeout":5000}), 10);
        let item = &overlay.snapshot().notifications[0];
        assert_eq!(
            (
                item.entry_transition.as_str(),
                item.exit_transition.as_str(),
                item.transition_ms
            ),
            ("Left", "Right", 300)
        );
        overlay.on_notification_at(
            &json!({"id":"fixed","timeout":0,"EntryTransition":2,"ExitTransition":"Down"}),
            20,
        );
        let snapshot = overlay.snapshot();
        let item = snapshot
            .notifications
            .iter()
            .find(|item| item.id == "fixed")
            .unwrap();
        assert_eq!(item.entry_transition, "Fade");
        assert_eq!(item.exit_transition, "Down");
        overlay.handle_message("BioInsights","ShowBitmapNotification",&json!({"ImageKey":"bitmap","Bitmap":{"png_base64":"test"},"ForceShow":true,"EntryTransition":3,"ExitTransition":4})).unwrap();
        let snapshot = overlay.snapshot();
        let bitmap = snapshot
            .alerts
            .iter()
            .find(|value| value["Key"] == "bitmap")
            .unwrap();
        assert_eq!(bitmap["ImageKey"], "bitmap");
        assert_eq!(bitmap["EntryTransition"], "Left");
        assert_eq!(bitmap["ExitTransition"], "Right");
        assert_eq!(bitmap["TTLms"], 3000);
        assert_eq!(bitmap["TransitionMS"], 300);
        assert_eq!(bitmap["Opacity"], 100.0);
        assert_eq!(bitmap["ForceShow"], true);
        overlay
            .handle_message(
                "BioInsights",
                "HideBitmapNotification",
                &json!({"ImageKey":"bitmap"}),
            )
            .unwrap();
        let expires = overlay.snapshot().alerts[0]["expires_ms"].as_u64().unwrap();
        overlay.tick_at(expires);
        assert!(overlay.snapshot().alerts.is_empty());
    }
    #[test]
    fn first_monotonic_tick_still_enforces_source_inter_notification_delay() {
        let mut overlay = StellarOverlay::new(json!({"DelayBetweenNotificationsMS":200}));
        for id in ["first", "second"] {
            overlay.on_notification_at(&json!({"id":id,"timeout":5000}), 0);
        }
        assert_eq!(overlay.snapshot().notifications.len(), 1);
        overlay.tick_at(199);
        assert_eq!(overlay.snapshot().notifications.len(), 1);
        overlay.tick_at(200);
        assert_eq!(overlay.snapshot().notifications.len(), 2);
    }
}

#[cfg(test)]
mod parity_regressions {
    use super::*;
    #[test]
    fn source_settings_changes_preserve_all_live_state() {
        let mut overlay = StellarOverlay::new(json!({"MaxNotifications":1}));
        overlay.on_notification_at(
            &json!({"Sender":"BioInsights","id":"one","Title":"Active","Timeout":60000}),
            10,
        );
        overlay.on_notification_at(
            &json!({"Sender":"BioInsights","id":"two","Title":"Queued","Timeout":60000}),
            11,
        );
        overlay.radar.system_address = 42;
        overlay.radar.context_id = 7;
        overlay.radar.sample_count = 2;
        overlay.radar.seen_genuses.insert(3);
        overlay.set_bio_panel(json!({"ImageKey":"bio","persistent":true}));
        let mut settings = overlay.settings().clone();
        settings["FontScaling"] = json!(150);
        overlay.set_settings(settings).unwrap();
        assert_eq!(overlay.active[0].id, "one");
        assert_eq!(overlay.pending[0].id, "two");
        assert_eq!(overlay.radar.system_address, 42);
        assert_eq!(overlay.radar.context_id, 7);
        assert_eq!(overlay.radar.sample_count, 2);
        assert!(overlay.radar.seen_genuses.contains(&3));
        assert!(overlay.alerts.contains_key("BioInsights"));
        assert_eq!(overlay.settings()["FontScaling"], 150);
    }
    #[test]
    fn custom_color_registration_merges_and_unregister_retains_brush_cache() {
        let mut overlay = StellarOverlay::new(json!({}));
        assert!(overlay
            .handle_message(
                "Bio",
                "RegisterPluginCustomColours",
                &json!({"TitleColor":"FFFF0000"})
            )
            .unwrap()
            .is_none());
        overlay.on_notification_at(&json!({"Sender":"Bio","id":"one","Timeout":0}), 10);
        let reply = overlay
            .handle_message(
                "Bio",
                "RegisterPluginCustomColours",
                &json!({"TextColor":"FF00FF00","GetDefaultColors":true}),
            )
            .unwrap()
            .unwrap();
        assert_eq!(reply["Payload"]["TitleColor"], "FFFF7B00");
        assert_eq!(overlay.active[0].colors["TitleColorHex"], "FFFF0000");
        assert_eq!(overlay.active[0].colors["ColorHex"], "FF00FF00");
        overlay
            .handle_message("Bio", "UnregisterPluginCustomColours", &json!({}))
            .unwrap();
        assert!(overlay.active[0].colors.is_null());
        overlay
            .handle_message(
                "Bio",
                "RegisterPluginCustomColours",
                &json!({"BorderColor":"FF0000FF"}),
            )
            .unwrap();
        assert_eq!(overlay.active[0].colors["TitleColorHex"], "FFFF0000");
        assert_eq!(overlay.active[0].colors["ColorHex"], "FF00FF00");
        assert_eq!(overlay.active[0].colors["BorderColorHex"], "FF0000FF");
    }
    #[test]
    fn source_settings_preview_actions_emit_actual_notifications_and_snapshot() {
        let mut overlay = StellarOverlay::new(json!({}));
        overlay
            .handle_message(
                "Core",
                "TestNotifications",
                &json!({"Settings":{"MaxNotifications":5,"HandleObservatoryNotifications":false}}),
            )
            .unwrap();
        assert_eq!(overlay.active.len() + overlay.pending.len(), 5);
        assert_eq!(overlay.active[0].title, "A Test Notification 1");
        assert_eq!(
            overlay.active[0].detail,
            "A sample notification (1 of 5) to test Overlay settings\nA multi-line test."
        );
        assert!(!overlay.handles_notifications());
        assert!(overlay.snapshot().force_show);
        let state = overlay
            .handle_message("Core", "DebugState", &json!({}))
            .unwrap()
            .unwrap();
        assert_eq!(state["Command"], "OverlayState");
        assert_eq!(state["Payload"]["settings"]["MaxNotifications"], 5);
        overlay
            .handle_message("Core", "TestRadar", &json!({}))
            .unwrap();
        assert!(overlay.active.is_empty());
        assert!(overlay.radar.points.is_empty());
        assert!(overlay.snapshot().radar.visible);
        assert!(overlay.radar_force_until > now_ms());
    }
}

#[cfg(test)]
mod source_notification_routing {
    use super::*;
    #[test]
    fn original_onnotificationevent_matches_all_1728_source_method_cases() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../../fixtures/stellar-notification-routing-oracle.json"
        ))
        .unwrap();
        for case in oracle["cases"].as_array().unwrap() {
            let mut overlay =
                StellarOverlay::new(json!({"HandleObservatoryNotifications":case["handle"]}));
            let mut value = json!({"Title":"Oracle","Rendering":case["rendering"],"Timeout":case["ttl"],"XPos":case["x"],"YPos":case["y"]});
            match case["guid"].as_str().unwrap() {
                "valid" => value["Guid"] = json!("11111111-1111-1111-1111-111111111111"),
                "empty" => value["Guid"] = json!("00000000-0000-0000-0000-000000000000"),
                _ => {}
            }
            overlay.on_notification_at(&value, 10);
            let mut native =
                StellarOverlay::new(json!({"HandleObservatoryNotifications":case["handle"]}));
            let mut lowercase = value.clone();
            lowercase["rendering"] = lowercase
                .as_object_mut()
                .unwrap()
                .remove("Rendering")
                .unwrap();
            native.on_notification_at(&lowercase, 10);
            assert_eq!(
                native.active.len(),
                overlay.active.len(),
                "lowercase rendering: {case}"
            );
            let output = if overlay.active.is_empty() {
                "ignored"
            } else if overlay.active[0].instant {
                "instant"
            } else {
                "queue"
            };
            assert_eq!(output, case["output"], "{case}");
            if let Some(item) = overlay.active.front() {
                let source_ttl = case["result_ttl"].as_i64().unwrap();
                let ttl = if source_ttl < 0 { 5000 } else { source_ttl };
                assert_eq!(
                    item.expires_ms,
                    if ttl == 0 {
                        None
                    } else {
                        Some(10 + ttl as u64)
                    },
                    "{case}"
                );
            }
        }
    }
    #[test]
    fn original_instant_update_changes_position_and_extended_cancel_removes_item() {
        let mut overlay = StellarOverlay::new(json!({}));
        overlay.on_notification_at(&json!({"Guid":"a","Title":"A","Timeout":0}), 10);
        overlay.on_notification_at(
            &json!({"Guid":"a","Action":"update","Title":"B","XPos":20,"YPos":30}),
            20,
        );
        assert_eq!(overlay.active[0].title, "B");
        assert_eq!((overlay.active[0].x, overlay.active[0].y), (20.0, 30.0));
        overlay.on_notification_at(
            &json!({"Guid":"a","Action":"update","ExtendedDetails":"CANCEL"}),
            30,
        );
        overlay.tick_at(30);
        assert!(overlay.active.is_empty());
        overlay.on_notification_at(&json!({"Guid":"b","Title":"A","Timeout":0}), 40);
        overlay
            .set_settings(json!({"HandleObservatoryNotifications":false}))
            .unwrap();
        overlay.on_notification_at(&json!({"Guid":"b","Action":"cancel"}), 50);
        assert_eq!(overlay.active.len(), 1);
    }
    #[test]
    fn empty_original_guids_receive_distinct_generated_ids() {
        let mut overlay = StellarOverlay::new(json!({}));
        for _ in 0..2 {
            overlay.on_notification_at(
                &json!({"Guid":"00000000-0000-0000-0000-000000000000","Timeout":0}),
                10,
            );
        }
        assert_eq!(overlay.active.len(), 2);
        assert_ne!(overlay.active[0].id, overlay.active[1].id);
        assert!(!overlay.active[0].persistent);
    }
    #[test]
    fn original_updatenotification_matches_32_source_method_cases() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../../fixtures/stellar-notification-routing-oracle.json"
        ))
        .unwrap();
        for case in oracle["updates"].as_array().unwrap() {
            let mut overlay = StellarOverlay::new(json!({}));
            if case["known"] == true {
                overlay.on_notification_at(&json!({"Guid":"11111111-1111-1111-1111-111111111111","Title":"Old","Timeout":0}),10);
            }
            overlay
                .set_settings(json!({"HandleObservatoryNotifications":case["handle"]}))
                .unwrap();
            overlay.on_notification_at(&json!({"Guid":"11111111-1111-1111-1111-111111111111","Action":"update","Title":"New","Rendering":case["rendering"]}),20);
            assert_eq!(
                overlay.active.iter().any(|n| n.title == "New"),
                case["instant"].as_bool().unwrap(),
                "{case}"
            );
        }
    }
}

#[cfg(test)]
mod source_queue_slots {
    use super::*;
    #[test]
    fn surviving_items_keep_their_original_allocated_source_reference_slot() {
        let mut overlay =
            StellarOverlay::new(json!({"MaxNotifications":2,"DelayBetweenNotificationsMS":0}));
        overlay.on_notification_at(&json!({"id":"a","Timeout":100}), 0);
        overlay.on_notification_at(&json!({"id":"b","Timeout":500}), 1);
        assert_eq!(overlay.active[0].queue_slot, Some(0));
        assert_eq!(overlay.active[1].queue_slot, Some(1));
        overlay.tick_at(100);
        assert_eq!(overlay.active[0].id, "b");
        assert_eq!(overlay.active[0].queue_slot, Some(1));
        overlay.on_notification_at(&json!({"id":"c","Timeout":500}), 101);
        assert_eq!(overlay.active[1].queue_slot, Some(0));
    }
}

#[cfg(test)]
mod source_missing_point_keys {
    use super::*;
    #[test]
    fn original_missing_coordinates_or_context_are_ignored_without_error_or_point() {
        let mut overlay = StellarOverlay::new(json!({}));
        for value in [
            json!({"ContextID":0}),
            json!({"X":1,"Y":2}),
            json!({"UseCurrentPosition":false,"ContextID":1}),
        ] {
            assert!(overlay
                .handle_message("BioInsights", "RadarAddVehicle", &value)
                .unwrap()
                .is_none());
            assert!(overlay.radar.points.is_empty());
        }
        overlay
            .handle_message(
                "BioInsights",
                "RadarAddVehicle",
                &json!({"UseCurrentPosition":true,"ContextID":0}),
            )
            .unwrap();
        assert_eq!(overlay.radar.points.len(), 1);
        assert_eq!(overlay.radar.points[0].latitude, 0.0);
    }
}
