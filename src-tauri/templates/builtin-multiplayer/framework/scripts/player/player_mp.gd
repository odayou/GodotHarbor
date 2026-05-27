extends CharacterBody2D

@export var speed: float = 300.0

@onready var sprite: Sprite2D = $Sprite2D

var _sync_position: Vector2 = Vector2.ZERO
var _sync_velocity: Vector2 = Vector2.ZERO

func _ready() -> void:
	if is_multiplayer_authority():
		_sync_position = global_position

func _physics_process(delta: float) -> void:
	if is_multiplayer_authority():
		var input_dir = Input.get_vector("move_left", "move_right", "move_up", "move_down")
		velocity = input_dir * speed
		move_and_slide()
		_sync_position = global_position
		_sync_velocity = velocity
	else:
		global_position = global_position.lerp(_sync_position, 0.2)

@rpc("authority", "unreliable_ordered")
func sync_transform(pos: Vector2, vel: Vector2) -> void:
	_sync_position = pos
	_sync_velocity = vel
