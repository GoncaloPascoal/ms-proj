extends KinematicBody

onready var _light: OmniLight = $Light

var id := 0
var orbital_plane: Dictionary
var alive := true setget _set_alive

func enable_light():
	_light.show()

func disable_light():
	_light.hide()

func _set_alive(v: bool):
	alive = v
