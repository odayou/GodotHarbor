extends Area2D
class_name NPCBase

@export var npc_name: String = "NPC"
@export var dialogic_timeline: String = ""
@export var interact_range: float = 64.0

var _is_player_nearby: bool = false


func _ready() -> void:
    add_to_group("interactable")
    body_entered.connect(_on_body_entered)
    body_exited.connect(_on_body_exited)


func interact(_actor: Node2D) -> void:
    if dialogic_timeline != "" and not Dialogic.timeline_ended.is_connected(_on_dialogue_ended):
        Dialogic.timeline_ended.connect(_on_dialogue_ended, ConnectFlags.ONE_SHOT)
        Dialogic.start(dialogic_timeline)


func _on_body_entered(body: Node2D) -> void:
    if body.is_in_group("player"):
        _is_player_nearby = true


func _on_body_exited(body: Node2D) -> void:
    if body.is_in_group("player"):
        _is_player_nearby = false


func _on_dialogue_ended() -> void:
    pass
