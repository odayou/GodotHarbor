extends Control

@onready var address_input: LineEdit = $VBoxContainer/AddressInput
@onready var port_input: SpinBox = $VBoxContainer/PortInput
@onready var host_button: Button = $VBoxContainer/ButtonContainer/HostButton
@onready var join_button: Button = $VBoxContainer/ButtonContainer/JoinButton
@onready var status_label: Label = $VBoxContainer/StatusLabel
@onready var player_list: ItemList = $VBoxContainer/PlayerList


func _ready() -> void:
    if host_button:
        host_button.pressed.connect(_on_host_pressed)
    if join_button:
        join_button.pressed.connect(_on_join_pressed)
    NetworkManager.player_connected.connect(_on_player_connected)
    NetworkManager.player_disconnected.connect(_on_player_disconnected)
    NetworkManager.server_disconnected.connect(_on_server_disconnected)


func _on_host_pressed() -> void:
    var port = int(port_input.value) if port_input else 7777
    if NetworkManager.host_game(port):
        _set_status("Hosting on port %d" % port)
        _show_lobby()
    else:
        _set_status("Failed to host")


func _on_join_pressed() -> void:
    var address = address_input.text if address_input else "127.0.0.1"
    var port = int(port_input.value) if port_input else 7777
    _set_status("Connecting...")
    if NetworkManager.join_game(address, port):
        _set_status("Connected!")
        _show_lobby()
    else:
        _set_status("Connection failed")


func _show_lobby() -> void:
    if host_button:
        host_button.disabled = true
    if join_button:
        join_button.disabled = true
    _refresh_player_list()


func _on_player_connected(_peer_id: int) -> void:
    _refresh_player_list()


func _on_player_disconnected(_peer_id: int) -> void:
    _refresh_player_list()


func _on_server_disconnected() -> void:
    _set_status("Server disconnected")
    if host_button:
        host_button.disabled = false
    if join_button:
        join_button.disabled = false


func _refresh_player_list() -> void:
    if not player_list:
        return
    player_list.clear()
    for player in NetworkManager.get_player_list():
        player_list.add_item(player.name)


func _set_status(text: String) -> void:
    if status_label:
        status_label.text = text
