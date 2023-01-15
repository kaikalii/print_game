import sys

def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)

# Init
print("title Pyton Example")
print("end_init")

mouse_x = 0
mouse_y = 0

while True:
    # Read input
    input_line = ""
    while input_line != "end_input":
        input_line = input()
        com = input_line.split(" ")
        if com[0] == "mouse_moved":
            mouse_x = com[1]
            mouse_y = com[2]

    # Draw frame
    print("color white")
    print("clear")

    print("anchor center")
    print("color red")
    print("rectangle {} {} 100 100".format(mouse_x, mouse_y))
    print("end_frame")