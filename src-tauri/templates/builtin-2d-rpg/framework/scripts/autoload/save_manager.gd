extends Node

const SAVE_PATH = "user://save_game.json"

var _save_data: Dictionary = {}


func _ready() -> void:
    load_game()


func save_game() -> bool:
    _save_data["timestamp"] = Time.get_datetime_string_from_system()
    _save_data["player"] = _collect_player_data()
    _save_data["quests"] = _collect_quest_data()
    _save_data["inventory"] = _collect_inventory_data()
    var file = FileAccess.open(SAVE_PATH, FileAccess.WRITE)
    if not file:
        return false
    file.store_string(JSON.stringify(_save_data, "\t"))
    file.close()
    return true


func load_game() -> bool:
    if not FileAccess.file_exists(SAVE_PATH):
        return false
    var file = FileAccess.open(SAVE_PATH, FileAccess.READ)
    if not file:
        return false
    var json = JSON.new()
    var error = json.parse(file.get_as_text())
    file.close()
    if error != OK:
        return false
    _save_data = json.data
    return true


func has_save() -> bool:
    return FileAccess.file_exists(SAVE_PATH)


func delete_save() -> void:
    if FileAccess.file_exists(SAVE_PATH):
        DirAccess.remove_absolute(SAVE_PATH)
    _save_data.clear()


func get_save_data() -> Dictionary:
    return _save_data


func _collect_player_data() -> Dictionary:
    var player = get_tree().get_first_node_in_group("player")
    if not player:
        return {}
    var data = {"position_x": player.global_position.x, "position_y": player.global_position.y}
    if player.get("health"):
        data["health"] = player.health
    return data


func _collect_quest_data() -> Dictionary:
    if not QuestManager:
        return {}
    var quests = {}
    for quest in QuestManager.get_active_quests():
        quests[quest.id] = {"current_step": quest.current_step}
    return quests


func _collect_inventory_data() -> Dictionary:
    if not InventoryManager:
        return {}
    return {"items": InventoryManager.get_save_data()}
