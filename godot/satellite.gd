extends KinematicBody

const VIEW_CONE_RATIO := 0.75

onready var light: OmniLight = $Light
onready var fire: Particles = $Fire
onready var view_cone: MeshInstance = $ViewConeAnchor/ViewCone

var id := 0
var orbital_plane: Dictionary
var status := true setget _set_status

# Note: altitude value is scaled and should not be displayed to user
var altitude: float
var view_angle: float

func _ready():
	set_coverage_visibility(false)
	
	# Initialize view cone
	view_cone.mesh.height = altitude
	view_cone.mesh.top_radius = altitude * tan(VIEW_CONE_RATIO * view_angle)
	view_cone.translation = (0.05 + 0.5 * altitude) * Vector3.UP

func _physics_process(_delta: float):
	look_at_earth()

func look_at_earth():
	look_at(Vector3.ZERO, Vector3.UP)

func set_coverage_visibility(value: bool):
	value = value and status
	view_cone.visible = value
	set_physics_process(value)
	if value:
		look_at_earth()

func set_selected(value: bool):
	if value:
		_enable_light()
	else:
		_disable_light()
	set_coverage_visibility(value)

func _enable_light():
	light.show()

func _disable_light():
	light.hide()

func _set_status(value: bool):
	status = value
	fire.emitting = !status
	if !value:
		set_coverage_visibility(value)
