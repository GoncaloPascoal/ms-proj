extends Spatial

signal satellite_selected(satellite)

const EARTH_RADIUS := 6.371e6
const SCALE := 2e-6
const MAX_RAY_LENGTH := 1000.0

export(String) var websocket_url = "ws://localhost:1234"
export(PackedScene) var satellite_scene
export(PackedScene) var connection_scene

onready var hud: Control = $HUD
onready var satellites_root: Spatial = $SatellitesRoot
onready var connections_root: Spatial = $ConnectionsRoot
onready var camera: Camera = $CameraGimbal/InnerGimbal/Camera
onready var orbital_plane: MeshInstance = $OrbitalPlane

var _orbital_planes: Dictionary
var _connections := []
var _inclination: float

var _client := WebSocketClient.new()

var _selected_satellite: KinematicBody

func _ready():
	connect("satellite_selected", hud, "on_satellite_selected")
	
	$Earth.scale = EARTH_RADIUS * SCALE * Vector3.ONE
	
	_client.connect("connection_established", self, "_connected")
	_client.connect("data_received", self, "_on_data")
	
	var err := _client.connect_to_url(websocket_url)
	if err != OK:
		print("Unable to connect to host.")
		set_physics_process(false)

func array_to_vector3(arr: Array) -> Vector3:
	return Vector3(arr[0], arr[1], arr[2])

func _init_simulation(json: Dictionary):
	var satellites: Dictionary = json["satellites"]
	var semimajor_axis: float = json["semimajor_axis"]
	
	_orbital_planes = json["orbital_planes"]
	_inclination = json["inclination"]
	
	var r = semimajor_axis * SCALE
	orbital_plane.mesh.top_radius = r
	orbital_plane.mesh.bottom_radius = r
	orbital_plane.mesh.height = r * 0.005
	orbital_plane.rotation.x = _inclination
	
	for id in satellites:
		var data = satellites[id]
		var instance = satellite_scene.instance()
		instance.id = int(id)
		instance.orbital_plane = _orbital_planes[data["orbital_plane"]]
		
		satellites_root.add_child(instance)
	
	hud.init_hud(json)

func _update_simulation(json: Dictionary):
	var satellites: Dictionary = json["satellites"]
	
	for id in satellites:
		var data = satellites[id]
		var satellite = satellites_root.get_child(int(id))
		
		var position = array_to_vector3(data["position"]) * SCALE
		var velocity = array_to_vector3(data["velocity"]) * SCALE
		
		satellite.global_translation = position
		satellite.velocity = velocity
		
		satellite.reset_physics_interpolation()
	
	if json.has("connections"):
		_update_connections(json["connections"])
	
	hud.update_hud(json)

func _update_connections(connections: Array):
	_connections = connections
	
	for child in connections_root.get_children():
		child.queue_free()
	
	for connection in _connections:
		var sat_a: KinematicBody = satellites_root.get_child(connection[0])
		var sat_b: KinematicBody = satellites_root.get_child(connection[1])
		
		var instance: ImmediateGeometry = connection_scene.instance()
		instance.sat_a = sat_a
		instance.sat_b = sat_b
		instance.set_selected(_selected_satellite == sat_a or _selected_satellite == sat_b)
		connect("satellite_selected", instance, "on_satellite_selected")
		
		connections_root.add_child(instance)

func _physics_process(_delta: float):
	if Input.is_action_just_pressed("select"):
		var mouse_pos := get_viewport().get_mouse_position()
		var from := camera.project_ray_origin(mouse_pos)
		var to := camera.project_ray_normal(mouse_pos) * MAX_RAY_LENGTH
		
		var ray_result := get_world().direct_space_state.intersect_ray(from, to, [self])
		
		if ray_result and ray_result.collider.get_collision_layer_bit(0):
			# Ray collided with a satellite
			_select_satellite(ray_result.collider)
		else:
			_select_satellite(null)
	
	_client.poll()

func _select_satellite(satellite: KinematicBody):
	if _selected_satellite:
		_selected_satellite.disable_light()
	
	_selected_satellite = satellite
	if _selected_satellite:
		_selected_satellite.enable_light()
		var longitude: float = _selected_satellite.orbital_plane["longitude"]
		orbital_plane.rotation.y = longitude
		orbital_plane.visible = true
		orbital_plane.reset_physics_interpolation()
	else:
		orbital_plane.visible = false
	
	emit_signal("satellite_selected", _selected_satellite)

func _connected(_proto: String = ""):
	print("Connected to host!")

func _on_data():
	var json := JSON.parse(_client.get_peer(1).get_packet().get_string_from_utf8())
	
	if json.error == OK:
		var result = json.result
		
		if result is Dictionary:
			match result["msg_type"]:
				"init": _init_simulation(result)
				"update": _update_simulation(result)
