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
    let mut keys = BTreeSet::new();

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
                ["key", key, pressed, ..] => {
                    if pressed.parse().unwrap() {
                        keys.insert(key.to_string());
                    } else {
                        keys.remove(*key);
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
        println!("color #505050");
        println!("clear");
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
        println!("color red");
        println!("rectangle {} {} 100 100", mouse_x - 50.0, mouse_y - 50.0);
        println!("end_frame");
    }
}
