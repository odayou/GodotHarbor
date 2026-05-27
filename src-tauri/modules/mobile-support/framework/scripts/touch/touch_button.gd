extends Control

@export var action_name: String = "jump"
@export var press_scale: float = 0.9

var _is_pressed: bool = false

@onready var button_label: Label = $Label

func _ready() -> void:
	if not TouchManager.is_touch_device:
		visible = false
	if button_label:
		button_label.text = action_name

func _gui_input(event: InputEvent) -> void:
	if event is InputEventScreenTouch:
		if event.pressed:
			_is_pressed = true
			scale = Vector2(press_scale, press_scale)
			Input.action_press(action_name)
			TouchManager.touch_action_pressed.emit(action_name)
		else:
			_is_pressed = false
			scale = Vector2.ONE
			Input.action_release(action_name)
			TouchManager.touch_action_released.emit(action_name)
