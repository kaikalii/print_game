# Description

`print_game` is a game engine that uses standard input and output to interface with the frontend.
It allows you to write games in any programming language, even those without FFI or builtin graphics support.

Drawing and other commands are executed by printing lines to stdout.
Input events are read as lines from stdin.

For example if you wanted to draw a square in Python, you could do this:
```python
print("/color red")
print("/rectangle 40 120 80 80")
```
To draw a sprite:
```python
print("/image assets/my_texture.png 100 100 200 200")
```

Handling input events is similar:
```python
event = ""
while event != "end_input":
    [event, *args] = input().split(" ")
    if event == "mouse_moved":
        print("mouse at {} {}".format(args[0], args[1]))
```

# Installation

To install, you can either download from github releases or build from source if you have Rust installed:
```
cargo install print_game
```

# Running the Examples

The examples don't do much on their own.
They have to be run as sub-processes of the `print_game` frontend.

To run the [Python example](example.py)
```
print_game python example.py
```

# Writing a Backend

The [Python example](example.py) is simple and easy to follow.

A backend program consists of a few parts:

#### Intialization
First is the initialization step.
Print lines to stdout to set up the window and other settings.
Print `/end_init` to open the window.
Initialization commands can be found [below](#init-commands).

#### Game Loop
Next is the main game loop.
The game loop should consist of two parts, an input reading phase and a frame drawing phase.

##### Reading Input Events
To read input events, read lines from stdin until you get `end_input`.
Input events can be found [below](#input-events).

##### Drawing a Frame
To draw a frame, print lines to stdout and end the frame with `/end_frame`.
Frame commands can be found [below](#frame-commands).

### Init Commands

- `/title` `title:string`
- `/window_size` `width:f32` `height:f32`
- `/vsync` `enabled:bool`
- `/end_init`

### Input Events

- `window_resized` `width:f32` `height:f32`
- `key` `key:string` `pressed:bool` `repeat:bool` `ctrl:bool` `shift:bool` `alt:bool`
- `mouse_button` `button:string` `pressed:bool` `ctrl:bool` `shift:bool` `alt:bool`
- `mouse_moved` `x:f32` `y:f32`
- `end_input`

### Frame commands

Frame commands are printed to stdout and are executed in order.
Commands that start with `get_*` have a response line that must be retrieved from stdin.

- `/clear`
  - set the clear color to the current color
- `/color` `r:f32` `g:f32` `b:f32` `a:f32`
  - set the current color
  - ex: `/color 1 0 0.5 1`
- `/color` `color:string`
  - set the current color
  - most css color strings should be valid
  - ex: `/color red`
  - ex: `/color #ff0000`
  - ex: `/color rgb(100%,0%,0%)`
- `/rectangle` `x:f32` `y:f32` `width:f32` `height:f32`
  - draw a rectangle
  - ex: `/rectangle 40 120 80 80`
- `/circle` `x:f32` `y:f32` `radius:f32`
  - draw a circle
  - ex: `/circle 100 200 50`
- `/polygon` [`x:f32` `y:f32`]
  - draw a polygon
  - ex: `/polygon 100 100 200 100 200 200 100 200`
- `/anchor` `horizontal:string` `vertical:string`
  - set the current anchor point for text and shape drawing
  - valid horizontal values: `left`, `center`, `right`
  - valid vertical values: `top`, `center`, `bottom`
- `/font_size` `size:f32`
  - set the current font size
- `/text` `x:f32` `y:f32` text:string
  - draw `text`
- `/image` `path:string` `x:f32` `y:f32` `width:f32` `height:f32` `uv_x:f32` `uv_y:f32` `uv_width:f32` `uv_height:f32`
  - draw an image
  - values after `path` are optional
  - if provided, `width` and `height` will scale the image
  - if provided, `uv_x`, `uv_y`, `uv_width`, and `uv_height` will crop the image
  - uv values are in the range [0, 1]
  - ex: `/image assets/my_texture.png 100 100 200 200`
- `/get_texture_size` `path:string`
  - get the size of an image
  - response: `width:u32` `height:u32`
  - ex: `/texture_size assets/my_texture.png`
- `/show_cursor` show:bool
  - show or hide the mouse cursor
- `/end_frame`