extends CharacterBody2D

@export var speed: float = 300.0
@export var jump_force: float = -400.0
@export var gravity: float = 980.0

var _sync_position: Vector2 = Vector2.ZERO


func _ready() -> void:
    set_multiplayer_authority(name.to_int())


func _physics_process(delta: float) -> void:
    if is_multiplayer_authority():
        if not is_on_floor():
            velocity.y += gravity * delta
        if Input.is_action_just_pressed("jump") and is_on_floor():
            velocity.y = jump_force
        var input_dir = Input.get_axis("move_left", "move_right")
        velocity.x = input_dir * speed
        move_and_slide()
        _sync_position = global_position
    else:
        global_position = global_position.lerp(_sync_position, 0.1)


func _process(_delta: float) -> void:
    if is_multiplayer_authority():
        rpc("_update_position", global_position)


@rpc("unreliable")
func _update_position(pos: Vector2) -> void:
    _sync_position = pos
