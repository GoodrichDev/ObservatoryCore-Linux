//! Evaluator 1.3.444.637 pricing. Formula order and banker's rounding are significant.
pub fn body_k(body_type: &str, star: bool, terraform: bool) -> f64 {
    let name = body_type.to_ascii_lowercase();
    if star {
        if name.starts_with('d') {
            14057.0
        } else if name == "n" || name == "h" {
            22628.0
        } else if name == "supermassiveblackhole" {
            80.5
        } else {
            1200.0
        }
    } else {
        match (name.as_str(), terraform) {
            ("sudarsky class i gas giant", false) => 1656.0,
            ("sudarsky class i gas giant", true) => 0.0,
            ("sudarsky class ii gas giant" | "high metal content body", false) => 9654.0,
            ("sudarsky class ii gas giant" | "high metal content body", true) => 100677.0,
            ("earthlike body", false) => 181126.0,
            ("earthlike body", true) => 0.0,
            ("water world", false) => 64831.0,
            ("water world", true) => 116295.0,
            ("ammonia world", false) => 96932.0,
            ("ammonia world", true) => 0.0,
            ("metal rich body", false) => 21790.0,
            ("metal rich body", true) => 105678.0,
            (_, false) => 300.0,
            (_, true) => 93328.0,
        }
    }
}

pub fn star_value(k: f64, mass: f64, first_discovery: bool, carrier: bool) -> i64 {
    ((k + k * mass / 66.25)
        * if first_discovery { 2.6 } else { 1.0 }
        * if carrier { 0.75 } else { 1.0 })
    .round_ties_even() as i64
}

#[allow(clippy::too_many_arguments)]
pub fn planet_value(
    k: f64,
    mass: f64,
    first_discovery: bool,
    mapped: bool,
    first_mapped: bool,
    efficient: bool,
    odyssey: bool,
    carrier: bool,
) -> i64 {
    let multiplier = if !mapped {
        1.0
    } else if first_discovery && first_mapped {
        3.699622554
    } else if first_mapped {
        8.0956
    } else {
        3.3333333333
    };
    let mut value = (k + k * 0.56591828 * mass.powf(0.2)) * multiplier;
    if mapped {
        if odyssey {
            value += (value * 0.3).max(555.0);
        }
        if efficient {
            value *= 1.25;
        }
    }
    value = value.max(500.0);
    value *= if first_discovery { 2.6 } else { 1.0 };
    value *= if carrier { 0.75 } else { 1.0 };
    value.round_ties_even() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn installed_source_price_oracle() {
        let oracle: serde_json::Value =
            serde_json::from_str(include_str!("../../../fixtures/evaluator-prices.json")).unwrap();
        for entry in oracle["k"].as_array().unwrap() {
            let kind = entry["body_type"].as_str().unwrap();
            let star = entry["is_star"].as_bool().unwrap();
            assert_eq!(
                body_k(kind, star, false),
                entry["k"].as_f64().unwrap(),
                "{kind}"
            );
            assert_eq!(
                body_k(kind, star, true),
                entry["terraform_k"].as_f64().unwrap(),
                "terraform {kind}"
            );
        }
        for case in oracle["planets"].as_array().unwrap() {
            assert_eq!(
                planet_value(
                    case[0].as_f64().unwrap(),
                    case[1].as_f64().unwrap(),
                    case[2].as_bool().unwrap(),
                    case[3].as_bool().unwrap(),
                    case[4].as_bool().unwrap(),
                    case[5].as_bool().unwrap(),
                    case[6].as_bool().unwrap(),
                    case[7].as_bool().unwrap()
                ),
                case[8].as_i64().unwrap(),
                "{case}"
            );
        }
        for case in oracle["stars"].as_array().unwrap() {
            assert_eq!(
                star_value(
                    case[0].as_f64().unwrap(),
                    case[1].as_f64().unwrap(),
                    case[2].as_bool().unwrap(),
                    case[3].as_bool().unwrap()
                ),
                case[4].as_i64().unwrap(),
                "{case}"
            );
        }
    }
}
