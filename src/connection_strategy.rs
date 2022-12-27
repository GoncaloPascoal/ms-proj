
use petgraph::{graphmap::GraphMap, Undirected};
use crate::model::Simulation;

fn add_adge(topology : &mut GraphMap<usize, f64, Undirected>, sim : &Simulation, a : usize, b : usize) {
    let pos_a = sim.satellites()[a].calc_position(sim.t());
    let pos_b = sim.satellites()[b].calc_position(sim.t());
    let length = (pos_a - pos_b).norm();

    topology.add_edge(a, b, length);
}

pub trait ConnectionStrategy: Send {
    fn run(&mut self, sim : &Simulation) -> GraphMap<usize, f64, Undirected>;
}

pub struct GridStrat {}

impl GridStrat{
    pub fn new() -> Self {
        GridStrat {}
    }
}

impl ConnectionStrategy for GridStrat {
    fn run(&mut self, sim : &Simulation) -> GraphMap<usize, f64, Undirected> {
        let mut topology: GraphMap<usize, f64, Undirected> = GraphMap::new();
        for sat in 0..sim.satellites().len() {
            topology.add_node(sat);
        }

        let sats = sim.satellites().len();
        let planes = sim.orbital_planes().len();
        let sats_per_plane = sats / planes;

        for plane in 0..planes {
            let start = plane * sats_per_plane;
            for sat in 0..(sats_per_plane - 1) {
                add_adge(&mut topology, sim, start + sat, start + sat + 1);
            }
            add_adge(&mut topology, sim, start, start + sats_per_plane - 1);
        }

        for sat in 0..sats_per_plane {
            for plane in 0..(planes - 1) {
                add_adge(&mut topology, sim, plane * sats_per_plane + sat, (plane + 1) * sats_per_plane + sat);
            }
            add_adge(&mut topology, sim, sat, (planes - 1) * sats_per_plane + sat);
        }

        topology
    }
}