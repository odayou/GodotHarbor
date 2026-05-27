extends Area2D

@export var checkpoint_sfx: AudioStream

func _on_body_entered(body: Node2D) -> void:
	if body is CharacterBody2D:
		GameManager.set_checkpoint(global_position)
		if checkpoint_sfx:
			AudioManager.play_sfx(checkpoint_sfx)
		monitoring = false
