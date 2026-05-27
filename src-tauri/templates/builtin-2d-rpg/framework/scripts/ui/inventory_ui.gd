extends CanvasLayer

@onready var grid_container: GridContainer = $Panel/GridContainer
var _items: Dictionary = {}

func _ready() -> void:
	visible = false

func toggle() -> void:
	visible = not visible
	if visible:
		_refresh()

func add_item(item_id: String, amount: int = 1) -> void:
	if _items.has(item_id):
		_items[item_id] += amount
	else:
		_items[item_id] = amount
	if visible:
		_refresh()

func remove_item(item_id: String, amount: int = 1) -> bool:
	if not _items.has(item_id) or _items[item_id] < amount:
		return false
	_items[item_id] -= amount
	if _items[item_id] <= 0:
		_items.erase(item_id)
	if visible:
		_refresh()
	return true

func has_item(item_id: String, amount: int = 1) -> bool:
	return _items.get(item_id, 0) >= amount

func _refresh() -> void:
	if not grid_container:
		return
	for child in grid_container.get_children():
		child.queue_free()
	for item_id in _items:
		var label = Label.new()
		label.text = "%s x%d" % [item_id, _items[item_id]]
		grid_container.add_child(label)
