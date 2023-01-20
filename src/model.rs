use std::{f64::consts::PI, sync::{Arc, mpsc::Sender}};

use nalgebra::{Rotation3, Vector3};
use petgraph::{algo::astar, graphmap::GraphMap, Undirected, visit::EdgeRef};
use rand::{Rng, rngs::StdRng, SeedableRng};

use crate::{connection_strategy::ConnectionStrategy, statistics::statistics_msg};

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

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn semimajor_axis(&self) -> f64 {
        self.semimajor_axis
    }

    pub fn inclination(&self) -> f64 {
        self.inclination
    }

    pub fn longitude(&self) -> f64 {
        self.longitude
    }
}

pub struct Satellite {
    id: usize,
    orbital_plane: Arc<OrbitalPlane>,
    arg_periapsis: f64,
    position: Vector3<f64>,
    status: bool,
}

impl Satellite {
    const HALF_ANGLE_DEGREES: f64 = 60.0;

    fn new(
        id: usize, 
        orbital_plane: Arc<OrbitalPlane>, 
        arg_periapsis: f64,
        status: bool,
    ) -> Self {
        Satellite {
            id,
            orbital_plane,
            arg_periapsis,
            position: Vector3::zeros(),
            status,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn orbital_plane(&self) -> &OrbitalPlane {
        &self.orbital_plane
    }

    pub fn arg_periapsis(&self) -> f64 {
        self.arg_periapsis
    }

    pub fn position(&self) -> &Vector3<f64> {
        &self.position
    }

    pub fn status(&self) -> bool {
        self.status
    }

    pub fn set_status(&mut self, status: bool) {
        self.status = status;
    }

    pub fn recalculate_position(&mut self, t: f64) {
        let r = self.orbital_plane.semimajor_axis;
        let true_anomaly = (t * self.orbital_plane.angular_speed) % (2.0 * PI);

        let new_position = Vector3::new(r, 0.0, 0.0);

        self.position = Rotation3::from_euler_angles(0.0, self.orbital_plane.longitude, 0.0) *
            Rotation3::from_euler_angles(self.orbital_plane.inclination, 0.0, 0.0) *
            Rotation3::from_euler_angles(0.0, self.arg_periapsis + true_anomaly, 0.0) *
            new_position
    }

    pub fn velocity(&self) -> Vector3<f64> {
        let direction = Rotation3::from_euler_angles(0.0, PI / 2.0, 0.0) * self.position.normalize();

        self.orbital_plane.orbital_speed * direction
    }

    /// Returns true if the satellite has an unobstructed line of sight towards
    /// a given point (it is not blocked by the Earth).
    pub fn has_line_of_sight(&self, point: &Vector3<f64>) -> bool {
        let distance_to_point = self.position.metric_distance(point);
        let segment_range = 0.0..distance_to_point;
        let direction = (point - self.position).normalize();

        let d = -direction.dot(&self.position);
        let nabla = direction.dot(&self.position).powi(2) - self.position.norm_squared() + EARTH_RADIUS.powi(2);

        if nabla < 0.0 {
            true
        }
        else if nabla == 0.0 {
            !segment_range.contains(&d)
        }
        else {
            let nabla = nabla.sqrt();
            !(segment_range.contains(&(d - nabla)) || segment_range.contains(&(d + nabla)))
        }
    }

    pub fn is_in_view_cone(&self, point: &Vector3<f64>) -> bool {
        let half_angle = Self::HALF_ANGLE_DEGREES.to_radians();
        let max_distance = self.orbital_plane.semimajor_axis * half_angle.cos();

        let cone_axis = -self.position.normalize();
        let to_point = point - self.position;

        let distance = to_point.norm();
        let point_angle = to_point.normalize().dot(&cone_axis).acos();

        point_angle <= half_angle && distance <= max_distance
    }
}

pub enum ConstellationType {
    Delta,
    Star
}

impl ConstellationType {
    pub fn angle(&self) -> f64 {
        match self {
            Self::Delta => 2.0 * PI,
            Self::Star => PI,
        }
    }
}

impl TryFrom<&str> for ConstellationType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "delta" => Ok(Self::Delta),
            "star" => Ok(Self::Star),
            _ => Err(()),
        }
    }
}

pub struct Model {
    orbital_planes: Vec<Arc<OrbitalPlane>>,
    satellites: Vec<Satellite>,
    t: f64,
    max_connections: usize,
}

impl Model {
    pub fn new(
        num_orbital_planes: usize,
        satellites_per_plane: usize,
        inclination: f64,
        constellation_type: ConstellationType,
        phasing: usize,
        semimajor_axis: f64,
        max_connections: usize,
    ) -> Self {
        let num_satellites = num_orbital_planes * satellites_per_plane;

        let mut orbital_planes = Vec::with_capacity(num_orbital_planes);
        let mut satellites = Vec::with_capacity(num_satellites);
        let phase_offset = phasing as f64 * PI / num_satellites as f64;

        for i in 0..num_orbital_planes {
            let longitude = constellation_type.angle() * i as f64 / num_orbital_planes as f64;

            let orbital_plane = Arc::new(OrbitalPlane::new(
                i, semimajor_axis, inclination, longitude,
            ));

            for j in 0..satellites_per_plane {
                satellites.push(Satellite::new(
                    i * satellites_per_plane + j,
                    Arc::clone(&orbital_plane),
                    (phase_offset * i as f64 + 2.0 * PI * j as f64 / satellites_per_plane as f64) % 360.0,
                    true,
                ));
            }

            orbital_planes.push(orbital_plane);
        }

        Model {
            orbital_planes,
            satellites,
            t: 0.0,
            max_connections,
        }
    }

    pub fn orbital_planes(&self) -> &[Arc<OrbitalPlane>] {
        &self.orbital_planes
    }

    pub fn satellites(&self) -> &[Satellite] {
        &self.satellites
    }

    pub fn satellites_mut(&mut self) -> &mut [Satellite] {
        &mut self.satellites
    }

    pub fn t(&self) -> f64 {
        self.t
    }

    pub fn increment_t(&mut self, time_step: f64) {
        self.t += time_step;
        let t = self.t;
        for sat in self.satellites_mut() {
            sat.recalculate_position(t);
        }
    }

    pub fn max_connections(&self) -> usize {
        self.max_connections
    }

    pub fn distance_between_satellites(&self, sat1: &Satellite, sat2: &Satellite) -> f64 {
        sat1.position().metric_distance(sat2.position())
    }

    /// Returns the point on the surface of the Earth with the given
    /// latitude and longitude (both in degrees).
    pub fn surface_point(&self, coordinates: &GeoCoordinates) -> Vector3<f64> {
        let angle_y = ((self.t / EARTH_ROTATION_PERIOD) * 2.0 * PI + coordinates.longitude().to_radians()) % (2.0 * PI);
        let angle_z = coordinates.latitude().to_radians();

        let v = Vector3::new(EARTH_RADIUS, 0.0, 0.0);

        Rotation3::from_euler_angles(0.0, angle_y, angle_z) * v
    }

    pub fn closest_active_satellite(&self, point: &Vector3<f64>) -> Option<&Satellite> {
        self.satellites.iter().filter(|s| s.status()).min_by(|s1, s2| {
            let dist1 = point.metric_distance(s1.position());
            let dist2 = point.metric_distance(s2.position());
            dist1.partial_cmp(&dist2).unwrap()
        })
    }
}

pub type ConnectionGraph = GraphMap<usize, f64, Undirected>;

pub struct Simulation {
    model: Model,
    time_step: f64,
    simulation_speed: f64,
    connection_refresh_interval: f64,
    rng: StdRng,
    recurrent_failure_probability: f64,
    last_update_timestamp: f64,
    topology: ConnectionGraph,
    strategy: Box<dyn ConnectionStrategy>,
    statistics_channel: Sender<String>,
}

impl Simulation {
    pub fn new(
        mut model: Model,
        time_step: f64,
        simulation_speed: f64,
        connection_refresh_interval: f64,
        rng_seed: Option<u64>,
        starting_failure_probability: f64,
        recurrent_failure_probability: f64,
        strategy: Box<dyn ConnectionStrategy>,
        statistics_channel: Sender<String>,
    ) -> Self {
        let mut rng = match rng_seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_entropy(),
        };

        if starting_failure_probability > 0.0 {
            for sat in model.satellites_mut() {
                if rng.gen::<f64>() < starting_failure_probability {
                    sat.set_status(false);
                }
            }
        }

        let mut sim = Simulation {
            model,
            time_step,
            simulation_speed,
            connection_refresh_interval,
            last_update_timestamp: 0.0,
            rng,
            recurrent_failure_probability,
            topology: GraphMap::new(),
            strategy,
            statistics_channel,
        };
        sim.update_connections();

        sim
    }

    pub fn simulation_speed(&self) -> f64 {
        self.simulation_speed
    }

    pub fn connection_refresh_interval(&self) -> f64 {
        self.connection_refresh_interval
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

    pub fn step(&mut self) {
        self.model.increment_t(self.time_step);
        if self.t() >= self.last_update_timestamp + self.connection_refresh_interval {
            // Simulate potential satellite failures
            if self.recurrent_failure_probability > 0.0 {
                for sat in self.model.satellites_mut() {
                    if sat.status() && self.rng.gen::<f64>() < self.recurrent_failure_probability {
                        sat.set_status(false);
                    }
                }
            }

            self.update_connections();
        }
    }

    pub fn update_connections(&mut self) {
        self.last_update_timestamp = self.t();
        self.topology = self.strategy.run(&self.model);

        // Send statistics message
        self.statistics_channel.send(statistics_msg(self)).unwrap();
    }

    /// Calculates round trip time (RTT) in seconds between two locations
    /// specified using geographical coordinates. Expensive calculation since it
    /// requires pathfinding algorithms and cloning the topology.
    pub fn calc_rtt(&self, c1: &GeoCoordinates, c2: &GeoCoordinates) -> Option<f64> {
        let mut topology = self.topology.clone();
        let satellites = self.satellites();

        // Update edge weights (distances between satellites) according to most recent timestamp
        for edge in topology.all_edges_mut() {
            let pos1 = satellites[edge.0].position();
            let pos2 = satellites[edge.1].position();
            *edge.2 = pos1.metric_distance(pos2);
        }

        let nodes: Vec<usize> = topology.nodes().collect();

        let p1 = self.model.surface_point(c1);
        let p2 = self.model.surface_point(c2);

        let id1 = satellites.len();
        let id2 = id1 + 1;

        // Add links between surface points and satellites when there is visibility between them
        for sat in nodes.iter().map(|id| &satellites[*id]) {
            if sat.is_in_view_cone(&p1) {
                topology.add_edge(id1, sat.id(), p1.metric_distance(sat.position()));
            }

            if sat.is_in_view_cone(&p2) {
                topology.add_edge(sat.id(), id2, sat.position().metric_distance(&p2));
            }
        }

        astar(
            &topology,
            id1,
            |n| n == id2,
            |e| *e.weight(),
            |n| match n {
                _ if n == id1 => p1.metric_distance(&p2),
                _ if n == id2 => 0.0,
                _ => satellites[n].position().metric_distance(&p2)
            }
        ).map(|(cost, _)| 2.0 * cost / LIGHT_SPEED)
    }

    pub fn simulate_failure(&mut self, id: usize) {
        self.model.satellites_mut()[id].set_status(false);
        self.topology.remove_node(id);
    }
}
