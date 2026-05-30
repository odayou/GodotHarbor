extends CharacterBody2D

enum State { IDLE, WALKING, ATTACKING, INTERACTING }

signal health_changed(new_health: int)
signal died

@export var speed: float = 200.0
@export var max_health: int = 100
@export var interact_range: float = 64.0
@export var attack_duration: float = 0.3

var state: State = State.IDLE
var health: int = max_health
var _attack_timer: float = 0.0
var _interact_timer: float = 0.0
var _facing_direction: Vector2 = Vector2.DOWN


func _ready() -> void:
    add_to_group("player")


func _physics_process(delta: float) -> void:
    match state:
        State.IDLE:
            _handle_idle()
        State.WALKING:
            _handle_walking()
        State.ATTACKING:
            _handle_attacking(delta)
        State.INTERACTING:
            _handle_interacting(delta)


func take_damage(amount: int) -> void:
    health -= amount
    health_changed.emit(health)
    if health <= 0:
        died.emit()
        GameManager.player_died.emit()


func heal(amount: int) -> void:
    health = mini(health + amount, max_health)
    health_changed.emit(health)


func _handle_idle() -> void:
    velocity = Vector2.ZERO
    if Input.is_action_just_pressed("attack"):
        _start_attack()
        return
    if Input.is_action_just_pressed("interact"):
        _start_interact()
        return
    var input_dir = Input.get_vector("move_left", "move_right", "move_up", "move_down")
    if input_dir != Vector2.ZERO:
        state = State.WALKING


func _handle_walking() -> void:
    var input_dir = Input.get_vector("move_left", "move_right", "move_up", "move_down")
    velocity = input_dir * speed
    if input_dir != Vector2.ZERO:
        _facing_direction = input_dir.normalized()
    if Input.is_action_just_pressed("attack"):
        _start_attack()
        return
    if Input.is_action_just_pressed("interact"):
        _start_interact()
        return
    if input_dir == Vector2.ZERO:
        state = State.IDLE
    move_and_slide()


func _handle_attacking(delta: float) -> void:
    velocity = Vector2.ZERO
    _attack_timer -= delta
    if _attack_timer <= 0.0:
        state = State.IDLE


func _handle_interacting(delta: float) -> void:
    velocity = Vector2.ZERO
    _interact_timer -= delta
    if _interact_timer <= 0.0:
        state = State.IDLE


func _start_attack() -> void:
    state = State.ATTACKING
    _attack_timer = attack_duration


func _start_interact() -> void:
    state = State.INTERACTING
    _interact_timer = 0.2
    var interactables = get_tree().get_nodes_in_group("interactable")
    var closest = _find_closest_interactable(interactables)
    if closest and closest.has_method("interact"):
        closest.interact(self)


func _find_closest_interactable(interactables: Array) -> Node:
    var closest = null
    var closest_dist = interact_range
    for obj in interactables:
        if not obj is Node2D:
            continue
        var dist = global_position.distance_to(obj.global_position)
        if dist < closest_dist:
            closest = obj
            closest_dist = dist
    return closest
