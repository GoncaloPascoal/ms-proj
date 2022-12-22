extends Spatial

export(float, 1.0, 5.0) var max_zoom = 3.0
export(float, 0.1, 1.0) var min_zoom = 0.5
export(float, 0.01, 1.0) var zoom_speed = 0.1

export(float) var mouse_sensitivity = 0.005

onready var inner_gimbal: Spatial = $InnerGimbal
onready var camera: Camera = $InnerGimbal/Camera

var zoom = 1.0

func _process(_delta: float):
	scale = lerp(scale, Vector3.ONE * zoom, zoom_speed)

func _input(event: InputEvent):
	if event.is_action_pressed("cam_zoom_in"):
		zoom -= zoom_speed
	if event.is_action_pressed("cam_zoom_out"):
		zoom += zoom_speed
	zoom = clamp(zoom, min_zoom, max_zoom)

	if Input.is_action_pressed("cam_rotate"):
		if event is InputEventMouseMotion:
			if event.relative.x != 0:
				rotate_object_local(Vector3.UP, -event.relative.x * mouse_sensitivity)
			if event.relative.y != 0:
				inner_gimbal.rotate_object_local(Vector3.RIGHT, -event.relative.y * mouse_sensitivity)
