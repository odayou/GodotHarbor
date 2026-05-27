extends NodePath

var patrol_points: PackedVector2Array = []
var speed: float = 100.0
var direction: int = 1

@onready var raycast_right: RayCast2D = $RayCastRight
@onready var raycast_left: RayCast2D = $RayCastLeft
@onready var sprite: AnimatedSprite2D = $AnimatedSprite2D

func _physics_process(delta: float) -> void:
	if not is_on_floor():
		velocity.y += 980.0 * delta
	velocity.x = speed * direction
	if raycast_right and raycast_right.is_colliding():
		direction = -1
	if raycast_left and raycast_left.is_colliding():
		direction = 1
	move_and_slide()
