extends GdUnitTestSuite


func test_get_greeting() -> void:
	var main = auto_free(Node2D.new())
	var script = load("res://scripts/main.gd")
	main.set_script(script)
	assert_str(main.get_greeting()).is_equal("Hello from Godot Harbor!")
