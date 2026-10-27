//! Public Galactic Codex data. The host executes queued GET requests on its
//! bounded HTTP workers; no journal or commander data is sent with this query.
use super::*;
use anyhow::{ensure, Context};
use base64::Engine as _;
use std::io::Read;

const MAX_FEED: usize = 16 * 1024 * 1024;
pub(super) fn request() -> Value {
    json!({"request_id":"bioinsights:galactic","url":"https://edjp.colacube.net/api/v2/codex/galactic","method":"GET","timeout_ms":10000})
}

#[derive(Default)]
pub(super) struct Galactic {
    pub data: Value,
    entries: BTreeMap<(String, String), u64>,
}
impl Galactic {
    pub fn parse(data: Value) -> Result<Self> {
        ensure!(
            data["Version"] == "1.0",
            "unsupported Galactic Codex feed version"
        );
        let seen = data["Seen"]
            .as_object()
            .context("Galactic Codex feed has no Seen object")?;
        let mut entries = BTreeMap::new();
        for (species, record) in seen {
            let Some((_, species)) = rules::species_key(species) else {
                continue;
            };
            let variants = record["Entry"]
                .as_object()
                .context("Galactic Codex species has no Entry object")?;
            for (colour, regions) in variants {
                let Some(colour) = rules::COLOURS
                    .iter()
                    .find(|c| c.eq_ignore_ascii_case(colour))
                else {
                    continue;
                };
                let regions = regions
                    .as_array()
                    .context("Galactic Codex regions must be an array")?;
                let mut bits = 0u64;
                for region in regions {
                    let region = region.as_u64().context("invalid Galactic Codex region")?;
                    ensure!(
                        (1..=42).contains(&region),
                        "Galactic Codex region outside game range"
                    );
                    bits |= 1 << region;
                }
                entries.insert((species.to_owned(), colour.to_string()), bits);
            }
        }
        Ok(Self { data, entries })
    }
    pub fn has_data(&self) -> bool {
        !self.entries.is_empty()
    }
    pub fn regions(&self, species: &str, colour: &str) -> u64 {
        *self
            .entries
            .get(&(species.to_owned(), colour.to_owned()))
            .unwrap_or(&0)
    }
    pub fn flags(&self, species: &str, colour: &str, current_regions: u64) -> (bool, bool) {
        if !self.has_data() {
            return (false, false);
        }
        let regions = self.regions(species, colour);
        (
            current_regions != 0 && regions & current_regions == 0,
            regions == 0,
        )
    }
}
impl BioInsights {
    pub fn take_http_requests(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.http_requests)
    }
    pub fn handle_http_response(&mut self, response: &Value) -> Result<()> {
        if response["request_id"] != "bioinsights:galactic" {
            return Ok(());
        }
        ensure!(
            response["ok"] == true,
            "Galactic Codex public lookup failed"
        );
        let bytes = if let Some(encoded) = response["body_base64"].as_str() {
            ensure!(
                encoded.len() <= MAX_FEED * 2,
                "Galactic Codex response too large"
            );
            base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .context("invalid Galactic Codex binary response")?
        } else {
            response["body"]
                .as_str()
                .context("Galactic Codex response has no body")?
                .as_bytes()
                .to_vec()
        };
        ensure!(bytes.len() <= MAX_FEED, "Galactic Codex response too large");
        let decoded = if bytes.starts_with(&[0x28, 0xb5, 0x2f, 0xfd]) {
            let decoder = zstd::stream::read::Decoder::new(bytes.as_slice())
                .context("invalid Galactic Codex Zstandard frame")?;
            let mut decoded = Vec::new();
            decoder
                .take((MAX_FEED + 1) as u64)
                .read_to_end(&mut decoded)
                .context("invalid Galactic Codex compressed data")?;
            ensure!(
                decoded.len() <= MAX_FEED,
                "Galactic Codex decompressed response too large"
            );
            decoded
        } else {
            bytes
        };
        let data: Value =
            serde_json::from_slice(&decoded).context("invalid Galactic Codex JSON")?;
        // Parse and validate a replacement before touching the last good cache.
        let galactic = Galactic::parse(data)?;
        self.galactic = galactic;
        *self.browser_cache.get_mut().expect("browser cache") = None;
        self.alert_cache.get_mut().expect("alert cache").clear();
        let bodies: Vec<_> = self
            .systems
            .iter()
            .flat_map(|(id, system)| {
                system
                    .bodies
                    .iter()
                    .filter(|(_, b)| b.bio_count.is_some_and(|n| n > 0) || !b.observed.is_empty())
                    .map(|(body, _)| (*id, *body))
            })
            .collect();
        for (system, body) in bodies {
            self.publish_body(system, body);
        }
        self.sync_overlay();
        Ok(())
    }
    pub(super) fn galactic_flags(&self, system: u64, predictions: &[Predicted]) -> (bool, bool) {
        let regions = self
            .systems
            .get(&system)
            .and_then(|s| s.position)
            .map(rules::regions_at)
            .unwrap_or(0);
        predictions.iter().filter(|p| p.region_matches).fold(
            (false, false),
            |(regional, global), p| {
                p.variants.iter().fold(
                    (regional, global),
                    |(regional, global), (colour, _, _, _)| {
                        let (r, g) = self.galactic.flags(&p.species, colour, regions);
                        (regional || r, global || g)
                    },
                )
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn payload() -> Value {
        json!({"Version":"1.0","Generated":"2026-10-26T00:00:00Z","Seen":{"$Codex_Ent_Stratum_07_Name;":{"Entry":{"Emerald":[18,23]}}}})
    }
    #[test]
    fn constructor_only_queues_public_get_and_configuration_disables_it() {
        let mut worker = BioInsights::new(json!({})).unwrap();
        let requests = worker.take_http_requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0]["method"], "GET");
        assert!(requests[0].get("body").is_none());
        assert!(BioInsights::new(json!({"GalacticCodexRemote":false}))
            .unwrap()
            .take_http_requests()
            .is_empty());
    }
    #[test]
    fn zstandard_feed_updates_flags_and_native_cache_without_losing_good_data() {
        let mut worker = BioInsights::new(json!({})).unwrap();
        let compressed = zstd::stream::encode_all(payload().to_string().as_bytes(), 1).unwrap();
        worker.handle_http_response(&json!({"request_id":"bioinsights:galactic","ok":true,"body_base64":base64::engine::general_purpose::STANDARD.encode(compressed)})).unwrap();
        assert_eq!(
            worker
                .galactic
                .flags("Codex_Ent_Stratum_07_Name", "Emerald", 1 << 18),
            (false, false)
        );
        assert_eq!(
            worker
                .galactic
                .flags("Codex_Ent_Stratum_07_Name", "Emerald", 1 << 1),
            (true, false)
        );
        assert_eq!(
            worker
                .galactic
                .flags("Codex_Ent_Stratum_07_Name", "Lime", 1 << 18),
            (true, true)
        );
        let good = worker.state();
        assert!(worker.handle_http_response(&json!({"request_id":"bioinsights:galactic","ok":true,"body":"{\"Version\":\"2.0\",\"Seen\":{}}"})).is_err());
        assert_eq!(worker.state(), good);
        let mut restored = BioInsights::new(json!({})).unwrap();
        restored.restore_state(&good);
        assert_eq!(
            restored
                .galactic
                .flags("Codex_Ent_Stratum_07_Name", "Emerald", 1 << 1),
            (true, false)
        );
    }
    #[test]
    fn plain_json_feed_is_accepted_and_failed_lookup_retains_cache() {
        let mut worker = BioInsights::new(json!({})).unwrap();
        worker.handle_http_response(&json!({"request_id":"bioinsights:galactic","ok":true,"body":payload().to_string()})).unwrap();
        let good = worker.state();
        assert!(worker
            .handle_http_response(&json!({"request_id":"bioinsights:galactic","ok":false}))
            .is_err());
        assert_eq!(worker.state(), good);
    }
    #[test]
    fn accepted_feed_refreshes_existing_evaluator_flags_in_both_directions() {
        let mut worker = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        for data in [
            json!({"event":"Location","SystemAddress":42,"StarSystem":"Test","StarPos":[0,0,0]}),
            json!({"event":"Scan","SystemAddress":42,"BodyID":0,"BodyName":"Test","StarType":"F","Radius":695700000,"SurfaceTemperature":6500}),
            json!({"event":"Scan","SystemAddress":42,"BodyID":1,"BodyName":"Test 1","PlanetClass":"High metal content body","Atmosphere":"thin carbon dioxide atmosphere","AtmosphereType":"CarbonDioxide","SurfaceGravity":1,"SurfaceTemperature":180,"SurfacePressure":100,"Parents":[{"Star":0}],"SemiMajorAxis":149597870700.0}),
            json!({"event":"FSSBodySignals","SystemAddress":42,"BodyID":1,"Signals":[{"Type":"$SAA_SignalType_Biological;","Count":1}]}),
        ] {
            worker.on_event(&JournalEvent::parse(&data.to_string()).unwrap());
        }
        worker.take_messages();
        worker.handle_http_response(&json!({"request_id":"bioinsights:galactic","ok":true,"body":json!({"Version":"1.0","Seen":{"$Codex_Ent_Stratum_07_Name;":{"Entry":{"Emerald":[23]}}}}).to_string()})).unwrap();
        assert!(worker
            .take_messages()
            .iter()
            .any(|m| m["command"] == "SetBioInsightsFlags" && m["payload"]["IsGCodex"] == true));
        let mut seen = serde_json::Map::new();
        for prediction in worker.body_predictions(1) {
            let mut entries = serde_json::Map::new();
            for variant in prediction["Variants"].as_array().unwrap() {
                entries.insert(variant["Colour"].as_str().unwrap().to_owned(), json!([18]));
            }
            seen.insert(
                prediction["SpeciesID"].as_str().unwrap().to_owned(),
                json!({"Entry":entries}),
            );
        }
        worker.handle_http_response(&json!({"request_id":"bioinsights:galactic","ok":true,"body":json!({"Version":"1.0","Seen":seen}).to_string()})).unwrap();
        assert!(worker
            .take_messages()
            .iter()
            .any(|m| m["command"] == "SetBioInsightsFlags" && m["payload"]["IsGCodex"] == false));
    }
}
