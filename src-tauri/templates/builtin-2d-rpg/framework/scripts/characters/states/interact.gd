extends State

var _interact_timer: float = 0.0
var _interact_duration: float = 0.2

func enter() -> void:
	_interact_timer = _interact_duration
	var interactables = get_tree().get_nodes_in_group("interactable")
	var actor_pos = actor.global_position
	var closest = null
	var closest_dist = 64.0
	for obj in interactables:
		var dist = actor_pos.distance_to(obj.global_position)
		if dist < closest_dist:
			closest = obj
			closest_dist = dist
	if closest and closest.has_method("interact"):
		closest.interact(actor)

func physics_update(delta: float) -> void:
	_interact_timer -= delta
	if _interact_timer <= 0.0:
		state_machine.transition_to("Idle")

func exit() -> void:
	pass
