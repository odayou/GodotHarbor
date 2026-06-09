extends CharacterBody2D

enum State { IDLE, RUNNING, JUMPING, FALLING, WALL_SLIDING, DASHING }

signal health_changed(new_health: int)
signal died

@export var speed: float = 300.0
@export var jump_force: float = -400.0
@export var gravity: float = 980.0
@export var coyote_time: float = 0.1
@export var jump_buffer_time: float = 0.1
@export var wall_slide_speed: float = 80.0
@export var wall_jump_force_x: float = 300.0
@export var wall_jump_force_y: float = -350.0
@export var wall_jump_control_time: float = 0.2
@export var dash_speed: float = 600.0
@export var dash_duration: float = 0.15
@export var dash_cooldown: float = 0.5
@export var max_health: int = 3
@export var invincible_time: float = 1.0

var state = State.IDLE
var health: int = max_health
var _coyote_timer: float = 0.0
var _jump_buffer_timer: float = 0.0
var _was_on_floor: bool = false
var _wall_dir: float = 0.0
var _wall_jump_control_timer: float = 0.0
var _dash_timer: float = 0.0
var _dash_cooldown_timer: float = 0.0
var _dash_dir: float = 1.0
var _invincible_timer: float = 0.0

@onready var sprite: AnimatedSprite2D = $AnimatedSprite2D
@onready var wall_ray_right: RayCast2D = $WallRayRight
@onready var wall_ray_left: RayCast2D = $WallRayLeft


func _ready() -> void:
	add_to_group("player")


func _physics_process(delta: float) -> void:
	_invincible_timer -= delta
	var was_on_floor = is_on_floor()
	if not is_on_floor():
		velocity.y += gravity * delta

	var input_dir = Input.get_axis("move_left", "move_right")
	_update_wall_direction()

	if not was_on_floor and not _was_on_floor:
		_coyote_timer -= delta
	else:
		_coyote_timer = coyote_time

	if Input.is_action_just_pressed("jump"):
		_jump_buffer_timer = jump_buffer_time
	else:
		_jump_buffer_timer -= delta

	if Input.is_action_just_pressed("dash") and _dash_cooldown_timer <= 0.0 and state != State.DASHING:
		_start_dash(input_dir if input_dir != 0.0 else _dash_dir)
	_dash_cooldown_timer -= delta

	match state:
		State.IDLE:
			velocity.x = 0.0
			if _try_jump():
				pass
			elif input_dir != 0.0:
				state = State.RUNNING
			elif not is_on_floor() and _coyote_timer <= 0.0:
				state = State.FALLING if _wall_dir == 0.0 else State.WALL_SLIDING

		State.RUNNING:
			velocity.x = input_dir * speed
			_flip_sprite(input_dir)
			if _try_jump():
				pass
			elif input_dir == 0.0 and is_on_floor():
				state = State.IDLE
			elif not is_on_floor() and _coyote_timer <= 0.0:
				state = State.FALLING if _wall_dir == 0.0 else State.WALL_SLIDING

		State.JUMPING:
			_apply_air_movement(delta, input_dir)
			if velocity.y > 0.0:
				state = State.FALLING
			if is_on_floor():
				state = State.IDLE if input_dir == 0.0 else State.RUNNING

		State.FALLING:
			_apply_air_movement(delta, input_dir)
			if is_on_floor():
				state = State.IDLE if input_dir == 0.0 else State.RUNNING
				_on_land()
			elif _wall_dir != 0.0 and input_dir == _wall_dir:
				state = State.WALL_SLIDING

		State.WALL_SLIDING:
			velocity.y = minf(velocity.y, wall_slide_speed)
			velocity.x = 0.0
			if _try_wall_jump():
				pass
			elif is_on_floor():
				state = State.IDLE
			elif _wall_dir == 0.0:
				state = State.FALLING
			if Input.is_action_just_released("move_left") or Input.is_action_just_released("move_right"):
				if _wall_dir == 0.0:
					state = State.FALLING

		State.DASHING:
			velocity.x = _dash_dir * dash_speed
			velocity.y = 0.0
			_dash_timer -= delta
			if _dash_timer <= 0.0:
				state = State.IDLE if is_on_floor() else State.FALLING
				velocity.x = 0.0

	_was_on_floor = is_on_floor()
	move_and_slide()
	_check_enemy_collision()


func take_damage(amount: int) -> void:
	if _invincible_timer > 0.0:
		return
	health -= amount
	health_changed.emit(health)
	_invincible_timer = invincible_time
	velocity.y = -200.0
	if health <= 0:
		died.emit()
		GameManager.lose_life()
		if GameManager.lives > 0:
			GameManager.respawn_player(self)
		else:
			GameManager.game_over.emit()


func _try_jump() -> bool:
	if _jump_buffer_timer > 0.0 and _coyote_timer > 0.0:
		velocity.y = jump_force
		_coyote_timer = 0.0
		_jump_buffer_timer = 0.0
		state = State.JUMPING
		return true
	return false


func _try_wall_jump() -> bool:
	if Input.is_action_just_pressed("jump") and _wall_dir != 0.0:
		velocity.x = -_wall_dir * wall_jump_force_x
		velocity.y = wall_jump_force_y
		_wall_jump_control_timer = wall_jump_control_time
		_flip_sprite(-_wall_dir)
		state = State.JUMPING
		return true
	return false


func _apply_air_movement(delta: float, input_dir: float) -> void:
	_wall_jump_control_timer -= delta
	if _wall_jump_control_timer > 0.0:
		return
	velocity.x = input_dir * speed
	_flip_sprite(input_dir)


func _start_dash(dir: float) -> void:
	_dash_dir = dir
	_dash_timer = dash_duration
	_dash_cooldown_timer = dash_cooldown
	state = State.DASHING
	velocity.y = 0.0


func _update_wall_direction() -> void:
	_wall_dir = 0.0
	if is_on_floor():
		return
	if wall_ray_right and wall_ray_right.is_colliding():
		_wall_dir = 1.0
	elif wall_ray_left and wall_ray_left.is_colliding():
		_wall_dir = -1.0


func _flip_sprite(dir: float) -> void:
	if sprite and dir != 0.0:
		sprite.flip_h = dir < 0.0


func _on_land() -> void:
	if _was_on_floor:
		return
	var camera = get_viewport().get_camera_2d()
	if camera and camera.has_method("apply_shake"):
		camera.apply_shake(2.0)


func _check_enemy_collision() -> void:
	for i in get_slide_collision_count():
		var collision = get_slide_collision(i)
		var collider = collision.get_collider()
		if collider and collider.is_in_group("enemy"):
			if global_position.y < collider.global_position.y - 4:
				collider.queue_free()
				velocity.y = jump_force * 0.6
			else:
				take_damage(1)
