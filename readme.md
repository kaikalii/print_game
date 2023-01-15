# Running the Examples

The examples don't do much on their own.
They have to be run as sub-processes of the print_game frontend.

```
cargo r run cargo r -- -p example
```

# Init Commands

- title title:string
- window_size width:f32 height:f32
- end_init

# Input Commands

- window_size width:f32 height:f32
- mouse_pos x:f32 y:f32
- key key:string pressed:bool ctrl:bool shift:bool alt:bool
- mouse_moved x:f32 y:f32
- end_input

# Frame commands

- clear
  - set the clear color to the current color
- color r:f32 g:f32 b:f32 a:f32
  - set the current color
- color color:string
  - set the current color
- rectangle x:f32 y:f32 width:f32 height:f32
  - draw a rectangle
- circle x:f32 y:f32 radius:f32
  - draw a circle
- polygon [x:f32 y:f32]
  - draw a polygon
- anchor horizontal:string vertical:string
  - set the current anchor point
- font_size size:f32
  - set the current font size
- text x:f32 y:f32 text:string
  - draw text
- show_cursor show:bool
  - show or hide the mouse cursor
- end_frame