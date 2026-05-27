extends CanvasLayer

@onready var resume_button: Button = $Panel/VBoxContainer/ResumeButton
@onready var restart_button: Button = $Panel/VBoxContainer/RestartButton
@onready var quit_button: Button = $Panel/VBoxContainer/QuitButton

func _ready() -> void:
	GameManager.game_paused.connect(_on_game_paused)
	GameManager.game_resumed.connect(_on_game_resumed)
	visible = false
	if resume_button:
		resume_button.pressed.connect(GameManager.resume_game)
	if restart_button:
		restart_button.pressed.connect(GameManager.restart_game)
	if quit_button:
		quit_button.pressed.connect(GameManager.quit_game)

func _input(event: InputEvent) -> void:
	if event.is_action_pressed("ui_cancel"):
		GameManager.toggle_pause()

func _on_game_paused() -> void:
	visible = true

func _on_game_resumed() -> void:
	visible = false
