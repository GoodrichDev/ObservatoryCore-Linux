//! Codex Companion projection of the installed catalogue and personal records.
//! UIHandler.buildSpeciesPage/buildVariantsTable are the presentation contract.
use super::*;
use std::sync::OnceLock;

const ALL_REGIONS: u64 = (1u64 << 43) - 2;

pub(super) fn full_name(genus: &str, species: &str) -> String {
    if rules::codex()
        .genus
        .get(genus)
        .is_some_and(|g| !g.is_horizon)
    {
        format!("{} {}", name(genus), name(species))
    } else {
        name(species)
    }
}

fn catalogue() -> &'static Vec<Value> {
    static ROWS: OnceLock<Vec<Value>> = OnceLock::new();
    ROWS.get_or_init(|| {
        // These strings are outputs of the original pure formatting methods,
        // captured for all 117 species. See fixtures/bioinsights-companion-oracle.
        let requirements: BTreeMap<String, Vec<String>> = serde_json::from_str(include_str!(
            "companion-requirements.json"
        ))
        .expect("installed Codex Companion requirements");
        let regions: Vec<Value> = serde_json::from_str(include_str!("region-names.json"))
            .expect("installed galactic region names");
        let mut rows = Vec::with_capacity(117);
        for (gkey, genus) in &rules::codex().genus {
            for (skey, species) in &genus.species {
                let genus_name = name(gkey);
                let species_name = name(skey);
                let full_name = full_name(gkey,skey);
                let possible = if species.regions == 0 { ALL_REGIONS } else { species.regions };
                let region_names: Vec<_> = (1..=42)
                    .filter(|i| species.regions & (1u64 << i) != 0)
                    .map(|i| regions[i].clone())
                    .collect();
                let mut causes: BTreeMap<u8, Vec<String>> = BTreeMap::new();
                for variant in &species.variant {
                    let cause = match species.variant_type {
                        3 => radiant_star(variant.star.unwrap_or(0)).to_owned(),
                        2 => material(variant.material.unwrap_or(255)).to_owned(),
                        _ => String::new(),
                    };
                    causes.entry(variant.colour).or_default().push(cause);
                }
                let variants: Vec<_> = causes.into_iter().map(|(colour_id, causes)| {
                    let colour = rules::COLOURS.get(colour_id as usize).copied().unwrap_or("Unknown");
                    let entry = if genus.is_horizon { full_name.clone() } else { format!("{full_name} - {colour}") };
                    json!({"Colour":colour,"ColourID":colour_id,"Cause":causes.join(" / "),
                        "BioForgeURL":format!("https://bioforge.canonn.tech/?entryid={entry}")})
                }).collect();
                let description = include_str!("species-description.tsv")
                    .lines().filter_map(|line|line.split_once('\t'))
                    .find(|(label,_)|*label==full_name).map(|(_,text)|text).unwrap_or("");
                rows.push(json!({"Genus":genus_name,"GenusID":gkey,"GenusNumber":genus_number(gkey),
                    "Species":species_name,"SpeciesID":skey,"SpeciesNumber":species_number(skey),
                    "FullName":full_name,"BaseValue":species.base_value,"FirstLoggedValue":species.base_value*5,
                    "MinimumSampleDistance":genus.minimum_sample_distance,"Description":description,
                    "Horizons":genus.is_horizon,"VariantType":match species.variant_type {2=>"SurfaceMaterial",3=>"ParentStar",4=>"Horizons",1=>"NotPredicted",_=>"Unknown"},
                    "Requirements":requirements.get(skey).expect("all installed species have source formatting"),
                    "RegionMask":species.regions,"PossibleRegionMask":possible,"RegionNames":region_names,
                    "BioscanURL":format!("https://elitetools.github.io/bioscans/{}.png",genus_name.replace(' ',"")),
                    "Variants":variants}));
            }
        }
        rows.sort_by_key(|row|(row["GenusNumber"].as_u64().unwrap_or(0),row["SpeciesNumber"].as_u64().unwrap_or(0)));
        rows
    })
}

pub(super) fn project(worker: &BioInsights) -> Vec<Value> {
    let mut rows = catalogue().clone();
    for row in &mut rows {
        let skey = row["SpeciesID"].as_str().unwrap().to_owned();
        for variant in row["Variants"].as_array_mut().unwrap() {
            let colour = variant["Colour"].as_str().unwrap().to_owned();
            let regions: Vec<_> = worker
                .personal_codex
                .iter()
                .filter(|(region, species, c)| {
                    (1..=42).contains(region) && species == &skey && c == &colour
                })
                .map(|(region, _, _)| *region)
                .collect();
            let seen = regions
                .iter()
                .fold(0u64, |mask, region| mask | (1u64 << region));
            let galactic = worker.galactic.regions(&skey, &colour);
            let galactic_regions: Vec<_> = (1..=42)
                .filter(|region| galactic & (1u64 << region) != 0)
                .collect();
            variant["Marked"] = json!(worker.is_marked(&skey, &colour));
            variant["Seen"] = json!(!regions.is_empty());
            variant["SeenRegionMask"] = json!(seen);
            variant["SeenRegionCount"] =
                // UIHandler.buildVariantsTable counts region bits 1..42
                // (exclusive upper bound); preserve its displayed count while
                // retaining every real region in Regions/SeenRegionMask.
                json!(regions.iter().filter(|r| (1..42).contains(*r)).count());
            variant["Regions"] = json!(regions);
            variant["GalacticSeen"] = json!(galactic != 0);
            variant["GalacticRegions"] = json!(galactic_regions);
            variant["GalacticDataAvailable"] = json!(worker.galactic.has_data());
        }
    }
    rows
}

fn radiant_star(id: u8) -> &'static str {
    match id {
        1 => "O star",
        2 => "B star",
        3 => "A star",
        4 => "F star",
        5 => "G star",
        6 => "K star",
        7 => "M star",
        8 => "L dwarf",
        9 => "T dwarf",
        10 => "T-Tauri star",
        11 => "Wolf-Rayet",
        12 => "White dwarf",
        13 => "Neutron star",
        14 => "Y dwarf",
        15 => "Herbig Ae/Be star",
        16 => "Black hole",
        _ => "",
    }
}

fn material(id: u8) -> &'static str {
    const NAMES: [&str; 12] = [
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
    NAMES.get(id as usize).copied().unwrap_or("")
}

/// Original FormSettingsStellarOverlay.SendTest fixture, used only by its Test
/// buttons. It goes through the real overlay transport and never enters journals.
pub(super) fn overlay_test(worker: &mut BioInsights, payload: &Value) {
    let original = worker.settings.clone();
    if payload["Settings"].is_object() {
        worker.settings = payload["Settings"].clone();
    }
    let mut panel = json!({"Key":"BIFSSNotiTest","ImageKey":"BIFSSNotiTest","Title":"Graea Hypue AG-V d3-25 10 a",
    "BodyName":"Graea Hypue AG-V d3-25 10 a","Signals":1,"Predictions":[
        {"Genus":"Bacterium","Species":"Cerbrus","BaseValue":1689800,"PersonalCodex":true,"GalacticNeverSeen":true,"Variants":[{"Colour":"Green","Likely":true}]},
        {"Genus":"Bacterium","Species":"Tela","BaseValue":1949000,"Variants":[{"Colour":"Gold","Likely":true},{"Colour":"Yellow","Likely":false}]},
        {"Genus":"Stratum","Species":"Tectonicas","BaseValue":19010800,"PersonalCodex":true,"Variants":[{"Colour":"Emerald","Likely":false}]}
    ]});
    worker.notification_style(&mut panel, payload["DSS"] == true);
    let lines:Vec<Value>=panel["Predictions"].as_array().into_iter().flatten().map(|row|{
          let variants:Vec<Value>=row["Variants"].as_array().into_iter().flatten().map(|v|json!({"Colour":v["Colour"],"ShowInUI":true,"Likelihood":if v["Likely"]==true{"Likely"}else{"Maybe"}})).collect();
          json!({"Genus":row["Genus"],"Species":row["Species"],"BaseValue":row["BaseValue"],"IsPersonalCodexEntry":row["PersonalCodex"],"IsGalacticCodexEntryNeverSeen":row["GalacticNeverSeen"],"PossibleVariants":variants})
      }).collect();
    panel["VisualAlert"] = json!({"BodyName":panel["BodyName"],"BioCount":1,"Lines":lines});
    panel["TTLms"] = worker.settings["FSSNotificationTTL"]
        .as_u64()
        .unwrap_or(6000)
        .into();
    panel["ForceShow"] = json!(true);
    worker.settings = original;
    worker.message("StellarOverlay", "ShowBitmapNotification", panel);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_source_requirements_and_detail_values() {
        let rows = catalogue();
        assert_eq!(rows.len(), 117);
        assert_eq!(
            rows.iter()
                .map(|r| r["Requirements"].as_array().unwrap().len())
                .sum::<usize>(),
            663
        );
        let alcyoneum = rows
            .iter()
            .find(|r| r["SpeciesID"] == "Codex_Ent_Bacterial_06_Name")
            .unwrap();
        assert_eq!(alcyoneum["FullName"], "Bacterium Alcyoneum");
        assert_eq!(alcyoneum["FirstLoggedValue"], 8_292_500);
        assert!(alcyoneum["Requirements"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str().unwrap().contains("Pressure < 0.0133 Atm")));
        let horizon = rows
            .iter()
            .find(|r| r["SpeciesID"] == "Codex_Ent_Cone_Name")
            .unwrap();
        assert_eq!(horizon["FullName"], "Bark Mounds");
        assert!(!horizon["Description"].as_str().unwrap().is_empty());
    }

    #[test]
    fn variant_causes_preserve_star_groups_and_materials() {
        let rows = catalogue();
        let araneamus = rows
            .iter()
            .find(|r| r["FullName"] == "Stratum Araneamus")
            .unwrap();
        assert_eq!(
            araneamus["Variants"][0]["Cause"],
            "B star / A star / Neutron star / T dwarf"
        );
        let acies = rows
            .iter()
            .find(|r| r["FullName"] == "Bacterium Acies")
            .unwrap();
        assert_eq!(acies["VariantType"], "SurfaceMaterial");
        assert!(acies["Variants"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["Cause"] == "Antimony"));
        assert_eq!(araneamus["PossibleRegionMask"], ALL_REGIONS);
    }

    #[test]
    fn requirements_match_all_original_formatter_captures() {
        let source: Value = serde_json::from_str(include_str!(
            "../../../fixtures/bioinsights-companion-oracle.json"
        ))
        .unwrap();
        for row in catalogue() {
            let key = row["SpeciesID"].as_str().unwrap();
            let expected: Vec<_> = source["requirements_html"][key]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .unwrap()
                        .replace("<br />", "\n")
                        .replace("&nbsp;", " ")
                        .replace("&#x25CF;", "●")
                        .replace("&#x25CB;", "○")
                })
                .collect();
            assert_eq!(row["Requirements"], json!(expected), "{key}");
        }
    }

    #[test]
    fn original_zoom_range_and_explicit_overlay_test() {
        let mut worker =
            BioInsights::new(json!({"GalacticCodexRemote":false,"FSSNotificationTTL":6000}))
                .unwrap();
        worker.on_message("SetZoomLevel", &json!({"ZoomLevel":600}));
        assert_eq!(worker.settings["ZoomLevel"], 600);
        worker.on_message("SetZoomLevel", &json!({"ZoomLevel":10}));
        assert_eq!(worker.settings["ZoomLevel"], 50);
        let settings = worker.settings.clone();
        worker.on_message("TestOverlay",&json!({"DSS":true,"Settings":{"DSSNotificationXPos":30,"DSSNotificationYPos":40,"FSSNotificationTTL":4000}}));
        let message = worker
            .take_messages()
            .into_iter()
            .find(|m| m["command"] == "ShowBitmapNotification")
            .unwrap();
        assert_eq!(message["payload"]["Title"], "Graea Hypue AG-V d3-25 10 a");
        assert_eq!(message["payload"]["XPos"], 30);
        assert_eq!(message["payload"]["YPos"], 40);
        assert_eq!(message["payload"]["TTLms"], 4000);
        assert_eq!(message["payload"]["ForceShow"], true);
        assert_eq!(worker.settings, settings);
        assert!(worker.systems.is_empty());
    }

    #[test]
    fn wipe_requires_confirmation_and_uses_hecate_commander_contract() {
        let mut worker = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        worker.commander = "F12345".to_owned();
        worker.vault_unsold = vec![json!([
            7,
            "2026-10-25T10:00:00Z",
            "Reference 1",
            11,
            1658500,
            true
        ])];
        worker.on_message("WipeUnsoldData", &json!({}));
        assert_eq!(worker.vault_unsold.len(), 1);
        assert!(worker.take_messages().is_empty());
        worker.on_message("WipeUnsoldData", &json!({"Confirmed":true}));
        assert!(worker.vault_unsold.is_empty());
        let messages = worker.take_messages();
        assert_eq!(messages[0]["command"], "BioInsightsUnsoldDelete");
        assert_eq!(messages[0]["payload"]["FID"], "F12345");
        assert!(messages
            .iter()
            .any(|m| m["command"] == "BioInsightsGetData"));
        assert!(messages
            .iter()
            .any(|m| m["command"] == "BioInsightsGetTotal"));
        assert!(worker.unsold_data().is_empty());
    }

    #[test]
    fn current_system_population_does_not_leak_across_jumps() {
        let mut worker = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        for (event, id, population) in [
            ("Location", 1, Some(12345)),
            ("FSDJump", 2, None),
            ("CarrierJump", 3, Some(456)),
            ("Location", 4, Some(0)),
        ] {
            let mut value = json!({"timestamp":"2026-10-25T10:00:00Z","event":event,"SystemAddress":id,"StarSystem":format!("Reference {id}")});
            if let Some(population) = population {
                value["Population"] = json!(population);
            }
            worker.on_event(&JournalEvent::parse(&value.to_string()).unwrap());
            assert_eq!(worker.main_view()["Population"], population.unwrap_or(0));
        }
    }
}
