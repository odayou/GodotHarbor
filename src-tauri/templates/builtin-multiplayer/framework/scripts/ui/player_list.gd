extends CanvasLayer

@onready var player_list_container: VBoxContainer = $Panel/VBoxContainer

func _ready() -> void:
	NetworkManager.player_connected.connect(_on_player_changed)
	NetworkManager.player_disconnected.connect(_on_player_changed)
	NetworkManager.server_disconnected.connect(_refresh_list)
	_refresh_list()

func _on_player_changed(_peer_id: int) -> void:
	_refresh_list()

func _refresh_list() -> void:
	if not player_list_container:
		return
	for child in player_list_container.get_children():
		child.queue_free()
	for player in NetworkManager.get_player_list():
		var label = Label.new()
		label.text = player.name
		player_list_container.add_child(label)
