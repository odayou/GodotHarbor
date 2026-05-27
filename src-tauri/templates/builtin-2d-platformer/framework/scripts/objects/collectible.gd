extends Area2D

@export var points: int = 10
@export var collect_sfx: AudioStream

func _on_body_entered(body: Node2D) -> void:
	if body is CharacterBody2D:
		GameManager.add_score(points)
		if collect_sfx:
			AudioManager.play_sfx(collect_sfx)
		queue_free()
