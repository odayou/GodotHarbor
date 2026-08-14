# SPDX-License-Identifier: MIT
# Harbor Bridge - 编辑器内 addon，连接 GodotHarbor MCP server
# 提供项目状态检测 + 一键 apply/restore/switch_engine
# 静默降级：Harbor 未装/未运行时不弹错不卡编辑器
@tool
extends EditorPlugin

const DEFAULT_PORT := 9876
const RECONNECT_INTERVAL := 5.0
const REQUEST_TIMEOUT := 3.0

var dock: Control = null
var tcp_client: StreamPeerTCP = null
var is_connected := false
var reconnect_timer: Timer = null
var status_label: Label = null
var action_buttons: Dictionary = {}
var pending_requests: Dictionary = {}  # id -> callback

func _enter_tree() -> void:
	_add_dock()
	_connect_to_server()

func _exit_tree() -> void:
	_disconnect_from_server()
	_remove_dock()

# ─── UI ───

func _add_dock() -> void:
	dock = VBoxContainer.new()
	dock.name = "Harbor Bridge"

	var title := Label.new()
	title.text = "Harbor Bridge"
	title.add_theme_font_size_override("font_size", 16)
	dock.add_child(title)

	status_label = Label.new()
	status_label.text = "未连接"
	status_label.add_theme_color_override("font_color", Color.GRAY)
	dock.add_child(status_label)

	# 操作按钮区
	var btn_row := HBoxContainer.new()
	dock.add_child(btn_row)

	_add_action_button(btn_row, "refresh", "刷新状态", "_on_refresh")
	_add_action_button(btn_row, "apply", "应用环境", "_on_apply")
	_add_action_button(btn_row, "restore", "还原环境", "_on_restore")

	add_control_to_dock(DOCK_SLOT_LEFT_UL, dock)

func _remove_dock() -> void:
	if dock != null:
		remove_control_from_docks(dock)
		dock.queue_free()
		dock = null

func _add_action_button(parent: Control, id: String, text: String, callback: String) -> void:
	var btn := Button.new()
	btn.text = text
	btn.disabled = true
	btn.pressed.connect(Callable(self, callback))
	parent.add_child(btn)
	action_buttons[id] = btn

# ─── TCP 连接 ───

func _connect_to_server() -> void:
	if tcp_client != null and tcp_client.get_status() == StreamPeerTCP.STATUS_CONNECTED:
		return

	tcp_client = StreamPeerTCP.new()
	var err = tcp_client.connect_to_host("127.0.0.1", DEFAULT_PORT)
	if err != OK:
		_schedule_reconnect()
		return

	# 等待连接（非阻塞轮询）
	_poll_connection()

func _poll_connection() -> void:
	if tcp_client == null:
		return
	var status = tcp_client.get_status()
	if status == StreamPeerTCP.STATUS_CONNECTED:
		if not is_connected:
			is_connected = true
			_update_ui_connected()
			_send_initialize()
	elif status == StreamPeerTCP.STATUS_CONNECTING:
		pass  # 继续等待
	else:
		# ERROR 或 NONE
		is_connected = false
		_update_ui_disconnected()
		_schedule_reconnect()

func _schedule_reconnect() -> void:
	if reconnect_timer == null:
		reconnect_timer = Timer.new()
		reconnect_timer.wait_time = RECONNECT_INTERVAL
		reconnect_timer.timeout.connect(_connect_to_server)
		add_child(reconnect_timer)
	reconnect_timer.start()

func _disconnect_from_server() -> void:
	if reconnect_timer != null:
		reconnect_timer.stop()
	if tcp_client != null:
		if tcp_client.get_status() == StreamPeerTCP.STATUS_CONNECTED:
			tcp_client.disconnect_from_host()
		tcp_client = null
	is_connected = false

func _process(_delta: float) -> void:
	if tcp_client != null:
		_poll_connection()
		_read_responses()

# ─── JSON-RPC ───

var next_id := 1

func _send_request(method: String, params: Dictionary, callback: Callable = Callable()) -> void:
	if not is_connected or tcp_client == null:
		return
	var id := next_id
	next_id += 1
	var request := {
		"jsonrpc": "2.0",
		"id": id,
		"method": method,
		"params": params
	}
	if callback.is_valid():
		pending_requests[id] = callback
	var json_str := JSON.stringify(request)
	tcp_client.put_data((json_str + "\n").to_utf8_buffer())

func _send_initialize() -> void:
	_send_request("initialize", {}, func(resp): _on_initialized(resp))

func _on_initialized(response: Variant) -> void:
	if response is Dictionary and response.has("result"):
		var result: Dictionary = response["result"]
		status_label.text = "已连接 (v" + str(result.get("serverInfo", {}).get("version", "?")) + ")"
		status_label.add_theme_color_override("font_color", Color.GREEN)
		_enable_buttons(true)
		_refresh_status()

func _read_responses() -> void:
	if tcp_client == null:
		return
	var available = tcp_client.get_available_bytes()
	if available <= 0:
		return
	var data = tcp_client.get_data(available)
	if data[0] != OK:
		return
	var text := (data[1] as PackedByteArray).get_string_from_utf8()
	# 按行解析（可能一次收到多行）
	for line in text.split("\n", false):
		if line.strip_edges() == "":
			continue
		var parsed = JSON.parse_string(line)
		if parsed == null or not parsed is Dictionary:
			continue
		if not parsed.has("id"):
			continue
		var id = int(parsed["id"])
		if pending_requests.has(id):
			var callback: Callable = pending_requests[id]
			pending_requests.erase(id)
			callback.call(parsed)

# ─── 操作 ───

func _refresh_status() -> void:
	_send_request("tools/list", {}, func(resp): _on_tools_listed(resp))

func _on_tools_listed(response: Variant) -> void:
	if not (response is Dictionary and response.has("result")):
		return
	var result: Dictionary = response["result"]
	var tools: Array = result.get("tools", [])
	for tool in tools:
		var name: String = tool.get("name", "")
		if action_buttons.has(name):
			(action_buttons[name] as Button).tooltip_text = tool.get("description", "")

func _on_refresh() -> void:
	_refresh_status()

func _on_apply() -> void:
	var project_path := ProjectSettings.globalize_path("res://")
	_send_request("tools/call", {"name": "apply_bindings", "arguments": {"project_path": project_path}},
		func(resp): _on_action_done("应用环境", resp))

func _on_restore() -> void:
	var project_path := ProjectSettings.globalize_path("res://")
	_send_request("tools/call", {"name": "restore_project_environment", "arguments": {"project_path": project_path}},
		func(resp): _on_action_done("还原环境", resp))

func _on_action_done(action_name: String, response: Variant) -> void:
	if response is Dictionary and response.has("result"):
		status_label.text = action_name + " 完成"
		status_label.add_theme_color_override("font_color", Color.GREEN)
	else:
		status_label.text = action_name + " 失败"
		status_label.add_theme_color_override("font_color", Color.RED)

# ─── UI 状态 ───

func _update_ui_connected() -> void:
	status_label.text = "连接中..."

func _update_ui_disconnected() -> void:
	status_label.text = "未连接 (Harbor 未运行)"
	status_label.add_theme_color_override("font_color", Color.GRAY)
	_enable_buttons(false)

func _enable_buttons(enabled: bool) -> void:
	for btn in action_buttons.values():
		(btn as Button).disabled = not enabled
