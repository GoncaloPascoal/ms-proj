
<div align="center">
    <h1>Interactive Satellite Megaconstellation Simulation</h1>
    <h3>Project for the Modelling and Simulation (MS) curricular unit.</h3>
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
python3 python/tcp_socket.py
```
- Optionally, you can also view statistics from simulation data which has been saved to a file:
```
python3 python/statistics.py path/to/simulation_data.json
```

## Configuration File

This section explains the parameters that can be used in TOML configuration files for the simulator.

- `name` (optional): name of the scenario

### Constellation Table (required)
- `altitude`: orbiting altitude of the satellites
- `num_orbital_planes`: number of orbital planes in the constellation
- `satellites_per_plane`: number of satellites per orbital plane
- `inclination`: angle between the orbital planes and the Earth's equatorial plane
- `max_connections`: maximum number of links that a single satellite can establish
- `type`: constellation type
  - in Walker Delta configurations (`"delta"`), the longitude values of the constellation's orbital planes span 360º around the Earth
  - in Walker Star Configurations (`"star"`), the longitude values of the constellation's orbital planes span 180º around the Earth
- `phasing`: used to calculate the phase offset (difference between the argument of periapsis of equivalent satellites from subsequent planes)

| Parameter | Required | Value Type | Default Value | Interval of Accepted Values |
| ---------------------- | --- | ------- | --------- | ------------------------- |
| `altitude`             | Yes | float   | N/A       | > 0                       |
| `num_orbital_planes`   | Yes | integer | N/A       | > 0                       |
| `satellites_per_plane` | Yes | integer | N/A       | > 0                       |
| `inclination`          | Yes | float   | N/A       | [0, 90]                   |
| `max_connections`      | Yes | integer | N/A       | > 0                       |
| `type`                 | No  | string  | `"delta"` | (`"delta"`, `"star"`)     |
| `phasing`              | No  | integer | 0         | [0, `num_orbital_planes`[ |

### Simulation Table (optional)
- `file_path`: file to which the statistics data from the simulation will be saved
  - When specified, the core simulation will run without artificial delays and will not communicate with the visualization or statistics component.
- `steps`: maximum number of time steps to run the simulation for
- `simulation_speed`: speed of the simulation (1 is real-time, 5 means that for every simulation second, five actual seconds have passed)
- `update_frequency`: frequency of updates to the simulation's state
- `update_frequency_server`: frequency of update messages sent to the visualization component
- `connection_refresh_interval`: interval between connection updates (in seconds)
- `rng_seed`: fixed seed for the random number generator; used to obtain reproducible scenarios
- `starting_failure_probability`: probability that a satellite will fail at the start of the simulation
- `recurrent_failure_probability`: probability that a satellite will fail at each connection update

| Parameter | Required | Value Type | Default Value | Interval of Accepted Values |
| ------------------------------- | ---------- | ------- | ------------------ | --------------- |
| `file_path`                     | No         | string  | None               | valid file path |
| `steps` | Only when `file_path` is specified | integer | None               | > 0             |
| `simulation_speed`              | No         | float   | 1.0                | > 0             |
| `update_frequency`              | No         | float   | 10.0               | > 0             |
| `update_frequency_server`       | No         | float   | `update_frequency` | > 0             |
| `connection_refresh_interval`   | No         | float   | 10.0               | > 0             |
| `rng_seed`                      | No         | integer | None               | >= 0            |
| `starting_failure_probability`  | No         | float   | 0.0                | [0.0, 1.0]      |
| `recurrent_failure_probability` | No         | float   | 0.0                | [0.0, 1.0]      |

### Strategy Table (optional)
- `type`: type of connection strategy
- `offset`: offset used to establish connections in the `"grid"` strategy

| Parameter | Required | Value Type | Default Value | Interval of Accepted Values |
| -------- | --- | ------- | -------- | -------------------------------- |
| `type`   | No  | string  | `"grid"` | (`"grid"`, `"nearest_neighbor"`) |
| `offset` | No  | integer | 0        | >= 0                             |

## Interactive Visualization Tool

Executable versions of the visualization application for both Windows and Linux can be found in the project's GitHub repository in the **releases** section.

### Controls

- **Left mouse button:** select satellite
- **Right mouse button:** rotate camera
- **Mouse wheel:** zoom in / out
