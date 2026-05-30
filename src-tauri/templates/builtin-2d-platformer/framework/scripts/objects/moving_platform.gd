extends AnimatableBody2D

@export var move_distance: float = 200.0
@export var move_speed: float = 100.0
@export var move_direction: Vector2 = Vector2.RIGHT
@export var wait_time: float = 0.5
@export var auto_start: bool = true

var _start_pos: Vector2
var _is_moving_forward: bool = true
var _is_waiting: bool = false


func _ready() -> void:
    _start_pos = global_position
    if auto_start:
        _start_moving()


func _physics_process(delta: float) -> void:
    if _is_waiting:
        return
    var target = _start_pos + move_direction * move_distance if _is_moving_forward else _start_pos
    var distance = global_position.distance_to(target)
    if distance < 2.0:
        global_position = target
        _is_waiting = true
        await get_tree().create_timer(wait_time).timeout
        _is_waiting = false
        _is_moving_forward = not _is_moving_forward
    else:
        var direction = global_position.direction_to(target)
        var step = minf(move_speed * delta, distance)
        global_position += direction * step
        position = position  # trigger sync


func _start_moving() -> void:
    _is_moving_forward = true
    _is_waiting = false
