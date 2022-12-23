
use std::{f64::consts::PI, rc::Rc};

use json::{object, JsonValue};
use nalgebra::{Vector3, Rotation3};

pub const GM: f64 = 3.986004418e14;
pub const EARTH_RADIUS: f64 = 6.371e6;

struct OrbitalPlane {
    id: usize,
    semimajor_axis: f64,
    inclination: f64,
    longitude: f64,
    // Calculated fields
    orbital_speed: f64,
    angular_speed: f64,
}

impl OrbitalPlane {
    fn new(id: usize, semimajor_axis: f64, inclination: f64, longitude: f64) -> Self {
        let orbital_speed = f64::sqrt(GM / semimajor_axis);

        OrbitalPlane {
            id,
            semimajor_axis,
            inclination,
            longitude,
            orbital_speed,
            angular_speed: orbital_speed / semimajor_axis,
        }
    }
}

struct Satellite {
    id: usize,
    orbital_plane: Rc<OrbitalPlane>,
    arg_periapsis: f64,
}

impl Satellite {
    fn new(id: usize, orbital_plane: Rc<OrbitalPlane>, arg_periapsis: f64) -> Self {
        Satellite {
            id,
            orbital_plane,
            arg_periapsis,
        }
    }

    fn calc_position(&self, t: f64) -> Vector3<f64> {
        let r = self.orbital_plane.semimajor_axis;
        let true_anomaly = (t * self.orbital_plane.angular_speed) % (2.0 * PI);

        let position = Vector3::new(r, 0.0, 0.0);

        Rotation3::from_euler_angles(0.0, self.orbital_plane.longitude, 0.0) *
        Rotation3::from_euler_angles(self.orbital_plane.inclination, 0.0, 0.0) *
        Rotation3::from_euler_angles(0.0, self.arg_periapsis + true_anomaly, 0.0) *
        position
    }

    fn calc_velocity(&self, t: f64) -> Vector3<f64> {
        let direction = Rotation3::from_axis_angle(&Vector3::y_axis(), PI / 2.0) * self.calc_position(t).normalize();

        self.orbital_plane.orbital_speed * direction
    }
}

pub struct Simulation {
    orbital_planes: Vec<Rc<OrbitalPlane>>,
    satellites: Vec<Satellite>,
    time_step: f64,
    t: f64,
}

impl Simulation {
    pub fn new(num_orbital_planes: usize, satellites_per_plane: usize, inclination: f64, semimajor_axis: f64, time_step: f64) -> Self {
        let mut orbital_planes = Vec::with_capacity(num_orbital_planes);
        let mut satellites = Vec::with_capacity(num_orbital_planes * satellites_per_plane);

        for i in 0..num_orbital_planes {
            let orbital_plane = Rc::new(OrbitalPlane::new(
                i, semimajor_axis, inclination, 2.0 * PI * i as f64 / num_orbital_planes as f64,
            ));

            for j in 0..satellites_per_plane {
                satellites.push(Satellite::new(
                    i * satellites_per_plane + j,
                    Rc::clone(&orbital_plane),
                    2.0 * PI * j as f64 / satellites_per_plane as f64,
                ));
            }

            orbital_planes.push(orbital_plane);
        }

        Simulation {
            orbital_planes,
            satellites,
            time_step,
            t: 0.0,
        }
    }

    pub fn step(&mut self) {
        self.t += self.time_step;
    }
}

pub fn init_msg(sim: &Simulation) -> String {
    let first_plane = sim.orbital_planes.get(0);

    let semimajor_axis = first_plane.map(|p| p.semimajor_axis).unwrap_or(0.0);
    let inclination = first_plane.map(|p| p.inclination).unwrap_or(0.0);

    let mut orbital_planes = JsonValue::new_object();
    for plane in &sim.orbital_planes {
        orbital_planes[plane.id.to_string()] = object! {
            longitude: plane.longitude,
        }
    }

    let mut satellites = JsonValue::new_object();
    for sat in &sim.satellites {
        satellites[sat.id.to_string()] = object! {
            orbital_plane: sat.orbital_plane.id.to_string(),
            arg_periapsis: sat.arg_periapsis,
        }
    }

    let obj = object! {
        msg_type: "init",
        semimajor_axis: semimajor_axis,
        inclination: inclination,
        orbital_planes: orbital_planes,
        satellites: satellites,
    };
    
    obj.dump()
}

pub fn update_msg(sim: &Simulation) -> String {
    let mut satellites = JsonValue::new_object();
    for sat in &sim.satellites {
        satellites[sat.id.to_string()] = object! {
            position: sat.calc_position(sim.t).as_slice(),
            velocity: sat.calc_velocity(sim.t).as_slice(),
        };
    }

    let obj = object! {
        msg_type: "update",
        t: sim.t,
        satellites: satellites,
    };

    obj.dump()
}
