extends Node

signal player_connected(peer_id: int)
signal player_disconnected(peer_id: int)
signal server_disconnected

var players: Dictionary = {}
var is_host: bool = false


func _ready() -> void:
    multiplayer.peer_connected.connect(_on_peer_connected)
    multiplayer.peer_disconnected.connect(_on_peer_disconnected)
    multiplayer.connected_to_server.connect(_on_connected_to_server)
    multiplayer.connection_failed.connect(_on_connection_failed)
    multiplayer.server_disconnected.connect(_on_server_disconnected)


func host_game(port: int) -> bool:
    var peer = ENetMultiplayerPeer.new()
    var error = peer.create_server(port)
    if error != OK:
        return false
    multiplayer.multiplayer_peer = peer
    is_host = true
    players[1] = {"id": 1, "name": "Host"}
    player_connected.emit(1)
    return true


func join_game(address: String, port: int) -> bool:
    var peer = ENetMultiplayerPeer.new()
    var error = peer.create_client(address, port)
    if error != OK:
        return false
    multiplayer.multiplayer_peer = peer
    is_host = false
    return true


func leave_game() -> void:
    multiplayer.multiplayer_peer = null
    players.clear()
    is_host = false


func kick_player(peer_id: int) -> void:
    if not is_host:
        return
    if multiplayer.multiplayer_peer is ENetMultiplayerPeer:
        multiplayer.multiplayer_peer.disconnect_peer(peer_id)


func get_player_list() -> Array:
    return players.values()


func _on_peer_connected(peer_id: int) -> void:
    if multiplayer.is_server():
        players[peer_id] = {"id": peer_id, "name": "Player_%d" % peer_id}
        _sync_player_list.rpc(players)
    player_connected.emit(peer_id)


func _on_peer_disconnected(peer_id: int) -> void:
    if multiplayer.is_server():
        players.erase(peer_id)
        _sync_player_list.rpc(players)
    player_disconnected.emit(peer_id)


func _on_connected_to_server() -> void:
    _request_player_list.rpc_id(1)


func _on_connection_failed() -> void:
    leave_game()


func _on_server_disconnected() -> void:
    players.clear()
    server_disconnected.emit()
    leave_game()


@rpc("authority", "call_local", "reliable")
func _sync_player_list(new_players: Dictionary) -> void:
    players = new_players


@rpc("any_peer", "reliable")
func _request_player_list() -> void:
    if multiplayer.is_server():
        var sender_id = multiplayer.get_remote_sender_id()
        _sync_player_list.rpc_id(sender_id, players)
