extends Node

signal transition_started
signal transition_finished

var _is_transitioning: bool = false
var _overlay: ColorRect = null


func change_scene(target: String, fade_duration: float = 0.5) -> void:
    if _is_transitioning:
        return
    _is_transitioning = true
    transition_started.emit()
    await _fade_out(fade_duration)
    get_tree().change_scene_to_file(target)
    await get_tree().process_frame
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
    await get_tree().process_frame
    await _fade_in(fade_duration)
    _is_transitioning = false
    transition_finished.emit()


func _fade_out(duration: float) -> void:
    _overlay = ColorRect.new()
    _overlay.color = Color.BLACK
    _overlay.mouse_filter = Control.MOUSE_FILTER_IGNORE
    _overlay.z_index = 100
    _overlay.set_anchors_preset(Control.PRESET_FULL_RECT)
    get_tree().root.add_child(_overlay)
    _overlay.modulate.a = 0.0
    var tween = create_tween()
    tween.tween_property(_overlay, "modulate:a", 1.0, duration)
    await tween.finished


func _fade_in(duration: float) -> void:
    if not _overlay or not is_instance_valid(_overlay):
        return
    var tween = create_tween()
    tween.tween_property(_overlay, "modulate:a", 0.0, duration)
    await tween.finished
    _overlay.queue_free()
    _overlay = null
