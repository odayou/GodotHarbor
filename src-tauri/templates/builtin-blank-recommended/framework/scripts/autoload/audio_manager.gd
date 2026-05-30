extends Node

var _bgm_player: AudioStreamPlayer
var _sfx_players: Array[AudioStreamPlayer] = []
var _sfx_pool_size: int = 8
var bgm_volume: float = 0.5
var sfx_volume: float = 0.8


func _ready() -> void:
    _bgm_player = AudioStreamPlayer.new()
    _bgm_player.bus = "Master"
    add_child(_bgm_player)
    for i in _sfx_pool_size:
        var player = AudioStreamPlayer.new()
        player.bus = "Master"
        add_child(player)
        _sfx_players.append(player)


func play_bgm(stream: AudioStream, fade_duration: float = 1.0) -> void:
    if _bgm_player.playing and _bgm_player.stream == stream:
        return
    if _bgm_player.playing:
        var tween = create_tween()
        tween.tween_property(_bgm_player, "volume_db", -80.0, fade_duration)
        await tween.finished
    _bgm_player.stream = stream
    _bgm_player.volume_db = linear_to_db(bgm_volume)
    _bgm_player.play()


func stop_bgm(fade_duration: float = 1.0) -> void:
    if not _bgm_player.playing:
        return
    var tween = create_tween()
    tween.tween_property(_bgm_player, "volume_db", -80.0, fade_duration)
    await tween.finished
    _bgm_player.stop()


func play_sfx(stream: AudioStream, pitch_variation: float = 0.0) -> void:
    var player = _get_available_sfx_player()
    if not player:
        return
    player.stream = stream
    player.volume_db = linear_to_db(sfx_volume)
    player.pitch_scale = 1.0 + randf_range(-pitch_variation, pitch_variation)
    player.play()


func set_bgm_volume(volume: float) -> void:
    bgm_volume = clampf(volume, 0.0, 1.0)
    _bgm_player.volume_db = linear_to_db(bgm_volume)


func set_sfx_volume(volume: float) -> void:
    sfx_volume = clampf(volume, 0.0, 1.0)


func _get_available_sfx_player() -> AudioStreamPlayer:
    for player in _sfx_players:
        if not player.playing:
            return player
    return null
