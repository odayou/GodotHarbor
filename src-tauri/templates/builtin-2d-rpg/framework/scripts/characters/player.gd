extends CharacterBody2D

enum State { IDLE, WALKING, ATTACKING, INTERACTING }

signal health_changed(new_health: int)
signal died

@export var speed: float = 200.0
@export var max_health: int = 100
@export var interact_range: float = 64.0
@export var attack_duration: float = 0.3
@export var invincible_time: float = 1.0
@export var attack_damage: int = 20

var state = State.IDLE
var health: int = max_health
var _attack_timer: float = 0.0
var _interact_timer: float = 0.0
var _facing_direction: Vector2 = Vector2.DOWN
var _invincible_timer: float = 0.0


func _ready() -> void:
	add_to_group("player")


func _physics_process(delta: float) -> void:
	_invincible_timer -= delta
	match state:
		State.IDLE:
			_handle_idle()
		State.WALKING:
			_handle_walking()
		State.ATTACKING:
			_handle_attacking(delta)
		State.INTERACTING:
			_handle_interacting(delta)
	move_and_slide()
	_check_enemy_collision()


func take_damage(amount: int) -> void:
	if _invincible_timer > 0.0:
		return
	health -= amount
	health_changed.emit(health)
	_invincible_timer = invincible_time
	velocity = -_facing_direction * 100.0
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
	var enemies = get_tree().get_nodes_in_group("enemy")
	for enemy in enemies:
		if not is_instance_valid(enemy):
			continue
		var dist = global_position.distance_to(enemy.global_position)
		var dir_to_enemy = (enemy.global_position - global_position).normalized()
		if dist < 50.0 and dir_to_enemy.dot(_facing_direction) > 0.3:
			if enemy.has_method("take_damage"):
				enemy.take_damage(attack_damage)


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


func _check_enemy_collision() -> void:
	for i in get_slide_collision_count():
		var collision = get_slide_collision(i)
		var collider = collision.get_collider()
		if collider and collider.is_in_group("enemy"):
			take_damage(collider.attack_damage if collider.get("attack_damage") else 10)
