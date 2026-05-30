extends Control

@onready var chat_log: RichTextLabel = $ChatLog
@onready var chat_input: LineEdit = $ChatInput
@onready var chat_toggle: Button = $ChatToggle

var _is_chat_open: bool = false


func _ready() -> void:
    if chat_input:
        chat_input.visible = false
        chat_input.text_submitted.connect(_on_text_submitted)
    if chat_toggle:
        chat_toggle.pressed.connect(_toggle_chat)


func _input(event: InputEvent) -> void:
    if event.is_action_pressed("chat"):
        _toggle_chat()


func _toggle_chat() -> void:
    _is_chat_open = not _is_chat_open
    if chat_input:
        chat_input.visible = _is_chat_open
        if _is_chat_open:
            chat_input.grab_focus()
        else:
            chat_input.release_focus()
    if chat_toggle:
        chat_toggle.visible = not _is_chat_open


func _on_text_submitted(text: String) -> void:
    if text.strip_edges() == "":
        return
    var sender_id = multiplayer.get_unique_id()
    rpc("_receive_message", sender_id, text)
    chat_input.text = ""
    _toggle_chat()


@rpc("any_peer", "call_local")
func _receive_message(sender_id: int, text: String) -> void:
    var sender_name = "Player_%d" % sender_id
    if NetworkManager and NetworkManager.players.has(sender_id):
        sender_name = NetworkManager.players[sender_id].get("name", sender_name)
    _add_to_log("[%s]: %s" % [sender_name, text])


func _add_to_log(msg: String) -> void:
    if chat_log:
        chat_log.append_text(msg + "\n")
