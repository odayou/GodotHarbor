extends ParallaxBackground

@export var auto_scroll_speed: Vector2 = Vector2.ZERO
@export var follow_camera: bool = true

var _camera: Camera2D


func _ready() -> void:
    if follow_camera:
        await get_tree().process_frame
        _camera = get_viewport().get_camera_2d()


func _process(delta: float) -> void:
    if auto_scroll_speed != Vector2.ZERO:
        scroll_offset += auto_scroll_speed * delta
