use std::collections::HashMap;

use json::object;
use petgraph::{algo::{connected_components, dijkstra}, visit::EdgeRef};

use crate::model::{ConnectionGraph, Simulation};

pub fn statistics_msg(sim: &Simulation) -> String {
    let diameter_and_average = calculate_diameter_and_average(&sim.topology());

    let edge_count = sim.topology().edge_count() as f64;
    let node_count = sim.topology().node_count() as f64;

    let obj = object! {
        t: sim.t(),
        connected_components: connected_components(&sim.topology()),
        articulation_points: count_articulation_points(&sim.topology()),
        graph_density: 2.0 * edge_count / (node_count * (node_count - 1.0)),
        d_graph_diameter: diameter_and_average.0,
        d_average_distance: diameter_and_average.1,
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

fn count_articulation_points(g: &ConnectionGraph) -> usize {
    let mut articulation_points = 0;

    if let Some(root) = g.nodes().next() {
        let mut info = TarjanInformation::new(g.node_count());

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

fn calculate_diameter_and_average(g: &ConnectionGraph) -> (f64, f64) {
    let mut diameter = 0.0;
    let mut average = 0.0;

    for source in g.nodes() { 
        let distances = dijkstra(
            &g,
            source,
            None,
            |e| *e.weight(),
        );
        for (_, distance) in distances {
            diameter = f64::max(diameter, distance);
            average += distance;
        }
    }
    average /= g.edge_count() as f64;
   
    (diameter, average)
}
