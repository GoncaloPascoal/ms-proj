use std::{f64::consts::PI, sync::Arc};

use json::{object, JsonValue};
use nalgebra::{Rotation3, Vector3};
use petgraph::{algo::astar, graphmap::GraphMap, Undirected, visit::EdgeRef};
use rand::{thread_rng, Rng};

use crate::connection_strategy::{ConnectionStrategy, GridStrategy};

/// Earth's standard gravitational parameter (gravitational constant times the Earth's mass).
pub const GM: f64 = 3.986004418e14;
/// Radius of the Earth, in meters.
pub const EARTH_RADIUS: f64 = 6.371e6;
/// Period of the Earth's rotation, in seconds.
pub const EARTH_ROTATION_PERIOD: f64 = 86400.0;
/// Speed of light, in meters per second.
pub const LIGHT_SPEED: f64 = 299792458.0;

pub struct GeoCoordinates {
    latitude: f64,
    longitude: f64,
}

impl GeoCoordinates {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        assert!(latitude.abs() <= 90.0);
        assert!(longitude.abs() <= 180.0);

        GeoCoordinates { latitude, longitude }
    }

    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    pub fn longitude(&self) -> f64 {
        self.longitude
    }

    pub fn haversine_distance(&self, other: &GeoCoordinates) -> f64 {
        let self_latitude  = self .latitude().to_radians();
        let other_latitude = other.latitude().to_radians();

        let delta_latitude  = (self.latitude () - other.latitude ()).to_radians();
        let delta_longitude = (self.longitude() - other.longitude()).to_radians();

        let central_angle_inner = (delta_latitude / 2.0).sin().powi(2)
            + self_latitude.cos() * other_latitude.cos() * (delta_longitude / 2.0).sin().powi(2);
        let central_angle = 2.0 * central_angle_inner.sqrt().asin();

        EARTH_RADIUS * central_angle
    }
}

pub struct OrbitalPlane {
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

pub struct Satellite {
    id: usize,
    orbital_plane: Arc<OrbitalPlane>,
    arg_periapsis: f64,
    alive: bool,
}

impl Satellite {
    fn new(
        id: usize, 
        orbital_plane: Arc<OrbitalPlane>, 
        arg_periapsis: f64, 
        alive: bool,
    ) -> Self {
        Satellite {
            id,
            orbital_plane,
            arg_periapsis,
            alive,
        }
    }

    pub fn calc_position(&self, t: f64) -> Vector3<f64> {
        let r = self.orbital_plane.semimajor_axis;
        let true_anomaly = (t * self.orbital_plane.angular_speed) % (2.0 * PI);

        let position = Vector3::new(r, 0.0, 0.0);

        Rotation3::from_euler_angles(0.0, self.orbital_plane.longitude, 0.0) *
        Rotation3::from_euler_angles(self.orbital_plane.inclination, 0.0, 0.0) *
        Rotation3::from_euler_angles(0.0, self.arg_periapsis + true_anomaly, 0.0) *
        position
    }

    pub fn calc_velocity(&self, t: f64) -> Vector3<f64> {
        let direction = Rotation3::from_euler_angles(0.0, PI / 2.0, 0.0) * self.calc_position(t).normalize();

        self.orbital_plane.orbital_speed * direction
    }
}

pub struct Model {
    orbital_planes: Vec<Arc<OrbitalPlane>>,
    satellites: Vec<Satellite>,
    t: f64,
    max_connections: usize,
    connection_range: f64,
}

impl Model {
    pub fn new(
        num_orbital_planes: usize,
        satellites_per_plane: usize,
        inclination: f64,
        semimajor_axis: f64,
        max_connections: usize,
        connection_range: f64,
        starting_failure_rate: f64,
    ) -> Self {
        let mut rng = thread_rng();
        let mut orbital_planes = Vec::with_capacity(num_orbital_planes);
        let mut satellites = Vec::with_capacity(num_orbital_planes * satellites_per_plane);

        for i in 0..num_orbital_planes {
            let orbital_plane = Arc::new(OrbitalPlane::new(
                i, semimajor_axis, inclination, 2.0 * PI * i as f64 / num_orbital_planes as f64,
            ));

            for j in 0..satellites_per_plane {
                satellites.push(Satellite::new(
                    i * satellites_per_plane + j,
                    Arc::clone(&orbital_plane),
                    2.0 * PI * j as f64 / satellites_per_plane as f64,
                    rng.gen::<f64>() <= starting_failure_rate,
                ));
            }

            orbital_planes.push(orbital_plane);
        }

        Model {
            orbital_planes,
            satellites,
            t: 0.0,
            max_connections,
            connection_range,
        }
    }

    pub fn orbital_planes(&self) -> &[Arc<OrbitalPlane>] {
        &self.orbital_planes
    }

    pub fn satellites(&self) -> &[Satellite] {
        &self.satellites
    }

    pub fn t(&self) -> f64 {
        self.t
    }

    pub fn increment_t(&mut self, time_step: f64) {
        self.t += time_step;
    }

    pub fn max_connections(&self) -> usize {
        self.max_connections
    }

    pub fn connection_range(&self) -> f64 {
        self.connection_range
    }

    /// Returns the point on the surface of the Earth with the given
    /// latitude and longitude (both in degrees).
    pub fn surface_point(&self, coordinates: &GeoCoordinates) -> Vector3<f64> {
        let angle_y = ((self.t / EARTH_ROTATION_PERIOD) * 2.0 * PI + coordinates.longitude().to_radians()) % (2.0 * PI);
        let angle_z = coordinates.latitude().to_radians();

        let v = Vector3::new(EARTH_RADIUS, 0.0, 0.0);

        Rotation3::from_euler_angles(0.0, angle_y, angle_z) * v
    }

    fn closest_satellite(&self, point: Vector3<f64>) -> &Satellite {
        self.satellites.iter().min_by(|s1, s2| {
            let dist1 = (point - s1.calc_position(self.t)).norm();
            let dist2 = (point - s2.calc_position(self.t)).norm();
            dist1.partial_cmp(&dist2).unwrap()
        }).unwrap()
    }
}

pub type ConnectionGraph = GraphMap<usize, f64, Undirected>;

pub struct Simulation {
    model: Model,
    time_step: f64,
    simulation_speed: f64,
    connection_refresh_interval: f64,
    last_update_timestamp: f64,
    topology: ConnectionGraph,
    strategy: Box<dyn ConnectionStrategy>,
}

impl Simulation {
    pub fn new(
        model: Model,
        time_step: f64,
        simulation_speed: f64,
        connection_refresh_interval: f64,
    ) -> Self {
        let mut topology = GraphMap::new();
        for sat in 0..model.satellites().len() {
            topology.add_node(sat);
        }

        Simulation {
            model,
            time_step,
            simulation_speed,
            connection_refresh_interval,
            last_update_timestamp: 0.0,
            topology,
            strategy: Box::new(GridStrategy::new()),
        }
    }

    pub fn step(&mut self) {
        self.model.increment_t(self.time_step);
        if self.t() >= self.last_update_timestamp + self.connection_refresh_interval {
            self.last_update_timestamp = self.t();
            self.update_connections()
        }
    }

    pub fn update_connections(&mut self) {
        // Updating the topology
        self.topology = self.strategy.run(&self.model);

        // Validating the topology
        for sat in self.topology.nodes() {
            assert!(self.topology.edges(sat).count() <= self.model.max_connections());
        }

        for (sat1, sat2, distance) in self.topology.all_edges() {
            assert!(self.satellites()[sat1].alive);
            assert!(self.satellites()[sat2].alive);
            assert!(*distance < self.model.connection_range());
        }
    }

    pub fn simulation_speed(&self) -> f64 {
        self.simulation_speed
    }

    pub fn satellites(&self) -> &[Satellite] {
        self.model.satellites()
    }

    pub fn t(&self) -> f64 {
        self.model.t()
    }

    pub fn topology(&self) -> &ConnectionGraph {
        &self.topology
    }

    pub fn orbital_planes(&self) -> &[Arc<OrbitalPlane>] {
        self.model.orbital_planes()
    }

    /// Calculates round trip time (RTT) in seconds between two locations
    /// specified using geographical coordinates.
    pub fn calc_rtt(&self, c1: &GeoCoordinates, c2: &GeoCoordinates) -> Option<f64> {
        let mut distance = 0.0;

        let p1 = self.model.surface_point(c1);
        let p2 = self.model.surface_point(c2);

        let sat1 = self.model.closest_satellite(p1);
        let sat2 = self.model.closest_satellite(p2);

        distance += (sat1.calc_position(self.t()) - p1).norm();
        if let Some((cost, _)) = astar(
            &self.topology,
            sat1.id,
            |n| n == sat2.id,
            |e| *e.weight(),
            |n| (sat2.calc_position(self.t()) - self.satellites()[n].calc_position(self.t())).norm()
        ) {
            distance += cost;
        }
        else {
            return None;
        }
        distance += (sat2.calc_position(self.t()) - p2).norm();

        Some(2.0 * distance / LIGHT_SPEED)
    }
}

pub fn init_msg(sim: &Simulation) -> String {
    let first_plane = sim.orbital_planes().get(0);

    let semimajor_axis = first_plane.map(|p| p.semimajor_axis).unwrap_or(0.0);
    let inclination = first_plane.map(|p| p.inclination).unwrap_or(0.0);

    let mut orbital_planes = JsonValue::new_object();
    for plane in sim.orbital_planes() {
        orbital_planes[plane.id.to_string()] = object! {
            longitude: plane.longitude,
        }
    }

    let mut satellites = JsonValue::new_object();
    for sat in sim.satellites() {
        satellites[sat.id.to_string()] = object! {
            orbital_plane: sat.orbital_plane.id.to_string(),
            arg_periapsis: sat.arg_periapsis,
        }
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

pub fn update_msg(sim: &Simulation) -> String {
    let mut satellites = JsonValue::new_object();
    for sat in sim.satellites() {
        satellites[sat.id.to_string()] = object! {
            position: sat.calc_position(sim.t()).as_slice(),
            alive: sat.alive,
        };
    }

    let mut obj = object! {
        msg_type: "update",
        t: sim.t(),
        satellites: satellites,
    };

    if sim.last_update_timestamp == sim.t() {
        let connections: Vec<_> = sim.topology().all_edges().map(|(a, b, _)| vec![a, b]).collect();
        let _ = obj.insert("connections", connections);
    }

    obj.dump()
}
