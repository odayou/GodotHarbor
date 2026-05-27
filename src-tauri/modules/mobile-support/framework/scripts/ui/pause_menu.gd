extends CanvasLayer

@onready var resume_button: Button = $Panel/VBoxContainer/ResumeButton
@onready var quit_button: Button = $Panel/VBoxContainer/QuitButton

func _ready() -> void:
	visible = false
	GameManager.game_paused.connect(func(): visible = true)
	GameManager.game_resumed.connect(func(): visible = false)
	if resume_button:
		resume_button.pressed.connect(GameManager.resume_game)
	if quit_button:
		quit_button.pressed.connect(GameManager.quit_game)

func _input(event: InputEvent) -> void:
	if event.is_action_pressed("ui_cancel"):
		GameManager.toggle_pause()
