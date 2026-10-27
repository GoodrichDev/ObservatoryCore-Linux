//! Journal orbit propagation and an open visiting route through unmapped targets.
use super::model::{Body, System};
use chrono::DateTime;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(super) type Point = [f64; 3];
pub(super) fn distance(a: Point, b: Point) -> f64 {
    a.into_iter()
        .zip(b)
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}
/// Installed SolveExact uses System.Numerics.Vector3.Distance: inputs and each
/// edge length narrow to float before Held-Karp accumulates the lengths in double.
fn source_distance(a: Point, b: Point) -> f64 {
    let x = a[0] as f32 - b[0] as f32;
    let y = a[1] as f32 - b[1] as f32;
    let z = a[2] as f32 - b[2] as f32;
    (x * x + y * y + z * z).sqrt() as f64
}
fn exact_order(points: &[Point]) -> Vec<usize> {
    let n = points.len();
    if n <= 1 {
        return (0..n).collect();
    }
    let states = 1usize << n;
    let mut costs = vec![f64::INFINITY; states * n];
    let mut previous = vec![0usize; states * n];
    costs[n] = 0.0;
    for mask in 0..states {
        for end in 0..n {
            let cost = costs[mask * n + end];
            if !cost.is_finite() {
                continue;
            }
            for next in 0..n {
                if mask & (1 << next) != 0 {
                    continue;
                }
                let next_mask = mask | (1 << next);
                let total = cost + source_distance(points[end], points[next]);
                let slot = next_mask * n + next;
                if total < costs[slot] {
                    costs[slot] = total;
                    previous[slot] = end;
                }
            }
        }
    }
    let full = states - 1;
    let mut last = 0usize;
    let mut best = f64::INFINITY;
    for end in 1..n {
        if costs[full * n + end] < best {
            best = costs[full * n + end];
            last = end;
        }
    }
    let mut order = Vec::with_capacity(n);
    let mut mask = full;
    loop {
        order.push(last);
        if mask == 1 {
            break;
        }
        let predecessor = previous[mask * n + last];
        mask ^= 1 << last;
        last = predecessor;
    }
    order.reverse();
    order
}
fn seconds(v: &Value) -> f64 {
    v.as_str()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map_or(0.0, |t| t.timestamp_millis() as f64 / 1000.0)
}
fn local(scan: &Value, at: f64) -> Point {
    let n = |key: &str| scan[key].as_f64().unwrap_or(0.0);
    let axis = n("SemiMajorAxis") / 1000.0;
    let period = n("OrbitalPeriod");
    if axis <= 0.0 || period <= 0.0 {
        return [0.0; 3];
    }
    let eccentricity = n("Eccentricity").clamp(0.0, 0.99999999);
    let mean = n("MeanAnomaly").to_radians()
        + (at - seconds(&scan["timestamp"])).rem_euclid(period) / period * std::f64::consts::TAU;
    let mut anomaly = mean + eccentricity * mean.sin() * (1.0 + eccentricity * mean.cos());
    for _ in 0..32 {
        let next = anomaly
            - (anomaly - eccentricity * anomaly.sin() - mean)
                / (1.0 - eccentricity * anomaly.cos());
        if (next - anomaly).abs() < 1e-9 {
            anomaly = next;
            break;
        }
        anomaly = next;
    }
    let x = axis * (anomaly.cos() - eccentricity);
    let y = axis * anomaly.sin() * (1.0 - eccentricity.powi(2)).sqrt();
    let node = -n("AscendingNode").to_radians();
    let inclination = -n("OrbitalInclination").to_radians();
    let peri = std::f64::consts::TAU - n("Periapsis").to_radians();
    let (sn, cn) = node.sin_cos();
    let (si, ci) = inclination.sin_cos();
    let (sp, cp) = peri.sin_cos();
    [
        x * (cp * cn - sp * ci * sn) - y * (sp * cn + cp * ci * sn),
        x * (cp * sn + sp * ci * cn) + y * (cp * ci * cn - sp * sn),
        x * sp * si + y * cp * si,
    ]
}
fn position(id: u64, system: &System, at: f64, seen: &mut BTreeSet<u64>) -> Point {
    if !seen.insert(id) {
        return [0.0; 3];
    }
    let scan = system
        .orbits
        .get(&id)
        .or_else(|| system.bodies.get(&id).map(|b| &b.orbit));
    let Some(scan) = scan else {
        return [0.0; 3];
    };
    let mut point = local(scan, at);
    if let Some(parent) = scan["Parents"]
        .as_array()
        .and_then(|v| v.first())
        .and_then(Value::as_object)
        .and_then(|v| v.values().next())
        .and_then(Value::as_u64)
    {
        let parent = position(parent, system, at, seen);
        for i in 0..3 {
            point[i] += parent[i];
        }
    }
    point
}
pub fn update(system: &mut System, settings: &Value, time: &str) {
    update_with_cancel(
        system,
        settings,
        time,
        &std::sync::atomic::AtomicBool::new(false),
    );
}
pub(super) fn update_with_cancel(
    system: &mut System,
    settings: &Value,
    time: &str,
    cancel: &std::sync::atomic::AtomicBool,
) {
    let at = seconds(&Value::String(time.into()));
    let mut ids = system.body_order.clone();
    for id in system.bodies.keys() {
        if !ids.contains(id) {
            ids.push(*id);
        }
    }
    let candidates: Vec<_> = ids
        .into_iter()
        .filter(|id| {
            system
                .bodies
                .get(id)
                .is_some_and(|b| !b.visited() && (b.worth(settings) || b.worth_visiting()))
        })
        .collect();
    let points: BTreeMap<_, _> = candidates
        .iter()
        .map(|&id| (id, position(id, system, at, &mut BTreeSet::new())))
        .collect();
    let start = system
        .from
        .or(system.main)
        .map(|id| position(id, system, at, &mut BTreeSet::new()))
        .unwrap_or([0.0; 3]);
    let input: Vec<_> = std::iter::once(start)
        .chain(candidates.iter().map(|id| points[id]))
        .collect();
    let order = if candidates.len() < 13 {
        exact_order(&input)
    } else {
        super::solver::solve(&input, cancel)
    };
    let route = order
        .into_iter()
        .skip(1)
        .map(|index| candidates[index - 1])
        .collect::<Vec<_>>();
    for body in system.bodies.values_mut() {
        body.order = 0;
    }
    for (index, id) in route.iter().enumerate() {
        if let Some(Body { order, .. }) = system.bodies.get_mut(id) {
            *order = index + 1;
        }
    }
    system.route = route;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn circular_orbit_quarter_period() {
        let scan = json!({"timestamp":"2026-01-01T00:00:00Z","SemiMajorAxis":1000000,"OrbitalPeriod":400,"MeanAnomaly":0});
        let p = local(&scan, seconds(&scan["timestamp"]) + 100.0);
        assert!(p[0].abs() < 1e-8);
        assert!((p[1] - 1000.0).abs() < 1e-8);
    }
    #[test]
    fn exact_route_order_matches_all_original_solver_captures() {
        let oracle: Value = serde_json::from_str(include_str!(
            "../../../fixtures/evaluator-route-oracle.json"
        ))
        .unwrap();
        for case in oracle["cases"].as_array().unwrap() {
            let points: Vec<Point> = case["points"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| {
                    [
                        p[0].as_f64().unwrap(),
                        p[1].as_f64().unwrap(),
                        p[2].as_f64().unwrap(),
                    ]
                })
                .collect();
            let order = exact_order(&points);
            let expected: Vec<usize> = case["order"]
                .as_array()
                .unwrap()
                .iter()
                .map(|i| i.as_u64().unwrap() as usize)
                .collect();
            assert_eq!(order, expected, "{} source cities", points.len());
            let length: f64 = order
                .windows(2)
                .map(|p| source_distance(points[p[0]], points[p[1]]))
                .sum();
            assert!((length - case["length"].as_f64().unwrap()).abs() < 0.002);
        }
    }
    #[test]
    fn original_orrery_relative_position_oracle() {
        let oracle: Value =
            serde_json::from_str(include_str!("../../../fixtures/evaluator-prices.json")).unwrap();
        for case in oracle["orbits"].as_array().unwrap() {
            let scan = json!({"timestamp":"2026-01-01T00:00:00Z","SemiMajorAxis":123456789,"OrbitalPeriod":50000,
                "Periapsis":case[0],"OrbitalInclination":case[1],"AscendingNode":case[2],"MeanAnomaly":case[3],"Eccentricity":case[4]});
            let point = local(
                &scan,
                seconds(&scan["timestamp"]) + case[5].as_f64().unwrap(),
            );
            for axis in 0..3 {
                assert!(
                    (point[axis] - case[axis + 6].as_f64().unwrap()).abs() < 0.0001,
                    "axis {axis} {point:?} source {case}"
                );
            }
        }
    }
}
