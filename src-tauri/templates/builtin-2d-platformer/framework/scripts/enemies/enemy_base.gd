extends CharacterBody2D

enum AIState { PATROL, CHASE, RETURN }

@export var speed: float = 100.0
@export var chase_speed: float = 160.0
@export var direction: int = 1
@export var gravity: float = 980.0
@export var detect_range: float = 80.0
@export var attack_range: float = 40.0
@export var attack_damage: int = 1
@export var attack_cooldown: float = 1.0
@export var lose_sight_range: float = 150.0

var ai_state: AIState = AIState.PATROL
var _attack_timer: float = 0.0
var _patrol_origin: Vector2
var _player_ref: CharacterBody2D = null

@onready var raycast_right: RayCast2D = $RayCastRight
@onready var raycast_left: RayCast2D = $RayCastLeft
@onready var edge_ray_right: RayCast2D = $EdgeRayRight
@onready var edge_ray_left: RayCast2D = $EdgeRayLeft
@onready var detect_area: Area2D = $DetectArea


func _ready() -> void:
    add_to_group("enemy")
    _patrol_origin = global_position
    if detect_area:
        detect_area.body_entered.connect(_on_detect_body_entered)
        detect_area.body_exited.connect(_on_detect_body_exited)


func _physics_process(delta: float) -> void:
    if not is_on_floor():
        velocity.y += gravity * delta
    _attack_timer -= delta
    match ai_state:
        AIState.PATROL:
            _handle_patrol()
        AIState.CHASE:
            _handle_chase()
        AIState.RETURN:
            _handle_return()
    move_and_slide()


func _handle_patrol() -> void:
    if _is_at_edge():
        direction = -direction
    if raycast_right and raycast_right.is_colliding():
        direction = -1
    if raycast_left and raycast_left.is_colliding():
        direction = 1
    velocity.x = speed * direction


func _handle_chase() -> void:
    if not _player_ref or not is_instance_valid(_player_ref):
        ai_state = AIState.RETURN
        return
    var dist = global_position.distance_to(_player_ref.global_position)
    if dist > lose_sight_range:
        ai_state = AIState.RETURN
        return
    var dir_to_player = signf(_player_ref.global_position.x - global_position.x)
    velocity.x = chase_speed * dir_to_player
    if dist < attack_range and _attack_timer <= 0.0:
        _attack_player()


func _handle_return() -> void:
    var dir_to_origin = signf(_patrol_origin.x - global_position.x)
    velocity.x = speed * dir_to_origin
    if absf(global_position.x - _patrol_origin.x) < 5.0:
        ai_state = AIState.PATROL


func _is_at_edge() -> bool:
    if direction > 0 and edge_ray_right and not edge_ray_right.is_colliding():
        return true
    if direction < 0 and edge_ray_left and not edge_ray_left.is_colliding():
        return true
    return false


func _attack_player() -> void:
    _attack_timer = attack_cooldown
    if _player_ref and _player_ref.has_method("take_damage"):
        _player_ref.take_damage(attack_damage)


func _on_detect_body_entered(body: Node2D) -> void:
    if body.is_in_group("player"):
        _player_ref = body
        ai_state = AIState.CHASE


func _on_detect_body_exited(body: Node2D) -> void:
    if body.is_in_group("player") and ai_state == AIState.CHASE:
        ai_state = AIState.RETURN
