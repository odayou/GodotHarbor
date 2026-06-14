extends CanvasLayer

@onready var grid_container: GridContainer = $Panel/GridContainer


func _ready() -> void:
	visible = false
	InventoryManager.inventory_changed.connect(_on_inventory_changed)


func toggle() -> void:
	visible = not visible
	if visible:
		_refresh()


func _on_inventory_changed(_item_id: String, _amount: int) -> void:
	if visible:
		_refresh()


func _refresh() -> void:
	if not grid_container:
		return
	for child in grid_container.get_children():
		child.queue_free()
	for item_id in InventoryManager._items:
		var amount = InventoryManager._items[item_id]
		var label = Label.new()
		label.text = "%s x%d" % [item_id, amount]
		grid_container.add_child(label)
