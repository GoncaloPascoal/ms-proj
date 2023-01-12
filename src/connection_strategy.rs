
use petgraph::graphmap::GraphMap;
use crate::model::{Model, ConnectionGraph};

fn add_edge(topology: &mut ConnectionGraph, model: &Model, a: usize, b: usize) {
    let t = model.t();

    let sat_a = &model.satellites()[a];
    let sat_b = &model.satellites()[b];

    let pos_b = sat_b.calc_position(t);

    if sat_a.status() && sat_b.status() && sat_a.has_line_of_sight(t, &pos_b) {
        let pos_a = sat_a.calc_position(t);
        let length = (pos_a - pos_b).norm();
    
        topology.add_edge(a, b, length);
    }
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
        for sat in model.satellites() {
            if sat.status() {
                topology.add_node(sat.id());
            }
        }

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
