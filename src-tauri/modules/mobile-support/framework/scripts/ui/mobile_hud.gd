extends CanvasLayer

@onready var score_label: Label = $ScoreLabel

func _ready() -> void:
	if GameManager.has_signal("score_changed"):
		GameManager.score_changed.connect(_on_score_changed)

func _on_score_changed(new_score: int) -> void:
	if score_label:
		score_label.text = "Score: %d" % new_score
