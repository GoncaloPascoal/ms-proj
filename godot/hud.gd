extends Control

onready var time_step: Label = $SimulationInfo/TimeStep

onready var satellite_info: Panel = $SatelliteInfo
onready var selected_id: Label = $SatelliteInfo/ID
onready var selected_position: Label = $SatelliteInfo/Position
onready var selected_velocity: Label = $SatelliteInfo/Velocity

var _selected_satellite: KinematicBody

func _ready():
	satellite_info.visible = false

func init_hud(json):
	$SimulationInfo/OrbitalPlanes.text = "Orbital Planes: " + str(len(json["orbital_planes"]))
	$SimulationInfo/Satellites.text = "Satellites: " + str(len(json["satellites"]))

func _process(_delta: float):
	if _selected_satellite:
		selected_position.text = "Position: " + str(_selected_satellite.global_translation)
		selected_velocity.text = "Velocity: " + str(_selected_satellite.velocity)

func update_hud(json):
	time_step.text = "Time: " + str(json["t"])

func on_satellite_selected(satellite: KinematicBody):
	_selected_satellite = satellite
	satellite_info.visible = _selected_satellite != null
	if _selected_satellite:
		selected_id.text = "ID: " + str(_selected_satellite.id)
