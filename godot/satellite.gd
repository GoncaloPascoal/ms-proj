extends KinematicBody

onready var _light: OmniLight = $Light

var id := 0
var orbital_plane: Dictionary
var status := true setget _set_status

func enable_light():
	_light.show()

func disable_light():
	_light.hide()

func _set_status(v: bool):
	status = v
