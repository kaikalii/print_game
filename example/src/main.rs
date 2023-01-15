#![allow(unused_assignments, unused_variables)]

use std::collections::BTreeSet;

pub fn main() {
    // Send init commands
    println!("title Example Game");
    println!("end_init");

    // Initialize engine state
    let mut window_width = 0f32;
    let mut window_height = 0.0;
    let mut mouse_x = 0.0;
    let mut mouse_y = 0.0;
    let mut dt = 1.0 / 60.0;
    let mut keys = BTreeSet::new();
    let mut show_cursor = true;

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
                }
                &["key", key, pressed, repeat, ..] => {
                    if pressed.parse().unwrap() {
                        keys.insert(key.to_string());
                        let repeat: bool = repeat.parse().unwrap();
                        if !repeat && key == "Escape" {
                            show_cursor = !show_cursor;
                        }
                    } else {
                        keys.remove(key);
                    }
                }
                ["dt", t] => {
                    dt = t.parse().unwrap();
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
        println!("show_cursor {show_cursor}");
        println!("anchor center");

        // Clear screen
        println!("color #404040");
        println!("clear");

        // Big circle
        if keys.contains("Space") {
            println!("color cyan");
        } else {
            println!("color white");
        }
        println!(
            "circle {} {} {}",
            window_width / 2.0,
            window_height / 2.0,
            window_width.min(window_height) / 2.0
        );

        // Triangle
        println!("color lightgreen");
        println!("polygon 0 0 100 50 0 100");

        // Some text
        println!("color orange");
        println!("font_size 50");
        println!(
            "text {} {} Hello, World!",
            window_width / 2.0,
            window_height / 4.0
        );

        // Rectangle that follows the mouse
        println!("color red");
        println!("rectangle {mouse_x} {mouse_y} 100 100");

        // End frame
        println!("end_frame");
    }
}
