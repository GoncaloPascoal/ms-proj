
use json::{JsonValue, object};

use crate::model::Simulation;

pub fn init_msg(sim: &Simulation) -> String {
    let first_plane = sim.orbital_planes().get(0);

    let semimajor_axis = first_plane.map(|p| p.semimajor_axis()).unwrap_or(0.0);
    let inclination = first_plane.map(|p| p.inclination()).unwrap_or(0.0);

    let mut orbital_planes = JsonValue::new_array();
    for plane in sim.orbital_planes() {
        let _ = orbital_planes.push(object! {
            longitude: plane.longitude(),
        });
    }

    let mut satellites = JsonValue::new_array();
    for sat in sim.satellites() {
        let _ = satellites.push(object! {
            orbital_plane: sat.orbital_plane().id(),
            arg_periapsis: sat.arg_periapsis(),
        });
    }

    let obj = object! {
        msg_type: "init",
        semimajor_axis: semimajor_axis,
        inclination: inclination,
        simulation_speed: sim.simulation_speed(),
        orbital_planes: orbital_planes,
        satellites: satellites,
    };
    
    obj.dump()
}

pub fn update_msg(sim: &Simulation, include_connections: bool) -> String {
    let mut satellites = JsonValue::new_array();
    for sat in sim.satellites() {
        let _ = satellites.push(object! {
            position: sat.position().as_slice(),
            status: sat.status(),
        });
    }

    let mut obj = object! {
        msg_type: "update",
        t: sim.t(),
        satellites: satellites,
    };

    if include_connections {
        let connections: Vec<_> = sim.topology().all_edges().map(|(a, b, _)| vec![a, b]).collect();
        let _ = obj.insert("connections", connections);
    }

    obj.dump()
}