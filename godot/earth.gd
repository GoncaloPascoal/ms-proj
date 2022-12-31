extends Area

const SECONDS_IN_DAY := 24 * 60 * 60

var simulation_speed: float setget set_simulation_speed

func _ready():
	self.simulation_speed = 0.0

func _physics_process(delta: float):
	rotate_y(2 * PI * simulation_speed * delta / SECONDS_IN_DAY)

func set_simulation_speed(value: float):
	simulation_speed = value
	set_physics_process(simulation_speed != 0.0)
