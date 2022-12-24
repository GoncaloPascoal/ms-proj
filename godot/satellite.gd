extends KinematicBody

export(int) var id := 0

var velocity := Vector3.ZERO

func _physics_process(_delta: float):
	velocity = move_and_slide(velocity)
