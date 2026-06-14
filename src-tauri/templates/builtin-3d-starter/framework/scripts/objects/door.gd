extends Interactable3D

@export var is_locked: bool = false
@export var required_item: String = ""
@export var open_rotation: float = -90.0

var _is_open: bool = false
var _tween: Tween = null


func interact(player: Node) -> void:
	if _is_open:
		_close()
		return
	if is_locked and required_item != "":
		if not player.has_method("has_item") or not player.has_item(required_item):
			AudioManager.play_sfx(preload("res://assets/audio/sfx/locked.wav"))
			return
		is_locked = false
	_open()


func _open() -> void:
	_is_open = true
	if _tween:
		_tween.kill()
	_tween = create_tween()
	_tween.tween_property(self, "rotation_degrees:y", open_rotation, 0.3).set_ease(Tween.EASE_OUT)
	AudioManager.play_sfx(preload("res://assets/audio/sfx/door_open.wav"))


func _close() -> void:
	_is_open = false
	if _tween:
		_tween.kill()
	_tween = create_tween()
	_tween.tween_property(self, "rotation_degrees:y", 0.0, 0.3).set_ease(Tween.EASE_OUT)
	AudioManager.play_sfx(preload("res://assets/audio/sfx/door_close.wav"))
