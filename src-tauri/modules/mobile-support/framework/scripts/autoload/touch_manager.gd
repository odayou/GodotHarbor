extends Node

signal touch_joystick_input(direction: Vector2)
signal touch_action_pressed(action: String)
signal touch_action_released(action: String)

var joystick_direction: Vector2 = Vector2.ZERO
var is_touch_device: bool = false

func _ready() -> void:
	is_touch_device = DisplayServer.is_touchscreen_available()
	if not is_touch_device:
		var touch_overlay = get_tree().get_first_node_in_group("touch_overlay")
		if touch_overlay:
			touch_overlay.visible = false

func _input(event: InputEvent) -> void:
	if event is InputEventScreenTouch:
		is_touch_device = true
	if event is InputEventScreenDrag:
		is_touch_device = true

func set_joystick_direction(dir: Vector2) -> void:
	joystick_direction = dir
	touch_joystick_input.emit(dir)
	Input.action_press("move_left", maxf(-dir.x, 0.0))
	Input.action_press("move_right", maxf(dir.x, 0.0))
	Input.action_press("move_up", maxf(-dir.y, 0.0))
	Input.action_press("move_down", maxf(dir.y, 0.0))
