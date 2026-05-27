extends Area2D
class_name NPCBase

@export var npc_name: String = "NPC"
@export var dialogue_id: String = ""
@export var interact_range: float = 64.0

var _is_player_nearby: bool = false

func _ready() -> void:
	add_to_group("interactable")
	body_entered.connect(_on_body_entered)
	body_exited.connect(_on_body_exited)

func interact(_actor: Node2D) -> void:
	if dialogue_id != "" and not DialogueManager.is_active():
		DialogueManager.start_dialogue(dialogue_id)

func _on_body_entered(body: Node2D) -> void:
	if body.is_in_group("player"):
		_is_player_nearby = true

func _on_body_exited(body: Node2D) -> void:
	if body.is_in_group("player"):
		_is_player_nearby = false
