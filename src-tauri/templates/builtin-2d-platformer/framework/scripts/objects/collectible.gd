extends Area2D

@export var score_value: int = 10
@export var sfx_stream: AudioStream


func _on_body_entered(body: Node2D) -> void:
    if body.is_in_group("player"):
        GameManager.add_score(score_value)
        if sfx_stream:
            AudioManager.play_sfx(sfx_stream)
        queue_free()
