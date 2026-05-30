extends Node

signal quest_started(quest_id: String)
signal quest_advanced(quest_id: String, step: int)
signal quest_completed(quest_id: String)
signal quest_failed(quest_id: String)

var _active_quests: Dictionary = {}
var _completed_quests: Dictionary = {}


func start_quest(quest_id: String, quest_data: Dictionary) -> void:
    if _active_quests.has(quest_id) or _completed_quests.has(quest_id):
        return
    _active_quests[quest_id] = {
        "id": quest_id,
        "title": quest_data.get("title", quest_id),
        "description": quest_data.get("description", ""),
        "steps": quest_data.get("steps", []),
        "current_step": 0,
        "rewards": quest_data.get("rewards", {}),
    }
    quest_started.emit(quest_id)


func advance_quest(quest_id: String) -> void:
    if not _active_quests.has(quest_id):
        return
    var quest = _active_quests[quest_id]
    quest.current_step += 1
    if quest.current_step >= quest.steps.size():
        complete_quest(quest_id)
    else:
        quest_advanced.emit(quest_id, quest.current_step)


func complete_quest(quest_id: String) -> void:
    if not _active_quests.has(quest_id):
        return
    var quest = _active_quests[quest_id]
    _completed_quests[quest_id] = quest
    _active_quests.erase(quest_id)
    _grant_rewards(quest.rewards)
    quest_completed.emit(quest_id)


func fail_quest(quest_id: String) -> void:
    if not _active_quests.has(quest_id):
        return
    _active_quests.erase(quest_id)
    quest_failed.emit(quest_id)


func is_quest_active(quest_id: String) -> bool:
    return _active_quests.has(quest_id)


func is_quest_completed(quest_id: String) -> bool:
    return _completed_quests.has(quest_id)


func get_quest_progress(quest_id: String) -> Dictionary:
    if _active_quests.has(quest_id):
        var q = _active_quests[quest_id]
        return {"step": q.current_step, "total": q.steps.size(), "current_text": q.steps[q.current_step] if q.current_step < q.steps.size() else ""}
    return {}


func get_active_quests() -> Array:
    return _active_quests.values()


func _grant_rewards(rewards: Dictionary) -> void:
    if rewards.has("score"):
        GameManager.add_score(rewards.score)
