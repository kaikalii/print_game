use std::io::stdin;

fn main() {
    println!("end_init");
    let mut window_width = 0.0;
    let mut window_height = 0.0;
    let mut mouse_x = 0.0;
    let mut mouse_y = 0.0;
    loop {
        // Read input
        for line in stdin().lines() {
            let line = line.unwrap();
            let command: Vec<&str> = line.split_whitespace().collect();
            match command.as_slice() {
                ["window_size", x, y] => {
                    window_width = x.parse().unwrap();
                    window_height = y.parse().unwrap();
                }
                ["mouse_pos", x, y] => {
                    mouse_x = x.parse().unwrap();
                    mouse_y = y.parse().unwrap();
                }
                ["end_input"] => {
                    break;
                }
                command => {
                    eprintln!("Invalid input command: {:?}", command);
                }
            }
        }
        eprintln!("{} {}", mouse_x, mouse_y);

        // Write frame
        println!("end_frame");
    }
}
