extends ImmediateGeometry

# Adapted from https://github.com/dbp8890/line-renderer

const COLOR_DEFAULT := Color.white
const COLOR_SELECTED := Color.red

var sat_a: KinematicBody
var sat_b: KinematicBody
var _color := COLOR_DEFAULT

var valid: bool setget set_valid

func _ready():
	self.valid = true

func _process(_delta: float):
	var pos_a := to_local(sat_a.global_translation)
	var pos_b := to_local(sat_b.global_translation)
	
	clear()
	
	begin(Mesh.PRIMITIVE_LINES)
	
	set_color(_color)
	
	add_vertex(pos_a)
	add_vertex(pos_b)
	
	end()

func on_satellite_selected(satellite: KinematicBody):
	set_selected(satellite == sat_a or satellite == sat_b)

func set_valid(value: bool):
	valid = value
	if !valid:
		set_active(valid)

func set_active(active: bool):
	set_process(active)
	visible = active

func set_selected(selected: bool):
	if selected:
		_color = COLOR_SELECTED
	else:
		_color = COLOR_DEFAULT
