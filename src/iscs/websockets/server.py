
import asyncio
import websockets
import itertools
import json

from vector import Vector3D
from typing import Tuple

from ..model.simulation import Simulation

def init_message(simulation: Simulation) -> str:
    semimajor_axis = simulation.orbital_planes[0].semimajor_axis if simulation.orbital_planes else 0
    inclination    = simulation.orbital_planes[0].inclination    if simulation.orbital_planes else 0

    msg = {
        "msg_type" : "init",
        "semimajor_axis": semimajor_axis,
        "inclination": inclination,
        "orbital_planes": {
            o.id: {
                "longitude": o.longitude,
            }
            for o in simulation.orbital_planes
        },
        "satellites" : {
            s.id: {
                "orbital_plane": s.orbital_plane.id,
                "arg_periapsis": s.arg_periapsis,
            }
            for s in simulation.satellites.values()
        }
    }

    return json.dumps(msg)

def to_tuple(v: Vector3D) -> Tuple[float, float, float]:
    return v.x, v.y, v.z

def update_message(simulation: Simulation) -> str:
    msg = {
        "msg_type": "update",
        "t": simulation.t,
        "satellites": {
            s.id: {
                "position": to_tuple(s.calc_position(simulation.t)),
                "velocity": to_tuple(s.calc_velocity(simulation.t)),
            }
            for s in simulation.satellites.values()
        },
        "connections": list(set(
            (s.id, other)
            for s in simulation.satellites.values()
            for other in s.connections
            if s < other
        )),
    }

    return json.dumps(msg)

async def echo(websocket):
    for _ in itertools.count():
        await asyncio.sleep(0.1)
        s.step()
        await websocket.send(update_message(s))

async def main():
    async with websockets.serve(echo, "localhost", 8765):
        await asyncio.Future()  # run forever

s = Simulation(inclination=0.6, num_orbital_planes=1, satellites_per_plane=1, semimajor_axis=6_921_000, time_step=10)
asyncio.run(main())