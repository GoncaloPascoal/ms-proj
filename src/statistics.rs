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

fn count_articulation_points(g: &ConnectionGraph) -> usize {
    let mut articulation_points = 0;

    let n_nodes = g.node_count();
    for node in 0..n_nodes {
        let mut g_copy = g.clone();
        g_copy.remove_node(node);
        if connected_components(&g_copy) > 1 {
            articulation_points += 1;
        }
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
