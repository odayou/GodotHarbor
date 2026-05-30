extends Area3D

@export var damage: int = 1
@export var respawn_player: bool = true


func _on_body_entered(body: Node3D) -> void:
    if body.is_in_group("player"):
        if body.has_method("take_damage"):
            body.take_damage(damage)
        if respawn_player:
            ScreenManager.reload_current_scene(0.3)
