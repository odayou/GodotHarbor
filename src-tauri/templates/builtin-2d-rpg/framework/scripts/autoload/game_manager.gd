extends Node

signal game_paused
signal game_resumed
signal player_died

var is_paused: bool = false

func _ready() -> void:
	process_mode = Node.PROCESS_MODE_ALWAYS

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

func quit_game() -> void:
	get_tree().quit()
