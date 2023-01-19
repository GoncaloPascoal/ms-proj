
use std::{fs::{self, File}, env, path::Path, net::{TcpListener, TcpStream, SocketAddrV4, Ipv4Addr}, sync::Arc, sync::{Mutex, mpsc::{self, Receiver}}, thread, time::Duration, io::{self, Write, Read}};
use connection_strategy::{ConnectionStrategy, GridStrategy};

use connection_strategy::NearestNeighborStrategy;
use model::{EARTH_RADIUS, Simulation, Model, ConstellationType};
use server::{init_msg, update_msg};

pub mod connection_strategy;
pub mod model;
pub mod server;
pub mod statistics;

const SERVER_PORT: u16 = 2000;
const STATISTICS_PORT: u16 = 2001;

fn main() -> thread::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Constellation parameters
    let orbiting_altitude: f64;
    let num_orbital_planes: usize;
    let satellites_per_plane: usize;
    let inclination: f64;
    let max_connections: usize;

    let constellation_type: ConstellationType;
    let phasing: i64;

    // Simulation parameters
    let file_path: Option<String>;
    let steps: Option<usize>;

    let simulation_speed: f64;
    let update_frequency: f64;
    let update_frequency_server: f64;
    let connection_refresh_interval: f64;

    let rng_seed: Option<u64>;
    let starting_failure_probability: f64;
    let recurrent_failure_probability: f64;

    let strategy: Box<dyn ConnectionStrategy>;

    if args.len() == 1 {
        orbiting_altitude = 0.55e6;
        num_orbital_planes = 10;
        satellites_per_plane = 20;
        inclination = 60.0;
        max_connections = 4;

        constellation_type = ConstellationType::Delta;
        phasing = 0;

        file_path = None;
        steps = None;

        simulation_speed = 10.0;
        update_frequency = 10.0;
        update_frequency_server = update_frequency;
        connection_refresh_interval = 10.0;

        rng_seed = None;
        starting_failure_probability = 0.0;
        recurrent_failure_probability = 0.0;

        strategy = Box::new(GridStrategy::new(0));
    } else if args.len() == 2 {
        use toml::Value;
        use toml::map::Map;

        let empty_parameters = Map::new();

        let path = Path::new(&args[1]);
        if !path.exists() {
            panic!("Specified path does not exist!");
        }
        let contents = fs::read_to_string(path).expect("Error when reading config file!")
            .parse::<Value>().expect("Error when parsing config file!");

        let constellation_parameters = match &contents["constellation"] {
            Value::Table(t) => t,
            _ => panic!(),
        };

        let simulation_parameters = match &contents["simulation"] {
            Value::Table(t) => t,
            _ => &empty_parameters,
        };

        orbiting_altitude    = constellation_parameters["altitude"]            .as_float()  .unwrap();
        num_orbital_planes   = constellation_parameters["num_orbital_planes"]  .as_integer().unwrap() as usize;
        satellites_per_plane = constellation_parameters["satellites_per_plane"].as_integer().unwrap() as usize;
        inclination          = constellation_parameters["inclination"]         .as_float()  .unwrap();
        max_connections      = constellation_parameters["max_connections"]     .as_integer().unwrap() as usize;

        constellation_type   = constellation_parameters.get("type")   .and_then(Value::as_str)
            .and_then(|v| ConstellationType::try_from(v).ok())
            .unwrap_or(ConstellationType::Delta);
        phasing              = constellation_parameters.get("phasing").and_then(Value::as_integer).unwrap_or(0); 
        assert!((0..num_orbital_planes as i64).contains(&phasing));

        file_path = simulation_parameters.get("file_path").and_then(Value::as_str).map(|s| s.to_owned());
        steps     = simulation_parameters.get("steps")    .and_then(Value::as_integer).map(|v| v as usize);

        if file_path.is_some() {
            steps.expect("Must specify a finite number of time steps when saving simulation data to a file!");
        }

        simulation_speed            = simulation_parameters.get("simulation_speed")           .and_then(Value::as_float).unwrap_or(1.0);
        update_frequency            = simulation_parameters.get("update_frequency")           .and_then(Value::as_float).unwrap_or(10.0);
        update_frequency_server     = simulation_parameters.get("update_frequency_server")    .and_then(Value::as_float).unwrap_or(update_frequency);
        connection_refresh_interval = simulation_parameters.get("connection_refresh_interval").and_then(Value::as_float).unwrap_or(10.0);

        rng_seed                      = simulation_parameters.get("rng_seed")                     .and_then(Value::as_integer).map(|v| v as u64);
        starting_failure_probability  = simulation_parameters.get("starting_failure_probability") .and_then(Value::as_float).unwrap_or(0.0);
        recurrent_failure_probability = simulation_parameters.get("recurrent_failure_probability").and_then(Value::as_float).unwrap_or(0.0);
        assert!((0.0..=1.0).contains(&recurrent_failure_probability));
        assert!((0.0..=1.0).contains(&starting_failure_probability));

        strategy = match &contents.get("strategy") {
            Some(Value::Table(params)) => {
                match params["type"].as_str().unwrap() {
                    "grid" => {
                        let offset = params.get("offset").and_then(Value::as_integer).unwrap_or(0) as usize;
                        Box::new(GridStrategy::new(offset))
                    },
                    "nearest_neighbor" => Box::new(NearestNeighborStrategy::new()),
                    _ => panic!("Invalid strategy type."),
                }
            }
            _ => Box::new(GridStrategy::new(0)),
        }
    } else {
        panic!("More than one argument!");
    }

    let (sender, receiver) = mpsc::channel();

    let sim = Arc::new(Mutex::new(Simulation::new(
        Model::new(
            num_orbital_planes,
            satellites_per_plane,
            inclination.to_radians(),
            constellation_type,
            phasing as usize,
            EARTH_RADIUS + orbiting_altitude,
            max_connections,
        ),
        simulation_speed / update_frequency,
        simulation_speed,
        connection_refresh_interval,
        rng_seed,
        starting_failure_probability,
        recurrent_failure_probability,
        strategy,
        sender,
    )));

    let sim_server = Arc::clone(&sim);

    let mut delay = Duration::from_secs_f64(1.0 / update_frequency);
    let delay_server = Duration::from_secs_f64(1.0 / update_frequency_server);

    let mut server_handle = None;
    let statistics_handle;

    if let Some(file_path) = file_path {
        let mut file = File::create(file_path).expect("Couldn't create file!");
        statistics_handle = thread::spawn(move || { file_thread(receiver, &mut file) });
        // Run simulation without artificial delays (only collect data)
        delay = Duration::ZERO;
    }
    else {
        server_handle = Some(thread::spawn(move || { server_thread(sim_server, delay_server) }));
        statistics_handle = thread::spawn(move || { statistics_thread(receiver) });
    }

    let simulation_handle = thread::spawn(move || { simulation_thread(sim, steps, delay) });
    simulation_handle.join().expect("Couldn't join simulation thread.");

    let _ = statistics_handle.join().expect("Couldn't join statistics thread.");
    if let Some(handle) = server_handle {
        let _ = handle.join().expect("Couldn't join server thread.");
    }

    Ok(())
}

fn simulation_thread(sim: Arc<Mutex<Simulation>>, steps: Option<usize>, delay: Duration) {
    let loop_step = || {
        thread::sleep(delay);
        {
            let mut lock = sim.lock().unwrap();
            lock.step();
        }
    };

    if let Some(steps) = steps {
        for _ in 0..steps {
            loop_step();
        }
    }
    else {
        loop {
            loop_step();
        }
    }
}

fn write(stream: &mut TcpStream, msg: String) -> io::Result<()> {
    let bytes = msg.as_bytes();

    stream.write_all(&(bytes.len() as u32).to_ne_bytes())?;
    stream.write_all(bytes)?;

    Ok(())
}

fn server_thread(sim: Arc<Mutex<Simulation>>, delay: Duration) -> io::Result<()> {
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, SERVER_PORT);
    let listener = TcpListener::bind(addr).unwrap();

    let mut msg;
    let mut size_buf = [0; 4];

    let mut t: f64;
    let mut last_connection_msg = 0.0;
    let mut include_connections: bool;

    for stream in listener.incoming() {
        let mut stream = stream?;

        {
            let lock = sim.lock().unwrap();
            msg = init_msg(&lock);
        }
        if write(&mut stream, msg).is_err() {
            continue;
        }

        stream.set_nonblocking(true)?;
        loop {
            thread::sleep(delay);
            {
                let lock = sim.lock().unwrap();
                t = lock.t();
                include_connections = t - last_connection_msg >= lock.connection_refresh_interval();
                msg = update_msg(&lock, include_connections);
            }
            if include_connections {
                last_connection_msg = t;
            }
            if write(&mut stream, msg).is_err() {
                break;
            }

            if let Ok(4) = stream.read(&mut size_buf) {
                let msg_size = u32::from_le_bytes(size_buf) as usize;

                let mut msg_buf = vec![0; msg_size];
                if stream.read_exact(&mut msg_buf).is_ok() {
                    if let Ok(msg) = String::from_utf8(msg_buf) {
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
        }
    }

    Ok(())
}

fn statistics_thread(channel: Receiver<String>) -> io::Result<()> {
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, STATISTICS_PORT);
    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming() {
        let mut stream = stream?;

        while let Ok(msg) = channel.recv() {
            write(&mut stream, msg)?;
        }
    }

    Ok(())
}

fn file_thread(channel: Receiver<String>, file: &mut File) -> io::Result<()> {
    while let Ok(msg) = channel.recv() {
        file.write_all(msg.as_bytes())?;
    }

    Ok(())
}
