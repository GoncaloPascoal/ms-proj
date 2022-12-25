use std::{fs, env, path::Path, net::TcpListener, time::Duration};
use tungstenite::{Message, accept};
use toml::{Value};

use model::{EARTH_RADIUS, Simulation, init_msg, update_msg};

pub mod model;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let orbiting_altitude : f64;
    let num_orbital_planes : usize;
    let satellites_per_plane : usize;
    let inclination : f64;
    let n_connections : usize;
    let connection_range : f64;

    let time_step : f64;
    let starting_failure_rate : f64;
    let steps_per_update : usize;

    if args.len() == 1 {
        orbiting_altitude = 0.55e6;
        num_orbital_planes = 10;
        satellites_per_plane = 20;
        inclination = 0.6;
        n_connections = 4;
        connection_range = 1e10;

        time_step = 1.0;
        starting_failure_rate = 0.0;
        steps_per_update = 10;
    } else if args.len() == 2 {
        let path = Path::new(&args[1]);
        if !path.exists() {
            panic!("Path doesn't exist!")
        }
        let contents = fs::read_to_string(path).unwrap().parse::<Value>().unwrap();

        let constellation_parameters = match &contents["constellation parameters"] {
            Value::Table(t) => t,
            _ => panic!(),
        };

        let simulation_parameters = match &contents["simulation parameters"] {
            Value::Table(t) => t,
            _ => panic!(),
        };

        orbiting_altitude    = constellation_parameters["altitude"]                .as_float()  .unwrap();
        num_orbital_planes   = constellation_parameters["number of orbital planes"].as_integer().unwrap() as usize;
        satellites_per_plane = constellation_parameters["satellites per plane"]    .as_integer().unwrap() as usize;
        inclination          = constellation_parameters["inclination"]             .as_float()  .unwrap();
        n_connections        = constellation_parameters["connections"]             .as_integer().unwrap() as usize;
        connection_range     = constellation_parameters["connection range"]        .as_float().unwrap();
        
        time_step            = simulation_parameters["timestep"]                   .as_float()  .unwrap();

        if simulation_parameters.contains_key("starting failure rate") {
            starting_failure_rate = simulation_parameters["starting failure rate"].as_float().unwrap();
            assert!(0.0 <= starting_failure_rate && starting_failure_rate <= 1.0);
        } else {
            starting_failure_rate = 0.0;
        }
        steps_per_update = simulation_parameters["number of steps per connection update"].as_integer().unwrap() as usize;
    } else {
        panic!("More than one argument!");
    }
    
    let mut sim = Simulation::new(
        num_orbital_planes,
        satellites_per_plane,
        inclination,
        EARTH_RADIUS + orbiting_altitude,
        time_step,
        starting_failure_rate,
        n_connections,
        connection_range,
        steps_per_update,
    );

    let server = TcpListener::bind("127.0.0.1:1234").unwrap();

    for stream in server.incoming() {
        let mut websocket = accept(stream?).unwrap();

        websocket.write_message(Message::Text(init_msg(&sim))).unwrap();

        loop {
            sim.step();
            std::thread::sleep(Duration::from_millis(100));
            websocket.write_message(Message::Text(update_msg(&sim))).unwrap();
        }
    }

    Ok(())
}
