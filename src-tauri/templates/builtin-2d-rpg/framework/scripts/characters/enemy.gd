extends CharacterBody2D

enum AIState { IDLE, PATROL, CHASE, ATTACK }

@export var speed: float = 80.0
@export var chase_speed: float = 120.0
@export var max_health: int = 30
@export var attack_damage: int = 10
@export var attack_range: float = 50.0
@export var detect_range: float = 150.0
@export var attack_cooldown: float = 1.5
@export var patrol_points: Array[Vector2] = []
@export var idle_time: float = 1.0

var health: int = max_health
var ai_state: AIState = AIState.IDLE
var _attack_timer: float = 0.0
var _patrol_index: int = 0
var _idle_timer: float = 0.0
var _player_ref: CharacterBody2D = null


func _ready() -> void:
    add_to_group("enemy")


func _physics_process(delta: float) -> void:
    _attack_timer -= delta
    match ai_state:
        AIState.IDLE:
            _handle_idle(delta)
        AIState.PATROL:
            _handle_patrol()
        AIState.CHASE:
            _handle_chase()
        AIState.ATTACK:
            _handle_attack()
    move_and_slide()


func take_damage(amount: int) -> void:
    health -= amount
    if health <= 0:
        _on_death()


func _handle_idle(delta: float) -> void:
    velocity.x = 0.0
    _idle_timer -= delta
    if _idle_timer <= 0.0:
        if _player_ref and is_instance_valid(_player_ref):
            ai_state = AIState.CHASE
        elif patrol_points.size() > 0:
            ai_state = AIState.PATROL


func _handle_patrol() -> void:
    if patrol_points.size() == 0:
        ai_state = AIState.IDLE
        return
    var target = patrol_points[_patrol_index]
    var dir = signf(target.x - global_position.x)
    velocity.x = speed * dir
    if absf(global_position.x - target.x) < 5.0:
        _patrol_index = (_patrol_index + 1) % patrol_points.size()
        ai_state = AIState.IDLE
        _idle_timer = idle_time
    _check_player_detection()


func _handle_chase() -> void:
    if not _player_ref or not is_instance_valid(_player_ref):
        ai_state = AIState.IDLE
        _idle_timer = idle_time
        return
    var dist = global_position.distance_to(_player_ref.global_position)
    if dist > detect_range * 2.0:
        ai_state = AIState.IDLE
        _idle_timer = idle_time
        return
    if dist < attack_range:
        ai_state = AIState.ATTACK
        return
    var dir = signf(_player_ref.global_position.x - global_position.x)
    velocity.x = chase_speed * dir


func _handle_attack() -> void:
    velocity.x = 0.0
    if _attack_timer <= 0.0 and _player_ref and is_instance_valid(_player_ref):
        if _player_ref.has_method("take_damage"):
            _player_ref.take_damage(attack_damage)
        _attack_timer = attack_cooldown
    if not _player_ref or not is_instance_valid(_player_ref):
        ai_state = AIState.IDLE
        _idle_timer = idle_time
        return
    var dist = global_position.distance_to(_player_ref.global_position)
    if dist > attack_range:
        ai_state = AIState.CHASE


func _check_player_detection() -> void:
    var player = get_tree().get_first_node_in_group("player")
    if player and global_position.distance_to(player.global_position) < detect_range:
        _player_ref = player
        ai_state = AIState.CHASE


func _on_death() -> void:
    queue_free()
