extends Spatial

const EARTH_RADIUS := 6.371e6
const SCALE := 2e-6

export(PackedScene) var satellite_scene

onready var earth: MeshInstance = $Earth
onready var satellites_root: Spatial = $SatellitesRoot

var _status := 0
var _stream := StreamPeerTCP.new()

func _ready():
	var mesh = SphereMesh.new()
	mesh.radius = EARTH_RADIUS * SCALE
	mesh.height = 2 * mesh.radius
	mesh.material = load("mat.tres")
	
	earth.mesh = mesh
	
	# Socket connection
	if _stream.connect_to_host("127.0.0.1", 1234) != OK:
		print("Error connecting to server.")
		return

func array_to_vector3(arr: Array) -> Vector3:
	return Vector3(arr[0], arr[1], arr[2])

func _init_simulation(json):
	var satellites: Dictionary = json["satellites"]

	for id in satellites:
		var _data = satellites[id] # TODO
		satellites_root.add_child(satellite_scene.instance())

func _update_simulation(json):
	var satellites = json["satellites"]

	for id in satellites:
		var data = satellites[id]
		var satellite = satellites_root.get_child(int(id))
		
		var position = array_to_vector3(data["position"]) * SCALE
		var velocity = array_to_vector3(data["velocity"]) * SCALE

		satellite.reset_physics_interpolation()
		satellite.global_translation = position
		satellite.velocity = velocity

func _physics_process(_delta: float):
	_status = _stream.get_status()

	if _status == _stream.STATUS_CONNECTED:
		var bytes := _stream.get_available_bytes()
		if bytes > 0:
			var data := _stream.get_partial_data(bytes)
			if data[0] == OK:
				var json = parse_json(data[1].get_string_from_utf8())
				_handle_msg(json)

func _handle_msg(json):
	match json["msg_type"]:
		"init":
			_init_simulation(json)
		"update":
			_update_simulation(json)
