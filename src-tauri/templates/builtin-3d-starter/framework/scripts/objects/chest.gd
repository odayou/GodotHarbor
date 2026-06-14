extends Interactable3D

@export var score_reward: int = 50
@export var heal_reward: int = 0
@export var item_id: String = ""

var _is_opened: bool = false
var _tween: Tween = null


func interact(player: Node) -> void:
	if _is_opened:
		return
	_is_opened = true
	if score_reward > 0:
		GameManager.add_score(score_reward)
	if heal_reward > 0 and player.has_method("heal"):
		player.heal(heal_reward)
	if item_id != "" and player.has_method("add_item"):
		player.add_item(item_id)
	AudioManager.play_sfx(preload("res://assets/audio/sfx/chest_open.wav"))
	if _tween:
		_tween.kill()
	_tween = create_tween()
	_tween.tween_property(self, "rotation_degrees:x", -30.0, 0.2).set_ease(Tween.EASE_OUT)
