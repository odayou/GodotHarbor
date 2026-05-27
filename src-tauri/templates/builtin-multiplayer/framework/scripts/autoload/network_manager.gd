extends Node

signal server_started(port: int)
signal client_connected(peer_id: int)
signal client_disconnected(peer_id: int)
signal connection_failed

const DEFAULT_PORT: int = 7777
const MAX_PLAYERS: int = 16

var peer: ENetMultiplayerPeer
var is_server: bool = false
var connected_players: Dictionary = {}

func create_server(port: int = DEFAULT_PORT) -> bool:
	peer = ENetMultiplayerPeer.new()
	var err = peer.create_server(port, MAX_PLAYERS)
	if err != OK:
		push_error("Failed to create server: %s" % error_string(err))
		return false
	multiplayer.multiplayer_peer = peer
	is_server = true
	server_started.emit(port)
	return true

func join_server(address: String, port: int = DEFAULT_PORT) -> bool:
	peer = ENetMultiplayerPeer.new()
	var err = peer.create_client(address, port)
	if err != OK:
		push_error("Failed to join server: %s" % error_string(err))
		return false
	multiplayer.multiplayer_peer = peer
	multiplayer.connection_failed.connect(func(): connection_failed.emit())
	return true

func disconnect_peer() -> void:
	if peer:
		peer.close()
		multiplayer.multiplayer_peer = OfflineMultiplayerPeer.new()
	is_server = false
	connected_players.clear()

func _ready() -> void:
	multiplayer.peer_connected.connect(_on_peer_connected)
	multiplayer.peer_disconnected.connect(_on_peer_disconnected)

func _on_peer_connected(peer_id: int) -> void:
	connected_players[peer_id] = {"id": peer_id, "name": "Player %d" % peer_id}
	client_connected.emit(peer_id)

func _on_peer_disconnected(peer_id: int) -> void:
	connected_players.erase(peer_id)
	client_disconnected.emit(peer_id)
