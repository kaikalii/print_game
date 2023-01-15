#![allow(unused_variables, unused_assignments)]

pub fn main() {
    // Send init commands
    println!("title Example Game");
    println!("end_init");

    // Initialize engine state
    let mut window_width = 0.0;
    let mut window_height = 0.0;
    let mut mouse_x = 0.0;
    let mut mouse_y = 0.0;

    // Main game loop
    loop {
        // Read input
        for line in std::io::stdin().lines() {
            let line = line.unwrap();
            let command: Vec<&str> = line.split_whitespace().collect();
            match command.as_slice() {
                ["window_size", x, y] => {
                    window_width = x.parse().unwrap();
                    window_height = y.parse().unwrap();
                }
                ["mouse_moved", x, y] => {
                    mouse_x = x.parse().unwrap();
                    mouse_y = y.parse().unwrap();
                    eprintln!("mouse pos: {} {}", mouse_x, mouse_y);
                }
                ["key", key, pressed, ..] => {
                    if pressed.parse().unwrap() {
                        eprintln!("key pressed: {}", key);
                    }
                }
                ["end_input"] => {
                    break;
                }
                command => {
                    eprintln!("Invalid input command: {:?}", command);
                }
            }
        }

        // Write frame
        println!("end_frame");
    }
}
