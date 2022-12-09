extends KinematicBody

var velocity := Vector3.ZERO

func _physics_process(_delta: float):
	velocity = move_and_slide(velocity)
