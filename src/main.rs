use std::{
    io::{self, BufRead, BufReader, Read, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use clap::{Parser, Subcommand};
use eframe::egui::*;

fn main() {
    let app = App::parse();
    match app.sub {
        Some(Sub::Run { command, args }) => run_command(command, args),
        None => run_terminal(),
    };
}

#[derive(Parser)]
struct App {
    #[command(subcommand)]
    sub: Option<Sub>,
}

#[derive(Clone, Subcommand)]
enum Sub {
    Run { command: String, args: Vec<String> },
}

fn run_command(command: String, args: Vec<String>) {
    let child = Command::new(command)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let mut server = Server::new(child);
    let mut native_options = eframe::NativeOptions::default();
    let mut window_title = "My Game".to_owned();
    let closed = server.handle_io(
        || (),
        |server| {
            for line in server.stdout.by_ref().lines() {
                let line = line?;
                let command: Vec<&str> = line.split_whitespace().collect();
                match command.as_slice() {
                    ["title", title] => window_title = (*title).into(),
                    ["window_size", width, height] => {
                        match (width.parse::<f32>(), height.parse::<f32>()) {
                            (Ok(width), Ok(height)) => {
                                native_options.initial_window_size = Some(vec2(width, height));
                            }
                            _ => eprintln!("Invalid window size: {} {}", width, height),
                        }
                    }
                    ["end_init"] => break,
                    command => {
                        eprintln!("Invalid init command: {:?}", command);
                    }
                }
            }
            Ok(())
        },
    );
    if closed.is_some() {
        return;
    }
    eframe::run_native(
        &window_title,
        Default::default(),
        Box::new(|_| Box::new(server)),
    );
}

fn run_terminal() {}

struct Server {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    mouse_pos: Pos2,
}

impl Server {
    fn new(mut child: Child) -> Self {
        Server {
            stdin: child.stdin.take().unwrap(),
            stdout: BufReader::new(child.stdout.take().unwrap()),
            child,
            mouse_pos: pos2(0.0, 0.0),
        }
    }
    fn handle_io<T>(
        &mut self,
        on_close: impl FnOnce() -> T,
        f: impl FnOnce(&mut Self) -> io::Result<()>,
    ) -> Option<T> {
        if let Err(e) = f(self) {
            if e.kind() == io::ErrorKind::BrokenPipe {
                println!("Child process ended");
                let _ = self.child.kill();
                return Some(on_close());
            }
        }
        None
    }
}

impl eframe::App for Server {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                // Update mouse pos
                if let Some(pos) = ui.input().pointer.hover_pos() {
                    self.mouse_pos = pos;
                }

                // Gather input lines
                let mut input_lines = Vec::new();
                input_lines.push(format!(
                    "window_size {} {}",
                    ui.available_width(),
                    ui.available_height()
                ));
                input_lines.push(format!(
                    "mouse_pos {} {}",
                    self.mouse_pos.x, self.mouse_pos.y
                ));
                input_lines.push("end_input".into());

                // Send input lines
                self.handle_io(
                    || frame.close(),
                    |server| {
                        for line in input_lines {
                            server.stdin.write_all(line.as_bytes())?;
                            server.stdin.write_all(b"\n")?;
                        }
                        Ok(())
                    },
                );

                // Handle frame lines
                self.handle_io(
                    || frame.close(),
                    |server| {
                        for line in server.stdout.by_ref().lines() {
                            let line = line?;
                            let command = line.split_whitespace().collect::<Vec<_>>();
                            match command.as_slice() {
                                ["end_frame"] => break,
                                command => {
                                    eprintln!("Invalid frame command: {:?}", command);
                                }
                            }
                        }
                        Ok(())
                    },
                );
            });
        ctx.request_repaint();
    }
    fn on_close_event(&mut self) -> bool {
        _ = self.child.kill();
        true
    }
}
