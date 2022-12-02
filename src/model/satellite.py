
from math import sqrt, pi
from vector import Vector3D
import vector

GM: float = 3.986004418e14      # m^3 s^-2
EARTH_RADIUS: float = 6.371e6   # m

class OrbitalPlane:
    def __init__(self, eccentricity: float = 0, semimajor_axis: float = 0,
            inclination: float = 0, longitude: float = 0):
        if eccentricity != 0:
            raise NotImplementedError

        self.eccentricity = eccentricity
        self.semimajor_axis = semimajor_axis
        self.inclination = inclination
        self.longitude = longitude

        self.orbital_speed = sqrt(GM / semimajor_axis)
        self.angular_speed = self.orbital_speed / semimajor_axis

class Satellite:
    __id = 0

    def __next_id() -> int:
        ret = Satellite.__id
        Satellite.__id += 1
        return ret

    def __init__(self, orbital_plane: OrbitalPlane, arg_periapsis: float = 0):
        self.id = Satellite.__next_id()
        self.orbital_plane = orbital_plane
        self.arg_periapsis = arg_periapsis

    def calc_position(self, t: float) -> Vector3D:
        r = self.orbital_plane.semimajor_axis
        true_anomaly = (t * self.orbital_plane.angular_speed) % (2 * pi)

        position: Vector3D = vector.obj(x=r, y=0, z=0)
        return position.rotateY(self.arg_periapsis + true_anomaly).rotateX(self.orbital_plane.inclination)

    def calc_velocity(self, t: float) -> Vector3D:
        direction = self.calc_position(t).rotateY(pi / 2).unit()
        return self.orbital_plane.orbital_speed * direction
