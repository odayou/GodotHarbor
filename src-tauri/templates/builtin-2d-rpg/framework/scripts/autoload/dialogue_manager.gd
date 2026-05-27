extends Node

signal dialogue_started(dialogue_id: String)
signal dialogue_ended(dialogue_id: String)
signal line_displayed(speaker: String, text: String)
signal choice_presented(choices: Array)

var _is_active: bool = false
var _current_dialogue_id: String = ""
var _dialogues: Dictionary = {}
var _current_line_index: int = 0
var _current_lines: Array = []

func _ready() -> void:
	_load_dialogues()

func start_dialogue(dialogue_id: String) -> void:
	if _is_active:
		return
	if not _dialogues.has(dialogue_id):
		push_warning("Dialogue not found: %s" % dialogue_id)
		return
	_is_active = true
	_current_dialogue_id = dialogue_id
	_current_lines = _dialogues[dialogue_id]
	_current_line_index = 0
	dialogue_started.emit(dialogue_id)
	_show_current_line()

func advance() -> void:
	if not _is_active:
		return
	_current_line_index += 1
	if _current_line_index >= _current_lines.size():
		end_dialogue()
		return
	_show_current_line()

func end_dialogue() -> void:
	_is_active = false
	var id = _current_dialogue_id
	_current_dialogue_id = ""
	_current_lines = []
	_current_line_index = 0
	dialogue_ended.emit(id)

func is_active() -> bool:
	return _is_active

func _show_current_line() -> void:
	var line = _current_lines[_current_line_index]
	if line.has("choices"):
		choice_presented.emit(line.choices)
	else:
		line_displayed.emit(line.get("speaker", ""), line.get("text", ""))

func _load_dialogues() -> void:
	var dir = DirAccess.open("res://data/dialogues/")
	if not dir:
		return
	dir.list_dir_begin()
	var file_name = dir.get_next()
	while file_name != "":
		if file_name.ends_with(".json"):
			var file = FileAccess.open("res://data/dialogues/" + file_name, FileAccess.READ)
			if file:
				var json = JSON.new()
				if json.parse(file.get_as_text()) == OK:
					var dialogue_id = file_name.replace(".json", "")
					_dialogues[dialogue_id] = json.data
				file.close()
		file_name = dir.get_next()
