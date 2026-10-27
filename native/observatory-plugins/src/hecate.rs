//! Native data-vault port of installed Hecate 1.3.444.761.
//!
//! Hecate is a background service: it intentionally produces no scan findings.
//! Its durable tables and message field names follow the installed assembly.
mod schema;
mod server;
mod spansh;
mod storage;
pub use spansh::from_spansh;

use anyhow::{bail, ensure, Context, Result};
use observatory_domain::{Finding, JournalEvent};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    path::Path,
    time::{Duration, Instant},
};
use storage::{epoch, now, timestamp, SystemStore};

pub const VERSION: &str = "1.3.444.761";
pub const NAME: &str = "Hecate - Data Vault";
pub const HAS_UI: bool = false;

pub fn default_settings() -> Value {
    json!({
        "InstallUpdates":true,"InstallBetas":false,"VerboseLogging":false,
        "RemoteLookups":true,
        "ServerSettings":{"EnableWebServer":false,"ServerPort":58080,"ServerLANIPAddress":""}
    })
}

struct PendingRequest {
    kind: &'static str,
    id: u64,
    replies: Vec<Value>,
}

pub struct Hecate {
    settings: Value,
    general: Connection,
    journals: SystemStore,
    spansh: SystemStore,
    canonn: SystemStore,
    current_id: u64,
    current_fid: i64,
    current_name: String,
    journal_text: String,
    last_timestamp: String,
    batch: bool,
    http_requests: Vec<Value>,
    pending: HashMap<String, PendingRequest>,
    request_counter: u64,
    automatic_lookup_started: HashMap<&'static str, Instant>,
    messages: Vec<Value>,
    errors: Vec<String>,
    registered_plugins: HashMap<String, Value>,
    server: Option<server::Server>,
}

fn archives_event(event: &str) -> bool {
    matches!(
        event,
        "ApproachBody"
            | "CarrierJump"
            | "CodexEntry"
            | "FSDJump"
            | "FSSAllBodiesFound"
            | "FSSBodySignals"
            | "Location"
            | "SAAScanComplete"
            | "SAASignalsFound"
            | "Scan"
            | "ScanBaryCentre"
            | "ScanOrganic"
            | "SupercruiseEntry"
            | "SupercruiseExit"
    )
}

impl Hecate {
    pub fn new(settings: Value) -> Result<Self> {
        Self::build(settings, None)
    }
    pub fn open(settings: Value, directory: &Path) -> Result<Self> {
        std::fs::create_dir_all(directory)?;
        Self::build(settings, Some(directory))
    }
    fn build(settings: Value, directory: Option<&Path>) -> Result<Self> {
        let general = match directory {
            Some(path) => Connection::open(path.join("general.db"))?,
            None => Connection::open_in_memory()?,
        };
        schema::initialize(&general)?;
        let open_store = |table| {
            let path = directory.map(|dir| dir.join(format!("{table}.db")));
            SystemStore::open(path.as_deref(), table)
        };
        let mut merged = default_settings();
        merge(&mut merged, &settings);
        let mut vault = Self {
            settings: merged,
            general,
            journals: open_store("journals")?,
            spansh: open_store("spansh")?,
            canonn: open_store("canonn")?,
            current_id: 0,
            current_fid: 0,
            current_name: String::new(),
            journal_text: String::new(),
            last_timestamp: String::new(),
            batch: false,
            http_requests: Vec::new(),
            pending: HashMap::new(),
            request_counter: 0,
            automatic_lookup_started: HashMap::new(),
            messages: Vec::new(),
            errors: Vec::new(),
            registered_plugins: HashMap::new(),
            server: None,
        };
        vault.restart_server();
        Ok(vault)
    }
    pub fn settings(&self) -> &Value {
        &self.settings
    }
    pub fn set_settings(&mut self, settings: Value) {
        if let Err(error) = self.try_set_settings(settings) {
            self.errors.push(format!("Hecate settings: {error:#}"));
        }
    }
    /// Apply source preferences in place. Listener replacement is transactional:
    /// a bind failure retains the old preferences and restores its listener.
    pub fn try_set_settings(&mut self, settings: Value) -> Result<()> {
        ensure!(settings.is_object(), "Hecate settings must be an object");
        let mut merged = default_settings();
        merge(&mut merged, &settings);
        let port = merged["ServerSettings"]["ServerPort"]
            .as_u64()
            .filter(|p| *p >= 8000 && *p < 59000)
            .unwrap_or(58080);
        merged["ServerSettings"]["ServerPort"] = json!(port);
        let address = merged["ServerSettings"]["ServerLANIPAddress"]
            .as_str()
            .unwrap_or("");
        if !address.is_empty() && address.parse::<std::net::IpAddr>().is_err() {
            merged["ServerSettings"]["ServerLANIPAddress"] = json!("");
        }
        if merged["ServerSettings"] != self.settings["ServerSettings"] {
            if merged["ServerSettings"]["EnableWebServer"].as_bool() == Some(true) {
                let same_port = self
                    .server
                    .as_ref()
                    .is_some_and(|server| server.port() as u64 == port);
                if same_port {
                    self.server = None;
                }
                let replacement = server::Server::start(
                    merged["ServerSettings"]["ServerLANIPAddress"]
                        .as_str()
                        .unwrap_or(""),
                    port as u16,
                );
                match replacement {
                    Ok(server) => {
                        for (name, target) in &self.registered_plugins {
                            server.register(name, target.clone())?;
                        }
                        self.server = Some(server);
                    }
                    Err(error) => {
                        if same_port {
                            self.restart_server();
                        }
                        return Err(error.context("Hecate server preferences were not applied"));
                    }
                }
            } else {
                self.server = None;
            }
            if let Some(server) = &self.server {
                for target in self.registered_plugins.values() {
                    self.messages.push(json!({"Target":target,"Command":"SetServerAddress","Payload":{"ServerAddress":server.address(),"ServerPort":server.port().to_string()}}));
                }
            }
        }
        self.settings = merged;
        Ok(())
    }
    fn restart_server(&mut self) {
        self.server = None;
        if self.settings["ServerSettings"]["EnableWebServer"].as_bool() != Some(true) {
            return;
        }
        let config = &self.settings["ServerSettings"];
        let port = config["ServerPort"]
            .as_u64()
            .filter(|p| *p >= 8000 && *p < 59000)
            .unwrap_or(58080) as u16;
        match server::Server::start(config["ServerLANIPAddress"].as_str().unwrap_or(""), port) {
            Ok(server) => {
                for (name, target) in &self.registered_plugins {
                    let _ = server.register(name, target.clone());
                    self.messages.push(json!({"Target":target,"Command":"SetServerAddress","Payload":{"ServerAddress":server.address(),"ServerPort":server.port().to_string()}}));
                }
                self.server = Some(server);
            }
            Err(error) => self.errors.push(format!("Hecate web server: {error:#}")),
        }
    }
    /// Register using sender identity, as the original plugin message bus did.
    pub fn register_plugin(&mut self, name: &str, target: Value) -> Result<()> {
        let name = sanitize_name(name);
        ensure!(
            !name.is_empty() && !matches!(name.as_str(), "status" | "debug" | "faviconico"),
            "reserved or empty Hecate plugin name"
        );
        self.registered_plugins.insert(name.clone(), target.clone());
        if let Some(server) = &self.server {
            server.register(&name, target.clone())?;
            self.messages.push(json!({"Target":target,"Command":"SetServerAddress","Payload":{"ServerAddress":server.address(),"ServerPort":server.port().to_string()}}));
        }
        Ok(())
    }
    /// Drain bridge requests on the host's regular worker tick.
    pub fn poll_bridge(&mut self) {
        if let Some(server) = &self.server {
            self.messages.extend(server.poll());
        }
    }
    /// Native equivalent of updating the original shared WebRequest Response dictionary.
    pub fn respond_web(&self, request: &str, response: Value) -> Result<()> {
        self.server
            .as_ref()
            .context("Hecate web server disabled")?
            .respond(request, response)
    }
    pub fn server_address(&self) -> Option<(String, u16)> {
        self.server
            .as_ref()
            .map(|server| (server.address().to_owned(), server.port()))
    }
    /// Clear transient visit state without destroying persisted commander data.
    pub fn reset(&mut self) {
        if let Err(error) = self.flush() {
            self.errors.push(format!("Hecate flush: {error:#}"));
        }
        self.current_id = 0;
        self.current_fid = 0;
        self.current_name.clear();
        self.journal_text.clear();
        self.pending.clear();
        self.http_requests.clear();
    }
    /// Explicit journal rebuild, corresponding to the original Read All action.
    pub fn begin_replay(&mut self) -> Result<()> {
        self.reset();
        self.general.execute_batch("BEGIN; DELETE FROM evaluator; DELETE FROM evaluator_systems; DELETE FROM bioinsights_unsold; DELETE FROM bioinsights_unsold_bonus; DELETE FROM census; DELETE FROM commanders; DELETE FROM syslookup_000; DELETE FROM pp_merits_000; COMMIT;")?;
        self.journals.clear()?;
        self.batch = true;
        Ok(())
    }
    pub fn set_batch(&mut self, batch: bool) {
        if self.batch && !batch {
            if let Err(error) = self.flush() {
                self.errors.push(format!("Hecate flush: {error:#}"));
            }
        }
        self.batch = batch;
    }
    pub fn flush(&mut self) -> Result<()> {
        if !self.journal_text.is_empty() && self.current_id != 0 {
            let date = if self.last_timestamp.is_empty() {
                now()
            } else {
                self.last_timestamp.clone()
            };
            self.journals.add(
                self.current_id,
                self.current_fid,
                &self.journal_text,
                &date,
                false,
            )?;
            self.journal_text.clear();
        }
        Ok(())
    }
    pub fn on_event(&mut self, event: &JournalEvent) -> Vec<Finding> {
        if let Err(error) = self.process(event) {
            self.errors
                .push(format!("Hecate {}: {error:#}", event.event));
        }
        Vec::new()
    }
    fn process(&mut self, event: &JournalEvent) -> Result<()> {
        if event.event == "LoadGame" {
            let fid = event
                .data
                .get("FID")
                .map(fid_value)
                .transpose()?
                .unwrap_or(0);
            if self.current_fid != fid {
                self.flush()?;
                self.current_id = 0;
            }
            self.current_fid = fid;
            self.current_name = event.str("Commander").to_owned();
            self.general.execute("INSERT INTO commanders(name,fid,lastseenepoch) VALUES(?1,?2,?3) ON CONFLICT(fid,name) DO UPDATE SET lastseenepoch=excluded.lastseenepoch",params![self.current_name,self.current_fid,epoch(&event.timestamp)?])?;
        }
        if matches!(event.event.as_str(), "Location" | "FSDJump")
            || (event.event == "CarrierJump"
                && (event.boolean("Docked") || event.boolean("OnFoot")))
        {
            if let Some(id) = event.id("SystemAddress") {
                if id != self.current_id {
                    self.flush()?;
                    self.current_id = id;
                }
                let position = event.data.get("StarPos").and_then(Value::as_array);
                let coordinate = |index| {
                    position
                        .and_then(|p| p.get(index))
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0)
                };
                self.general.execute("INSERT OR REPLACE INTO syslookup_000(id64,name,x,y,z,population) VALUES(?1,?2,?3,?4,?5,?6)",params![id as i64,event.str("StarSystem"),coordinate(0),coordinate(1),coordinate(2),event.data.get("Population").and_then(Value::as_i64).unwrap_or(0)])?;
                if !self.batch {
                    self.queue_cache("spansh", id, None)?;
                    self.queue_cache("canonn", id, None)?;
                }
            }
        }
        if !self.batch
            && (event.event == "CarrierJumpRequest"
                || (event.event == "StartJump" && event.str("JumpType") == "Hyperspace"))
        {
            if let Some(id) = event.id("SystemAddress") {
                self.queue_cache("spansh", id, None)?;
                self.queue_cache("canonn", id, None)?;
            }
        }
        // Hecate's census records decorated NPC pilots, not player commanders.
        let census_name = match event.event.as_str() {
            "ReceiveText"
                if event.str("Channel").eq_ignore_ascii_case("npc")
                    && event.str("From").starts_with("$npc_name_decorate:") =>
            {
                event.str("From_Localised")
            }
            "ShipTargeted" | "Bounty"
                if event.str("PilotName").starts_with("$npc_name_decorate:") =>
            {
                event.str("PilotName_Localised")
            }
            _ => "",
        };
        if !census_name.is_empty() && self.current_id != 0 {
            self.record_census(census_name, &event.timestamp)?;
        }
        if !event.timestamp.is_empty() {
            self.last_timestamp = event.timestamp.clone();
        }
        // Original JournalEvent archives only these fourteen exploration
        // subclasses. Ancillary files, arbitrary future events and census/
        // commander events have separate side effects but are not visit rows.
        if self.current_id != 0 && archives_event(&event.event) {
            let text = event.json();
            // Flush bounded chunks while retaining every event in visit order.
            if self.journal_text.len() + text.len() + 2 > 8 * 1024 * 1024 {
                self.flush()?;
            }
            self.journal_text.push_str(&text);
            self.journal_text.push_str("\r\n");
        }
        if event.event == "Shutdown" {
            self.flush()?;
        }
        Ok(())
    }
    fn record_census(&self, name: &str, date: &str) -> Result<()> {
        self.general.execute("INSERT INTO census(name,lastseenepoch,lastseensystem) VALUES(?1,?2,?3) ON CONFLICT(name) DO UPDATE SET count=count+1,lastseenepoch=excluded.lastseenepoch,lastseensystem=excluded.lastseensystem WHERE census.lastseenepoch+7200<excluded.lastseenepoch OR census.lastseensystem<>excluded.lastseensystem",params![name,epoch(date)?,self.current_id as i64])?;
        Ok(())
    }
    pub fn errors(&mut self) -> Vec<String> {
        std::mem::take(&mut self.errors)
    }
    pub fn rows(&self) -> Vec<Value> {
        let count = |table: &str| {
            self.general
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| {
                    r.get::<_, i64>(0)
                })
                .unwrap_or(0)
        };
        vec![
            self.journals
                .details()
                .unwrap_or(json!({"Store":"journals","Records":0})),
            self.spansh
                .details()
                .unwrap_or(json!({"Store":"spansh","Records":0})),
            self.canonn
                .details()
                .unwrap_or(json!({"Store":"canonn","Records":0})),
            json!({"Store":"commanders","Records":count("commanders")}),
            json!({"Store":"evaluator","Records":count("evaluator")}),
            json!({"Store":"bioinsights_unsold","Records":count("bioinsights_unsold")}),
        ]
    }
    pub fn take_http_requests(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.http_requests)
    }
    pub fn take_messages(&mut self) -> Vec<Value> {
        self.poll_bridge();
        std::mem::take(&mut self.messages)
    }

    /// Returns an original-command response envelope, or None for a mutation.
    pub fn handle_message(&mut self, command: &str, payload: &Value) -> Result<Option<Value>> {
        ensure!(
            payload.is_object(),
            "Hecate message payload must be an object"
        );
        match command {
            "Ready" | "LegacyPluginMessage" => Ok(None),
            "ResetJournalDatabase" => {
                ensure!(
                    payload["Confirmed"] == true,
                    "Reset Journal Database requires confirmation"
                );
                // HecateWorker.ResetAllBatabases calls ResetDatabase(journalDBH)
                // only: general ledgers, remote caches and active visit survive.
                self.journals.clear()?;
                Ok(reply("JournalDatabaseReset", json!({"Success":true})))
            }
            "GetSystemsVisitedFromList" => {
                self.flush()?;
                let ids = required(payload, "ID64s")?
                    .as_array()
                    .context("ID64s must be an array")?
                    .iter()
                    .map(unsigned)
                    .collect::<Result<Vec<_>>>()?;
                Ok(reply(
                    "SystemsVisitedFromListResponse",
                    json!({"ID64s":self.journals.visited(&ids)?}),
                ))
            }
            "LoadSystem" => {
                self.flush()?;
                let id = id(payload, "ID64")?;
                let before = date(payload, "DateTime")?;
                let (events, fids) = self.journals.visits(id, &before)?;
                Ok(reply(
                    "LoadSystemResponse",
                    json!({"ID64":id,"JournalEvents":events,"FIDs":fids}),
                ))
            }
            "SpanshSystem" => {
                let system = id(payload, "ID64")?;
                if let Some((text, date)) = self.spansh.latest(system)? {
                    if self.settings["RemoteLookups"].as_bool() == Some(false)
                        || chrono::Utc::now().timestamp() - epoch(&date)? <= 86_400
                    {
                        let data: Value = serde_json::from_str(&text)?;
                        return Ok(reply(
                            "SpanshSystemResponse",
                            json!({"ID64":system,"System":data.get("system").unwrap_or(&data),"JournalEvents":from_spansh(&data,None)?,"Response":payload.get("Response").cloned().unwrap_or(Value::Null)}),
                        ));
                    }
                }
                self.queue_cache("spansh", system, Some(payload.clone()))?;
                self.queue_cache("canonn", system, None)?;
                Ok(None)
            }
            "EvaluatorAdd" => {
                require_keys(
                    payload,
                    &["ID64", "BodyID", "Value", "Name", "DateTime", "FID"],
                )?;
                self.general.execute("INSERT OR REPLACE INTO evaluator(fid,id64,bodyid,value,bodyname,dtepoch) VALUES(?1,?2,?3,?4,?5,?6)",params![fid(payload)?,id(payload,"ID64")? as i64,integer(payload,"BodyID")?,integer(payload,"Value")?,text(payload,"Name")?,epoch(&date(payload,"DateTime")?)?])?;
                Ok(None)
            }
            "EvaluatorSysAdd" => {
                self.general.execute("INSERT OR REPLACE INTO evaluator_systems(fid,id64,name,dtepoch) VALUES(?1,?2,?3,?4)",params![fid(payload)?,id(payload,"ID64")? as i64,text(payload,"Name")?,epoch(&date(payload,"DateTime")?)?])?;
                Ok(None)
            }
            "EvaluatorDelete" => {
                self.general
                    .execute("DELETE FROM evaluator WHERE fid=?1", [fid(payload)?])?;
                Ok(None)
            }
            "EvaluatorGetData" => Ok(reply(
                "EvaluatorGetDataResponse",
                json!({"Response":payload.get("Response").cloned().unwrap_or(Value::Null),"Data":self.evaluator_data(fid(payload)?)?,"Processed":true}),
            )),
            "EvaluatorGetTotal" => {
                let total = self.evaluator_total(fid(payload)?)?;
                let mut result = json!({"Total":total});
                if payload["SubtractCurrent"].as_bool() == Some(true) {
                    result["SubtractCurrent"] = json!(true);
                }
                Ok(reply("EvaluatorSetTotal", result))
            }
            "BioInsightsUnsoldAdd" => {
                require_keys(
                    payload,
                    &[
                        "FID", "ID64", "BodyID", "BioName", "DateTime", "Species", "Value",
                        "BodyName",
                    ],
                )?;
                self.general.execute("INSERT OR REPLACE INTO bioinsights_unsold(fid,id64,bodyid,dtepoch,bioname,species,basevalue,bodyname) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![fid(payload)?,id(payload,"ID64")? as i64,integer(payload,"BodyID")?,epoch(&date(payload,"DateTime")?)?,text(payload,"BioName")?,integer(payload,"Species")?,integer(payload,"Value")?,text(payload,"BodyName")?])?;
                Ok(None)
            }
            "BioInsightsUnsoldDelete" => {
                self.general.execute(
                    "DELETE FROM bioinsights_unsold WHERE fid=?1",
                    [fid(payload)?],
                )?;
                Ok(None)
            }
            "BioInsightsSetBonus" => {
                let bonus = required(payload, "Value")?
                    .as_bool()
                    .context("Value must be boolean")?;
                self.general.execute("INSERT OR REPLACE INTO bioinsights_unsold_bonus(fid,id64,bodyid,species,bonus) SELECT fid,id64,bodyid,species,?1 FROM bioinsights_unsold WHERE id=?2",params![bonus,integer(payload,"ID")?])?;
                Ok(None)
            }
            "BioInsightsGetData" => Ok(reply(
                "BioInsightsGetDataResponse",
                json!({"Data":self.bioinsights_data(fid(payload)?)?,"Processed":true,"Response":payload.get("Response").cloned().unwrap_or(Value::Null)}),
            )),
            "BioInsightsGetTotal" => Ok(reply(
                "BioInsightsGetTotalResponse",
                json!({"FID":fid(payload)?,"Total":self.bioinsights_total(fid(payload)?)?}),
            )),
            "GetFID" => {
                let name = text(payload, "Name")?;
                let value = self
                    .general
                    .query_row(
                        "SELECT fid FROM commanders WHERE name=?1 ORDER BY lastseenepoch LIMIT 1",
                        [name],
                        |r| r.get::<_, i64>(0),
                    )
                    .optional()?
                    .unwrap_or(0);
                Ok(reply("GetFIDResponse", json!({"FID":value,"Name":name})))
            }
            "PIAddMerits" => {
                self.add_merits(payload)?;
                Ok(None)
            }
            "PIGetMerits" => Ok(reply(
                "PIGetMeritsResponse",
                json!({"Data":self.merit_history(fid(payload)?,integer(payload,"Power")?)?,"Processed":true,"Response":payload.get("Response").cloned().unwrap_or(Value::Null)}),
            )),
            "PIGetCycle" => Ok(reply(
                "PIGetCycleResponse",
                json!({"Data":self.merit_cycle(fid(payload)?,integer(payload,"Power")?,integer(payload,"Cycle")?)?,"Processed":true,"Response":payload.get("Response").cloned().unwrap_or(Value::Null)}),
            )),
            "GetCurrentMerits" => {
                let mut totals = self.current_merits(payload)?;
                if let Some(update) = payload.get("UpdateUI") {
                    totals["UpdateUI"] = update.clone();
                }
                Ok(reply("GetCurrentMeritsResponse", totals))
            }
            "WebServerRegisterPlugin" => {
                let name = payload
                    .get("Name")
                    .or_else(|| payload.get("Plugin"))
                    .and_then(Value::as_str)
                    .context("web registration needs a plugin name")?;
                let target = payload
                    .get("Target")
                    .or_else(|| payload.get("Guid"))
                    .cloned()
                    .unwrap_or(json!(sanitize_name(name)));
                self.register_plugin(name, target)?;
                Ok(None)
            }
            "WebSocketResponse" => {
                if let Some(server) = &self.server {
                    server.send(text(payload, "ClientID")?, text(payload, "Json")?)?;
                }
                Ok(None)
            }
            "WebSocketBroadcast" => {
                if let Some(server) = &self.server {
                    server.broadcast(text(payload, "Plugin")?, text(payload, "Json")?);
                }
                Ok(None)
            }
            "WebResponse" => {
                self.respond_web(
                    text(payload, "RequestID")?,
                    required(payload, "Response")?.clone(),
                )?;
                Ok(None)
            }
            _ => bail!("unknown Hecate command {command}"),
        }
    }
    pub fn evaluator_total(&self, fid: i64) -> Result<i64> {
        Ok(self.general.query_row(
            "SELECT COALESCE(SUM(value),0) FROM evaluator WHERE fid=?1",
            [fid],
            |r| r.get(0),
        )?)
    }
    pub fn evaluator_data(&self, fid: i64) -> Result<Vec<Value>> {
        let mut statement=self.general.prepare("SELECT COALESCE(s.name,'unknown'),COALESCE(s.dtepoch,0),SUM(e.value) FROM evaluator e LEFT JOIN evaluator_systems s ON e.fid=s.fid AND e.id64=s.id64 WHERE e.fid=?1 GROUP BY s.name,s.dtepoch ORDER BY s.dtepoch")?;
        let rows = statement
            .query_map([fid], |r| {
                Ok(json!([
                    r.get::<_, String>(0)?,
                    timestamp(r.get(1)?),
                    r.get::<_, i64>(2)?
                ]))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }
    pub fn bioinsights_data(&self, fid: i64) -> Result<Vec<Value>> {
        let mut statement=self.general.prepare("SELECT u.id,u.dtepoch,u.bodyname,u.species,u.basevalue,CASE WHEN b.bonus IS NULL OR b.bonus=1 THEN 1 ELSE 0 END FROM bioinsights_unsold u LEFT JOIN bioinsights_unsold_bonus b ON b.fid=u.fid AND b.id64=u.id64 AND b.bodyid=u.bodyid AND b.species=u.species WHERE u.fid=?1 ORDER BY u.dtepoch")?;
        let rows = statement
            .query_map([fid], |r| {
                Ok(json!([
                    r.get::<_, i64>(0)?,
                    timestamp(r.get(1)?),
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, i64>(4)?,
                    r.get::<_, i64>(5)? == 1
                ]))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }
    pub fn bioinsights_total(&self, fid: i64) -> Result<i64> {
        Ok(self
            .bioinsights_data(fid)?
            .iter()
            .map(|row| {
                row[4].as_i64().unwrap_or(0) * if row[5].as_bool() == Some(true) { 5 } else { 1 }
            })
            .sum())
    }
    fn add_merits(&self, p: &Value) -> Result<()> {
        require_keys(
            p,
            &[
                "System",
                "TargetSystem",
                "FID",
                "Power",
                "Cycle",
                "Session",
                "Activity",
                "Gained",
                "Total",
                "DateTime",
                "Dupe",
            ],
        )?;
        self.general.execute("INSERT INTO pp_merits_000(system,targetsystem,fid,power,cycle,session,activity,meritsgained,totalmerits,flags,dtepoch) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",params![id(p,"System")? as i64,id(p,"TargetSystem")? as i64,fid(p)?,integer(p,"Power")?,integer(p,"Cycle")?,integer(p,"Session")?,integer(p,"Activity")?,integer(p,"Gained")?,integer(p,"Total")?,if p["Dupe"].as_bool()==Some(true){1}else{0},epoch(&date(p,"DateTime")?)?])?;
        Ok(())
    }
    fn merit_history(&self, fid: i64, power: i64) -> Result<Vec<Value>> {
        let mut statement=self.general.prepare("SELECT COALESCE(s.name,'unknown'),COALESCE(t.name,'unknown'),p.cycle,p.session,p.activity,p.meritsgained,p.totalmerits,p.flags,p.dtepoch FROM pp_merits_000 p LEFT JOIN syslookup_000 s ON s.id64=p.system LEFT JOIN syslookup_000 t ON t.id64=p.targetsystem WHERE p.fid=?1 AND p.power=?2 ORDER BY p.dtepoch,p.id")?;
        let rows=statement.query_map(params![fid,power],|r|Ok(json!({"System":r.get::<_,String>(0)?,"TargetSystem":r.get::<_,String>(1)?,"Cycle":r.get::<_,i64>(2)?,"Session":r.get::<_,i64>(3)?,"Activity":r.get::<_,i64>(4)?,"MeritsGained":r.get::<_,i64>(5)?,"TotalMerits":r.get::<_,i64>(6)?,"Flags":r.get::<_,i64>(7)?,"DateTime":timestamp(r.get(8)?)})))?.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }
    fn merit_cycle(&self, fid: i64, power: i64, cycle: i64) -> Result<Vec<Value>> {
        let mut statement=self.general.prepare("SELECT COALESCE(s.name,'unknown'),SUM(p.meritsgained) FROM pp_merits_000 p LEFT JOIN syslookup_000 s ON s.id64=p.targetsystem WHERE p.fid=?1 AND p.power=?2 AND p.cycle=?3 AND (p.flags&1)=0 GROUP BY COALESCE(s.name,'unknown')")?;
        let rows = statement
            .query_map(params![fid, power, cycle], |r| {
                Ok(json!([r.get::<_, String>(0)?, r.get::<_, i64>(1)?]))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }
    fn current_merits(&self, p: &Value) -> Result<Value> {
        let (fid, power, cycle, session, since, system) = (
            fid(p)?,
            integer(p, "Power")?,
            integer(p, "Cycle")?,
            integer(p, "Session")?,
            integer(p, "Since")?,
            id(p, "System")? as i64,
        );
        let sum = |cycle: Option<i64>,
                   session: Option<i64>,
                   since: Option<i64>,
                   system: Option<i64>|
         -> Result<i64> {
            Ok(self.general.query_row("SELECT COALESCE(SUM(meritsgained),0) FROM pp_merits_000 WHERE fid=?1 AND power=?2 AND (flags&1)=0 AND (?3 IS NULL OR cycle=?3) AND (?4 IS NULL OR session=?4) AND (?5 IS NULL OR dtepoch>=?5) AND (?6 IS NULL OR targetsystem=?6)",params![fid,power,cycle,session,since,system],|r|r.get(0))?)
        };
        Ok(
            json!({"TotalCycleMerits":sum(Some(cycle),None,None,None)?,"TotalDailyMerits":sum(None,None,Some(since),None)?,"TotalSessionMerits":sum(None,Some(session),None,None)?,"SystemCycleMerits":sum(Some(cycle),None,None,Some(system))?,"SystemDailyMerits":sum(None,None,Some(since),Some(system))?,"SystemSessionMerits":sum(None,Some(session),None,Some(system))?}),
        )
    }
    fn queue_cache(&mut self, kind: &'static str, id: u64, reply: Option<Value>) -> Result<()> {
        if id == 0 || self.settings["RemoteLookups"].as_bool() == Some(false) {
            return Ok(());
        }
        if let Some(pending) = self
            .pending
            .values_mut()
            .find(|p| p.kind == kind && p.id == id)
        {
            if let Some(reply) = reply {
                pending.replies.push(reply);
            }
            return Ok(());
        }
        let store = if kind == "spansh" {
            &self.spansh
        } else {
            &self.canonn
        };
        if reply.is_none() {
            if let Some((_, date)) = store.latest(id)? {
                if chrono::Utc::now().timestamp() - epoch(&date)? <= 86_400 {
                    return Ok(());
                }
            }
            // Source SpanshTimer and CanonnTimer release independent one-second
            // automatic lookup cooldowns. Explicit request/reply lookups still run.
            if self
                .automatic_lookup_started
                .get(kind)
                .is_some_and(|started| started.elapsed() < Duration::from_secs(1))
            {
                return Ok(());
            }
            self.automatic_lookup_started.insert(kind, Instant::now());
        }
        self.request_counter = self.request_counter.wrapping_add(1);
        let request_id = format!("hecate:{kind}:{id}:{}", self.request_counter);
        let url = if kind == "spansh" {
            format!("https://spansh.co.uk/api/dump/{id}")
        } else {
            format!(
                "https://us-central1-canonn-api-236217.cloudfunctions.net/query/codex/bodies/{id}"
            )
        };
        self.pending.insert(
            request_id.clone(),
            PendingRequest {
                kind,
                id,
                replies: reply.into_iter().collect(),
            },
        );
        self.http_requests
            .push(json!({"request_id":request_id,"url":url,"method":"GET","timeout_ms":10000}));
        Ok(())
    }
    pub fn handle_http_response(&mut self, response: &Value) -> Result<()> {
        let request = response["request_id"]
            .as_str()
            .context("HTTP response missing request_id")?;
        let Some(pending) = self.pending.remove(request) else {
            return Ok(());
        };
        if response["ok"].as_bool() != Some(true) {
            self.errors
                .push(format!("Hecate {} lookup failed", pending.kind));
            return Ok(());
        }
        let body = response["body"]
            .as_str()
            .context("Hecate response has no text body")?;
        let value: Value = serde_json::from_str(body).context("invalid Hecate API JSON")?;
        let store = if pending.kind == "spansh" {
            &self.spansh
        } else {
            &self.canonn
        };
        store.add(pending.id, 0, body, &now(), true)?;
        for reply in pending.replies {
            self.messages.push(json!({"Command":"SpanshSystemResponse","Payload":{"ID64":pending.id,"System":value.get("system").unwrap_or(&value),"JournalEvents":from_spansh(&value,None)?,"Response":reply.get("Response").cloned().unwrap_or(Value::Null)}}));
        }
        Ok(())
    }
}
impl Drop for Hecate {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

fn merge(destination: &mut Value, source: &Value) {
    if let (Some(destination), Some(source)) = (destination.as_object_mut(), source.as_object()) {
        for (key, value) in source {
            if value.is_object() && destination.get(key).is_some_and(Value::is_object) {
                merge(destination.get_mut(key).unwrap(), value);
            } else {
                destination.insert(key.clone(), value.clone());
            }
        }
    }
}
fn reply(command: &str, payload: Value) -> Option<Value> {
    Some(json!({"Command":command,"Payload":payload}))
}
fn required<'a>(p: &'a Value, key: &str) -> Result<&'a Value> {
    p.get(key)
        .with_context(|| format!("Hecate message missing {key}"))
}
fn require_keys(p: &Value, keys: &[&str]) -> Result<()> {
    for key in keys {
        required(p, key)?;
    }
    Ok(())
}
fn integer(p: &Value, key: &str) -> Result<i64> {
    required(p, key)?
        .as_i64()
        .with_context(|| format!("{key} must be an integer"))
}
fn unsigned(value: &Value) -> Result<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
        .context("ID64 must be an unsigned integer")
}
fn id(p: &Value, key: &str) -> Result<u64> {
    unsigned(required(p, key)?)
}
fn fid_value(value: &Value) -> Result<i64> {
    if let Some(fid) = value.as_i64() {
        return Ok(fid);
    }
    let text = value
        .as_str()
        .context("FID must be an integer or Frontier ID")?;
    Ok(text.trim_start_matches('F').parse::<i64>()?)
}
fn fid(p: &Value) -> Result<i64> {
    fid_value(required(p, "FID")?)
}
fn text<'a>(p: &'a Value, key: &str) -> Result<&'a str> {
    required(p, key)?
        .as_str()
        .with_context(|| format!("{key} must be text"))
}
fn date(p: &Value, key: &str) -> Result<String> {
    let value = required(p, key)?;
    if let Some(s) = value.as_str() {
        epoch(s)?;
        Ok(s.to_owned())
    } else if let Some(n) = value.as_i64() {
        Ok(timestamp(n))
    } else {
        bail!("{key} must be a date or Unix seconds")
    }
}
pub fn sanitize_name(name: &str) -> String {
    let value: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .flat_map(char::to_lowercase)
        .collect();
    if value.len() > 130 {
        value[..128].to_owned()
    } else {
        value
    }
}

#[cfg(test)]
mod tests;
