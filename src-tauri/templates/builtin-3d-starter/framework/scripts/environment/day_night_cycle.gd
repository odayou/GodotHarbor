extends DirectionalLight3D

@export var cycle_duration: float = 120.0
@export var max_energy: float = 1.0
@export var min_energy: float = 0.1

var _time: float = 0.0

func _process(delta: float) -> void:
	_time += delta / cycle_duration
	_time = fmod(_time, 1.0)
	var sun_angle = _time * 2.0 * PI
	rotation_degrees.x = -90.0 + rad_to_deg(sun_angle) * 0.5
	var energy_factor = maxf(0.0, cos(sun_angle - PI * 0.5))
	light_energy = lerpf(min_energy, max_energy, energy_factor)
