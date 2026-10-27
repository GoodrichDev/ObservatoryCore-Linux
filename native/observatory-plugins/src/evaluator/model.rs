use super::pricing::{body_k, planet_value, star_value};
use observatory_domain::JournalEvent;
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub fn flag(settings: &Value, key: &str) -> bool {
    settings[key].as_bool().unwrap_or(false)
}
pub fn number(settings: &Value, key: &str) -> f64 {
    settings[key].as_f64().unwrap_or(0.0)
}

#[derive(Debug, Clone, Default)]
pub struct Extra {
    pub bio_count: Option<u64>,
    pub geo_count: Option<u64>,
    pub high_value: bool,
    pub is_high_value: bool,
    pub personal: bool,
    pub galactic: bool,
    pub marked: bool,
    pub worth_visiting: bool,
    pub visited: bool,
}

#[derive(Debug, Clone)]
pub struct Body {
    pub id: u64,
    pub name: String,
    pub kind: String,
    pub star: bool,
    pub discovered: bool,
    pub was_mapped: bool,
    pub footfalled: bool,
    pub mapped: bool,
    pub efficient: bool,
    pub landable: bool,
    pub terraform: bool,
    pub atmospheric: bool,
    pub odyssey: bool,
    pub mass: f64,
    pub distance: f64,
    pub gravity: f64,
    pub volcanism: String,
    pub extra: Extra,
    pub override_mapping: Option<bool>,
    pub order: usize,
    pub orbit: Value,
}

impl Body {
    pub fn from_scan(e: &JournalEvent, odyssey: bool, extra: Extra) -> Option<Self> {
        let star = !e.str("StarType").is_empty();
        if !star && e.str("PlanetClass").is_empty() {
            return None;
        }
        let mut extra = extra;
        if extra.geo_count.is_none()
            && e.str("ScanType") == "Detailed"
            && e.str("Volcanism").is_empty()
        {
            extra.geo_count = Some(0);
        }
        Some(Self {
            id: e.id("BodyID")?,
            name: e.str("BodyName").into(),
            kind: e.str(if star { "StarType" } else { "PlanetClass" }).into(),
            star,
            discovered: e.boolean("WasDiscovered"),
            was_mapped: e.boolean("WasMapped"),
            footfalled: e.boolean("WasFootfalled"),
            mapped: false,
            efficient: false,
            landable: e.boolean("Landable"),
            terraform: !star && e.str("TerraformState").starts_with("Terraform"),
            atmospheric: odyssey
                && !star
                && e.boolean("Landable")
                && !e.str("AtmosphereType").is_empty()
                && e.str("AtmosphereType") != "None"
                && e.number("SurfacePressure") > 0.0,
            odyssey,
            mass: (e.number(if star { "StellarMass" } else { "MassEM" }) as f32) as f64,
            distance: e.number("DistanceFromArrivalLS"),
            gravity: if e.boolean("Landable") {
                (e.number("SurfaceGravity") as f32) as f64 / 9.80665
            } else {
                0.0
            },
            volcanism: e.str("Volcanism").into(),
            extra,
            override_mapping: None,
            order: 0,
            orbit: e.data.clone(),
        })
    }
    fn k(&self) -> f64 {
        body_k(&self.kind, self.star, false)
            + if self.terraform {
                body_k(&self.kind, false, true)
            } else {
                0.0
            }
    }
    pub fn current(&self) -> i64 {
        if self.star {
            star_value(self.k(), self.mass, !self.discovered, false)
        } else {
            planet_value(
                self.k(),
                self.mass,
                !self.discovered,
                self.mapped,
                !self.was_mapped && self.mapped,
                self.efficient,
                self.odyssey,
                false,
            )
        }
    }
    pub fn potential(&self) -> i64 {
        if self.star {
            self.current()
        } else {
            planet_value(
                self.k(),
                self.mass,
                !self.discovered,
                true,
                !self.was_mapped,
                true,
                self.odyssey,
                false,
            )
        }
    }
    pub fn main_bonus(&self) -> i64 {
        if self.star {
            self.current() / 3
        } else {
            (planet_value(
                self.k(),
                self.mass,
                !self.discovered,
                false,
                false,
                false,
                false,
                false,
            ) / 3)
                .max(500)
        }
    }
    pub fn visited(&self) -> bool {
        self.mapped || self.extra.visited
    }
    pub fn worth_visiting(&self) -> bool {
        self.override_mapping.unwrap_or(self.extra.worth_visiting)
    }
    pub fn worth(&self, settings: &Value) -> bool {
        if self.star {
            return false;
        }
        if let Some(value) = self.override_mapping {
            return value;
        }
        let max = number(
            settings,
            match self.kind.as_str() {
                "Earthlike body" => "MaxDistanceELW",
                "Water world" => "MaxDistanceWW",
                "Ammonia world" => "MaxDistanceAW",
                _ => "MaxDistanceOther",
            },
        );
        let value = number(settings, "MinimumValueForWorthMapping");
        ((max == 0.0 || self.distance <= max) && (value == 0.0 || self.potential() as f64 >= value))
            || (flag(settings, "BIHighValueWorthMapping") && self.extra.high_value)
            || (flag(settings, "BIPersonalCodexWorthMapping") && self.extra.personal)
            || (flag(settings, "BIGalacticCodexWorthMapping") && self.extra.galactic)
            || (flag(settings, "BIMarkedWorthMapping") && self.extra.marked)
    }
    pub fn visible(&self, settings: &Value, main: Option<u64>) -> bool {
        let extra = &self.extra;
        self.potential() as f64 >= number(settings, "MinimumBodyValue")
            || self.override_mapping.is_some()
            || self.worth(settings)
            || self.worth_visiting()
            || (flag(settings, "AlwaysShowMain") && Some(self.id) == main)
            || (flag(settings, "AlwaysShowELW") && self.kind == "Earthlike body")
            || (flag(settings, "AlwaysShowWW") && self.kind == "Water world")
            || (flag(settings, "AlwaysShowTerraformables") && self.terraform)
            || (flag(settings, "AlwaysShowAtmospherics") && self.atmospheric)
            || (flag(settings, "AlwaysShowAtmosphericsBios")
                && self.atmospheric
                && extra.bio_count.is_some_and(|v| v > 0))
            || (flag(settings, "AlwaysShowNonAtmosphericsBios")
                && !self.atmospheric
                && extra.bio_count.is_some_and(|v| v > 0))
            || (flag(settings, "AlwaysShowBIHighValue") && extra.high_value)
            || (flag(settings, "AlwaysShowBIPersonalCodex") && extra.personal)
            || (flag(settings, "AlwaysShowBIGalacticCodex") && extra.galactic)
            || (flag(settings, "AlwaysShowBIMarked") && extra.marked)
    }
    pub fn row(&self, sys: &System, settings: &Value) -> Value {
        let short = self
            .name
            .strip_prefix(&sys.name)
            .unwrap_or(&self.name)
            .trim();
        json!({"System":sys.name,"ID64":sys.address,"VisitID":sys.visit,"BodyID":self.id,"BodyName":self.name,
            "Body":if short.is_empty() {"0"} else {short},"Type":self.kind,"Distance":self.distance,
            "Gravity":self.gravity,"Terraformable":self.terraform,"IsStar":self.star,"IsLandable":self.landable,
            "Atmospheric":self.atmospheric,"CurrentValue":self.current()+if Some(self.id)==sys.main {sys.main_bonus()} else {0},
            "PotentialMaxValue":self.potential()+if Some(self.id)==sys.main {sys.main_bonus()} else {0},
            "Map":self.worth(settings)||self.worth_visiting(),"Order":self.order,"Done":!self.star&&self.mapped,
            "IsNextToVisit":self.order==1,"Volcanism":self.volcanism,"WasDiscovered":self.discovered,
            "WasMapped":self.was_mapped,"WasFootfalled":self.footfalled,"IsMapped":self.mapped,
            "BioCount":self.extra.bio_count,"GeoCount":self.extra.geo_count,"BioInsightsHighValue":self.extra.high_value,
            "IsHighValue":self.extra.is_high_value,
            "PluginWorthVisiting":self.extra.worth_visiting,"BioInsightsPersonalCodex":self.extra.personal,"BioInsightsGalacticCodex":self.extra.galactic,"BioInsightsMarked":self.extra.marked})
    }
}

#[derive(Debug, Clone, Default)]
pub struct System {
    pub name: String,
    pub address: u64,
    pub visit: u64,
    pub date: String,
    pub total: usize,
    pub bodies: BTreeMap<u64, Body>,
    /// Source Dictionary order is the first scan insertion order (also resolves route ties).
    pub body_order: Vec<u64>,
    pub extras: BTreeMap<u64, Extra>,
    pub orbits: BTreeMap<u64, Value>,
    pub main: Option<u64>,
    pub active: bool,
    pub route: Vec<u64>,
    pub from: Option<u64>,
}
impl System {
    pub fn main_bonus(&self) -> i64 {
        let known: i64 = self
            .bodies
            .values()
            .filter(|b| Some(b.id) != self.main)
            .map(Body::main_bonus)
            .sum();
        let unknown = self.total.saturating_sub(self.bodies.len());
        known
            + self
                .main
                .and_then(|id| self.bodies.get(&id))
                .map_or(0, |b| {
                    (unknown as f64 * 500.0 * if b.discovered { 1.0 } else { 2.6 })
                        .round_ties_even() as i64
                })
    }
    pub fn scanned_bonus(&self) -> i64 {
        if self.total > 0 && self.bodies.len() == self.total {
            self.total as i64 * 1000
        } else {
            0
        }
    }
    pub fn mapped_bonus(&self) -> i64 {
        if self.total > 0
            && self.bodies.len() == self.total
            && self.bodies.values().filter(|b| !b.star).all(|b| b.mapped)
        {
            self.bodies.values().filter(|b| !b.star).count() as i64 * 10000
        } else {
            0
        }
    }
    pub fn current(&self) -> i64 {
        self.bodies.values().map(Body::current).sum::<i64>()
            + self.main_bonus()
            + self.scanned_bonus()
            + self.mapped_bonus()
    }
    pub fn potential(&self, s: &Value) -> i64 {
        self.bodies
            .values()
            .map(|b| {
                if b.worth(s) {
                    b.potential()
                } else {
                    b.current()
                }
            })
            .sum::<i64>()
            + self.main_bonus()
            + self.scanned_bonus()
            + self.mapped_bonus()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn scan(value: Value) -> JournalEvent {
        JournalEvent {
            raw_json: None,
            event: "Scan".into(),
            timestamp: String::new(),
            data: value,
        }
    }
    fn oracle() -> Value {
        serde_json::from_str(include_str!("../../../fixtures/evaluator-prices.json")).unwrap()
    }
    #[test]
    fn body_constructor_and_mapping_match_installed_source() {
        for expected in oracle()["models"].as_array().unwrap() {
            let e = scan(
                json!({"BodyID":1,"BodyName":"Oracle 1","PlanetClass":expected["kind"],"MassEM":1.07317,
                "Landable":true,"SurfaceGravity":2.12345,"SurfacePressure":0.12345,"AtmosphereType":"CarbonDioxide",
                "DistanceFromArrivalLS":999,"WasDiscovered":expected["discovered"],
                "TerraformState":if expected["terraform"]==true {"Terraformable"} else {""}}),
            );
            let mut body =
                Body::from_scan(&e, expected["odyssey"] == true, Extra::default()).unwrap();
            body.mapped = expected["mapped"] == true;
            body.efficient = body.mapped;
            assert_eq!(
                body.current(),
                expected["current"].as_i64().unwrap(),
                "current {expected}"
            );
            assert_eq!(
                body.potential(),
                expected["potential"].as_i64().unwrap(),
                "potential {expected}"
            );
            assert_eq!(
                body.main_bonus(),
                expected["main_bonus"].as_i64().unwrap(),
                "bonus {expected}"
            );
            assert_eq!(body.atmospheric, expected["atmospheric"] == true);
            assert!((body.gravity - expected["gravity"].as_f64().unwrap()).abs() < 1e-12);
            assert_eq!(
                body.worth(&super::super::default_settings()),
                expected["worth"] == true,
                "worth {expected}"
            );
        }
    }
    #[test]
    fn distance_threshold_and_bio_overrides_match_installed_source() {
        for case in oracle()["worth"].as_array().unwrap() {
            let body=Body::from_scan(&scan(json!({"BodyID":1,"BodyName":"Oracle 1","PlanetClass":case[0],"MassEM":1,"WasDiscovered":true,"DistanceFromArrivalLS":999})),true,Extra{high_value:true,..Extra::default()}).unwrap();
            let settings = json!({"MaxDistanceELW":case[1],"MaxDistanceWW":case[1],"MaxDistanceAW":case[1],"MaxDistanceOther":case[1],"MinimumValueForWorthMapping":case[2],"BIHighValueWorthMapping":case[3]});
            assert_eq!(body.worth(&settings), case[4] == true, "{case}");
        }
    }
    #[test]
    fn system_discovery_and_completion_bonuses_match_installed_source() {
        let mut sys = System {
            name: "Oracle".into(),
            main: Some(0),
            ..System::default()
        };
        let star = Body::from_scan(
            &scan(json!({"BodyID":0,"BodyName":"Oracle","StarType":"G","StellarMass":1})),
            true,
            Extra::default(),
        )
        .unwrap();
        sys.bodies.insert(0, star);
        let fixture = oracle();
        for (index, total) in [0, 1, 3, 2, 2].into_iter().enumerate() {
            sys.total = total;
            if index == 3 {
                let b=Body::from_scan(&scan(json!({"BodyID":1,"BodyName":"Oracle 1","PlanetClass":"Earthlike body","MassEM":1})),true,Extra::default()).unwrap();
                sys.bodies.insert(1, b);
            }
            if index == 4 {
                let body = sys.bodies.get_mut(&1).unwrap();
                body.mapped = true;
                body.efficient = true;
            }
            let actual = json!([
                total,
                sys.main_bonus(),
                sys.current(),
                sys.scanned_bonus(),
                sys.mapped_bonus()
            ]);
            assert_eq!(actual, fixture["bonuses"][index]);
        }
    }
}
