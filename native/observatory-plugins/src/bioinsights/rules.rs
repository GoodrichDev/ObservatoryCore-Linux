//! Data driven Codex predicates. The data and comparison boundaries come from
//! the locally installed CodexLib 1.3.444.738; see PROVENANCE.md.
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

const AU: f64 = 149_597_870_700.0;
const ATM: f64 = 101_325.0;
const GRAVITY: f64 = 9.798;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct Codex {
    pub genus: BTreeMap<String, Genus>,
    #[serde(rename = "EntryIDSpecies")]
    pub entry_id_species: BTreeMap<String, u64>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct Genus {
    pub species: BTreeMap<String, Species>,
    pub minimum_sample_distance: u32,
    pub is_horizon: bool,
    pub journal_name: String,
    pub requirements: Vec<Requirement>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct Species {
    pub variant: Vec<Variant>,
    pub variant_type: u8,
    pub base_value: u64,
    pub journal_name: String,
    pub requirements: Vec<Requirement>,
    pub post_requirements: Vec<Requirement>,
    pub star_requirements: Vec<Requirement>,
    pub regions: u64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct Requirement {
    pub value: Vec<Vec<Predicate>>,
}
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub(super) struct Predicate {
    pub r#type: u8,
    pub d_value: f64,
    pub i_value: u64,
    pub s_value: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct Variant {
    pub colour: u8,
    pub star: Option<u8>,
    pub material: Option<u8>,
}

pub(super) fn codex() -> &'static Codex {
    static CODEX: OnceLock<Codex> = OnceLock::new();
    CODEX.get_or_init(|| {
        serde_json::from_str(include_str!("codex.json")).expect("bundled Codex data")
    })
}
pub(super) fn name(key: &str) -> String {
    static NAMES: OnceLock<BTreeMap<String, String>> = OnceLock::new();
    NAMES
        .get_or_init(|| serde_json::from_str(include_str!("names.json")).expect("bundled names"))
        .get(key)
        .cloned()
        .unwrap_or_else(|| key.to_owned())
}
pub(super) fn token(s: &str) -> String {
    s.trim_matches(['$', ';'])
        .to_ascii_lowercase()
        .replace("clepeus", "clypeus")
}
pub(super) fn genus_key(s: &str) -> Option<&'static str> {
    let s = token(s);
    codex()
        .genus
        .iter()
        .find(|(key, genus)| {
            token(key) == s
                || token(&name(key)) == s
                || (!genus.journal_name.is_empty() && token(&genus.journal_name) == s)
        })
        .map(|(key, _)| key.as_str())
}
pub(super) fn species_key(s: &str) -> Option<(&'static str, &'static str)> {
    let s = token(s);
    codex().genus.iter().find_map(|(gkey, genus)| {
        genus
            .species
            .iter()
            .find(|(key, species)| {
                token(key) == s
                    || token(&format!("{} {}", name(gkey), name(key))) == s
                    || (!species.journal_name.is_empty()
                        && !genus.journal_name.is_empty()
                        && token(&format!(
                            "Codex_Ent_{}_{}_Name",
                            genus.journal_name, species.journal_name
                        )) == s)
            })
            .map(|(key, _)| (gkey.as_str(), key.as_str()))
    })
}
pub(super) const COLOURS: [&str; 26] = [
    "Unknown",
    "Amethyst",
    "Aquamarine",
    "Blue",
    "Cobalt",
    "Cyan",
    "Emerald",
    "Gold",
    "Green",
    "Grey",
    "Indigo",
    "Lime",
    "Magenta",
    "Maroon",
    "Mauve",
    "Mulberry",
    "Ocher",
    "Orange",
    "Peach",
    "Red",
    "Sage",
    "Teal",
    "Turquoise",
    "White",
    "Yellow",
    "Horizons",
];
const MATERIALS: [&str; 12] = [
    "Antimony",
    "Polonium",
    "Ruthenium",
    "Technetium",
    "Tellurium",
    "Yttrium",
    "Cadmium",
    "Mercury",
    "Molybdenum",
    "Niobium",
    "Tungsten",
    "Tin",
];
const ATMOSPHERES: [&str; 21] = [
    "Ammonia",
    "AmmoniaOxygen",
    "AmmoniaRich",
    "Argon",
    "ArgonRich",
    "CarbonDioxide",
    "CarbonDioxideRich",
    "EarthLike",
    "Helium",
    "MetallicVapour",
    "Methane",
    "MethaneRich",
    "Neon",
    "NeonRich",
    "Nitrogen",
    "None",
    "Oxygen",
    "SilicateVapour",
    "SulphurDioxide",
    "Water",
    "WaterRich",
];
const PLANETS: [&str; 20] = [
    "Unknown",
    "Ammonia world",
    "Earthlike body",
    "Gas giant with ammonia based life",
    "Gas giant with water based life",
    "Helium rich gas giant",
    "High metal content body",
    "Icy body",
    "Metal rich body",
    "Rocky body",
    "Rocky ice body",
    "Sudarsky class I gas giant",
    "Sudarsky class II gas giant",
    "Sudarsky class III gas giant",
    "Sudarsky class IV gas giant",
    "Sudarsky class V gas giant",
    "Water giant",
    "Water world",
    "Helium gas giant",
    "Water giant with life",
];
const COMPOSITION: [&str; 13] = [
    "Water",
    "Oxygen",
    "CarbonDioxide",
    "SulphurDioxide",
    "Ammonia",
    "Methane",
    "Nitrogen",
    "Hydrogen",
    "Helium",
    "Neon",
    "Argon",
    "Silicates",
    "Iron",
];
pub(super) fn compact(s: &str) -> String {
    s.chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect()
}
pub(super) fn planet_class(scan: &Value) -> u8 {
    let c = compact(text(scan, "PlanetClass"));
    PLANETS.iter().position(|v| compact(v) == c).unwrap_or(0) as u8
}
pub(super) fn atmosphere_type(scan: &Value) -> u8 {
    let explicit = text(scan, "AtmosphereType");
    let a = if explicit.is_empty() {
        text(scan, "Atmosphere")
            .trim_start_matches("thin ")
            .trim_start_matches("thick ")
            .trim_start_matches("hot ")
    } else {
        explicit
    };
    let a = compact(a.trim_end_matches(" atmosphere"));
    if a.is_empty() {
        return 15;
    }
    ATMOSPHERES
        .iter()
        .position(|s| compact(s) == a)
        .unwrap_or(15) as u8
}
pub(super) fn atmosphere_name(scan: &Value) -> &'static str {
    ATMOSPHERES[atmosphere_type(scan) as usize]
}
pub(super) fn star_type(s: &str) -> u8 {
    match s {
        "O" => 1,
        "B" => 2,
        "A" => 3,
        "F" => 4,
        "G" => 5,
        "K" => 6,
        "M" => 7,
        "L" => 8,
        "T" => 9,
        "TTS" => 10,
        "N" => 13,
        "Y" => 14,
        "AeBe" => 15,
        "H" => 16,
        "SupermassiveBlackHole" => 17,
        "M_RedSuperGiant" => 6,
        "M_RedGiant" => 7,
        "K_OrangeGiant" => 6,
        "F_WhiteSuperGiant" => 4,
        "G_WhiteSuperGiant" => 5,
        "A_BlueWhiteSuperGiant" => 3,
        "B_BlueWhiteSuperGiant" => 2,
        _ if s.starts_with('W') => 11,
        _ if s.starts_with('D') => 12,
        _ if ["C", "CN", "CJ", "CH", "CHd", "CS", "MS", "S"].contains(&s) => 18,
        _ => 0,
    }
}
pub(super) fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}
pub(super) fn number(v: &Value, key: &str) -> f64 {
    v.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}
fn single(v: &Value, key: &str) -> f64 {
    number(v, key) as f32 as f64
}

#[derive(Clone, Debug)]
pub(super) struct Predicted {
    pub genus: String,
    pub species: String,
    pub value: u64,
    pub distance: u32,
    pub horizons: bool,
    pub variants: Vec<(String, String, u8, bool)>,
    pub region_matches: bool,
    pub provisional: bool,
}

pub(super) struct Context<'a> {
    pub scan: &'a Value,
    pub bodies: &'a BTreeMap<u64, Value>,
    pub regions: u64,
    pub spheres: &'a BTreeSet<u8>,
    pub complete: bool,
}
impl Context<'_> {
    fn all_stars(&self) -> Vec<&Value> {
        self.bodies
            .values()
            .filter(|b| !text(b, "StarType").is_empty() && b["_Hint"] != true)
            .collect()
    }
    fn scanned_stars(&self) -> Vec<&Value> {
        self.all_stars()
            .into_iter()
            .filter(|s| s["_SystemOnly"] != true && s["_Hint"] != true)
            .collect()
    }
    fn hint_star(&self) -> Option<&Value> {
        self.bodies.values().find(|s| s["_Hint"] == true)
    }
    fn parent_stars(&self) -> Vec<&Value> {
        if self.good_stellar_context() {
            self.scanned_stars()
        } else {
            self.fallback_radiant().into_iter().collect()
        }
    }
    fn orbital_reference(&self) -> &Value {
        // EnrichStars inherits the outer planet's stellar environment for a
        // moon: its own local orbit does not become an extra star distance.
        self.scan
            .get("Parents")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|p| {
                p.get("Planet")
                    .and_then(Value::as_u64)
                    .and_then(|id| self.bodies.get(&id))
            })
            .next_back()
            .unwrap_or(self.scan)
    }
    fn orbital_path(body: &Value) -> Vec<u64> {
        body.get("BodyID")
            .and_then(Value::as_u64)
            .into_iter()
            .chain(
                body.get("Parents")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .flat_map(|p| p.as_object().into_iter().flatten())
                    .filter_map(|(_, id)| id.as_u64()),
            )
            .collect()
    }
    fn star_distance(&self, star: &Value) -> f64 {
        let reference = self.orbital_reference();
        let parents = reference.get("Parents").and_then(Value::as_array);
        let primary = parents
            .into_iter()
            .flatten()
            .find_map(|p| p.get("Star").and_then(Value::as_u64));
        if primary == star["BodyID"].as_u64() {
            return single(reference, "SemiMajorAxis");
        }
        // For a planet attached to one member of a multiple-star hierarchy,
        // source enrichment measures other stars from that parent star.
        let origin = primary
            .and_then(|id| self.bodies.get(&id))
            .unwrap_or(reference);
        let origin_path = Self::orbital_path(origin);
        let star_path = Self::orbital_path(star);
        if let Some(common) = origin_path.iter().find(|id| star_path.contains(id)) {
            let mut distance = 0f32;
            for path in [&origin_path, &star_path] {
                if let Some(id) = path.iter().take_while(|id| *id != common).last() {
                    distance += self
                        .bodies
                        .get(id)
                        .map(|b| single(b, "SemiMajorAxis") as f32)
                        .unwrap_or(0.0);
                }
            }
            return distance as f64;
        }
        // A source synthetic context or incomplete hierarchy has no common
        // parent yet. The known planet orbit remains its source estimate.
        single(reference, "SemiMajorAxis")
    }
    fn missing_stellar_parent(&self) -> bool {
        self.scan["Parents"].as_array().is_some_and(|parents| {
            parents.iter().any(|p| {
                p.as_object().is_some_and(|p| {
                    p.iter().any(|(kind, id)| {
                        let Some(id) = id.as_u64() else {
                            return true;
                        };
                        match kind.as_str() {
                            "Star" => !self.scanned_stars().iter().any(|s| s["BodyID"] == id),
                            "Planet" => !self.bodies.get(&id).is_some_and(|b| {
                                b["PlanetClass"].is_string() || b["_ContextPlanet"] == true
                            }),
                            "Null" => {
                                !self.bodies.get(&id).is_some_and(|b| {
                                    text(b, "StarType").is_empty() && !b["PlanetClass"].is_string()
                                }) && !(id == 0
                                    && self.scanned_stars().iter().any(|s| s["BodyID"] == 0))
                                    && !self.scanned_stars().iter().any(|s| {
                                        s["Parents"]
                                            .as_array()
                                            .is_some_and(|p| p.iter().any(|p| p["Null"] == id))
                                    })
                            }
                            _ => false,
                        }
                    })
                })
            })
        })
    }
    fn good_stellar_context(&self) -> bool {
        if self.scan["Parents"].as_array().is_none_or(|p| p.is_empty())
            || self.missing_stellar_parent()
        {
            return false;
        }
        let origin = self.scan["Parents"]
            .as_array()
            .unwrap()
            .iter()
            .find_map(|p| p["Star"].as_u64());
        // EnrichStars cannot establish distances from a detached root star:
        // findCommonParent returns no common root and falls back to the hint.
        !origin.is_some_and(|id| {
            id != 0
                && self
                    .bodies
                    .get(&id)
                    .is_some_and(|s| Self::orbital_path(s).len() == 1)
                && self.scanned_stars().len() > 1
        })
    }
    fn fallback_radiant(&self) -> Option<&Value> {
        let stars = self.scanned_stars();
        let hint = self.hint_star();
        let has_main = self.scan["Parents"]
            .as_array()
            .is_some_and(|p| p.iter().any(|p| p["Star"] == 0));
        if let Some(hint) = hint {
            if self.missing_stellar_parent()
                && has_main
                && stars
                    .iter()
                    .any(|s| star_type(text(s, "StarType")) == star_type(text(hint, "StarType")))
            {
                return stars.into_iter().find(|s| s["BodyID"] == 0);
            }
            return Some(hint);
        }
        if !self.missing_stellar_parent() {
            return None;
        }
        stars.into_iter().max_by(|a, b| {
            single(a, "SurfaceTemperature").total_cmp(&single(b, "SurfaceTemperature"))
        })
    }
    pub(super) fn radiant(&self) -> Option<&Value> {
        if !self.good_stellar_context() {
            return self.fallback_radiant();
        }
        // Source orders visual apparent magnitude; temperatures and radii feed
        // the separate black-body/star requirement checks.
        self.scanned_stars().into_iter().min_by(|a, b| {
            self.apparent_magnitude(a)
                .total_cmp(&self.apparent_magnitude(b))
        })
    }
    fn apparent_magnitude(&self, star: &Value) -> f64 {
        number(star, "AbsoluteMagnitude")
            + 5.0 * (self.star_distance(star).max(1.0) / 3.085_677_581_491_367e16).log10()
            - 5.0
    }
    fn relevant_parent_stars(&self) -> Vec<&Value> {
        let parents = self.parent_stars();
        let Some(radiant) = self.radiant() else {
            return parents;
        };
        let radiant_type = star_type(text(radiant, "StarType"));
        let other = parents
            .iter()
            .filter(|s| star_type(text(s, "StarType")) != radiant_type)
            .map(|s| self.apparent_magnitude(s))
            .min_by(f64::total_cmp)
            .unwrap_or(1_000_000.0);
        if other - self.apparent_magnitude(radiant) > 2.0 {
            vec![radiant]
        } else {
            parents
        }
    }
    fn nearest(&self) -> Option<f64> {
        self.all_stars()
            .into_iter()
            .map(|s| self.star_distance(s))
            .filter(|d| *d > 0.0)
            .min_by(f64::total_cmp)
    }
    fn requirements(&self, rs: &[Requirement], star: Option<&Value>) -> bool {
        rs.iter().all(|r| {
            r.value
                .iter()
                .any(|or| or.iter().all(|p| self.test(p, star)))
        })
    }
    fn test(&self, p: &Predicate, star: Option<&Value>) -> bool {
        let d = p.d_value;
        let i = p.i_value as u8;
        let s = p.s_value.as_str();
        let v = self.scan;
        match p.r#type {
            0 => match s {
                "THIN" => text(v, "Atmosphere").to_ascii_lowercase().contains("thin"),
                "ANY" => true,
                "NONE" => atmosphere_type(v) == 15,
                _ => false,
            },
            1 => atmosphere_type(v) == i,
            2 => {
                let chemical = COMPOSITION
                    .get(p.i_value.trailing_zeros() as usize)
                    .copied()
                    .unwrap_or("");
                v.get("AtmosphereComposition")
                    .and_then(Value::as_array)
                    .is_some_and(|a| {
                        a.iter().any(|c| {
                            compact(text(c, "Name")) == compact(chemical)
                                && single(c, "Percent") >= d
                        })
                    })
            }
            3 => self
                .parent_stars()
                .iter()
                .any(|star| star_type(text(star, "StarType")) == star_name(s)),
            4 => self
                .radiant()
                .is_some_and(|s| self.star_distance(s) > d * AU),
            5 => {
                let volcanism = text(v, "Volcanism").to_ascii_uppercase();
                match s {
                    "ANY" => !volcanism.is_empty(),
                    "NONE" => volcanism.is_empty(),
                    "MAJOR" => volcanism.starts_with("MAJOR "),
                    "MINOR" => volcanism.starts_with("MINOR "),
                    "REGULAR" => {
                        !volcanism.is_empty()
                            && !volcanism.starts_with("MAJOR ")
                            && !volcanism.starts_with("MINOR ")
                    }
                    "NOT_MAJOR" => !volcanism.is_empty() && !volcanism.starts_with("MAJOR "),
                    "NOT_MINOR" => !volcanism.is_empty() && !volcanism.starts_with("MINOR "),
                    "IRON_BASED" => volcanism.contains("METALLIC "),
                    "ROCKY" => volcanism.contains("ROCKY"),
                    _ => volcanism.contains(&s.replace('_', " ").replace(" BASED", "")),
                }
            }
            6 => has_material(v, s),
            7 => single(v, "SurfacePressure") / ATM >= d,
            8 => single(v, "SurfacePressure") / ATM < d,
            9 => single(v, "SurfaceGravity") / GRAVITY >= d,
            10 => single(v, "SurfaceGravity") / GRAVITY < d,
            11 => single(v, "SurfaceTemperature") >= d,
            12 => single(v, "SurfaceTemperature") < d,
            13 => single(v, "MassEM") >= d,
            14 => single(v, "MassEM") < d,
            15 => planet_class(v) == i,
            16 => self.spheres.contains(&i),
            17 => self
                .bodies
                .values()
                .any(|v| planet_class(v) == i && text(v, "StarType").is_empty()),
            18 => self
                .all_stars()
                .iter()
                .any(|v| star_type(text(v, "StarType")) == i),
            19 => single(v, "SemiMajorAxis") / AU <= d,
            20 | 22 => false, // Reserved, also unimplemented by this installed CodexLib.
            21 => number(v, "DistanceFromArrivalLS") >= d,
            27 => star
                .is_some_and(|s| text(s, "Luminosity").trim_end_matches(['a', 'b']) == p.s_value),
            28 => is_procedural(text(v, "BodyName")) == (p.i_value != 0),
            29 => star.is_some_and(|s| self.star_distance(s) <= d * AU),
            30 => star.is_some_and(|s| self.star_distance(s) >= d * AU),
            31 => self.nearest().is_some_and(|distance| distance <= d * AU),
            32 => self.nearest().is_some_and(|distance| distance >= d * AU),
            33 => atmosphere_type(v) != i,
            40 => star.is_some_and(|s| star_type(text(s, "StarType")) == i),
            41 | 42 => {
                let flux: f64 = self
                    .parent_stars()
                    .iter()
                    .map(|s| {
                        single(s, "SurfaceTemperature").powi(4)
                            * (single(s, "Radius") / self.star_distance(s).max(1.0)).powi(2)
                            / 4.0
                    })
                    .sum();
                let temperature = flux.powf(0.25);
                if p.r#type == 41 {
                    temperature >= d
                } else {
                    temperature <= d
                }
            }
            23..=26 | 34..=39 => star.is_some_and(|star| {
                let distance = self.star_distance(star).max(1.0);
                let radius = single(star, "Radius");
                let temperature = single(star, "SurfaceTemperature");
                let flux = 5.670_374_419e-8 * temperature.powi(4) * (radius / distance).powi(2);
                let absolute = number(star, "AbsoluteMagnitude");
                let apparent = absolute + 5.0 * (distance / 3.085_677_581_491_367e16).log10() - 5.0;
                let bolometric = -2.5 * flux.log10() - 18.997_351;
                let value = match p.r#type {
                    23 | 24 => flux,
                    25 | 26 => bolometric,
                    36 | 37 => absolute,
                    _ => apparent,
                };
                if matches!(p.r#type, 23 | 25 | 34 | 36 | 38) {
                    value >= d
                } else {
                    value <= d
                }
            }),
            43 => p.i_value < 64 && self.regions & (1 << p.i_value) != 0,
            _ => false,
        }
    }
    pub fn predict(&self, include_horizons: bool) -> Vec<Predicted> {
        let mut result = Vec::new();
        for (gkey, genus) in &codex().genus {
            if (!include_horizons && genus.is_horizon)
                || !self.requirements(&genus.requirements, None)
            {
                continue;
            }
            for (skey, species) in &genus.species {
                if !self.requirements(&species.requirements, None)
                    || (self.complete && !self.requirements(&species.post_requirements, None))
                {
                    continue;
                }
                let regions = species.regions == 0 || self.regions & species.regions != 0;
                // Region mismatches remain visible as unlikely predictions in the
                // original. A scan can subsequently provide an actual sighting.
                let parents = self.parent_stars();
                let radiant = self.radiant();
                let good_context = self.good_stellar_context();
                let star_requirements =
                    !good_context || self.requirements(&species.star_requirements, radiant);
                let mut variants = Vec::new();
                for variant in &species.variant {
                    let colour = COLOURS
                        .get(variant.colour as usize)
                        .unwrap_or(&"Unknown")
                        .to_string();
                    match species.variant_type {
                        2 => {
                            let material = MATERIALS
                                .get(variant.material.unwrap_or(255) as usize)
                                .copied()
                                .unwrap_or("");
                            if has_material(self.scan, material) {
                                variants.push((
                                    colour,
                                    format!("Surface Material: {material} present"),
                                    if !regions {
                                        2
                                    } else if !good_context {
                                        1
                                    } else {
                                        0
                                    },
                                    true,
                                ));
                            }
                        }
                        3 => {
                            if parents.is_empty() {
                                // AllParentStars falls back to UNKNOWN when
                                // no parent/hint exists. The source emits no
                                // unsupported colour candidates in that case.
                            } else if parents
                                .iter()
                                .any(|s| Some(star_type(text(s, "StarType"))) == variant.star)
                            {
                                let likely = radiant.is_some_and(|s| {
                                    Some(star_type(text(s, "StarType"))) == variant.star
                                }) && regions;
                                variants.push((
                                    colour,
                                    format!(
                                        "Star Class: {}",
                                        parents
                                            .iter()
                                            .filter(|s| Some(star_type(text(s, "StarType")))
                                                == variant.star)
                                            .map(|s| [
                                                "UNKNOWN", "O", "B", "A", "F", "G", "K", "M", "L",
                                                "T", "TTS", "W", "D", "N", "Y", "AEBE", "H",
                                                "SUPER", "C"
                                            ]
                                                [star_type(text(s, "StarType")) as usize])
                                            .collect::<Vec<_>>()
                                            .join(",")
                                    ),
                                    if likely {
                                        if good_context {
                                            0
                                        } else {
                                            1
                                        }
                                    } else {
                                        2
                                    },
                                    true,
                                ));
                            }
                        }
                        4 => variants.push((
                            colour,
                            "Horizons Bio".to_owned(),
                            if !regions {
                                2
                            } else if !good_context {
                                1
                            } else {
                                0
                            },
                            true,
                        )),
                        _ => variants.push((
                            colour,
                            "Variant not predicted".to_owned(),
                            if regions { 0 } else { 2 },
                            true,
                        )),
                    }
                }
                // Source VariantResult merges same-colour candidates and hides
                // unlikely alternatives when a radiant colour was predicted.
                let mut merged: Vec<(String, String, u8, bool)> = Vec::new();
                for variant in variants {
                    if let Some(previous) = merged.iter_mut().find(|v| v.0 == variant.0) {
                        previous.1.push(',');
                        previous
                            .1
                            .push_str(variant.1.strip_prefix("Star Class: ").unwrap_or(&variant.1));
                        previous.2 = previous.2.min(variant.2);
                    } else {
                        merged.push(variant);
                    }
                }
                let mut variants = merged;
                if species.variant_type == 3
                    && (self.relevant_parent_stars().len() == 1
                        || variants.iter().any(|v| v.2 == 0))
                {
                    for variant in &mut variants {
                        variant.3 = variant.2 != 2;
                    }
                }
                // The plugin runs radiant-star requirements after CheckScan and
                // downgrades each source confidence by one level, capped Unlikely.
                if !star_requirements {
                    for variant in &mut variants {
                        variant.2 = (variant.2 + 1).min(2);
                    }
                }
                variants.sort_by(|a, b| a.0.cmp(&b.0));
                variants.dedup_by(|a, b| a.0 == b.0);
                if variants.iter().any(|v| v.3) {
                    result.push(Predicted {
                        genus: gkey.clone(),
                        species: skey.clone(),
                        value: species.base_value,
                        distance: genus.minimum_sample_distance,
                        horizons: genus.is_horizon,
                        provisional: variants.iter().any(|v| v.2 == 1),
                        variants,
                        region_matches: regions,
                    });
                }
            }
        }
        result
    }
}
fn star_name(s: &str) -> u8 {
    if s == "D" {
        12
    } else if s == "W" {
        11
    } else {
        star_type(s)
    }
}
fn has_material(scan: &Value, material: &str) -> bool {
    scan.get("Materials")
        .and_then(Value::as_array)
        .is_some_and(|a| {
            a.iter().any(|m| {
                text(m, "Name").eq_ignore_ascii_case(material) && number(m, "Percent") > 0.0
            })
        })
}
fn is_procedural(body: &str) -> bool {
    let words: Vec<_> = body.split_whitespace().collect();
    words.windows(2).any(|w| {
        let v = w[0].as_bytes();
        v.len() == 5
            && v[2] == b'-'
            && [v[0], v[1], v[3], v[4]].iter().all(u8::is_ascii_uppercase)
            && w[1]
                .as_bytes()
                .first()
                .is_some_and(|c| (b'a'..=b'h').contains(c))
    })
}

pub(super) fn regions_at(position: [f64; 3]) -> u64 {
    static ROWS: OnceLock<Vec<Vec<(u16, u8)>>> = OnceLock::new();
    let rows = ROWS.get_or_init(|| {
        let bytes = include_bytes!("regions.bin");
        let mut offset = 0;
        let mut rows = Vec::with_capacity(2048);
        for _ in 0..2048 {
            let n = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]) as usize;
            offset += 2;
            let mut row = Vec::with_capacity(n);
            for _ in 0..n {
                row.push((
                    u16::from_le_bytes([bytes[offset], bytes[offset + 1]]),
                    bytes[offset + 2],
                ));
                offset += 3;
            }
            rows.push(row);
        }
        rows
    });
    let [x, _, z] = position;
    let mut result = 0;
    for (x, z) in [
        (x, z),
        (x - 20.0, z),
        (x + 20.0, z),
        (x, z - 20.0),
        (x, z + 20.0),
    ] {
        let ix = ((x + 49_985.0) * 83.0 / 4096.0) as i32;
        let iz = ((z + 24_105.0) * 83.0 / 4096.0) as i32;
        if !(0..2048).contains(&ix) || !(0..2048).contains(&iz) {
            continue;
        }
        let mut limit = 0;
        for (length, region) in &rows[iz as usize] {
            limit += *length as i32;
            if ix < limit {
                if *region != 0 {
                    result |= 1u64 << region;
                }
                break;
            }
        }
    }
    result
}
pub(super) fn spheres_at([x, y, z]: [f64; 3]) -> BTreeSet<u8> {
    static SPHERES: OnceLock<Vec<([f64; 3], f64, u8)>> = OnceLock::new();
    let spheres = SPHERES.get_or_init(|| {
        include_str!("spheres.csv")
            .lines()
            .filter_map(|line| {
                let v: Vec<_> = line.split(',').collect();
                if v.len() != 5 {
                    return None;
                }
                Some((
                    [v[0].parse().ok()?, v[1].parse().ok()?, v[2].parse().ok()?],
                    v[3].parse().ok()?,
                    v[4].trim().parse().ok()?,
                ))
            })
            .collect()
    });
    spheres
        .iter()
        .filter_map(|([a, b, c], r, t)| {
            (((x - a).powi(2) + (y - b).powi(2) + (z - c).powi(2)).sqrt() < *r).then_some(*t)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn stellar_context_matches_original_bsystem_enrichment() {
        let cases: Vec<Value> = serde_json::from_str(include_str!(
            "../../../fixtures/bioinsights-radiant-oracle.json"
        ))
        .unwrap();
        assert_eq!(cases.len(), 241);
        let mut failures = Vec::new();
        for case in cases {
            let scans = case["Input"]["Scans"].as_array().unwrap();
            let mut bodies: BTreeMap<_, _> = scans
                .iter()
                .map(|s| (s["BodyID"].as_u64().unwrap(), s.clone()))
                .collect();
            if let Some(hint) = case["Input"]["StarHint"].as_str().filter(|h| !h.is_empty()) {
                bodies.insert(
                    u64::MAX,
                    json!({"BodyID":0,"StarType":if hint=="AEBE"{"AeBe"}else{hint},"_Hint":true}),
                );
            }
            let scan = &bodies[&case["Input"]["BodyID"].as_u64().unwrap()];
            let ctx = Context {
                scan,
                bodies: &bodies,
                regions: regions_at([0.0; 3]),
                spheres: &spheres_at([0.0; 3]),
                complete: true,
            };
            let selected = ctx.radiant().and_then(|s| s["BodyID"].as_u64());
            if let Some(raw) = ctx.radiant() {
                let kind = [
                    "UNKNOWN", "O", "B", "A", "F", "G", "K", "M", "L", "T", "TTS", "W", "D", "N",
                    "Y", "AEBE", "H", "SUPER", "C",
                ][star_type(text(raw, "StarType")) as usize];
                if kind != text(&case, "RadiantType") {
                    failures.push(format!(
                        "{} type {} expected {}",
                        case["Input"]["Name"], kind, case["RadiantType"]
                    ));
                }
            }
            if case["RadiantType"] != "UNKNOWN" && selected != case["Radiant"].as_u64() {
                failures.push(format!(
                    "{} selected {:?}, expected {}",
                    case["Input"]["Name"], selected, case["Radiant"]
                ));
            }
            let mut predicted:Vec<Value>=ctx.predict(true).iter().filter(|p|p.region_matches).map(|p|{
                let mut variants:Vec<Value>=p.variants.iter().map(|v|json!({"Colour":v.0,"Reason":v.1,"Likelihood":if v.2==0{"Likely"}else if v.2==1{"Maybe"}else{"Unlikely"},"ShowInUI":v.3})).collect();variants.sort_by_key(|v|text(v,"Colour").to_owned());
                let kind=match codex().genus[&p.genus].species[&p.species].variant_type{2=>"SurfaceMaterial",3=>"ParentStar",4=>"Horizons",_=>"Unknown"};
                json!({"Genus":p.genus,"Species":p.species,"Value":p.value,"Distance":p.distance,"Type":kind,"Variants":variants})
            }).collect();
            predicted.sort_by_key(|p| text(p, "Species").to_owned());
            let mut expected = case["Predictions"].as_array().unwrap().clone();
            for p in &mut expected {
                p["Variants"]
                    .as_array_mut()
                    .unwrap()
                    .sort_by_key(|v| text(v, "Colour").to_owned());
            }
            expected.sort_by_key(|p| text(p, "Species").to_owned());
            if predicted != expected {
                failures.push(format!(
                    "{} full predictions expected {} got {}",
                    case["Input"]["Name"],
                    json!(expected),
                    json!(predicted)
                ));
            }
            for source in case["Stars"].as_array().unwrap() {
                let star = &bodies[&source["BodyID"].as_u64().unwrap()];
                let distance = ctx.star_distance(star);
                let expected = number(source, "CalculatedDistance");
                if (distance - expected).abs() > expected.abs().max(1.0) * 1e-6 {
                    failures.push(format!(
                        "{} star{} distance {} expected {}",
                        case["Input"]["Name"], source["BodyID"], distance, expected
                    ));
                }
            }
        }
        assert!(
            failures.is_empty(),
            "{} mismatches: {}",
            failures.len(),
            failures.join("\n")
        );
    }
    #[test]
    fn full_predictions_match_original_codexchecker_synthetic_scans() {
        use serde_json::json;
        let cases: Value = serde_json::from_str(include_str!(
            "../../../fixtures/bioinsights-predictions-oracle.json"
        ))
        .unwrap();
        let mut failures = Vec::new();
        assert_eq!(cases.as_array().unwrap().len(), 4026);
        let covered: BTreeSet<_> = cases
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|c| c["Results"].as_array().unwrap())
            .map(|p| text(p, "Species"))
            .collect();
        assert_eq!(
            covered.len(),
            117,
            "Every original catalogue species needs a positive source prediction"
        );
        for case in cases.as_array().unwrap() {
            let position: [f64; 3] = serde_json::from_value(case["Position"].clone()).unwrap();
            let spheres = if case["AllSpheres"] == true {
                (0..=6).collect()
            } else {
                spheres_at(position)
            };
            let mut bodies = BTreeMap::new();
            let star = text(case, "Star");
            if !star.is_empty() {
                bodies.insert(0,json!({"BodyID":0,"BodyName":"Synthetic A","StarType":star,"SurfaceTemperature":5800,"Radius":700_000_000,"AbsoluteMagnitude":4.83,"Luminosity":"V"}));
            }
            let mut scan = case["Scan"].clone();
            let distance = number(case, "StarDistance");
            let orbit = number(&scan, "SemiMajorAxis");
            if (distance - orbit).abs() > 1.0 {
                scan["Parents"] = json!([{"Planet":2},{"Star":0}]);
                bodies.insert(
                    2,
                    json!({"BodyID":2,"SemiMajorAxis":distance-orbit,"_ContextPlanet":true}),
                );
            }
            let parent = text(case, "ParentStar");
            if !parent.is_empty() {
                let delta = number(case, "MagnitudeDelta");
                scan["Parents"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"Star":10}));
                bodies.insert(10,json!({"BodyID":10,"StarType":parent,"SurfaceTemperature":5800,"Radius":700_000_000.0*10f64.powf(-delta/5.0),"AbsoluteMagnitude":4.83+delta,"Luminosity":"V"}));
            }
            for (index, kind) in case["ExtraPlanets"]
                .as_array()
                .into_iter()
                .flatten()
                .enumerate()
            {
                bodies.insert(100 + index as u64, json!({"PlanetClass":kind}));
            }
            for (index, kind) in case["ExtraStars"]
                .as_array()
                .into_iter()
                .flatten()
                .enumerate()
            {
                bodies.insert(200+index as u64,json!({"BodyID":200+index,"StarType":kind,"DistanceFromArrivalLS":1_000_000_000,"_SystemOnly":true}));
            }
            bodies.insert(1, scan.clone());
            let ctx = Context {
                scan: &scan,
                bodies: &bodies,
                regions: if case["AllRegions"] == true {
                    (1u64 << 43) - 2
                } else {
                    regions_at(position)
                },
                spheres: &spheres,
                complete: case["Complete"] == true,
            };
            let mut predicted:Vec<Value>=ctx.predict(true).iter().filter(|p|p.region_matches).map(|p|{
                let mut variants:Vec<Value>=p.variants.iter().map(|v|json!({"Colour":v.0,"Reason":v.1,"Likelihood":if v.2==0{"Likely"}else if v.2==1{"Maybe"}else{"Unlikely"},"ShowInUI":v.3})).collect();variants.sort_by_key(|v|text(v,"Colour").to_owned());
                let kind=match codex().genus[&p.genus].species[&p.species].variant_type{2=>"SurfaceMaterial",3=>"ParentStar",4=>"Horizons",_=>"Unknown"};
                json!({"Genus":p.genus,"Species":p.species,"Value":p.value,"Distance":p.distance,"Type":kind,"Variants":variants})
            }).collect();
            predicted.sort_by_key(|p| text(p, "Species").to_owned());
            let mut expected:Vec<Value>=case["Results"].as_array().unwrap().iter().map(|p|{
                let mut variants=p["Variants"].as_array().unwrap().clone();variants.sort_by_key(|v|text(v,"Colour").to_owned());
                json!({"Genus":p["Genus"],"Species":p["Species"],"Value":p["Value"],"Distance":p["Distance"],"Type":p["Type"],"Variants":variants})
            }).collect();
            expected.sort_by_key(|p| text(p, "Species").to_owned());

            if predicted != expected {
                failures.push(format!(
                    "Case {} scan {} expected {} got {}",
                    case["Case"],
                    case["Scan"],
                    json!(expected),
                    json!(predicted)
                ));
            }
        }
        assert!(
            failures.is_empty(),
            "{} of {} source cases differ:\n{}",
            failures.len(),
            cases.as_array().unwrap().len(),
            failures
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
    use super::*;
    use serde_json::json;
    #[test]
    fn complete_installed_catalogue_and_all_active_predicate_kinds_are_supported() {
        let data = codex();
        assert_eq!(data.genus.len(), 21);
        assert_eq!(
            data.genus.values().map(|g| g.species.len()).sum::<usize>(),
            117
        );
        assert_eq!(data.entry_id_species.len(), 772);
        let supported = [
            0, 1, 2, 3, 5, 7, 8, 9, 10, 11, 12, 15, 16, 17, 18, 19, 21, 31, 32, 33, 43,
        ];
        for (gkey, g) in &data.genus {
            assert_ne!(name(gkey), *gkey);
            assert!(g.minimum_sample_distance > 0);
            for (skey, s) in &g.species {
                assert_ne!(name(skey), *skey);
                assert!(s.base_value > 0);
                assert!(!s.variant.is_empty());
                for requirement in g
                    .requirements
                    .iter()
                    .chain(&s.requirements)
                    .chain(&s.post_requirements)
                    .chain(&s.star_requirements)
                {
                    for alternative in &requirement.value {
                        for predicate in alternative {
                            assert!(
                                supported.contains(&predicate.r#type),
                                "uncovered predicate {} for {skey}",
                                predicate.r#type
                            );
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn volcanism_matches_84_original_predicate_source_vectors() {
        let vectors: Vec<Value> =
            serde_json::from_str(include_str!("predicate-vectors.json")).unwrap();
        assert_eq!(vectors.len(), 84);
        let bodies = BTreeMap::new();
        let spheres = BTreeSet::new();
        for vector in vectors {
            let scan = json!({"Volcanism":vector["Volcanism"]});
            let context = Context {
                scan: &scan,
                bodies: &bodies,
                regions: 0,
                spheres: &spheres,
                complete: true,
            };
            let predicate = Predicate {
                r#type: 5,
                s_value: vector["Rule"].as_str().unwrap().to_owned(),
                ..Default::default()
            };
            assert_eq!(
                context.test(&predicate, None),
                vector["Result"].as_bool().unwrap(),
                "{vector}"
            );
        }
    }
    #[test]
    fn strict_maximum_and_inclusive_minimum_climate_bounds_match_original() {
        let scan = json!({"SurfaceTemperature":180.0,"SurfaceGravity":9.798,"SurfacePressure":101325.0,"MassEM":1.0});
        let bodies = BTreeMap::new();
        let spheres = BTreeSet::new();
        let c = Context {
            scan: &scan,
            bodies: &bodies,
            regions: 0,
            spheres: &spheres,
            complete: true,
        };
        for (min, max, value) in [
            (7, 8, single(&scan, "SurfacePressure") / 101325.0),
            (9, 10, single(&scan, "SurfaceGravity") / 9.798),
            (11, 12, 180.0),
            (13, 14, 1.0),
        ] {
            assert!(c.test(
                &Predicate {
                    r#type: min,
                    d_value: value,
                    ..Default::default()
                },
                None
            ));
            assert!(!c.test(
                &Predicate {
                    r#type: max,
                    d_value: value,
                    ..Default::default()
                },
                None
            ));
            assert!(c.test(
                &Predicate {
                    r#type: max,
                    d_value: value + 0.000001,
                    ..Default::default()
                },
                None
            ));
        }
    }
    #[test]
    fn galactic_region_and_nebula_lookup_use_installed_spatial_resources() {
        assert_eq!(regions_at([0.0, 0.0, 0.0]), 1 << 18);
        assert_eq!(regions_at([1e9, 0.0, 1e9]), 0);
        assert!(spheres_at([-719.23047, -17.45312, 694.55273]).contains(&0));
    }
    #[test]
    fn giant_and_compact_star_classes_follow_installed_conversion() {
        for (star, expected) in [
            ("M_RedGiant", 7),
            ("M_RedSuperGiant", 6),
            ("F_WhiteSuperGiant", 4),
            ("A_BlueWhiteSuperGiant", 3),
            ("DAZ", 12),
            ("SupermassiveBlackHole", 17),
            ("AeBe", 15),
            ("CS", 18),
        ] {
            assert_eq!(star_type(star), expected, "{star}");
        }
    }
}
