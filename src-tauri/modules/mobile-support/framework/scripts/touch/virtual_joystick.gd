extends Control

@export var max_distance: float = 100.0
@export var deadzone: float = 10.0

var _is_pressed: bool = false
var _start_position: Vector2 = Vector2.ZERO
var _current_direction: Vector2 = Vector2.ZERO

@onready var knob: Control = $Knob
@onready var base: Control = $Base

func _ready() -> void:
	if not TouchManager.is_touch_device:
		visible = false

func _gui_input(event: InputEvent) -> void:
	if event is InputEventScreenTouch:
		if event.pressed:
			_is_pressed = true
			_start_position = event.position
		else:
			_is_pressed = false
			_current_direction = Vector2.ZERO
			if knob:
				knob.position = Vector2.ZERO
			TouchManager.set_joystick_direction(Vector2.ZERO)
	elif event is InputEventScreenDrag and _is_pressed:
		var diff = event.position - _start_position
		var dist = diff.length()
		if dist > deadzone:
			_current_direction = diff.normalized() * minf(dist, max_distance) / max_distance
		else:
			_current_direction = Vector2.ZERO
		if knob:
			knob.position = diff.limit_length(max_distance)
		TouchManager.set_joystick_direction(_current_direction)
