
use petgraph::{graphmap::GraphMap, Undirected};
use crate::model::Model;

fn add_edge(topology: &mut GraphMap<usize, f64, Undirected>, model: &Model, a: usize, b: usize) {
    let pos_a = model.satellites()[a].calc_position(model.t());
    let pos_b = model.satellites()[b].calc_position(model.t());
    let length = (pos_a - pos_b).norm();

    topology.add_edge(a, b, length);
}

pub trait ConnectionStrategy: Send {
    fn run(&mut self, model: &Model) -> GraphMap<usize, f64, Undirected>;
}

pub struct GridStrat;

impl GridStrat {
    pub fn new() -> Self {
        GridStrat
    }
}

impl ConnectionStrategy for GridStrat {
    fn run(&mut self, model: &Model) -> GraphMap<usize, f64, Undirected> {
        let mut topology = GraphMap::new();
        for sat in 0..model.satellites().len() {
            topology.add_node(sat);
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