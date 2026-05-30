extends Node

signal inventory_changed
signal item_used(item_id: String)

var _items: Dictionary = {}


func add_item(item_id: String, amount: int = 1) -> void:
    if _items.has(item_id):
        _items[item_id]["amount"] += amount
    else:
        _items[item_id] = {"id": item_id, "amount": amount}
    inventory_changed.emit()


func remove_item(item_id: String, amount: int = 1) -> bool:
    if not _items.has(item_id):
        return false
    _items[item_id]["amount"] -= amount
    if _items[item_id]["amount"] <= 0:
        _items.erase(item_id)
    inventory_changed.emit()
    return true


func has_item(item_id: String, amount: int = 1) -> bool:
    if not _items.has(item_id):
        return false
    return _items[item_id]["amount"] >= amount


func get_item_count(item_id: String) -> int:
    if not _items.has(item_id):
        return 0
    return _items[item_id]["amount"]


func use_item(item_id: String) -> bool:
    if not has_item(item_id):
        return false
    item_used.emit(item_id)
    remove_item(item_id)
    return true


func get_all_items() -> Array:
    return _items.values()


func clear_inventory() -> void:
    _items.clear()
    inventory_changed.emit()


func get_save_data() -> Dictionary:
    return _items.duplicate()


func load_save_data(data: Dictionary) -> void:
    _items = data.duplicate()
    inventory_changed.emit()
