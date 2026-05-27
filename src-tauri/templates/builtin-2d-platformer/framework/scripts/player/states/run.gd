extends State

func enter() -> void:
	pass

func physics_update(delta: float) -> void:
	var input_dir = Input.get_vector("move_left", "move_right", "move_up", "move_down")
	actor.velocity.x = input_dir.x * actor.speed
	if not actor.is_on_floor():
		state_machine.transition_to("Fall")
		return
	if Input.is_action_just_pressed("jump"):
		actor.velocity.y = actor.jump_force
		state_machine.transition_to("Jump")
		return
	if input_dir.x == 0.0:
		state_machine.transition_to("Idle")
	actor.move_and_slide()

func exit() -> void:
	pass
