extends Node2D

@export var player_scene: PackedScene
@export var spawn_position: Vector2 = Vector2(100, 300)


func _ready() -> void:
    if multiplayer.is_server():
        multiplayer.peer_connected.connect(_on_peer_connected)
        multiplayer.peer_disconnected.connect(_on_peer_disconnected)
    if player_scene:
        _spawn_player(multiplayer.get_unique_id())


func _on_peer_connected(peer_id: int) -> void:
    _spawn_player(peer_id)


func _on_peer_disconnected(peer_id: int) -> void:
    var player = get_node_or_null(str(peer_id))
    if player:
        player.queue_free()


func _spawn_player(peer_id: int) -> void:
    if not player_scene:
        return
    var player = player_scene.instantiate()
    player.name = str(peer_id)
    player.global_position = spawn_position
    add_child(player, true)
