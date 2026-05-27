extends CharacterBody2D

@export var speed: float = 300.0
@export var jump_force: float = -400.0
@export var gravity: float = 980.0

@onready var state_machine: StateMachine = $StateMachine

func _ready() -> void:
	pass

func _physics_process(delta: float) -> void:
	if not is_on_floor():
		velocity.y += gravity * delta
