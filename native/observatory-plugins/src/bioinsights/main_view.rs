//! Projection of UIHandler.GenerateHTMLBodyHeader/GenerateHTMLBodyDetails.
//! This is the plugin's main grouped view, distinct from its NativeOutput grid.
use super::*;

impl BioInsights {
    pub(super) fn visual_alert(&self, system: u64, body: u64) -> Value {
        if let Some(cached) = self
            .alert_cache
            .lock()
            .expect("alert cache")
            .get(&(system, body))
        {
            return cached.clone();
        }
        let view = self.project_main_view(Some((system, body)));
        let group = &view["Groups"][0];
        let lines:Vec<Value>=group["Rows"].as_array().into_iter().flatten().filter(|r|r["Seen"]!="NotPresent").map(|row|{
            let variants:Vec<Value>=row["Variants"].as_array().into_iter().flatten().filter(|v|v["NotPresent"]!=true).map(|v|json!({"Colour":v["Colour"],"ShowInUI":v["ShowInUI"],"PersonalCodexEntry":v["PersonalCodex"],"CCMark":v["Marked"],"PossibleCodexEntry":v["GalacticCodex"],"PossibleCodexEntryNeverSeen":v["GalacticNeverSeen"],"Likelihood":v["Likelihood"]})).collect();
            json!({"Genus":row["Genus"],"Species":row["Species"],"Type":row["PredictionType"],"BaseValue":row["BaseValue"],"Distance":row["Distance"],"IsPersonalCodexEntry":row["PersonalCodex"],"IsMarked":row["Marked"],"IsGalacticCodexEntry":row["GalacticCodex"],"IsGalacticCodexEntryNeverSeen":row["GalacticNeverSeen"],"Seen":row["Seen"],"Samples":row["SampleState"],"PossibleVariants":variants})
        }).collect();
        let alert = json!({"BodyName":group["BodyName"],"BodyID":body,"BioCount":group["Signals"],"Lines":lines});
        self.alert_cache
            .lock()
            .expect("alert cache")
            .insert((system, body), alert.clone());
        alert
    }
    pub fn main_view(&self) -> Value {
        self.project_main_view(None)
    }
    pub(super) fn project_main_view(&self, only: Option<(u64, u64)>) -> Value {
        let mut order = self.presentation_order.clone();
        for system in self.systems.keys() {
            if !order.contains(system) {
                order.push(*system);
            }
        }
        let mut groups = Vec::new();
        for system in order {
            if only.is_some_and(|(selected, _)| selected != system) {
                continue;
            }
            let Some(sys) = self.systems.get(&system) else {
                continue;
            };
            let bodies = sys.rule_bodies();
            let regions = sys.position.map(rules::regions_at).unwrap_or(0);
            let spheres = sys.position.map(rules::spheres_at).unwrap_or_default();
            for (body, state) in &sys.bodies {
                if only.is_some_and(|(_, selected)| selected != *body) {
                    continue;
                }
                // GenerateSystemTable requires a positive FSS/DSS count, unless
                // the body was explicitly forced into output by an observation.
                if state.bio_count.unwrap_or(0) == 0
                    && state.observed.is_empty()
                    && state.scan["ForceDisplay"] != true
                {
                    continue;
                }
                if !text(&state.scan, "StarType").is_empty()
                    || state.scan.get("PlanetClass").is_none()
                {
                    continue;
                }
                if self.setting("HideAutoscans") && text(&state.scan, "ScanType") == "AutoScan" {
                    continue;
                }
                let context = rules::Context {
                    scan: &state.scan,
                    bodies: &bodies,
                    regions,
                    spheres: &spheres,
                    complete: sys.complete,
                };
                let radiant = context
                    .radiant()
                    .map(|s| text(s, "StarType"))
                    .unwrap_or("Unknown");
                // CodexChecker.CheckScan skips a species when IsInRegion is
                // false (source flag2/case8). The diagnostic/native output grid
                // can retain those candidates, but they are not main-view rows.
                let mut predictions: Vec<_> = self
                    .all_predictions(system, *body)
                    .into_iter()
                    .filter(|p| p.region_matches)
                    .collect();
                let predicted_species: BTreeSet<_> =
                    predictions.iter().map(|p| p.species.clone()).collect();
                for species in state.observed.keys() {
                    if predicted_species.contains(species) {
                        continue;
                    }
                    if let Some((genus, species)) = rules::species_key(species) {
                        let genus_data = &rules::codex().genus[genus];
                        let data = &genus_data.species[species];
                        predictions.push(Predicted {
                            genus: genus.into(),
                            species: species.into(),
                            value: data.base_value,
                            distance: genus_data.minimum_sample_distance,
                            horizons: genus_data.is_horizon,
                            variants: Vec::new(),
                            region_matches: true,
                            provisional: false,
                        });
                    }
                }
                predictions.sort_by_cached_key(|p| (name(&p.genus), name(&p.species)));
                if text(&state.scan, "ScanType") == "AutoScan"
                    && self.settings["HideAutoscansNoPredictions"]
                        .as_bool()
                        .unwrap_or(true)
                    && predictions.is_empty()
                {
                    continue;
                }
                let mut rows = Vec::new();
                for p in predictions {
                    let observed = state.observed.get(&p.species);
                    let present = eligible(state, &p);
                    if !present && self.setting("HideNotPresent") {
                        continue;
                    }
                    let data = &rules::codex().genus[&p.genus].species[&p.species];
                    let was_predicted = predicted_species.contains(&p.species);
                    let mut variants = p.variants.clone();
                    if let Some(observed) = observed.filter(|s| !s.variant.is_empty()) {
                        if !variants.iter().any(|v| v.0 == observed.variant) {
                            variants.push((
                                observed.variant.clone(),
                                "Observed in journal".into(),
                                0,
                                true,
                            ));
                        }
                    }
                    // The installed checker retains Codex catalogue order;
                    // alphabetical colour sorting reverses e.g. Red or Blue.
                    variants.sort_by_key(|v| {
                        data.variant
                            .iter()
                            .position(|source| {
                                rules::COLOURS
                                    .get(source.colour as usize)
                                    .is_some_and(|c| *c == v.0)
                            })
                            .unwrap_or(usize::MAX)
                    });
                    let mut projected = Vec::new();
                    for (colour, reason, likely, show_in_ui) in variants {
                        let seen = observed.is_some_and(|s| s.seen && s.variant == colour);
                        let not_present = !present
                            || observed.is_some_and(|s| {
                                s.seen && !s.variant.is_empty() && s.variant != colour
                            });
                        let any_new = !self
                            .personal_codex
                            .iter()
                            .any(|(_, s, c)| s == &p.species && c == &colour);
                        let region_new = !self.personal_codex.iter().any(|(region, s, c)| {
                            *region < 64
                                && regions & (1 << region) != 0
                                && s == &p.species
                                && c == &colour
                        });
                        let personal = !not_present
                            && (self.setting("PersonalCodexPossibleAny") && any_new
                                || self.setting("PersonalCodexPossibleCurrent") && region_new);
                        let (regional, never) = self.galactic.flags(&p.species, &colour, regions);
                        let marked = self.is_marked(&p.species, &colour)
                            || observed.is_some_and(|s| s.marked);
                        let likelihood = if likely == 0 {
                            "Likely"
                        } else if likely == 1 {
                            "Maybe"
                        } else {
                            "Unlikely"
                        };
                        projected.push(json!({"Colour":colour,"Reason":reason,"Likelihood":likelihood,"ShowInUI":show_in_ui,"Seen":seen,"NotPresent":not_present,"PersonalCodex":personal,"Marked":marked,"GalacticCodex":!not_present&&regional,"GalacticNeverSeen":!not_present&&never}));
                    }
                    let personal = projected.iter().any(|v| v["PersonalCodex"] == true);
                    let marked = projected.iter().any(|v| v["Marked"] == true)
                        || self.is_marked(&p.species, "");
                    let galactic = projected.iter().any(|v| v["GalacticCodex"] == true);
                    let never = projected.iter().any(|v| v["GalacticNeverSeen"] == true);
                    let high =
                        self.meets_value_threshold("MinIndividualBiologicalValueMillions", p.value);
                    let prediction_type = if !was_predicted {
                        "NotPredicted"
                    } else {
                        match data.variant_type {
                            2 => "SurfaceMaterial",
                            3 => "ParentStar",
                            4 => "Horizons",
                            _ => "Unknown",
                        }
                    };
                    let seen = if !present {
                        "NotPresent"
                    } else if observed.is_some_and(|s| s.seen) {
                        "Seen"
                    } else {
                        "NotSeen"
                    };
                    let samples = observed.map_or("No", |s| {
                        if s.analysed {
                            "Complete"
                        } else {
                            match s.sample_count {
                                1 => "Logged",
                                2 => "Sampled",
                                _ => "No",
                            }
                        }
                    });
                    rows.push(json!({"Genus":name(&p.genus),"GenusID":p.genus,"GenusNumber":genus_number(&p.genus),"Species":name(&p.species),"SpeciesID":p.species,"SpeciesNumber":species_number(&p.species),"PredictionType":prediction_type,"Seen":seen,"SampleState":samples,"HighValue":high,"PersonalCodex":personal,"Marked":marked,"GalacticCodex":galactic,"GalacticNeverSeen":never,"BaseValue":p.value,"ValueText":millions(p.value),"Distance":p.distance,"Variants":projected}));
                }
                let mut previous = String::new();
                let mut stripe = 0usize;
                for row in &mut rows {
                    let genus = text(row, "Genus");
                    if !previous.is_empty() && previous != genus {
                        stripe += 1;
                    }
                    previous = genus.to_owned();
                    row["Stripe"] = json!(stripe);
                }
                let summary = self.summary(system, *body);
                let minimum = summary["BaseMinimumValue"].as_u64().unwrap_or(0);
                let maximum = summary["BaseMaximumValue"].as_u64().unwrap_or(0);
                let class = text(&state.scan, "PlanetClass");
                let high = rows.iter().any(|r| r["HighValue"] == true)
                    || self.meets_value_threshold("MinBodyValueMillions", maximum);
                let personal = rows.iter().any(|r| r["PersonalCodex"] == true);
                let marked = rows.iter().any(|r| r["Marked"] == true);
                let galactic = rows.iter().any(|r| r["GalacticCodex"] == true);
                let never = rows.iter().any(|r| r["GalacticNeverSeen"] == true);
                let value_text = if minimum == 0 {
                    String::new()
                } else if minimum == maximum {
                    millions(minimum)
                } else {
                    format!("{} - {}", millions(minimum), millions(maximum))
                };
                let genus_count = rows
                    .iter()
                    .filter(|r| r["Seen"] != "NotPresent")
                    .filter_map(|r| r["GenusID"].as_str())
                    .collect::<BTreeSet<_>>()
                    .len();
                let signal_symbol = if text(&state.scan, "ScanType") == "AutoScan" {
                    "BioCountAutoscan"
                } else if state
                    .bio_count
                    .is_some_and(|count| (genus_count as u64) < count)
                {
                    "BioCountMismatch"
                } else {
                    "BioCount"
                };
                groups.push(json!({"ID64":system,"System":sys.name,"BodyID":body,"BodyName":text(&state.scan,"BodyName"),"BodyType":class,"PlanetSymbol":planet_symbol(class),"Current":system==self.current_system&&self.current_body==Some(*body),"CurrentSystem":system==self.current_system,"Signals":state.bio_count,"SignalSymbol":signal_symbol,"Mapped":state.mapped,"RadiantStar":radiant,"MinimumValue":minimum,"MaximumValue":maximum,"ValueText":value_text,"FirstFootfall":state.footfall,"HighValue":high,"PersonalCodex":personal,"Marked":marked,"GalacticCodex":galactic,"GalacticNeverSeen":never,"Gravity":number(&state.scan,"SurfaceGravity")/9.80665,"Temperature":number(&state.scan,"SurfaceTemperature"),"Atmosphere":text(&state.scan,"Atmosphere"),"Pressure":number(&state.scan,"SurfacePressure")/101325.0,"Volcanism":text(&state.scan,"Volcanism"),"DistanceLS":number(&state.scan,"DistanceFromArrivalLS"),"ExtraFields":extra_fields(&state.scan),"Rows":rows}));
            }
        }
        json!({"Commander":self.commander_name,"FID":self.commander,"SystemAddress":self.current_system,"System":self.systems.get(&self.current_system).map(|s|s.name.as_str()).unwrap_or(""),"Population":self.systems.get(&self.current_system).map(|s|s.population).unwrap_or(0),"Groups":groups,"UnsoldTotal":self.overlay()["UnsoldTotal"],"SourceView":"Main","DependencyNag":self.browser_route.first().is_some_and(|p|p=="hecatenag"),"UIAlerts":self.ui_alerts()})
    }
}
fn decimal(value: f64, precision: usize) -> String {
    let s = format!("{value:.precision$}");
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').into()
    } else {
        s
    }
}
fn extra_fields(scan: &Value) -> Value {
    let atmosphere = rules::atmosphere_name(scan);
    let empty = atmosphere == "None";
    let raw = text(scan, "Volcanism");
    let volcano = raw.replace(" volcanism", "");
    let volcano = if volcano.is_empty() {
        "No Volcanism".to_owned()
    } else {
        volcano
            .split_whitespace()
            .map(|w| {
                let mut c = w.chars();
                c.next()
                    .map(|v| v.to_uppercase().collect::<String>() + &c.as_str().to_lowercase())
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>()
            .join(" ")
    };
    json!({"Atmosphere":if empty{"No Atmosphere"}else{atmosphere},"Pressure":if empty{"-".to_owned()}else{format!("{}atm",decimal(number(scan,"SurfacePressure")/101325.0,4))},"Volcanism":volcano,"Gravity":format!("{}G",decimal(number(scan,"SurfaceGravity")/9.80665,3)),"Temperature":format!("{}K",decimal(number(scan,"SurfaceTemperature"),1))})
}
fn planet_symbol(class: &str) -> &'static str {
    match class.to_ascii_lowercase().as_str() {
        "metal rich body" => "MR",
        "high metal content body" => "HMC",
        "rocky body" => "RB",
        "rocky ice body" => "RIB",
        "icy body" => "IB",
        _ => "Unknown",
    }
}
fn millions(value: u64) -> String {
    let formatted = format!(
        "{:.2}",
        ((value as f64 / 1_000_000.0) * 100.0).round_ties_even() / 100.0
    );
    format!("{}M", formatted.trim_end_matches('0').trim_end_matches('.'))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event(value: Value) -> JournalEvent {
        JournalEvent::parse(&value.to_string()).unwrap()
    }
    #[test]
    fn main_view_is_body_grouped_uses_commander_name_and_source_values() {
        let mut worker = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        worker.on_event(&event(
            json!({"event":"LoadGame","Commander":"CascadianX","FID":"F1234"}),
        ));
        worker.on_event(&event(json!({"event":"Location","SystemAddress":42,"StarSystem":"Eolls Flyiae OX-A d1-27","StarPos":[0,0,0]})));
        let genus = |s: &str| {
            rules::codex()
                .genus
                .keys()
                .find(|g| name(g) == s)
                .unwrap()
                .clone()
        };
        let species = |g: &str, s: &str| {
            rules::codex().genus[&genus(g)]
                .species
                .keys()
                .find(|key| name(key) == s)
                .unwrap()
                .clone()
        };
        let selections: [(&str, &str, &[&str]); 8] = [
            ("Bacterium", "Alcyoneum", &["Emerald"]),
            ("Concha", "Renibus", &["Red", "Blue"]),
            ("Frutexa", "Flabellum", &["Emerald"]),
            ("Fungoida", "Gelata", &["Cyan", "Green"]),
            ("Fungoida", "Setisis", &["Orange"]),
            ("Osseus", "Spiralis", &["Grey"]),
            ("Tubus", "Rosarium", &["Red"]),
            ("Tussock", "Divisa", &["Lime"]),
        ];
        let mut sys = SystemState {
            name: "Eolls Flyiae OX-A d1-27".into(),
            ..Default::default()
        };
        sys.bodies.insert(0,BodyState{scan:json!({"BodyID":0,"StarType":"G","SurfaceTemperature":5500,"Radius":695700000}),..Default::default()});
        let mut body = BodyState {
            scan: json!({"BodyName":"Eolls Flyiae OX-A d1-27 1 e","PlanetClass":"Rocky body","Parents":[{"Star":0}],"SemiMajorAxis":149597870700.0,"WasFootfalled":false}),
            bio_count: Some(4),
            footfall: true,
            ..Default::default()
        };
        let mut predictions = Vec::new();
        for (g, s, colours) in selections {
            let g = genus(g);
            let s = species(&name(&g), s);
            let data = &rules::codex().genus[&g];
            let data_species = &data.species[&s];
            predictions.push(Predicted {
                genus: g,
                species: s,
                value: data_species.base_value,
                distance: data.minimum_sample_distance,
                horizons: false,
                variants: colours
                    .iter()
                    .map(|c| {
                        (
                            (*c).to_owned(),
                            "Screenshot presentation fixture".into(),
                            0,
                            true,
                        )
                    })
                    .collect(),
                region_matches: true,
                provisional: false,
            });
        }
        body.genera = predictions.iter().map(|p| p.genus.clone()).collect();
        sys.bodies.insert(7, body);
        worker.systems.insert(42, sys);
        worker
            .predictions_cache
            .lock()
            .unwrap()
            .insert((42, 7), predictions);
        let view = worker.main_view();
        assert_eq!(view["Commander"], "CascadianX");
        assert_eq!(view["FID"], "F1234");
        let group = &view["Groups"][0];
        assert_eq!(group["Signals"], 4);
        assert_eq!(group["PlanetSymbol"], "RB");
        assert_eq!(group["RadiantStar"], "G");
        assert_eq!(group["Rows"].as_array().unwrap().len(), 8);
        assert_eq!(group["Rows"][0]["Genus"], "Bacterium");
        assert_eq!(group["Rows"][0]["Species"], "Alcyoneum");
        assert_eq!(group["Rows"][0]["BaseValue"], 1658500);
        assert_eq!(group["Rows"][0]["ValueText"], "1.66M");
        assert_eq!(group["ValueText"], "6.9M - 12.94M");
        assert_eq!(group["SignalSymbol"], "BioCount");
        assert_eq!(group["Rows"][1]["Variants"][0]["Colour"], "Red");
        assert_eq!(group["Rows"][1]["Variants"][1]["Colour"], "Blue");
    }
    #[test]
    fn source_main_hides_unconfirmed_bodies_but_preserves_no_prediction_headers() {
        let mut worker = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        worker.systems.entry(42).or_default().bodies.insert(
            1,
            BodyState {
                scan: json!({"BodyName":"Unknown 1","PlanetClass":"Icy body"}),
                ..Default::default()
            },
        );
        assert!(worker.main_view()["Groups"].as_array().unwrap().is_empty());
        worker
            .systems
            .get_mut(&42)
            .unwrap()
            .bodies
            .get_mut(&1)
            .unwrap()
            .bio_count = Some(1);
        assert_eq!(worker.main_view()["Groups"][0]["BodyName"], "Unknown 1");
    }
    #[test]
    fn original_main_excludes_out_of_region_candidates_but_keeps_actual_sightings() {
        let mut worker =
            BioInsights::new(json!({"GalacticCodexRemote":false,"ShowUnlikely":true})).unwrap();
        let candidates = [
            ("Cactoida", "Lapis"),
            ("Cactoida", "Peperatis"),
            ("Frutexa", "Flammasis"),
        ];
        let mut predictions = Vec::new();
        for (genus, species) in candidates {
            let (g, data) = rules::codex()
                .genus
                .iter()
                .find(|(g, _)| name(g) == genus)
                .unwrap();
            let (s, data_species) = data
                .species
                .iter()
                .find(|(s, _)| name(s) == species)
                .unwrap();
            predictions.push(Predicted {
                genus: g.clone(),
                species: s.clone(),
                value: data_species.base_value,
                distance: data.minimum_sample_distance,
                horizons: false,
                variants: vec![("Orange".into(), "Star Class: G".into(), 2, true)],
                region_matches: false,
                provisional: false,
            });
        }
        let observed = predictions[0].species.clone();
        worker.systems.entry(42).or_default().bodies.insert(
            1,
            BodyState {
                scan: json!({"BodyName":"Reference 1","PlanetClass":"Rocky body"}),
                bio_count: Some(4),
                ..Default::default()
            },
        );
        worker
            .predictions_cache
            .lock()
            .unwrap()
            .insert((42, 1), predictions);
        assert_eq!(
            worker.all_predictions(42, 1).len(),
            3,
            "diagnostic candidates remain available"
        );
        assert!(worker.main_view()["Groups"][0]["Rows"]
            .as_array()
            .unwrap()
            .is_empty());
        worker
            .systems
            .get_mut(&42)
            .unwrap()
            .bodies
            .get_mut(&1)
            .unwrap()
            .observed
            .insert(
                observed,
                SpeciesState {
                    seen: true,
                    variant: "Orange".into(),
                    ..Default::default()
                },
            );
        let view = worker.main_view();
        let rows = view["Groups"][0]["Rows"].as_array().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["Species"], "Lapis");
        assert_eq!(rows[0]["PredictionType"], "NotPredicted");
        assert_eq!(rows[0]["Seen"], "Seen");
    }
}
