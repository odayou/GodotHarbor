extends Node

const SAVE_DIR = "user://saves/"

signal save_completed(slot: int)
signal load_completed(slot: int)

var _current_slot: int = 0

func _ready() -> void:
	DirAccess.make_dir_recursive_absolute(SAVE_DIR)

func save_game(slot: int = _current_slot) -> bool:
	var data = _collect_save_data()
	var file_path = SAVE_DIR + "save_%d.json" % slot
	var file = FileAccess.open(file_path, FileAccess.WRITE)
	if not file:
		push_error("Failed to open save file: %s" % file_path)
		return false
	file.store_string(JSON.stringify(data, "\t"))
	file.close()
	_current_slot = slot
	save_completed.emit(slot)
	return true

func load_game(slot: int = _current_slot) -> bool:
	var file_path = SAVE_DIR + "save_%d.json" % slot
	if not FileAccess.file_exists(file_path):
		push_warning("Save file not found: %s" % file_path)
		return false
	var file = FileAccess.open(file_path, FileAccess.READ)
	if not file:
		return false
	var json = JSON.new()
	var err = json.parse(file.get_as_text())
	file.close()
	if err != OK:
		return false
	_apply_save_data(json.data)
	_current_slot = slot
	load_completed.emit(slot)
	return true

func delete_save(slot: int) -> bool:
	var file_path = SAVE_DIR + "save_%d.json" % slot
	if FileAccess.file_exists(file_path):
		DirAccess.remove_absolute(file_path)
		return true
	return false

func has_save(slot: int) -> bool:
	return FileAccess.file_exists(SAVE_DIR + "save_%d.json" % slot)

func list_saves() -> Array[int]:
	var saves: Array[int] = []
	var dir = DirAccess.open(SAVE_DIR)
	if dir:
		dir.list_dir_begin()
		var file_name = dir.get_next()
		while file_name != "":
			if file_name.begins_with("save_") and file_name.ends_with(".json"):
				var slot_str = file_name.replace("save_", "").replace(".json", "")
				var slot = slot_str.to_int()
				if slot > 0:
					saves.append(slot)
			file_name = dir.get_next()
	return saves

func _collect_save_data() -> Dictionary:
	var player = get_tree().get_first_node_in_group("player")
	var data = {
		"timestamp": Time.get_datetime_string_from_system(),
		"slot": _current_slot,
		"scene": get_tree().current_scene.scene_file_path,
	}
	if player:
		data["player"] = {
			"position_x": player.global_position.x,
			"position_y": player.global_position.y,
		}
	return data

func _apply_save_data(data: Dictionary) -> void:
	if data.has("scene"):
		get_tree().change_scene_to_file(data["scene"])
	await get_tree().tree_changed
	if data.has("player"):
		var player = get_tree().get_first_node_in_group("player")
		if player:
			player.global_position = Vector2(data.player.position_x, data.player.position_y)
