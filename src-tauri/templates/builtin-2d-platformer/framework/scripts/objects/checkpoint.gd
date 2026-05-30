extends Area2D

@export var checkpoint_id: String = ""
@export var sfx_stream: AudioStream

var _is_activated: bool = false


func _on_body_entered(body: Node2D) -> void:
    if body.is_in_group("player") and not _is_activated:
        _is_activated = true
        GameManager.set_checkpoint(body.global_position)
        if sfx_stream:
            AudioManager.play_sfx(sfx_stream)
        _update_visual()


func _update_visual() -> void:
    var flag = $Flag
    if flag:
        flag.modulate = Color.GREEN
