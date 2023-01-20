
use json::object;
use petgraph::algo::connected_components;

use crate::model::{ConnectionGraph, Simulation, GeoCoordinates};

fn round(x: f64, decimal_places: u32) -> f64 {
    let y = 10f64.powi(decimal_places as i32);
    (x * y).round() / y
}

pub fn statistics_msg(sim: &Simulation) -> String {
    let edge_count = sim.topology().edge_count() as f64;
    let node_count = sim.topology().node_count() as f64;

    let num_satellites = sim.satellites().len();
    let failed_satellites = sim.satellites().iter().filter(|s| !s.status()).count();
    let failure_ratio = failed_satellites as f64 / num_satellites as f64 * 100.0;

    let london       = GeoCoordinates::new(51.507222, -0.1275);
    let nyc          = GeoCoordinates::new(40.712778, -74.006111);
    let johannesburg = GeoCoordinates::new(-26.204444, 28.045556);
    let singapore    = GeoCoordinates::new(1.291667, 103.85);

    let rtt_london_nyc          = sim.calc_rtt(&london, &nyc         );
    let rtt_london_singapore    = sim.calc_rtt(&london, &singapore   );
    let rtt_london_johannesburg = sim.calc_rtt(&london, &johannesburg);
    let dist_london_nyc          = GeoCoordinates::haversine_distance(&london, &nyc         );
    let dist_london_singapore    = GeoCoordinates::haversine_distance(&london, &singapore   );
    let dist_london_johannesburg = GeoCoordinates::haversine_distance(&london, &johannesburg);

    let obj = object! {
        t: round(sim.t(), 3),
        connected_components: connected_components(sim.topology()),
        articulation_points: count_articulation_points(sim.satellites().len(), sim.topology()),
        graph_density: 2.0 * edge_count / (node_count * (node_count - 1.0)),
        active_connections: edge_count,
        failure_ratio: failure_ratio,
        rtt_nyc         : rtt_london_nyc.map(|rtt| rtt * 1e3),
        rtt_singapore   : rtt_london_singapore.map(|rtt| rtt * 1e3),
        rtt_johannesburg: rtt_london_johannesburg.map(|rtt| rtt * 1e3),
        latency_nyc         : rtt_london_nyc.map(|rtt| rtt / dist_london_nyc),
        latency_singapore   : rtt_london_singapore.map(|rtt| rtt / dist_london_singapore),
        latency_johannesburg: rtt_london_johannesburg.map(|rtt| rtt / dist_london_johannesburg),
    };

    obj.dump()
}

struct TarjanInformation {
    visited: Vec<bool>,
    depth: Vec<u32>,
    low: Vec<u32>,
    parent: Vec<Option<usize>>,
}

impl TarjanInformation {
    fn new(node_count: usize) -> Self {
        TarjanInformation {
            visited: vec![false; node_count],
            depth: vec![0; node_count],
            low: vec![0; node_count],
            parent: vec![None; node_count],
        }
    }
}

fn count_articulation_points(num_satellites: usize, g: &ConnectionGraph) -> usize {
    let mut articulation_points = 0;

    if let Some(root) = g.nodes().next() {
        let mut info = TarjanInformation::new(num_satellites);

        fn dfs(g: &ConnectionGraph, info: &mut TarjanInformation,
                articulation_points: &mut usize, idx: usize, d: u32) {
            info.visited[idx] = true;
            info.depth[idx] = d;
            info.low[idx] = d;

            let mut children = 0;
            let mut is_articulation = false;

            for n_idx in g.neighbors(idx) {
                if !info.visited[n_idx] {
                    info.parent[n_idx] = Some(idx);
                    dfs(g, info, articulation_points, n_idx, d + 1);
                    children += 1;
                    if info.low[n_idx] >= info.depth[idx] {
                        is_articulation = true;
                    }
                    info.low[idx] = u32::min(info.low[idx], info.low[n_idx]);
                }
                else {
                    let not_parent = match info.parent[idx] {
                        None => true,
                        Some(p_idx) => n_idx != p_idx,
                    };

                    if not_parent {
                        info.low[idx] = u32::min(info.low[idx], info.depth[n_idx]);
                    }
                }
            }

            is_articulation = match info.parent[idx] {
                None => children > 1,
                Some(_) => is_articulation,
            };

            if is_articulation {
                *articulation_points += 1;
            }
        }

        dfs(g, &mut info, &mut articulation_points, root, 0);
    }

    articulation_points
}
