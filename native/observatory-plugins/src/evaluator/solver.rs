//! Free-end route search from installed Tsp3DSolver.Engine.
//! Candidate edges, degree-two greedy construction, 2-opt and 1..3 Or-opt,
//! double-bridge perturbation, MST bound, 2% gap and 200-step stagnation.
use super::routing::Point;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
#[cfg(test)]
std::thread_local! {static TRACE:std::cell::RefCell<Vec<serde_json::Value>>=const{std::cell::RefCell::new(Vec::new())};}
struct Graph {
    n: usize,
    d: Vec<f64>,
    neighbors: Vec<Vec<usize>>,
    penalty: f64,
}
impl Graph {
    fn edge(&self, a: usize, b: usize) -> f64 {
        self.d[a * self.n + b]
    }
    fn new(input: &[Point]) -> Self {
        let real = input.len();
        let n = real + 1;
        let mut lo = [f64::INFINITY; 3];
        let mut hi = [f64::NEG_INFINITY; 3];
        let points: Vec<Point> = input.iter().map(|p| p.map(|v| v as f32 as f64)).collect();
        for p in &points {
            for i in 0..3 {
                lo[i] = lo[i].min(p[i]);
                hi[i] = hi[i].max(p[i]);
            }
        }
        let diagonal = (0..3).map(|i| (hi[i] - lo[i]).powi(2)).sum::<f64>().sqrt();
        let penalty = 2.0 * real as f64 * diagonal + 1.0;
        let mut d = vec![0.0; n * n];
        for a in 0..n {
            for b in 0..a {
                let value = if a == real {
                    if b == 0 {
                        0.0
                    } else {
                        penalty
                    }
                } else {
                    super::routing::distance(points[a], points[b])
                };
                d[a * n + b] = value;
                d[b * n + a] = value;
            }
        }
        let mut neighbors = Vec::with_capacity(n);
        for a in 0..real {
            let mut nearest: Vec<usize> = (0..real).filter(|b| *b != a).collect();
            nearest.sort_by(|a1, b1| d[a * n + *a1].total_cmp(&d[a * n + *b1]));
            nearest.truncate(12);
            if a == 0 {
                nearest.push(real);
            }
            neighbors.push(nearest);
        }
        neighbors.push(vec![0]);
        Self {
            n,
            d,
            neighbors,
            penalty,
        }
    }
    fn length(&self, tour: &[usize]) -> f64 {
        let dummy = self.n - 1;
        let mut connected_start = false;
        let mut total = 0.0;
        for i in 0..tour.len() {
            let a = tour[i];
            let b = tour[(i + 1) % tour.len()];
            if a == dummy || b == dummy {
                if a == 0 || b == 0 {
                    connected_start = true;
                }
            } else {
                total += self.edge(a, b);
            }
        }
        if connected_start {
            total
        } else {
            total + self.penalty
        }
    }
    fn lower_bound(&self) -> f64 {
        let mut seen = vec![false; self.n];
        let mut best = vec![f64::INFINITY; self.n];
        best[0] = 0.0;
        let mut total = 0.0;
        for _ in 0..self.n {
            let a = (0..self.n)
                .filter(|i| !seen[*i])
                .min_by(|a, b| best[*a].total_cmp(&best[*b]))
                .unwrap();
            seen[a] = true;
            total += best[a];
            for b in 0..self.n {
                if !seen[b] {
                    best[b] = best[b].min(self.edge(a, b));
                }
            }
        }
        total
    }
    fn greedy(&self) -> Vec<usize> {
        fn root(parent: &mut [usize], mut a: usize) -> usize {
            while parent[a] != a {
                parent[a] = parent[parent[a]];
                a = parent[a];
            }
            a
        }
        fn connect(adj: &mut [Vec<usize>], parent: &mut [usize], a: usize, b: usize) {
            adj[a].push(b);
            adj[b].push(a);
            let ra = root(parent, a);
            let rb = root(parent, b);
            parent[ra] = rb;
        }
        let mut edges = std::collections::BTreeSet::new();
        for a in 0..self.n {
            for &b in &self.neighbors[a] {
                edges.insert((a.min(b), a.max(b)));
            }
        }
        let mut edges: Vec<_> = edges.into_iter().collect();
        edges.sort_by(|&(a, b), &(c, d)| {
            let weight = |x, y| {
                if x == 0 && y == self.n - 1 {
                    f64::NEG_INFINITY
                } else {
                    self.edge(x, y)
                }
            };
            weight(a, b).total_cmp(&weight(c, d))
        });
        let mut adj = vec![Vec::new(); self.n];
        let mut parent: Vec<usize> = (0..self.n).collect();
        let mut connected = 0;
        for (a, b) in edges {
            if adj[a].len() < 2 && adj[b].len() < 2 && root(&mut parent, a) != root(&mut parent, b)
            {
                connect(&mut adj, &mut parent, a, b);
                connected += 1;
                if connected == self.n - 1 {
                    break;
                }
            }
        }
        while connected < self.n - 1 {
            let mut pair = None;
            let mut length = f64::INFINITY;
            for a in 0..self.n {
                for b in a + 1..self.n {
                    if adj[a].len() < 2
                        && adj[b].len() < 2
                        && root(&mut parent, a) != root(&mut parent, b)
                        && self.edge(a, b) < length
                    {
                        length = self.edge(a, b);
                        pair = Some((a, b));
                    }
                }
            }
            let (a, b) = pair.expect("disconnected path endpoints");
            connect(&mut adj, &mut parent, a, b);
            connected += 1;
        }
        let ends: Vec<usize> = (0..self.n).filter(|a| adj[*a].len() < 2).collect();
        connect(&mut adj, &mut parent, ends[0], ends[1]);
        let mut tour = Vec::with_capacity(self.n);
        let mut previous = usize::MAX;
        let mut next = 0;
        for _ in 0..self.n {
            tour.push(next);
            let following = *adj[next].iter().find(|i| **i != previous).unwrap();
            previous = next;
            next = following;
        }
        tour
    }
    fn improve(&self, tour: &mut Vec<usize>, cancel: &AtomicBool, _deadline: Instant) {
        use std::collections::VecDeque;
        let mut pos = vec![0; self.n];
        let mut queue: VecDeque<usize> = tour.iter().copied().collect();
        let mut queued = vec![true; self.n];
        for (i, &city) in tour.iter().enumerate() {
            pos[city] = i;
        }
        let enqueue = |city: usize, queue: &mut VecDeque<usize>, queued: &mut [bool]| {
            if !queued[city] {
                queued[city] = true;
                queue.push_back(city);
            }
        };
        for _ in 0..300 * self.n + 20000 {
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            let Some(c1) = queue.pop_front() else {
                return;
            };
            queued[c1] = false;
            let prev = tour[(pos[c1] + self.n - 1) % self.n];
            let next = tour[(pos[c1] + 1) % self.n];
            let mut changed = false;
            'two: for direction in 0..2 {
                let adjacent = if direction == 0 { prev } else { next };
                let old = self.edge(c1, adjacent);
                for &c2 in &self.neighbors[c1] {
                    if c2 == c1 || c2 == adjacent {
                        continue;
                    }
                    if self.edge(c1, c2) >= old {
                        break;
                    }
                    let other = tour[if direction == 0 {
                        (pos[c2] + self.n - 1) % self.n
                    } else {
                        (pos[c2] + 1) % self.n
                    }];
                    if other == c1 || other == c2 {
                        continue;
                    }
                    if old + self.edge(c2, other) - (self.edge(c1, c2) + self.edge(adjacent, other))
                        <= 1e-10
                    {
                        continue;
                    }
                    let (mut start, end) = if direction == 0 {
                        (pos[c2], pos[adjacent])
                    } else {
                        (pos[adjacent], pos[c2])
                    };
                    #[cfg(test)]
                    TRACE.with(|trace| {
                        trace
                            .borrow_mut()
                            .push(serde_json::json!({"op":"two","i":start,"j":end,"tour":tour}))
                    });
                    let mut length = (end + self.n - start) % self.n + 1;
                    if length * 2 > self.n {
                        start = (end + 1) % self.n;
                        length = self.n - length;
                    }
                    for offset in 0..length / 2 {
                        tour.swap(
                            (start + offset) % self.n,
                            (start + length - 1 - offset) % self.n,
                        );
                    }
                    for (i, &city) in tour.iter().enumerate() {
                        pos[city] = i;
                    }
                    for city in [c1, adjacent, c2, other] {
                        enqueue(city, &mut queue, &mut queued);
                    }
                    changed = true;
                    break 'two;
                }
            }
            if !changed {
                'or: for length in 1..=3 {
                    if length >= self.n - 2 {
                        break;
                    }
                    let start = pos[c1];
                    let last = tour[(start + length - 1) % self.n];
                    let before = tour[(start + self.n - 1) % self.n];
                    let after = tour[(start + length) % self.n];
                    if before == last || after == c1 || before == after {
                        continue;
                    }
                    let removal =
                        self.edge(before, c1) + self.edge(last, after) - self.edge(before, after);
                    if removal <= 1e-10 {
                        continue;
                    }
                    for neighbors in [&self.neighbors[c1], &self.neighbors[last]] {
                        for &insert in neighbors {
                            let successor = tour[(pos[insert] + 1) % self.n];
                            let in_segment =
                                |city: usize| (pos[city] + self.n - start) % self.n < length;
                            if insert == before || in_segment(insert) || in_segment(successor) {
                                continue;
                            }
                            let forward = removal
                                - (self.edge(insert, c1) + self.edge(last, successor)
                                    - self.edge(insert, successor));
                            let reverse = removal
                                - (self.edge(insert, last) + self.edge(c1, successor)
                                    - self.edge(insert, successor));
                            let reversed = reverse > forward;
                            if (if reversed { reverse } else { forward }) <= 1e-10 {
                                continue;
                            }
                            #[cfg(test)]TRACE.with(|trace|trace.borrow_mut().push(serde_json::json!({"op":"or","segStartPos":start,"segLen":length,"insertAfterCity":insert,"reversed":reversed,"tour":tour})));
                            let mut segment: Vec<_> = (0..length)
                                .map(|offset| tour[(start + offset) % self.n])
                                .collect();
                            if reversed {
                                segment.reverse();
                            }
                            let mut result = Vec::with_capacity(self.n);
                            for offset in 0..self.n - length {
                                let city = tour[(start + length + offset) % self.n];
                                result.push(city);
                                if city == insert {
                                    result.extend_from_slice(&segment);
                                }
                            }
                            *tour = result;
                            for (i, &city) in tour.iter().enumerate() {
                                pos[city] = i;
                            }
                            for city in [before, after, insert, successor, c1, last] {
                                enqueue(city, &mut queue, &mut queued);
                            }
                            changed = true;
                            break 'or;
                        }
                    }
                }
            }
            if changed {
                enqueue(c1, &mut queue, &mut queued);
            }
        }
    }
    fn order(&self, tour: &[usize]) -> Vec<usize> {
        let start = tour.iter().position(|id| *id == 0).unwrap();
        let backwards = tour[(start + 1) % self.n] == self.n - 1;
        (0..self.n)
            .map(|offset| {
                tour[if backwards {
                    (start + self.n - offset) % self.n
                } else {
                    (start + offset) % self.n
                }]
            })
            .filter(|city| *city != self.n - 1)
            .collect()
    }
}
// System.Random(int) keeps the compatibility subtractive generator. Seeded
// restart perturbations are captured independently from the original class.
struct Random {
    seed: [i32; 56],
    next: usize,
    other: usize,
}
impl Random {
    fn new(seed: u64) -> Self {
        let value = seed as i32;
        let subtraction = if value == i32::MIN {
            i32::MAX
        } else {
            value.abs()
        };
        let mut data = [0i32; 56];
        let mut mj = 161803398i32.wrapping_sub(subtraction);
        data[55] = mj;
        let mut mk = 1i32;
        for i in 1..55 {
            let index = (21 * i) % 55;
            data[index] = mk;
            mk = mj.wrapping_sub(mk);
            if mk < 0 {
                mk = mk.wrapping_add(i32::MAX);
            }
            mj = data[index];
        }
        for _ in 0..4 {
            for i in 1..56 {
                data[i] = data[i].wrapping_sub(data[1 + (i + 30) % 55]);
                if data[i] < 0 {
                    data[i] = data[i].wrapping_add(i32::MAX);
                }
            }
        }
        Self {
            seed: data,
            next: 0,
            other: 21,
        }
    }
    fn next(&mut self, max: usize) -> usize {
        self.next += 1;
        if self.next >= 56 {
            self.next = 1;
        }
        self.other += 1;
        if self.other >= 56 {
            self.other = 1;
        }
        let mut value = self.seed[self.next].wrapping_sub(self.seed[self.other]);
        if value == i32::MAX {
            value -= 1;
        }
        if value < 0 {
            value = value.wrapping_add(i32::MAX);
        }
        self.seed[self.next] = value;
        (value as f64 * (1.0 / i32::MAX as f64) * max as f64) as usize
    }
}
fn double_bridge(tour: &[usize], rng: &mut Random) -> Vec<usize> {
    let n = tour.len();
    let a = 1 + rng.next(n - 3);
    let b = a + 1 + rng.next(n - a - 2);
    let c = b + 1 + rng.next(n - b - 1);
    let mut kicked = Vec::with_capacity(n);
    kicked.extend_from_slice(&tour[..a]);
    kicked.extend_from_slice(&tour[b..c]);
    kicked.extend_from_slice(&tour[a..b]);
    kicked.extend_from_slice(&tour[c..]);
    kicked
}
fn chain(graph: &Graph, seed: u64, cancel: &AtomicBool, budget: Duration) -> (Vec<usize>, f64) {
    let mut tour = graph.greedy();
    let deadline = Instant::now() + budget;
    graph.improve(&mut tour, cancel, deadline);
    let mut best = tour.clone();
    let mut length = graph.length(&best);
    let bound = graph.lower_bound();
    let mut stagnant = 0;
    let mut rng = Random::new(seed);
    while !cancel.load(Ordering::Relaxed)
        && Instant::now() < deadline
        && stagnant < 200
        && (bound <= 1e-9 || (length - bound) / bound * 100.0 > 2.0)
    {
        if graph.n < 8 {
            break;
        }
        tour.clone_from(&best);
        tour = double_bridge(&tour, &mut rng);
        graph.improve(&mut tour, cancel, deadline);
        let value = graph.length(&tour);
        if value < length - 1e-9 {
            best.clone_from(&tour);
            length = value;
            stagnant = 0;
        } else {
            stagnant += 1;
        }
    }
    (graph.order(&best), length)
}
pub(super) fn solve(points: &[Point], cancel: &AtomicBool) -> Vec<usize> {
    let graph = Graph::new(points);
    let budget = Duration::from_secs(2);
    let workers = std::thread::available_parallelism().map_or(1, usize::from);
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(1, |d| d.as_nanos() as u64);
    std::thread::scope(|scope| {
        let mut jobs = Vec::new();
        for worker in 1..workers {
            let graph = &graph;
            jobs.push(scope.spawn(move || {
                chain(
                    graph,
                    seed.wrapping_add(worker as u64 * 7919 + 17),
                    cancel,
                    budget,
                )
            }));
        }
        let mut best = chain(
            &graph,
            if workers > 1 {
                seed.wrapping_add(17)
            } else {
                seed
            },
            cancel,
            budget,
        );
        for job in jobs {
            if let Ok(candidate) = job.join() {
                if candidate.1 < best.1 {
                    best = candidate;
                }
            }
        }
        best.0
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_fixed_iteration_traces_match_all_128_candidate_and_best_routes() {
        let oracle: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/evaluator-large-route-oracle.json"
        ))
        .unwrap();
        for (case, primitive) in oracle["cases"]
            .as_array()
            .unwrap()
            .iter()
            .zip(oracle["kicks"].as_array().unwrap())
        {
            let points: Vec<Point> = case["points"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| {
                    [
                        v[0].as_f64().unwrap(),
                        v[1].as_f64().unwrap(),
                        v[2].as_f64().unwrap(),
                    ]
                })
                .collect();
            let graph = Graph::new(&points);
            let mut best: Vec<usize> = primitive["optimized"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap() as usize)
                .collect();
            let mut best_length = graph.length(&best);
            assert!((best_length - primitive["initialLength"].as_f64().unwrap()).abs() < 1e-7);
            let mut rng = Random::new(case["seed"].as_u64().unwrap());
            for (iteration, step) in primitive["trace"].as_array().unwrap().iter().enumerate() {
                let mut candidate = double_bridge(&best, &mut rng);
                TRACE.with(|t| t.borrow_mut().clear());
                graph.improve(
                    &mut candidate,
                    &AtomicBool::new(false),
                    Instant::now() + Duration::from_secs(5),
                );
                let actual_ops = TRACE.with(|t| t.borrow().clone());
                let source_ops = step["ops"].as_array().unwrap();
                for (i, (actual, source)) in actual_ops.iter().zip(source_ops).enumerate() {
                    assert_eq!(
                        actual, source,
                        "ops seed{} iteration{iteration} move{i}",
                        case["seed"]
                    );
                }
                assert_eq!(
                    actual_ops.len(),
                    source_ops.len(),
                    "move count seed{} iteration{iteration}",
                    case["seed"]
                );
                let expected: Vec<usize> = step["candidate"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_u64().unwrap() as usize)
                    .collect();
                assert_eq!(
                    candidate, expected,
                    "seed{} iteration{iteration}",
                    case["seed"]
                );
                let length = graph.length(&candidate);
                assert!((length - step["length"].as_f64().unwrap()).abs() < 1e-7);
                if length < best_length - 1e-9 {
                    best = candidate;
                    best_length = length;
                }
                assert!((best_length - step["bestLength"].as_f64().unwrap()).abs() < 1e-7);
            }
        }
    }
    #[test]
    fn source_greedy_mst_and_local_pass_captures() {
        let oracle: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/evaluator-large-route-oracle.json"
        ))
        .unwrap();
        for (case, primitive) in oracle["cases"]
            .as_array()
            .unwrap()
            .iter()
            .zip(oracle["kicks"].as_array().unwrap())
        {
            let points: Vec<Point> = case["points"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| {
                    [
                        v[0].as_f64().unwrap(),
                        v[1].as_f64().unwrap(),
                        v[2].as_f64().unwrap(),
                    ]
                })
                .collect();
            let graph = Graph::new(&points);
            assert_eq!(
                graph.neighbors,
                primitive["neighbors"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|row| row
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_u64().unwrap() as usize)
                        .collect::<Vec<_>>())
                    .collect::<Vec<_>>()
            );
            assert!((graph.lower_bound() - primitive["bound"].as_f64().unwrap()).abs() < 1e-7);
            let original: Vec<usize> = primitive["greedy"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap() as usize)
                .collect();
            assert_eq!(graph.greedy(), original, "greedy seed{}", case["seed"]);
            let mut tour = original;
            graph.improve(
                &mut tour,
                &AtomicBool::new(false),
                Instant::now() + Duration::from_secs(5),
            );
            let expected: Vec<usize> = primitive["optimized"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap() as usize)
                .collect();
            assert_eq!(tour, expected, "local pass seed{}", case["seed"]);
        }
    }
    #[test]
    fn seeded_random_and_double_bridge_match_original_engine() {
        let oracle: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/evaluator-large-route-oracle.json"
        ))
        .unwrap();
        for case in oracle["randomSeeds"].as_array().unwrap() {
            let mut random = Random::new(case["seed"].as_i64().unwrap() as u64);
            for (i, value) in case["values"].as_array().unwrap().iter().enumerate() {
                assert_eq!(
                    random.next((i + 1) * 1000),
                    value.as_u64().unwrap() as usize,
                    "seed{}",
                    case["seed"]
                );
            }
        }
        let mut random = Random::new(1234);
        for (i, value) in oracle["random"].as_array().unwrap().iter().enumerate() {
            assert_eq!(
                random.next((i + 1) * 1000),
                value.as_u64().unwrap() as usize
            );
        }
        for case in oracle["kicks"].as_array().unwrap() {
            let tour: Vec<_> = (0..case["n"].as_u64().unwrap() as usize).collect();
            assert_eq!(
                double_bridge(&tour, &mut Random::new(case["seed"].as_u64().unwrap())),
                case["tour"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_u64().unwrap() as usize)
                    .collect::<Vec<_>>()
            );
        }
    }
    #[test]
    fn complete_seeded_chains_match_original_orders_and_costs() {
        let fixtures: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/evaluator-large-route-oracle.json"
        ))
        .unwrap();
        for case in fixtures["cases"].as_array().unwrap() {
            let points: Vec<Point> = case["points"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| {
                    [
                        v[0].as_f64().unwrap(),
                        v[1].as_f64().unwrap(),
                        v[2].as_f64().unwrap(),
                    ]
                })
                .collect();
            let graph = Graph::new(&points);
            let (order, length) = chain(
                &graph,
                case["seed"].as_u64().unwrap(),
                &AtomicBool::new(false),
                Duration::from_secs(2),
            );
            if case["hitTimeLimit"] == false {
                assert_eq!(
                    order,
                    case["order"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_u64().unwrap() as usize)
                        .collect::<Vec<_>>(),
                    "full chain seed{}",
                    case["seed"]
                );
            }
            assert_eq!(order[0], 0);
            let mut ids = order.clone();
            ids.sort_unstable();
            assert_eq!(ids, (0..points.len()).collect::<Vec<_>>());
            assert!(
                (length - case["length"].as_f64().unwrap()).abs() < 1e-7,
                "{} cities native{length} source{}",
                points.len(),
                case["length"]
            );
        }
    }
}
