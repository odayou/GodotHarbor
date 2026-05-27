extends CanvasLayer

@onready var joystick: Control = $VBoxContainer/JoystickArea
@onready var action_button_jump: Control = $VBoxContainer/HBoxContainer/JumpButton
@onready var action_button_attack: Control = $VBoxContainer/HBoxContainer/AttackButton
@onready var pause_button: Button = $PauseButton

func _ready() -> void:
	add_to_group("touch_overlay")
	if not TouchManager.is_touch_device:
		visible = false
	if pause_button:
		pause_button.pressed.connect(GameManager.toggle_pause)
