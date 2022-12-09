
from model.satellite import Satellite, OrbitalPlane

def main():
    s = Satellite(OrbitalPlane(semimajor_axis=100, inclination=1))

    print(s.calc_position(0))
    print(s.calc_position(1))
    print(s.calc_position(2))
 
       
    # {
    #     "msg_type": "update",
    #     "timestep" : 15,
    #     "satellites" : {
    #         "1234" : {
    #             "coordinates" : [1, 2, 3],
    #             "fail" : 24
    #         },
    #         "..."
    #     },
    #     "connections" : [["1", "2"], ["1", "3"], "..."]
    # }

if __name__ == '__main__':
    main()
