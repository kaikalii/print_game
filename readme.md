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
- color r:f32 g:f32 b:f32 a:f32
- color color:string
- end_frame