extends Node

signal transition_started
signal transition_finished

var _is_transitioning: bool = false


func change_scene(target: String, fade_duration: float = 0.5) -> void:
    if _is_transitioning:
        return
    _is_transitioning = true
    transition_started.emit()
    await _fade_out(fade_duration)
    get_tree().change_scene_to_file(target)
    await get_tree().tree_changed
    await _fade_in(fade_duration)
    _is_transitioning = false
    transition_finished.emit()


func change_scene_to_packed(scene: PackedScene, fade_duration: float = 0.5) -> void:
    if _is_transitioning:
        return
    _is_transitioning = true
    transition_started.emit()
    await _fade_out(fade_duration)
    get_tree().change_scene_to_packed(scene)
    await get_tree().tree_changed
    await _fade_in(fade_duration)
    _is_transitioning = false
    transition_finished.emit()


func reload_current_scene(fade_duration: float = 0.3) -> void:
    if _is_transitioning:
        return
    _is_transitioning = true
    transition_started.emit()
    await _fade_out(fade_duration)
    get_tree().reload_current_scene()
    await _fade_in(fade_duration)
    _is_transitioning = false
    transition_finished.emit()


func _fade_out(duration: float) -> void:
    var overlay = ColorRect.new()
    overlay.color = Color.BLACK
    overlay.mouse_filter = Control.MOUSE_FILTER_IGNORE
    overlay.z_index = 100
    overlay.set_anchors_preset(Control.PRESET_FULL_RECT)
    get_tree().root.add_child(overlay)
    overlay.modulate.a = 0.0
    var tween = create_tween()
    tween.tween_property(overlay, "modulate:a", 1.0, duration)
    await tween.finished


func _fade_in(duration: float) -> void:
    var overlay = get_tree().root.get_child(get_tree().root.get_child_count() - 1)
    if not overlay is ColorRect:
        return
    var tween = create_tween()
    tween.tween_property(overlay, "modulate:a", 0.0, duration)
    await tween.finished
    overlay.queue_free()
