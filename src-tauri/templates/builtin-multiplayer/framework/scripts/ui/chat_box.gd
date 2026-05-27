extends CanvasLayer

@onready var chat_input: LineEdit = $Panel/VBoxContainer/ChatInput
@onready var chat_log: RichTextLabel = $Panel/VBoxContainer/ChatLog
@onready var send_button: Button = $Panel/VBoxContainer/SendButton

func _ready() -> void:
	if send_button:
		send_button.pressed.connect(_on_send)
	if chat_input:
		chat_input.text_submitted.connect(func(_text): _on_send())

func _input(event: InputEvent) -> void:
	if event.is_action_pressed("chat"):
		if chat_input:
			chat_input.grab_focus()

func _on_send() -> void:
	if not chat_input or chat_input.text.strip_edges() == "":
		return
	var msg = chat_input.text.strip_edges()
	chat_input.text = ""
	_send_message.rpc(msg)

@rpc("any_peer", "call_local", "reliable")
func _send_message(msg: String) -> void:
	var sender_id = multiplayer.get_remote_sender_id()
	var sender_name = "Player %d" % sender_id if sender_id != 0 else "You"
	_append_message(sender_name, msg)

func _append_message(sender: String, text: String) -> void:
	if chat_log:
		chat_log.append_text("[color=gray][%s]:[/color] %s\n" % [sender, text])
