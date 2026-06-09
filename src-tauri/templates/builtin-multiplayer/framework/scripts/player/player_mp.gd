extends CharacterBody2D

@export var speed: float = 300.0

var _sync_position: Vector2 = Vector2.ZERO

func _ready() -> void:
	add_to_group("player")
	if is_multiplayer_authority():
		_sync_position = global_position

func _physics_process(delta: float) -> void:
	if is_multiplayer_authority():
		var input_dir = Input.get_vector("move_left", "move_right", "move_up", "move_down")
		velocity = input_dir * speed
		move_and_slide()
		_sync_position = global_position
		sync_position.rpc(global_position)
	else:
		global_position = global_position.lerp(_sync_position, 0.2)

@rpc("authority", "unreliable_ordered")
func sync_position(pos: Vector2) -> void:
	_sync_position = pos
