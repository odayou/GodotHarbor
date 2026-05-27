extends State

func enter() -> void:
	pass

func physics_update(_delta: float) -> void:
	actor.velocity = Vector2.ZERO
	if Input.is_action_just_pressed("attack"):
		state_machine.transition_to("Attack")
		return
	if Input.is_action_just_pressed("interact"):
		state_machine.transition_to("Interact")
		return
	var input_dir = Input.get_vector("move_left", "move_right", "move_up", "move_down")
	if input_dir != Vector2.ZERO:
		state_machine.transition_to("Walk")

func exit() -> void:
	pass
