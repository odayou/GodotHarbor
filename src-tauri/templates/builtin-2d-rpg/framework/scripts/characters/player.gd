extends CharacterBody2D

@export var speed: float = 200.0
@export var max_health: int = 100
var health: int = max_health

@onready var state_machine: StateMachine = $StateMachine

func _ready() -> void:
	add_to_group("player")

func take_damage(amount: int) -> void:
	health -= amount
	if health <= 0:
		GameManager.player_died.emit()

func heal(amount: int) -> void:
	health = mini(health + amount, max_health)
