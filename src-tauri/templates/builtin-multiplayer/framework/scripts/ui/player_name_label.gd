extends Label

@export var offset: Vector2 = Vector2(0, -40)


func _ready() -> void:
    horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
    text = "Player"
    var parent = get_parent()
    if parent and parent.is_in_group("player"):
        var peer_id = parent.name.to_int()
        if NetworkManager and NetworkManager.players.has(peer_id):
            text = NetworkManager.players[peer_id].get("name", "Player_%d" % peer_id)


func _process(_delta: float) -> void:
    var parent = get_parent()
    if parent is Node2D:
        global_position = parent.global_position + offset
