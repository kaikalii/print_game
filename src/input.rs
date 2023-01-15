use std::{
    io::{self, Write},
    time::Instant,
};

use eframe::egui::*;

use crate::Server;

impl Server {
    pub fn send_input_lines(&mut self, ui: &mut Ui) -> io::Result<()> {
        let now = Instant::now();
        let dt = (now - self.last_instant).as_secs_f32();
        self.last_instant = now;

        // Gather input lines
        let mut input_lines = Vec::new();
        // Window resized
        if self.last_window_size != vec2(ui.available_width(), ui.available_height()) {
            self.last_window_size = vec2(ui.available_width(), ui.available_height());
            input_lines.push(format!(
                "window_resized {} {}",
                ui.available_width(),
                ui.available_height()
            ));
        }
        // Events
        for event in &ui.input().events {
            match event {
                Event::Key {
                    key,
                    pressed,
                    modifiers,
                } => {
                    input_lines.push(format!(
                        "key {key:?} {pressed} {} {} {} {}",
                        self.keys_down.contains(key),
                        modifiers.ctrl,
                        modifiers.shift,
                        modifiers.alt
                    ));
                    if *pressed {
                        self.keys_down.insert(*key);
                    } else {
                        self.keys_down.remove(key);
                    }
                }
                Event::PointerButton {
                    button,
                    pressed,
                    modifiers,
                    ..
                } => {
                    input_lines.push(format!(
                        "mouse_button {button:?} {pressed} {} {} {}",
                        modifiers.ctrl, modifiers.shift, modifiers.alt
                    ));
                }
                Event::PointerMoved(pos) => {
                    input_lines.push(format!("mouse_moved {} {}", pos.x, pos.y));
                }
                _ => {}
            }
        }
        // ΔT
        input_lines.push(format!("dt {dt}"));
        // End
        input_lines.push("end_input".into());

        // Send input lines
        for line in input_lines {
            self.stdin.write_all(line.as_bytes())?;
            self.stdin.write_all(b"\n")?;
        }
        Ok(())
    }
}
