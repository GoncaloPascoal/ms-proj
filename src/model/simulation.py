
from typing import Dict, List

from .satellite import OrbitalPlane, Satellite
from math import pi

class Simulation:
    def __init__(self, num_orbital_planes: int = 0, satellites_per_plane: int = 0,
            inclination: float = 0, semimajor_axis: float = 0,
            time_step: float = 0.01, sim_speed: float = 1):
        self.orbital_planes: List[OrbitalPlane] = [
            OrbitalPlane(
                semimajor_axis=semimajor_axis,
                inclination=inclination,
                longitude=2 * pi * i / num_orbital_planes,
            )
            for i in range(num_orbital_planes)
        ]

        self.satellites: Dict[int, Satellite] = dict()
        for plane in self.orbital_planes:
            for i in range(satellites_per_plane):
                s = Satellite(plane, 2 * pi * i / satellites_per_plane)
                self.satellites[s.id] = s

        self.time_step = time_step
        self.t = 0
        self.sim_speed = sim_speed

    def step(self):
        self.t += self.time_step
