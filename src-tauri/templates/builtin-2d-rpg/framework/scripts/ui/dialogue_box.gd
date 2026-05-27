extends CanvasLayer

@onready var speaker_label: Label = $Panel/SpeakerLabel
@onready var text_label: RichTextLabel = $Panel/TextLabel
@onready var advance_button: Button = $Panel/AdvanceButton

var _typewriter_speed: float = 0.03
var _is_typing: bool = false
var _full_text: String = ""

func _ready() -> void:
	visible = false
	DialogueManager.line_displayed.connect(_on_line_displayed)
	DialogueManager.dialogue_ended.connect(_on_dialogue_ended)
	DialogueManager.choice_presented.connect(_on_choice_presented)
	if advance_button:
		advance_button.pressed.connect(_on_advance)

func _input(event: InputEvent) -> void:
	if not DialogueManager.is_active():
		return
	if event.is_action_pressed("interact") or event.is_action_pressed("ui_accept"):
		if _is_typing:
			_is_typing = false
			text_label.visible_characters = -1
		else:
			DialogueManager.advance()

func _on_line_displayed(speaker: String, text: String) -> void:
	visible = true
	speaker_label.text = speaker
	_full_text = text
	text_label.text = text
	text_label.visible_characters = 0
	_is_typing = true
	_typewriter_effect()

func _on_dialogue_ended(_dialogue_id: String) -> void:
	visible = false

func _on_choice_presented(choices: Array) -> void:
	pass

func _on_advance() -> void:
	if _is_typing:
		_is_typing = false
		text_label.visible_characters = -1
	else:
		DialogueManager.advance()

func _typewriter_effect() -> void:
	for i in range(_full_text.length()):
		if not _is_typing:
			break
		text_label.visible_characters = i + 1
		await get_tree().create_timer(_typewriter_speed).timeout
	_is_typing = false
