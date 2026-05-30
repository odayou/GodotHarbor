extends Control

@onready var health_bar: ProgressBar = $MarginContainer/HBoxContainer/HealthBar
@onready var health_label: Label = $MarginContainer/HBoxContainer/HealthLabel
@onready var crosshair: CenterContainer = $Crosshair

var _player: CharacterBody3D = null
var _max_health: int = 100


func _ready() -> void:
    await get_tree().process_frame
    _player = get_tree().get_first_node_in_group("player")
    if _player and _player.has_signal("health_changed"):
        _player.health_changed.connect(_on_health_changed)
        _max_health = _player.max_health if _player.get("max_health") else 100
        _on_health_changed(_player.health if _player.get("health") else _max_health)


func _on_health_changed(new_health: int) -> void:
    if health_bar:
        health_bar.max_value = _max_health
        health_bar.value = new_health
    if health_label:
        health_label.text = "%d / %d" % [new_health, _max_health]
