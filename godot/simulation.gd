extends Spatial

const EARTH_RADIUS := 6.371e6
const SCALE := 2e-6

const FPS = 10.0

export(PackedScene) var satellite_scene

onready var earth: MeshInstance = $Earth
onready var satellites_root: Spatial = $SatellitesRoot

var time_since_start = 0
var timestamp_index = 1
var sim_data = {}

func _ready():
	var mesh = SphereMesh.new()
	mesh.radius = EARTH_RADIUS * SCALE
	mesh.height = 2 * mesh.radius
	mesh.material = load("mat.tres")

	earth.mesh = mesh

	var file = File.new()
	var filename = "../data/test.sim"
	if not file.file_exists(filename):
		print("File not found")
		return
	file.open(filename, File.READ)
	sim_data = parse_json(file.get_as_text())
	_init_simulation(sim_data[0])

func list_to_vector3(l) -> Vector3:
	return Vector3(l[0], l[1], l[2])

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
		
		var position = list_to_vector3(data["position"]) * SCALE
		var velocity = list_to_vector3(data["velocity"]) * SCALE

		satellite.reset_physics_interpolation()
		satellite.global_translation = position
		satellite.velocity = velocity

func _physics_process(_delta: float):
	# ativar o update_simulation conforme necessário
	time_since_start += _delta
	if timestamp_index < len(sim_data) and timestamp_index / FPS < time_since_start:
		_update_simulation(sim_data[timestamp_index])
		timestamp_index += 1

func _input(event: InputEvent):
	if event is InputEventMouseMotion:
		pass
