extends Spatial

const EARTH_RADIUS := 6.371e6
const SCALE := 2e-6

export(String) var websocket_url = "ws://localhost:1234"
export(PackedScene) var satellite_scene

onready var satellites_root: Spatial = $SatellitesRoot
onready var camera: Camera = $CameraGimbal/InnerGimbal/Camera

var _client := WebSocketClient.new()
var _selected_satellite: KinematicBody

func _ready():
	_client.connect("connection_established", self, "_connected")
	_client.connect("data_received", self, "_on_data")

	var err := _client.connect_to_url(websocket_url)
	if err != OK:
		print("Unable to connect to host.")
		set_physics_process(false)

func array_to_vector3(arr: Array) -> Vector3:
	return Vector3(arr[0], arr[1], arr[2])

func _init_simulation(json):
	var satellites: Dictionary = json["satellites"]
	
	for id in satellites:
		var instance = satellite_scene.instance()
		instance.id = int(id)
		satellites_root.add_child(instance)

func _update_simulation(json):
	var satellites: Dictionary = json["satellites"]
	
	for id in satellites:
		var data = satellites[id]
		var satellite = satellites_root.get_child(int(id))
		
		var position = array_to_vector3(data["position"]) * SCALE
		var velocity = array_to_vector3(data["velocity"]) * SCALE

		satellite.reset_physics_interpolation()
		satellite.global_translation = position
		satellite.velocity = velocity

func _physics_process(_delta: float):
	if Input.is_action_just_pressed("select"):
		var mouse_pos := get_viewport().get_mouse_position()
		var from := camera.project_ray_origin(mouse_pos)
		var to := camera.project_ray_normal(mouse_pos) * camera.far

		var ray_result := get_world().direct_space_state.intersect_ray(from, to, [self])
		
		if ray_result:
			_selected_satellite = ray_result.collider

	_client.poll()

func _connected(_proto: String = ""):
	print("Connected to host!")

func _on_data():
	var json := JSON.parse(_client.get_peer(1).get_packet().get_string_from_utf8())

	if json.error == OK:
		var result = json.result
		match result["msg_type"]:
			"init": _init_simulation(result)
			"update": _update_simulation(result)
