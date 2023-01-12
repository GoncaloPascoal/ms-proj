
use std::time::Instant;

use petgraph::graphmap::GraphMap;
use crate::model::{Model, ConnectionGraph};

fn is_edge_valid(topology: &ConnectionGraph, model: &Model, a: usize, b: usize) -> bool {
    let sat_a = &model.satellites()[a];
    let sat_b = &model.satellites()[b];

    let pos_b = sat_b.position();

    let both_alive = sat_a.status() && sat_b.status();
    let connections_available = topology.edges(a).count() < model.max_connections() && topology.edges(b).count() < model.max_connections();

    both_alive && connections_available && sat_a.has_line_of_sight(&pos_b)
}

fn add_edge(topology: &mut ConnectionGraph, model: &Model, a: usize, b: usize) {
    if !is_edge_valid(topology, model, a, b) {
        return;
    }

    let pos_a = model.satellites()[a].position();
    let pos_b = model.satellites()[b].position();

    let length = (pos_a - pos_b).norm();

    topology.add_edge(a, b, length);
}

pub trait ConnectionStrategy: Send {
    fn run(&mut self, model: &Model) -> ConnectionGraph;
}

pub struct GridStrategy;

impl GridStrategy {
    pub fn new() -> Self {
        GridStrategy
    }
}

impl ConnectionStrategy for GridStrategy {
    fn run(&mut self, model: &Model) -> ConnectionGraph {
        let mut topology = GraphMap::new();
        model.satellites().iter().filter(|s| s.status()).for_each(|s| {
            topology.add_node(s.id());
        });

        let sats = model.satellites().len();
        let planes = model.orbital_planes().len();
        let sats_per_plane = sats / planes;

        for plane in 0..planes {
            let start = plane * sats_per_plane;
            for sat in 0..(sats_per_plane - 1) {
                add_edge(&mut topology, model, start + sat, start + sat + 1);
            }
            add_edge(&mut topology, model, start, start + sats_per_plane - 1);
        }

        for sat in 0..sats_per_plane {
            for plane in 0..(planes - 1) {
                add_edge(&mut topology, model, plane * sats_per_plane + sat, (plane + 1) * sats_per_plane + sat);
            }
            add_edge(&mut topology, model, sat, (planes - 1) * sats_per_plane + sat);
        }

        topology
    }
}

pub struct NearestNeighborStrategy;

impl NearestNeighborStrategy {
    pub fn new() -> Self {
        NearestNeighborStrategy
    }
}

impl ConnectionStrategy for NearestNeighborStrategy {
    fn run(&mut self, model: &Model) -> ConnectionGraph {
        let instant = Instant::now();

        let mut topology: ConnectionGraph = GraphMap::new();
        let max_connections = model.max_connections();

        model.satellites().iter().filter(|s| s.status()).for_each(|s| {
            topology.add_node(s.id());
        });

        for sat in model.satellites() {
            let mut other_satellites: Vec<usize> = model.satellites().iter().filter_map(|s|
                if s.id() != sat.id() && is_edge_valid(&topology, model, sat.id(), s.id()) {
                    Some(s.id())
                } else {
                    None
                }
            ).collect();

            other_satellites.sort_by(|a, b| {
                let sat_a = &model.satellites()[*a];
                let sat_b = &model.satellites()[*b];

                model.distance_between_satellites(sat, sat_a)
                    .partial_cmp(&model.distance_between_satellites(sat, sat_b))
                    .unwrap()
            });

            for other in other_satellites {
                if topology.edges(sat.id()).count() == max_connections {
                    break;
                }
                add_edge(&mut topology, model, sat.id(), other);
            }
        }

        println!("{} ms", instant.elapsed().as_millis());

        topology
    }
}
