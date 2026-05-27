extends State

func enter() -> void:
	pass

func physics_update(delta: float) -> void:
	actor.velocity.y += actor.gravity * delta
	var input_dir = Input.get_vector("move_left", "move_right", "move_up", "move_down")
	actor.velocity.x = input_dir.x * actor.speed
	if actor.is_on_floor():
		state_machine.transition_to("Idle")
		return
	if actor.velocity.y > 0.0:
		state_machine.transition_to("Fall")
	actor.move_and_slide()

func exit() -> void:
	pass
