
GM: float = 3.986004418e14      # m^3 s^-2
EARTH_RADIUS: float = 6.371e6   # m

from math import sqrt
from vector import Vector3D

class OrbitalPlane:
    def __init__(self, eccentricity: float = 0, semimajor_axis: float = 0,
            inclination: float = 0, longitude: float = 0):
        self.eccentricity = eccentricity
        self.semimajor_axis = semimajor_axis
        self.inclination = inclination
        self.longitude = longitude

        self.orbital_speed = sqrt(GM / semimajor_axis)

class Satellite:
    def __init__(self, orbital_plane: OrbitalPlane, arg_periapsis: float = 0):
        self.orbital_plane = orbital_plane
        self.arg_periapsis = arg_periapsis

    def calc_position(true_anomaly: float) -> Vector3D:
        # TODO
        pass
