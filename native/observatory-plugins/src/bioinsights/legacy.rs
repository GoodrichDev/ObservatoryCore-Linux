//! Original per-commander Codex binary format and distance-cache migration.
//! BiologicalCodexEntry.makeKey/ReadFileVersion100 and DistanceNotifierJSON are
//! the reference; the Windows files are read by the host and never modified.
use super::*;
use anyhow::{ensure, Context};

pub fn decode_legacy_codex(bytes: &[u8]) -> Result<Value> {
    ensure!(
        bytes.len() >= 12,
        "BioInsights Codex file has a truncated header"
    );
    let u32_at = |offset| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
    ensure!(
        u32_at(0) == 1_112_100_681,
        "Unrecognized BioInsights Codex magic"
    );
    ensure!(u32_at(4) == 100, "Unsupported BioInsights Codex version");
    let count = u32_at(8) as usize;
    let required = count
        .checked_mul(16)
        .and_then(|n| n.checked_add(12))
        .context("BioInsights Codex entry count overflow")?;
    ensure!(
        required <= bytes.len(),
        "BioInsights Codex entries are truncated"
    );
    ensure!(
        count <= 1_000_000,
        "BioInsights Codex entry count exceeds limit"
    );
    let mut rows = BTreeMap::new();
    for row in bytes[12..required].as_chunks::<16>().0 {
        let key = u64::from_le_bytes(row[..8].try_into().unwrap());
        let regions = u64::from_le_bytes(row[8..].try_into().unwrap());
        rows.insert(key, regions); // Source ReadFileVersion100 replaces duplicates.
    }
    Ok(json!(rows
        .into_iter()
        .map(|(key, regions)| json!({"Key":key,"Regions":regions}))
        .collect::<Vec<_>>()))
}

pub(super) fn valid_fid(fid: &str) -> bool {
    let digits = fid.strip_prefix('F').unwrap_or(fid);
    !digits.is_empty() && digits.len() <= 20 && digits.bytes().all(|b| b.is_ascii_digit())
}
fn entries(value: &Value) -> Result<Vec<(String, String, u64)>> {
    let mut entries = Vec::new();
    for row in value
        .as_array()
        .context("Legacy Codex entries must be an array")?
    {
        let key = row["Key"]
            .as_u64()
            .context("Legacy Codex Key must be unsigned")?;
        let mask = row["Regions"]
            .as_u64()
            .context("Legacy Codex Regions must be unsigned")?;
        let genus = key >> 32;
        let species = (key >> 16) & 0xffff;
        let colour = (key & 0xffff) as usize;
        let pair = rules::codex().genus.iter().find_map(|(g, data)| {
            (genus_number(g) as u64 == genus)
                .then(|| data.species.keys().find(|s| species_number(s) == species))
                .flatten()
        });
        if let (Some(species), Some(colour)) = (pair, rules::COLOURS.get(colour)) {
            entries.push((species.clone(), (*colour).to_owned(), mask));
        }
    }
    Ok(entries)
}
fn distance_cache(value: &Value) -> Result<Value> {
    if value.is_null() {
        return Ok(Value::Null);
    }
    ensure!(
        value.is_object(),
        "DistanceNotifier cache must be an object"
    );
    let id = value["SystemID64"]
        .as_u64()
        .context("DistanceNotifier SystemID64 must be unsigned")?;
    let minimum = value["MinimumDistance"]
        .as_u64()
        .context("DistanceNotifier minimum distance must be unsigned")?;
    let radius = value["RadiusKM"]
        .as_f64()
        .filter(|n| n.is_finite() && *n >= 0.0)
        .context("DistanceNotifier radius must be nonnegative")?;
    let points = value["ScanPosition"]
        .as_array()
        .context("DistanceNotifier scan positions must be an array")?;
    ensure!(
        points.len() <= 1024,
        "DistanceNotifier scan position count exceeds limit"
    );
    for p in points {
        ensure!(
            p["Latitude"]
                .as_f64()
                .is_some_and(|n| n.is_finite() && n.abs() <= 90.0)
                && p["Longitude"]
                    .as_f64()
                    .is_some_and(|n| n.is_finite() && n.abs() <= 180.0),
            "DistanceNotifier position is invalid"
        );
    }
    Ok(json!({"SystemID64":id,"MinimumDistance":minimum,"RadiusKM":radius,"ScanPosition":points}))
}
impl BioInsights {
    pub(super) fn commander_snapshot(&self) -> Value {
        let mut state = self
            .commander_states
            .get(&self.commander)
            .cloned()
            .unwrap_or_else(|| json!({}));
        state["PersonalCodex"] = json!(self.personal_codex);
        state["MarkedVariants"] = json!(self.marked_variants);
        state["DistanceNotifier"] = self.distance_cache.clone();
        state
    }
    pub(super) fn store_commander(&mut self) {
        if !self.commander.is_empty() {
            self.commander_states
                .insert(self.commander.clone(), self.commander_snapshot());
        }
    }
    pub(super) fn restore_commander(&mut self, state: &Value) {
        self.personal_codex =
            serde_json::from_value(state["PersonalCodex"].clone()).unwrap_or_default();
        self.marked_variants =
            serde_json::from_value(state["MarkedVariants"].clone()).unwrap_or_default();
        self.distance_cache = state
            .get("DistanceNotifier")
            .cloned()
            .unwrap_or(Value::Null);
        self.distance_notified = false;
    }
    pub(super) fn activate_commander(&mut self, fid: &str) {
        if fid == self.commander {
            return;
        }
        self.store_commander();
        let state = self.commander_states.get(fid).cloned();
        if let Some(state) = state {
            self.restore_commander(&state);
        } else if !self.commander_states.is_empty() {
            self.personal_codex.clear();
            self.marked_variants.clear();
            self.distance_cache = Value::Null;
        }
        self.commander = fid.to_owned();
        self.active_sample = None;
        self.distance_notified = false;
        self.announced.clear();
        self.vault_unsold.clear();
        self.vault_total = None;
    }
    /// Payload contains decoded CCSeen/BISeen/CCMark arrays and original
    /// dnotifier.json. Validate the whole import before replacing native state.
    pub fn restore_legacy_state(&mut self, payload: &Value) -> Result<usize> {
        let fid = payload["FID"]
            .as_str()
            .context("Legacy BioInsights FID is required")?;
        ensure!(valid_fid(fid), "Legacy BioInsights FID is invalid");
        let mut seen = BTreeSet::new();
        let mut marked = BTreeSet::new();
        for kind in ["CCSeen", "BISeen"] {
            if let Some(rows) = payload.get(kind) {
                for (species, colour, mask) in entries(rows)? {
                    for region in 1..=42 {
                        if mask & (1u64 << region) != 0 {
                            seen.insert((region, species.clone(), colour.clone()));
                        }
                    }
                }
            }
        }
        if let Some(rows) = payload.get("CCMark") {
            for (species, colour, mask) in entries(rows)? {
                if mask != 0 {
                    marked.insert((species, colour));
                }
            }
        }
        let cache = distance_cache(payload.get("DistanceNotifier").unwrap_or(&Value::Null))?;
        let mut state = self
            .commander_states
            .get(fid)
            .cloned()
            .unwrap_or_else(|| json!({}));
        let mut previous_seen: BTreeSet<(u64, String, String)> =
            serde_json::from_value(state["PersonalCodex"].clone()).unwrap_or_default();
        let mut previous_marks: BTreeSet<(String, String)> =
            serde_json::from_value(state["MarkedVariants"].clone()).unwrap_or_default();
        let count = seen.len() + marked.len();
        previous_seen.extend(seen);
        previous_marks.extend(marked);
        state["PersonalCodex"] = json!(previous_seen);
        state["MarkedVariants"] = json!(previous_marks);
        state["DistanceNotifier"] = cache;
        // Keep unmapped future IDs and original bytes' logical entries intact.
        state["LegacySource"] = payload.clone();
        self.commander_states.insert(fid.to_owned(), state.clone());
        if self.commander == fid {
            self.restore_commander(&state);
            self.resume_distance_cache();
        }
        Ok(count)
    }
    pub(super) fn cache_distance_sample(&mut self) {
        let Some((system, body, species)) = &self.active_sample else {
            self.distance_cache = Value::Null;
            return;
        };
        let Some(observed) = self
            .systems
            .get(system)
            .and_then(|s| s.bodies.get(body))
            .and_then(|b| b.observed.get(species))
        else {
            return;
        };
        let minimum = rules::species_key(species)
            .map(|(g, _)| rules::codex().genus[g].minimum_sample_distance)
            .unwrap_or(0);
        self.distance_cache = json!({"SystemID64":system,"BodyID":body,"SpeciesID":species,"MinimumDistance":minimum,"RadiusKM":self.radius,"ScanPosition":observed.samples.iter().map(|p|json!({"Latitude":p[0],"Longitude":p[1]})).collect::<Vec<_>>()});
    }
    pub(super) fn resume_distance_cache(&mut self) {
        if self.batch || self.distance_cache["SystemID64"].as_u64() != Some(self.current_system) {
            return;
        }
        let points: Vec<[f64; 2]> = self.distance_cache["ScanPosition"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|p| Some([p["Latitude"].as_f64()?, p["Longitude"].as_f64()?]))
            .collect();
        if points.is_empty() || points.len() > 2 {
            return;
        }
        self.distance_notified = false;
        if let Some(radius) = self.distance_cache["RadiusKM"]
            .as_f64()
            .filter(|r| *r > 0.0)
        {
            self.radius = radius;
        }
        if let Some((system, body, species)) = self.active_sample.clone() {
            if let Some(observed) = self
                .systems
                .get_mut(&system)
                .and_then(|s| s.bodies.get_mut(&body))
                .and_then(|b| b.observed.get_mut(&species))
            {
                observed.samples = points;
                observed.sample_count = observed.samples.len() as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;
    fn event(value: Value) -> JournalEvent {
        JournalEvent::parse(&value.to_string()).unwrap()
    }
    fn oracle() -> Value {
        serde_json::from_str(include_str!(
            "../../../fixtures/bioinsights-state-oracle.json"
        ))
        .unwrap()
    }
    #[test]
    fn settings_match_all_original_constructor_values() {
        let source: Value = serde_json::from_str(include_str!(
            "../../../fixtures/bioinsights-settings-oracle.json"
        ))
        .unwrap();
        let worker = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        for (key, expected) in source.as_object().unwrap() {
            assert_eq!(worker.settings[key], *expected, "original setting {key}");
        }
        assert_eq!(source.as_object().unwrap().len(), 55);
    }
    #[test]
    fn binary_reader_matches_original_csharp_keys_masks_and_duplicate_rule() {
        let original = oracle();
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(original["BytesBase64"].as_str().unwrap())
            .unwrap();
        let mut expected = original["Entries"].as_array().unwrap().clone();
        expected.sort_by_key(|row| row["Key"].as_u64().unwrap());
        assert_eq!(decode_legacy_codex(&bytes).unwrap(), json!(expected));
        assert_eq!(expected.last().unwrap()["Key"], u64::MAX);
        for length in [0, 4, 8, 11, bytes.len() - 1] {
            assert!(decode_legacy_codex(&bytes[..length]).is_err());
        }
        let mut bad = bytes.clone();
        bad[0] = 0;
        assert!(decode_legacy_codex(&bad).is_err());
        let mut bad = bytes;
        bad[4] = 101;
        assert!(decode_legacy_codex(&bad).is_err());
    }
    #[test]
    fn imported_codex_and_marks_are_commander_scoped_and_roundtrip() {
        let mut worker = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        let rows = oracle()["Entries"].clone();
        assert!(
            worker
                .restore_legacy_state(&json!({"FID":"F123","CCSeen":rows,"CCMark":rows}))
                .unwrap()
                > 0
        );
        worker.on_event(&event(
            json!({"event":"LoadGame","FID":"F123","Commander":"First"}),
        ));
        assert!(worker.personal_codex.contains(&(
            18,
            "Codex_Ent_Bacterial_01_Name".into(),
            "Green".into()
        )));
        assert!(worker.personal_codex.contains(&(
            42,
            "Codex_Ent_Bacterial_01_Name".into(),
            "Green".into()
        )));
        assert!(worker.is_marked("Codex_Ent_Bacterial_01_Name", "Green"));
        worker.on_event(&event(
            json!({"event":"LoadGame","FID":"F456","Commander":"Second"}),
        ));
        assert!(worker.personal_codex.is_empty());
        assert!(worker.marked_variants.is_empty());
        worker.on_event(&event(
            json!({"event":"LoadGame","FID":"F123","Commander":"First"}),
        ));
        let saved = worker.state();
        assert_eq!(saved["version"], 2);
        let mut restored = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        restored.restore_state(&saved);
        assert_eq!(restored.personal_codex, worker.personal_codex);
        assert_eq!(restored.marked_variants, worker.marked_variants);
        assert_eq!(
            restored.state()["Commanders"]["F123"]["LegacySource"]["CCSeen"],
            rows,
            "unknown future keys retained losslessly"
        );
        let before = worker.state();
        assert!(worker
            .restore_legacy_state(&json!({"FID":"../invalid","CCSeen":[]}))
            .is_err());
        assert_eq!(worker.state(), before);
    }
    #[test]
    fn original_distance_cache_resumes_once_after_preread_and_rejects_other_system() {
        let mut worker = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        worker.restore_legacy_state(&json!({"FID":"F123","DistanceNotifier":{"SystemID64":42,"MinimumDistance":500,"RadiusKM":1_000_000.0,"ScanPosition":[{"Latitude":0.0,"Longitude":0.0}]}})).unwrap();
        worker.set_batch(true);
        worker.on_event(&event(
            json!({"event":"LoadGame","FID":"F123","Commander":"First"}),
        ));
        worker.on_event(&event(
            json!({"event":"Location","SystemAddress":42,"StarSystem":"Test"}),
        ));
        worker.set_batch(false);
        assert!(worker
            .on_status(&json!({"Latitude":0.0,"Longitude":0.001,"PlanetRadius":1_000_000.0}))
            .is_empty());
        let notification =
            worker.on_status(&json!({"Latitude":0.0,"Longitude":0.04,"PlanetRadius":1_000_000.0}));
        assert_eq!(notification.len(), 1);
        assert_eq!(notification[0].title, "Minimum Distance Reached");
        assert!(worker
            .on_status(&json!({"Latitude":0.0,"Longitude":0.05}))
            .is_empty());
        worker.current_system = 43;
        worker.distance_notified = false;
        assert!(worker
            .on_status(&json!({"Latitude":0.0,"Longitude":0.06}))
            .is_empty());
    }
    #[test]
    fn settings_edits_preserve_history_commander_marks_and_partial_samples() {
        let mut worker = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        worker.on_event(&event(
            json!({"event":"LoadGame","FID":"F123","Commander":"First"}),
        ));
        worker.on_event(&event(json!({"event":"Scan","SystemAddress":42,"BodyID":1,"BodyName":"Test A 1","PlanetClass":"Rocky body"})));
        worker.on_status(&json!({"Latitude":0.0,"Longitude":0.0,"PlanetRadius":1_000_000.0}));
        worker.on_event(&event(json!({"event":"ScanOrganic","SystemAddress":42,"BodyID":1,"Genus":"$Codex_Ent_Stratum_Genus_Name;","Species":"$Codex_Ent_Stratum_07_Name;","ScanType":"Log"})));
        worker.on_message(
            "MarkVariant",
            &json!({"SpeciesID":"Codex_Ent_Stratum_07_Name","Variant":"Emerald","Marked":true}),
        );
        let state = worker.state();
        let sample = worker.active_sample.clone();
        let points = worker.systems[&42].bodies[&1].observed["Codex_Ent_Stratum_07_Name"]
            .samples
            .clone();
        let mut preferences = worker.settings.clone();
        preferences["ShowHeaders"] = json!(false);
        worker.set_settings(preferences).unwrap();
        assert_eq!(worker.state(), state);
        assert_eq!(worker.active_sample, sample);
        assert_eq!(
            worker.systems[&42].bodies[&1].observed["Codex_Ent_Stratum_07_Name"].samples,
            points
        );
        assert!(!worker.setting("ShowHeaders"));
        assert!(worker.set_settings(json!(3)).is_err());
        assert_eq!(worker.state(), state);
    }
    #[test]
    fn debug_body_and_verbose_logs_are_local_bounded_and_never_upload_logs() {
        let mut worker =
            BioInsights::new(json!({"GalacticCodexRemote":false,"DebugBody":"TEST A 1"})).unwrap();
        worker.on_event(&event(json!({"event":"Scan","SystemAddress":42,"BodyID":1,"BodyName":"Test A 1","PlanetClass":"Rocky body"})));
        let diagnostics = worker.take_diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0]["Level"], "Debug");
        assert!(diagnostics[0]["Predictions"].is_array());
        assert!(worker.take_diagnostics().is_empty());
        let mut preferences = worker.settings.clone();
        preferences["VerboseLogging"] = json!(true);
        worker.set_settings(preferences).unwrap();
        for _ in 0..300 {
            worker.on_event(&event(json!({"event":"Music","MusicTrack":"Test"})));
        }
        assert_eq!(worker.take_diagnostics().len(), 256);
        assert!(worker.http_requests.is_empty());
    }
}
