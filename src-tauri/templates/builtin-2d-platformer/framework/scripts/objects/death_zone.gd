extends Area2D

@export var damage: int = 1
@export var respawn_on_touch: bool = true


func _on_body_entered(body: Node2D) -> void:
    if body.is_in_group("player"):
        if body.has_method("take_damage"):
            body.take_damage(damage)
        if respawn_on_touch and body.has_method("global_position"):
            GameManager.lose_life()
            if GameManager.lives > 0:
                GameManager.respawn_player(body)
