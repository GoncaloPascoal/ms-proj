extends ImmediateGeometry

# Adapted from https://github.com/dbp8890/line-renderer

const THICKNESS := 0.05
const COLOR_DEFAULT := Color.white
const COLOR_SELECTED := Color.red

var sat_a: KinematicBody
var sat_b: KinematicBody
var _color := COLOR_DEFAULT
var _thickness := THICKNESS

func _process(_delta: float):
	var pos_a := to_local(sat_a.global_translation)
	var pos_b := to_local(sat_b.global_translation)
	var ab := pos_b - pos_a;
	
	var camera_origin := to_local(get_viewport().get_camera().global_transform.origin)
	var orthogonal_ab := (camera_origin - ((pos_a + pos_b) / 2)).cross(ab).normalized() * _thickness
	
	var a_to_ab = pos_a + orthogonal_ab
	var a_from_ab = pos_a - orthogonal_ab
	var b_to_ab = pos_b + orthogonal_ab
	var b_from_ab = pos_b - orthogonal_ab
	
	clear()
	
	begin(Mesh.PRIMITIVE_TRIANGLES)
	
	set_color(_color)
	
	set_uv(Vector2(1, 0))
	add_vertex(a_to_ab)
	set_uv(Vector2.ZERO)
	add_vertex(b_to_ab)
	set_uv(Vector2.ONE)
	add_vertex(a_from_ab)
	set_uv(Vector2.ZERO)
	add_vertex(b_to_ab)
	set_uv(Vector2(0, 1))
	add_vertex(b_from_ab)
	set_uv(Vector2.ONE)
	add_vertex(a_from_ab)
	
	end()

func on_satellite_selected(satellite: KinematicBody):
	set_selected(satellite == sat_a or satellite == sat_b)

func set_selected(selected: bool):
	if selected:
		_color = COLOR_SELECTED
		_thickness = 2 * THICKNESS
	else:
		_color = COLOR_DEFAULT
		_thickness = THICKNESS
