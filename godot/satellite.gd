extends KinematicBody

export(int) var id := 0

onready var _light: OmniLight = $Light

var orbital_plane: Dictionary

func enable_light():
	_light.show()

func disable_light():
	_light.hide()
