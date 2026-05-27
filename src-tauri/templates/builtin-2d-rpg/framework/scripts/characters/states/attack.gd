extends State

var _attack_timer: float = 0.0
var _attack_duration: float = 0.3

func enter() -> void:
	_attack_timer = _attack_duration

func physics_update(delta: float) -> void:
	_attack_timer -= delta
	if _attack_timer <= 0.0:
		state_machine.transition_to("Idle")

func exit() -> void:
	pass
