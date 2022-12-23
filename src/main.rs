use std::{io::Write, net::TcpListener};

pub mod model;
use model::{EARTH_RADIUS, Simulation, init_msg, update_msg};

fn main() -> std::io::Result<()> {
    let orbiting_altitude = 0.55e6;
    let mut sim = Simulation::new(
        10,
        20,
        0.6,
        EARTH_RADIUS + orbiting_altitude,
        10.0
    );
    
    let listener = TcpListener::bind("127.0.0.1:1234")?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        let mut stream = stream?;
        stream.write_all(init_msg(&sim).as_bytes());

        for _ in 0..1000 {
            sim.step();
            stream.write_all(update_msg(&sim).as_bytes());
        }
    }

    Ok(())
}
