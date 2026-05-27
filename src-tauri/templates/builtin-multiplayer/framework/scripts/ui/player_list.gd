extends CanvasLayer

@onready var player_list_container: VBoxContainer = $Panel/VBoxContainer

func _ready() -> void:
	NetworkManager.client_connected.connect(_on_client_connected)
	NetworkManager.client_disconnected.connect(_on_client_disconnected)
	_refresh_list()

func _on_client_connected(_peer_id: int) -> void:
	_refresh_list()

func _on_client_disconnected(_peer_id: int) -> void:
	_refresh_list()

func _refresh_list() -> void:
	if not player_list_container:
		return
	for child in player_list_container.get_children():
		child.queue_free()
	for peer_id in NetworkManager.connected_players:
		var label = Label.new()
		label.text = NetworkManager.connected_players[peer_id].name
		player_list_container.add_child(label)
