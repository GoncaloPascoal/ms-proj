extends Control

onready var time_step: Label = $SimulationInfo/TimeStep

onready var satellite_info: Panel = $SatelliteInfo
onready var selected_position: Label = $SatelliteInfo/Position
onready var selected_velocity: Label = $SatelliteInfo/Velocity

var _selected_satellite: KinematicBody2D

func _ready():
	satellite_info.visible = false

func init_hud(json):
	$SimulationInfo/OrbitalPlanes.text = "Orbital Planes: " + str(len(json["orbital_planes"]))
	$SimulationInfo/Satellites.text = "Satellites: " + str(len(json["satellites"]))

func _process(delta: float):
	pass

func update_hud(json):
	time_step.text = "Time: " + str(json["t"])
