extends Node

signal game_paused
signal game_resumed
signal player_died
signal score_changed(new_score: int)

var score: int = 0
var is_paused: bool = false
var _spawn_point: Vector2 = Vector2.ZERO


func _ready() -> void:
    process_mode = Node.PROCESS_MODE_ALWAYS


func add_score(points: int) -> void:
    score += points
    score_changed.emit(score)


func set_spawn_point(pos: Vector2) -> void:
    _spawn_point = pos


func respawn_player(player: CharacterBody2D) -> void:
    if _spawn_point != Vector2.ZERO:
        player.global_position = _spawn_point
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
    _spawn_point = Vector2.ZERO
    get_tree().paused = false
    get_tree().reload_current_scene()


func quit_game() -> void:
    get_tree().quit()
