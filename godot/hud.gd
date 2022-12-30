extends Control

signal connection_visibility_changed(value)

onready var time_step: Label = $SimulationInfo/TimeStep

onready var satellite_info: Panel = $SatelliteInfo
onready var selected_id: Label = $SatelliteInfo/ID
onready var selected_position: Label = $SatelliteInfo/Position
onready var selected_connections: Label = $SatelliteInfo/Connections

onready var check_box_connection_visibility: CheckBox = $Settings/ConnectionVisibility

onready var fps: Label = $MiscInfo/FPS

var _selected_satellite: KinematicBody
var _connections := []

func _ready():
	satellite_info.visible = false
	check_box_connection_visibility.connect("toggled", self, "_on_connection_visibility_toggled")

func init_hud(json: Dictionary):
	$SimulationInfo/OrbitalPlanes.text = "Orbital Planes: " + str(len(json["orbital_planes"]))
	$SimulationInfo/Satellites.text = "Satellites: " + str(len(json["satellites"]))

func _process(_delta: float):
	fps.text = "FPS: " + str(Engine.get_frames_per_second())
	
	if _selected_satellite:
		selected_position.text = "Position: " + str(_selected_satellite.global_translation)

func _is_selected_connection(connection: Array) -> bool:
	return _selected_satellite.id in connection

func _other_satellite(connection: Array) -> int:
	return connection[0] if connection[0] != _selected_satellite.id else connection[1]

func format_time(t: int) -> String:
	var hours := t / 3600
	var minutes := (t % 3600) / 60
	var seconds := t % 60
	
	return "%02d:%02d:%02d" % [hours, minutes, seconds]

func update_hud(json: Dictionary):
	time_step.text = "Time: " + format_time(json["t"])
	if json.has("connections"):
		_connections = json["connections"]
		_update_connections()

func _on_connection_visibility_toggled(value: bool):
	emit_signal("connection_visibility_changed", value)

func on_satellite_selected(satellite: KinematicBody):
	_selected_satellite = satellite
	satellite_info.visible = _selected_satellite != null
	if _selected_satellite:
		selected_id.text = "ID: " + str(_selected_satellite.id)
		_update_connections()

func _update_connections():
	if _selected_satellite:
		var sc := []
		for c in _connections:
			if _is_selected_connection(c):
				sc.append(str(_other_satellite(c)))
		selected_connections.text = "Connections: " + ", ".join(sc)
