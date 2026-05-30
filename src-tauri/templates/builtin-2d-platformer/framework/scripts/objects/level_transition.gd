extends Area2D

@export_file("*.tscn") var target_scene: String = ""
@export var spawn_point: String = ""


func _on_body_entered(body: Node2D) -> void:
    if body is CharacterBody2D and target_scene != "":
        GameManager.set_spawn_point(spawn_point)
        ScreenManager.change_scene(target_scene)
