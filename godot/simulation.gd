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

var _tcp := StreamPeerTCP.new()

var _visible_connections := true
var _selected_satellite: KinematicBody

func _ready():
	connect("satellite_selected", hud, "on_satellite_selected")
	hud.connect("connection_visibility_changed", self, "_on_connection_visibility_changed")
	
	$Earth.scale = EARTH_RADIUS * SCALE * Vector3.ONE
	
	if _tcp.connect_to_host("127.0.0.1", 1234) != OK:
		print("Unable to connect to host.")
		set_physics_process(false)
	else:
		print("Connected to host!")

func array_to_vector3(arr: Array) -> Vector3:
	return Vector3(arr[0], arr[1], arr[2])

func _init_simulation(json: Dictionary):
	var satellites: Dictionary = json["satellites"]
	var semimajor_axis: float = json["semimajor_axis"]
	
	_orbital_planes = json["orbital_planes"]
	_inclination = json["inclination"]
	
	var r = semimajor_axis * SCALE
	orbital_plane.mesh.top_radius = r * 0.99
	orbital_plane.mesh.bottom_radius = r * 0.99
	orbital_plane.mesh.height = r * 0.003
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
		satellite.global_translation = position
	
	if json.has("connections"):
		_update_connections(json["connections"])
	
	hud.update_hud(json)

func _update_connections(connections: Array):
	_connections = connections
	var n_connections = len(connections)
	
	for i in n_connections:
		var connection = _connections[i]
		var instance: ImmediateGeometry
		var new: bool = i >= connections_root.get_child_count()
		
		if new:
			instance = connection_scene.instance()
			connect("satellite_selected", instance, "on_satellite_selected")
		else:
			instance = connections_root.get_child(i)
		
		var sat_a: KinematicBody = satellites_root.get_child(connection[0])
		var sat_b: KinematicBody = satellites_root.get_child(connection[1])
		
		instance.sat_a = sat_a
		instance.sat_b = sat_b
		instance.set_selected(_selected_satellite == sat_a or _selected_satellite == sat_b)
		
		if _visible_connections:
			instance.set_active(true)
		
		if new:
			connections_root.add_child(instance)
		else:
			instance.valid = true
	
	for i in range(n_connections, connections_root.get_child_count()):
		var node: ImmediateGeometry = connections_root.get_child(i)
		node.valid = false

func _physics_process(_delta: float):
	if _tcp.get_status() == StreamPeerTCP.STATUS_CONNECTED:
		var bytes = _tcp.get_available_bytes()
		if bytes > 0:
			var data := _tcp.get_utf8_string()
			var json := JSON.parse(data)
			if json.error == OK:
				var result = json.result
				if result is Dictionary:
					match result["msg_type"]:
						"init": _init_simulation(result)
						"update": _update_simulation(result)
			else:
				print(json.error_string)

func _unhandled_input(event: InputEvent):
	if event.is_action_pressed("select"):
		var mouse_pos := get_viewport().get_mouse_position()
		var from := camera.project_ray_origin(mouse_pos)
		var to := camera.project_ray_normal(mouse_pos) * MAX_RAY_LENGTH
		
		var ray_result := get_world().direct_space_state.intersect_ray(from, to, [self])
		
		if ray_result and ray_result.collider.get_collision_layer_bit(0):
			# Ray collided with a satellite
			_select_satellite(ray_result.collider)
		else:
			_select_satellite(null)

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

func _on_connection_visibility_changed(value: bool):
	_visible_connections = value
	for node in connections_root.get_children():
		if node.valid:
			node.set_active(_visible_connections)
