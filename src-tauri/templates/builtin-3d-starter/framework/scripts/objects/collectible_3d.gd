extends Area3D

@export var item_name: String = "Collectible"
@export var score_value: int = 10
@export var heal_amount: int = 0
@export var sfx_stream: AudioStream


func _on_body_entered(body: Node3D) -> void:
	if body.is_in_group("player"):
		if heal_amount > 0 and body.has_method("heal"):
			body.heal(heal_amount)
		if score_value > 0:
			GameManager.add_score(score_value)
		if sfx_stream:
			AudioManager.play_sfx(sfx_stream)
		queue_free()
