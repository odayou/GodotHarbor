extends Area2D

@export var item_id: String = ""
@export var item_name: String = ""
@export var amount: int = 1
@export var heal_amount: int = 0
@export var sfx_stream: AudioStream


func _on_body_entered(body: Node2D) -> void:
    if not body.is_in_group("player"):
        return
    if heal_amount > 0 and body.has_method("heal"):
        body.heal(heal_amount)
    if item_id != "" and InventoryManager:
        InventoryManager.add_item(item_id, amount)
    if sfx_stream:
        AudioManager.play_sfx(sfx_stream)
    queue_free()
