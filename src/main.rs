
use std::{net::TcpListener, time::Duration};
use tungstenite::{Message, accept};

use model::{EARTH_RADIUS, Simulation, init_msg, update_msg};

pub mod model;

fn main() -> std::io::Result<()> {
    let orbiting_altitude = 0.55e6;
    let mut sim = Simulation::new(
        10,
        20,
        0.6,
        EARTH_RADIUS + orbiting_altitude,
        1.0,
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
