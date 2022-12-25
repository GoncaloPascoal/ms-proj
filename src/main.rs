use std::{fs, env, path::Path, net::TcpListener, time::Duration};
use tungstenite::{Message, accept};
use toml::Value;

use model::{EARTH_RADIUS, Simulation, init_msg, update_msg};

pub mod model;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let orbiting_altitude : f64;
    let num_orbital_planes : usize;
    let satellites_per_plane : usize;
    let inclination : f64;
    let time_step : f64;

    if args.len() == 1 {
        orbiting_altitude = 0.55e6;
        num_orbital_planes = 10;
        satellites_per_plane = 20;
        inclination = 0.6;
        time_step = 1.0;
    } else if args.len() == 2 {
        let path = Path::new(&args[1]);
        if !path.exists() {
            panic!("Path doesn't exist!")
        }

        let contents = fs::read_to_string(path).unwrap().parse::<Value>().unwrap();
        println!("{}", contents["constellation parameters"]["altitude"]);

        orbiting_altitude = contents["constellation parameters"]["altitude"].as_float().unwrap();
        num_orbital_planes = contents["constellation parameters"]["number of orbital planes"].as_integer().unwrap() as usize;
        satellites_per_plane = contents["constellation parameters"]["satellites per plane"].as_integer().unwrap() as usize;
        inclination = contents["constellation parameters"]["inclination"].as_float().unwrap();
        time_step = contents["simulation parameters"]["timestep"].as_float().unwrap();
    } else {
        panic!("More than one argument!");
    }
    
    let mut sim = Simulation::new(
        num_orbital_planes,
        satellites_per_plane,
        inclination,
        EARTH_RADIUS + orbiting_altitude,
        time_step,
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
