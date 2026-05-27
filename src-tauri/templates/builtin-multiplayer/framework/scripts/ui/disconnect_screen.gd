extends CanvasLayer

@onready var reconnect_button: Button = $Panel/VBoxContainer/ReconnectButton
@onready var quit_button: Button = $Panel/VBoxContainer/QuitButton

func _ready() -> void:
	visible = false
	NetworkManager.connection_failed.connect(func(): visible = true)
	NetworkManager.client_disconnected.connect(_on_disconnected)
	if reconnect_button:
		reconnect_button.pressed.connect(_on_reconnect)
	if quit_button:
		quit_button.pressed.connect(GameManager.quit_game)

func _on_disconnected(peer_id: int) -> void:
	if peer_id == 1:
		visible = true

func _on_reconnect() -> void:
	visible = false
