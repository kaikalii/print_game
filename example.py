import sys, time

def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)

# Init
print("title Python Example")
print("window_size 800 600")
print("vsync true")
print("end_init")

mouse_x = 0.0
mouse_y = 0.0

while True:
    # Read input
    input_line = ""
    while input_line != "end_input":
        input_line = input()
        com = input_line.split(" ")
        args = com[1:]
        com = com[0]
        if com == "mouse_moved":
            mouse_x = float(args[0])
            mouse_y = float(args[1])
        elif com == "dt":
            dt = float(args[0])

    # Draw frame
    print("color white")
    print("clear")

    print("anchor center")
    print("color red")
    print("rectangle {} {} 100 100".format(mouse_x, mouse_y))

    print("color black")
    print("anchor left top")
    print("text 1 1 {} fps".format(int(1.0 / dt)))
    
    print("end_frame")