extends Area2D

@export var item_id: String = ""
@export var item_name: String = ""
@export var amount: int = 1
@export var sfx_stream: AudioStream


func _on_body_entered(body: Node2D) -> void:
    if body.is_in_group("player") and item_id != "":
        InventoryManager.add_item(item_id, amount)
        if sfx_stream:
            AudioManager.play_sfx(sfx_stream)
        queue_free()
