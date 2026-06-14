extends CharacterBody3D

signal health_changed(new_health: int)
signal died

@export var walk_speed: float = 5.0
@export var sprint_speed: float = 8.0
@export var crouch_speed: float = 2.5
@export var jump_velocity: float = 4.5
@export var mouse_sensitivity: float = 0.002
@export var interact_range: float = 3.0
@export var attack_range: float = 2.0
@export var attack_damage: int = 25
@export var attack_cooldown: float = 0.5
@export var max_health: int = 100
@export var crouch_height: float = 1.0
@export var stand_height: float = 2.0

var gravity: float = ProjectSettings.get_setting("physics/3d/default_gravity")
var health: int = max_health
var _is_sprinting: bool = false
var _is_crouching: bool = false
var _current_speed: float = walk_speed
var _is_dead: bool = false
var _attack_timer: float = 0.0

@onready var camera_pivot: Node3D = $CameraPivot
@onready var camera: Camera3D = $CameraPivot/Camera3D
@onready var collision_shape: CollisionShape3D = $CollisionShape3D
@onready var interact_ray: RayCast3D = $CameraPivot/InteractRay


func _ready() -> void:
	Input.mouse_mode = Input.MOUSE_MODE_CAPTURED
	add_to_group("player")


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseMotion and Input.mouse_mode == Input.MOUSE_MODE_CAPTURED:
		rotate_y(-event.relative.x * mouse_sensitivity)
		camera_pivot.rotate_x(-event.relative.y * mouse_sensitivity)
		camera_pivot.rotation.x = clampf(camera_pivot.rotation.x, -PI / 2.0, PI / 2.0)
	if event.is_action_pressed("ui_cancel"):
		if Input.mouse_mode == Input.MOUSE_MODE_CAPTURED:
			Input.mouse_mode = Input.MOUSE_MODE_VISIBLE
		else:
			Input.mouse_mode = Input.MOUSE_MODE_CAPTURED
	if event.is_action_pressed("interact"):
		_try_interact()
	if event.is_action_pressed("attack"):
		_try_attack()


func _physics_process(delta: float) -> void:
	if _is_dead:
		return
	_attack_timer -= delta
	_handle_movement_state()
	if not is_on_floor():
		velocity.y -= gravity * delta
	if Input.is_action_just_pressed("jump") and is_on_floor() and not _is_crouching:
		velocity.y = jump_velocity
	var input_dir = Input.get_vector("move_left", "move_right", "move_forward", "move_backward")
	var direction = (transform.basis * Vector3(input_dir.x, 0.0, input_dir.y)).normalized()
	if direction:
		velocity.x = direction.x * _current_speed
		velocity.z = direction.z * _current_speed
	else:
		velocity.x = move_toward(velocity.x, 0.0, _current_speed)
		velocity.z = move_toward(velocity.z, 0.0, _current_speed)
	move_and_slide()


func take_damage(amount: int) -> void:
    if _is_dead:
        return
    health -= amount
    health_changed.emit(health)
    if health <= 0:
        health = 0
        _is_dead = true
        died.emit()
        Input.mouse_mode = Input.MOUSE_MODE_VISIBLE
        ScreenManager.reload_current_scene(1.0)


func heal(amount: int) -> void:
    health = mini(health + amount, max_health)
    health_changed.emit(health)


func _handle_movement_state() -> void:
    if Input.is_action_pressed("sprint") and is_on_floor() and not _is_crouching:
        _is_sprinting = true
        _current_speed = sprint_speed
    elif Input.is_action_pressed("crouch"):
        _is_crouching = true
        _is_sprinting = false
        _current_speed = crouch_speed
        if collision_shape and collision_shape.shape is CapsuleShape3D:
            collision_shape.shape.height = crouch_height
            collision_shape.position.y = crouch_height / 2.0
    else:
        _is_sprinting = false
        _is_crouching = false
        _current_speed = walk_speed
        if collision_shape and collision_shape.shape is CapsuleShape3D:
            collision_shape.shape.height = stand_height
            collision_shape.position.y = stand_height / 2.0


func _try_interact() -> void:
    if not interact_ray:
        return
    if interact_ray.is_colliding():
        var collider = interact_ray.get_collider()
        if collider and collider.has_method("interact"):
            collider.interact(self)


func _try_attack() -> void:
    if _attack_timer > 0.0:
        return
    _attack_timer = attack_cooldown
    var space_state = get_world_3d().direct_space_state
    var origin = global_position
    var forward = -global_transform.basis.z
    var end = origin + forward * attack_range
    var query = PhysicsRayQueryParameters3D.create(origin, end, 0b101)
    var result = space_state.intersect_ray(query)
    if result and result.collider and result.collider.has_method("take_damage"):
        result.collider.take_damage(attack_damage)
