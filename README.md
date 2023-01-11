
<div align="center">
    <h1>Interactive Satellite Megaconstellation Simulation</h1>
    Project for the Modelling and Simulation (MS) curricular unit.
</div>

## Setup

- Ensure that the Rust compiler and `cargo` are installed. The quickest way to do so is to visit https://rustup.rs.
- Build the project in release mode:
```
cargo build --release
```
- Run the simulation, optionally providing a path to a TOML configuration file:
```
cargo run --release path/to/config.toml
```
- Run the visualization script, which will automatically connect to the simulation through a TCP socket.
```
python3 python/websocket.py
```
