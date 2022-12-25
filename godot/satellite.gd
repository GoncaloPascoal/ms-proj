extends KinematicBody

export(int) var id := 0

const LIGHT_ENERGY := 10.0

onready var _light: OmniLight = $Light

var orbital_plane: Dictionary
var velocity := Vector3.ZERO

func _physics_process(_delta: float):
	velocity = move_and_slide(velocity)

func enable_light():
	_light.light_energy = LIGHT_ENERGY

func disable_light():
	_light.light_energy = 0.0
