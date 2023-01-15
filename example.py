import math

mouse_x = 0.0
mouse_y = 0.0
window_width = 0.0
window_height = 0.0
keys = set()

pos_x = 100.0
pos_y = 100.0
speed = 200.0

character_sprite = "assets/character_sprite.png"
grass_sprite = "assets/grass.png"

# Init
print("title Python Example")
print("window_size 800 600")
print("vsync true")
print("end_init")

while True:
    # Read input events
    input_line = ""
    while input_line != "end_input":
        # Split the line into event and arguments
        input_line = input()
        split_line = input_line.split(" ")
        event = split_line[0]
        args = split_line[1:]
        # Handle events
        if event == "mouse_moved":
            mouse_x = float(args[0])
            mouse_y = float(args[1])
        elif event == "window_resized":
            window_width = float(args[0])
            window_height = float(args[1])
        elif event == "dt":
            dt = float(args[0])
        elif event == "key":
            key = args[0]
            pressed = args[1] == "true"
            if pressed:
                keys.add(key)
            else:
                keys.discard(key)
        elif event == "mouse_button":
            button = args[0]
            pressed = args[1] == "true"
            if pressed and button == "Primary":
                print("/click")

    # Move character
    vel_x = float("D" in keys) - float("A" in keys)
    vel_y = float("S" in keys) - float("W" in keys)
    vel_mag = pow(vel_x * vel_x + vel_y * vel_y, 0.5)
    if vel_mag > 0.0:
        vel_x /= vel_mag
        vel_y /= vel_mag
    pos_x += vel_x * speed * dt
    pos_y += vel_y * speed * dt

    # Draw frame

    # Set clear color
    print("color white")
    print("clear")

    # Draw tiling grass
    print("get_texture_size {}".format(grass_sprite))
    size = input().split(" ")
    size = [float(size[0]), float(size[1])]
    print("anchor left top")
    for i in range(int(math.ceil(window_width / size[0]))):
        for j in range(int(math.ceil(window_height / size[1]))):
            print("image {} {} {}".format(grass_sprite, i * size[0], j * size[1]))

    # Draw character
    print("anchor center")
    print("image {} {} {} 100 100".format(character_sprite, pos_x, pos_y))

    # Draw mouse position
    print("color red")
    print("circle {} {} 20".format(mouse_x, mouse_y))

    # Draw FPS
    print("color black")
    print("anchor left top")
    print("text 1 1 {} fps".format(int(1.0 / dt)))
    
    # End frame
    print("end_frame")