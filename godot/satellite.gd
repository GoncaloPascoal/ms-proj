extends KinematicBody

onready var light: OmniLight = $Light
onready var fire: Particles = $Fire

var id := 0
var orbital_plane: Dictionary
var status := true setget _set_status

func enable_light():
	light.show()

func disable_light():
	light.hide()

func _set_status(v: bool):
	status = v
	fire.emitting = !status
