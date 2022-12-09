extends Spatial

const EARTH_RADIUS := 6.371e6
const SCALE := 2e-6

export var websocket_url = "ws://localhost:8765"

onready var earth: MeshInstance = $Earth
onready var satellite: MeshInstance = $Satellite
onready var ray: RayCast = $RayCast

var _client = WebSocketClient.new()

func _ready():
	var mesh = SphereMesh.new()
	mesh.radius = EARTH_RADIUS * SCALE
	mesh.height = 2 * mesh.radius
	mesh.material = load("mat.tres")

	earth.mesh = mesh

	_client.connect("connection_established", self, "_connected")
	_client.connect("data_received", self, "_on_data")

	var err = _client.connect_to_url(websocket_url)

	if err != OK:
		print("Unable to connect")
		set_process(false)

func _connected(_proto: String = ""):
	print("Connected!")

func list_to_vector3(l) -> Vector3:
	return Vector3(l[0], l[1], l[2])

func _on_data():
	var json = JSON.parse(_client.get_peer(1).get_packet().get_string_from_utf8()).result

	var position = list_to_vector3(json["satellites"]["0"]["position"]) * SCALE
	var velocity = list_to_vector3(json["satellites"]["0"]["velocity"]) * SCALE

	satellite.global_translation = position
	ray.global_translation = position
	ray.cast_to = velocity * 200

func _process(_delta: float):
	_client.poll()
