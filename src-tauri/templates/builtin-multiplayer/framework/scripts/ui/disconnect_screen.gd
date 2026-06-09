extends CanvasLayer

@onready var reconnect_button: Button = $Panel/VBoxContainer/ReconnectButton
@onready var quit_button: Button = $Panel/VBoxContainer/QuitButton

func _ready() -> void:
	visible = false
	NetworkManager.server_disconnected.connect(func(): visible = true)
	if reconnect_button:
		reconnect_button.pressed.connect(_on_reconnect)
	if quit_button:
		quit_button.pressed.connect(func(): get_tree().quit())

func _on_reconnect() -> void:
	visible = false
