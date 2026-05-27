extends Node
class_name StateMachine3D

@export var initial_state: State3D
var current_state: State3D
var states: Dictionary = {}

func _ready() -> void:
	for child in get_children():
		if child is State3D:
			states[child.name.to_lower()] = child
			child.state_machine = self
			child.actor = get_parent()
	if initial_state:
		current_state = initial_state
		current_state.enter()

func _process(delta: float) -> void:
	if current_state:
		current_state.update(delta)

func _physics_process(delta: float) -> void:
	if current_state:
		current_state.physics_update(delta)

func transition_to(state_name: String) -> void:
	var new_state = states.get(state_name.to_lower())
	if new_state and new_state != current_state:
		if current_state:
			current_state.exit()
		current_state = new_state
		current_state.enter()
