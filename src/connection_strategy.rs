
use kiddo::{KdTree, distance::squared_euclidean};
use petgraph::graphmap::GraphMap;
use crate::model::{Model, ConnectionGraph};

fn is_edge_valid(topology: &ConnectionGraph, model: &Model, a: usize, b: usize) -> bool {
    let sat_a = &model.satellites()[a];
    let sat_b = &model.satellites()[b];

    let pos_b = sat_b.position();

    let both_alive = sat_a.status() && sat_b.status();
    let connections_available = topology.edges(a).count() < model.max_connections() && topology.edges(b).count() < model.max_connections();

    both_alive && connections_available && sat_a.has_line_of_sight(pos_b)
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

pub struct GridStrategy {
    offset: usize,
}

impl GridStrategy {
    pub fn new(offset: usize) -> Self {
        GridStrategy {
            offset,
        }
    }
}

impl ConnectionStrategy for GridStrategy {
    fn run(&mut self, model: &Model) -> ConnectionGraph {
        let mut topology = GraphMap::new();
        model.satellites().iter().filter(|s| s.status()).for_each(|s| {
            topology.add_node(s.id());
        });

        let num_sats = model.satellites().len();
        let num_planes = model.orbital_planes().len();
        let sats_per_plane = num_sats / num_planes;

        for plane in 0..num_planes {
            let start = plane * sats_per_plane;
            for sat in 0..sats_per_plane {
                add_edge(
                    &mut topology,
                    model,
                    start + sat,
                    start + (sat + 1) % sats_per_plane
                );
            }
        }

        for sat in 0..sats_per_plane {
            for plane in 0..num_planes {
                add_edge(
                    &mut topology,
                    model,
                    plane * sats_per_plane + sat,
                    ((plane + 1) % num_planes) * sats_per_plane + (sat + self.offset) % sats_per_plane
                );
            }
        }

        topology
    }
}

pub struct NearestNeighborStrategy {
    kd_tree: KdTree<f64, usize, 3>,
}

impl NearestNeighborStrategy {
    pub fn new() -> Self {
        NearestNeighborStrategy {
            kd_tree: KdTree::new(),
        }
    }
}

impl Default for NearestNeighborStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionStrategy for NearestNeighborStrategy {
    fn run(&mut self, model: &Model) -> ConnectionGraph {
        let mut topology: ConnectionGraph = GraphMap::new();
        let max_connections = model.max_connections();

        self.kd_tree = KdTree::new();

        model.satellites().iter().filter(|s| s.status()).for_each(|s| {
            topology.add_node(s.id());
            let _ = self.kd_tree.add(s.position().as_slice().try_into().unwrap(), s.id());
        });

        for sat in model.satellites() {
            let pos = sat.position().as_slice().try_into().unwrap();
            for other in self.kd_tree.iter_nearest(pos, &squared_euclidean).unwrap() {
                if topology.edges(sat.id()).count() == max_connections {
                    break;
                }
                add_edge(&mut topology, model, sat.id(), *other.1);
            }
        }

        topology
    }
}
