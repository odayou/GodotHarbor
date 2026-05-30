extends Camera2D

@export var shake_decay: float = 5.0

var _shake_strength: float = 0.0


func apply_shake(strength: float) -> void:
    _shake_strength = strength


func _process(delta: float) -> void:
    if _shake_strength > 0.0:
        offset = Vector2(randf_range(-_shake_strength, _shake_strength), randf_range(-_shake_strength, _shake_strength))
        _shake_strength = lerpf(_shake_strength, 0.0, shake_decay * delta)
    else:
        offset = Vector2.ZERO
