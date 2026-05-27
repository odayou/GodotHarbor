extends Node

signal game_paused
signal game_resumed
signal score_changed(new_score: int)
signal lives_changed(new_lives: int)
signal game_over

var score: int = 0
var lives: int = 3
var is_paused: bool = false
var _checkpoint_position: Vector2 = Vector2.ZERO

func _ready() -> void:
	process_mode = Node.PROCESS_MODE_ALWAYS

func add_score(points: int) -> void:
	score += points
	score_changed.emit(score)

func lose_life() -> void:
	lives -= 1
	lives_changed.emit(lives)
	if lives <= 0:
		game_over.emit()

func set_checkpoint(pos: Vector2) -> void:
	_checkpoint_position = pos

func respawn_player(player: CharacterBody2D) -> void:
	player.global_position = _checkpoint_position
	player.velocity = Vector2.ZERO

func pause_game() -> void:
	is_paused = true
	get_tree().paused = true
	game_paused.emit()

func resume_game() -> void:
	is_paused = false
	get_tree().paused = false
	game_resumed.emit()

func toggle_pause() -> void:
	if is_paused:
		resume_game()
	else:
		pause_game()

func restart_game() -> void:
	score = 0
	lives = 3
	_checkpoint_position = Vector2.ZERO
	get_tree().paused = false
	get_tree().reload_current_scene()

func quit_game() -> void:
	get_tree().quit()
