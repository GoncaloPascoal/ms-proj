
use std::{fs, env, path::Path, net::{TcpListener, TcpStream, SocketAddrV4, Ipv4Addr}, sync::Arc, sync::Mutex, thread, time::Duration, io::{Write, self, Read}};
use toml::Value;

use model::{EARTH_RADIUS, Simulation, init_msg, update_msg, Model};
use statistics::statistics_msg;

pub mod model;
pub mod connection_strategy;
pub mod statistics;

const SERVER_PORT: u16 = 2000;
const STATISTICS_PORT: u16 = 2001;

fn main() -> thread::Result<()> {
    let args: Vec<String> = env::args().collect();

    let orbiting_altitude: f64;
    let num_orbital_planes: usize;
    let satellites_per_plane: usize;
    let inclination: f64;
    let max_connections: usize;
    let connection_range: f64;

    let simulation_speed: f64;
    let update_frequency: f64;
    let update_frequency_server: f64;
    let starting_failure_rate: f64;
    let connection_refresh_interval: f64;

    if args.len() == 1 {
        orbiting_altitude = 0.55e6;
        num_orbital_planes = 10;
        satellites_per_plane = 20;
        inclination = 0.6;
        max_connections = 4;
        connection_range = 1e10;

        simulation_speed = 10.0;
        update_frequency = 10.0;
        update_frequency_server = update_frequency;
        starting_failure_rate = 0.0;
        connection_refresh_interval = 10.0;
    } else if args.len() == 2 {
        let path = Path::new(&args[1]);
        if !path.exists() {
            panic!("Specified path does not exist!")
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

        simulation_speed            = simulation_parameters["simulation_speed"]           .as_float().unwrap_or(1.0);
        update_frequency            = simulation_parameters["update_frequency"]           .as_float().unwrap_or(10.0);
        update_frequency_server     = simulation_parameters["update_frequency_server"]    .as_float().unwrap_or(update_frequency);
        connection_refresh_interval = simulation_parameters["connection_refresh_interval"].as_float().unwrap();
        starting_failure_rate       = simulation_parameters["starting_failure_rate"]      .as_float().unwrap_or(0.0);
        assert!((0.0..=1.0).contains(&starting_failure_rate));
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
            starting_failure_rate,
        ),
        simulation_speed / update_frequency,
        simulation_speed,
        connection_refresh_interval,
    )));

    let sim_server = Arc::clone(&sim);
    let sim_statistics = Arc::clone(&sim);

    let steps = 10000;
    let delay = Duration::from_secs_f64(1.0 / update_frequency);
    let delay_server = Duration::from_secs_f64(1.0 / update_frequency_server);
    let server_steps = (steps as f64 * update_frequency_server / update_frequency) as usize;

    let simulation_handle = thread::spawn(move || { simulation_thread(sim, steps, delay) });
    let server_handle = thread::spawn(move || { server_thread(sim_server, server_steps, delay_server) });
    let statistics_handle = thread::spawn(move || { statistics_thread(sim_statistics, server_steps, delay_server) });

            simulation_handle.join().expect("Couldn't join simulation thread.");
    let _ = server_handle    .join().expect("Couldn't join visualization server thread.");
    let _ = statistics_handle.join().expect("Couldn't join statistics server thread.");

    Ok(())
}

fn simulation_thread(sim: Arc<Mutex<Simulation>>, simulation_steps: usize, delay: Duration) {
    for _ in 0..simulation_steps {
        thread::sleep(delay);
        {
            let mut lock = sim.lock().unwrap();
            lock.step();
        }
    }
}

fn write(stream: &mut TcpStream, msg: String) {
    let bytes = msg.as_bytes();
    stream.write_all(&(bytes.len() as u32).to_ne_bytes()).unwrap();
    stream.write_all(bytes).unwrap();
}

fn server_thread(sim: Arc<Mutex<Simulation>>, communication_steps: usize, delay: Duration) -> io::Result<()> {
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, SERVER_PORT);
    let listener = TcpListener::bind(addr).unwrap();
    let mut msg;

    for stream in listener.incoming() {
        let mut stream = stream?;

        {
            let lock = sim.lock().unwrap();
            msg = init_msg(&lock);
        }
        write(&mut stream, msg);

        stream.set_nonblocking(true)?;
        for _ in 0..communication_steps {
            thread::sleep(delay);
            {
                let lock = sim.lock().unwrap();
                msg = update_msg(&lock);
            }
            write(&mut stream, msg);

            let mut msg = String::new();
            if stream.read_to_string(&mut msg).is_ok() {
                if let Ok(json) = json::parse(&msg) {
                    if json["msg_type"].as_str() == Some("simulate_failure") {
                        if let Some(id) = json["satellite_id"].as_usize() {
                            let mut lock = sim.lock().unwrap();
                            lock.simulate_failure(id);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn statistics_thread(sim: Arc<Mutex<Simulation>>, communication_steps: usize, delay: Duration) -> io::Result<()> {
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, STATISTICS_PORT);
    let listener = TcpListener::bind(addr).unwrap();
    let mut msg;

    for stream in listener.incoming() {
        let mut stream = stream?;

        for _ in 0..communication_steps {
            thread::sleep(delay);
            {
                let lock = sim.lock().unwrap();
                msg = statistics_msg(&lock);
            }
            write(&mut stream, msg);
        }
    }

    Ok(())
}
