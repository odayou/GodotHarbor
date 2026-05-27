extends Node

var master_volume: float = 1.0
var music_volume: float = 0.8
var sfx_volume: float = 1.0

var _music_players: Dictionary = {}
var _sfx_players: Array = []

func _ready() -> void:
	for i in range(8):
		var player = AudioStreamPlayer.new()
		player.bus = "Master"
		add_child(player)
		_sfx_players.append(player)

func play_music(stream: AudioStream, bus: String = "Music") -> void:
	if _music_players.has(bus):
		var old = _music_players[bus]
		if old.playing:
			old.stop()
	var player = AudioStreamPlayer.new()
	player.stream = stream
	player.bus = bus
	player.volume_db = linear_to_db(music_volume)
	add_child(player)
	player.play()
	_music_players[bus] = player
	player.finished.connect(func(): player.queue_free())

func play_sfx(stream: AudioStream) -> void:
	for player in _sfx_players:
		if not player.playing:
			player.stream = stream
			player.volume_db = linear_to_db(sfx_volume)
			player.play()
			return

func set_master_volume(value: float) -> void:
	master_volume = clampf(value, 0.0, 1.0)
	AudioServer.set_bus_volume_db(AudioServer.get_bus_index("Master"), linear_to_db(master_volume))

func set_music_volume(value: float) -> void:
	music_volume = clampf(value, 0.0, 1.0)

func set_sfx_volume(value: float) -> void:
	sfx_volume = clampf(value, 0.0, 1.0)
