
use std::{fs, env, path::Path, net::TcpListener, sync::Arc, sync::Mutex, thread, time::Duration};
use tungstenite::{Message, accept};
use toml::Value;

use model::{EARTH_RADIUS, Simulation, init_msg, update_msg, Model};

pub mod model;
pub mod connection_strategy;

fn main() -> thread::Result<()> {
    let args: Vec<String> = env::args().collect();

    let orbiting_altitude: f64;
    let num_orbital_planes: usize;
    let satellites_per_plane: usize;
    let inclination: f64;
    let max_connections: usize;
    let connection_range: f64;

    let time_step: f64;
    let starting_failure_rate: f64;
    let connection_refresh_time: f64;

    if args.len() == 1 {
        orbiting_altitude = 0.55e6;
        num_orbital_planes = 10;
        satellites_per_plane = 20;
        inclination = 0.6;
        max_connections = 4;
        connection_range = 1e10;

        time_step = 1.0;
        starting_failure_rate = 0.0;
        connection_refresh_time = 10.0;
    } else if args.len() == 2 {
        let path = Path::new(&args[1]);
        if !path.exists() {
            panic!("Path doesn't exist!")
        }
        let contents = fs::read_to_string(path).unwrap().parse::<Value>().unwrap();

        let constellation_parameters = match &contents["constellation"] {
            Value::Table(t) => t,
            _ => panic!(),
        };

        let simulation_parameters = match &contents["simulation"] {
            Value::Table(t) => t,
            _ => panic!(),
        };

        orbiting_altitude    = constellation_parameters["altitude"]            .as_float()  .unwrap();
        num_orbital_planes   = constellation_parameters["num_orbital_planes"]  .as_integer().unwrap() as usize;
        satellites_per_plane = constellation_parameters["satellites_per_plane"].as_integer().unwrap() as usize;
        inclination          = constellation_parameters["inclination"]         .as_float()  .unwrap();
        max_connections      = constellation_parameters["max_connections"]     .as_integer().unwrap() as usize;
        connection_range     = constellation_parameters["connection_range"]    .as_float()  .unwrap();

        time_step            = simulation_parameters["time_step"]               .as_float()  .unwrap();

        if simulation_parameters.contains_key("starting_failure_rate") {
            starting_failure_rate = simulation_parameters["starting_failure_rate"].as_float().unwrap();
            assert!(0.0 <= starting_failure_rate && starting_failure_rate <= 1.0);
        } else {
            starting_failure_rate = 0.0;
        }
        connection_refresh_time = simulation_parameters["connection_refresh_time"].as_float().unwrap();
    } else {
        panic!("More than one argument!");
    }

    let sim = Arc::new(Mutex::new(Simulation::new(
        Model::new(
            num_orbital_planes,
            satellites_per_plane,
            inclination,
            EARTH_RADIUS + orbiting_altitude,
            max_connections,
            connection_range,
        ),
        time_step,
        starting_failure_rate,
        connection_refresh_time,
    )));

    let sim_server = Arc::clone(&sim);

    let steps = 10000;

    let simulation_handle = thread::spawn(move || { simulation_thread(sim, steps, 100) });
    let server_handle = thread::spawn(move || { server_thread(sim_server, steps, 100) });

    simulation_handle.join().expect("Couldn't join simulation thread.");
    let _ = server_handle.join().expect("Couldn't join server thread.");

    Ok(())
}

fn simulation_thread(sim: Arc<Mutex<Simulation>>, simulation_steps: usize, delay_ms: u64) {
    for _ in 0..simulation_steps {
        thread::sleep(Duration::from_millis(delay_ms));
        {
            let mut lock = sim.lock().unwrap();
            lock.step();
        }
    }
}

fn server_thread(sim: Arc<Mutex<Simulation>>, communication_steps: usize, delay_ms: u64) -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:1234").unwrap();

    for stream in listener.incoming() {
        let mut websocket = accept(stream?).unwrap();

        {
            let lock = sim.lock().unwrap();
            websocket.write_message(Message::Text(init_msg(&lock))).unwrap();
        }

        for _ in 0..communication_steps {
            thread::sleep(Duration::from_millis(delay_ms));
            {
                let lock = sim.lock().unwrap();
                websocket.write_message(Message::Text(update_msg(&lock))).unwrap();
            }
        }

        break;
    }

    Ok(())
}
