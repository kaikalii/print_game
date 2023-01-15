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
                let (command, args) = line.split_once(' ').unwrap_or((&line, ""));
                let split_args: Vec<&str> = args.split_whitespace().collect();
                match (command, split_args.as_slice()) {
                    ("title", _) => window_title = args.into(),
                    ("window_size", [width, height]) => {
                        match (width.parse::<f32>(), height.parse::<f32>()) {
                            (Ok(width), Ok(height)) => {
                                native_options.initial_window_size = Some(vec2(width, height));
                            }
                            _ => eprintln!("Invalid window size: {} {}", width, height),
                        }
                    }
                    ("end_init", _) => break,
                    _ => {
                        eprintln!("Invalid init command: {command} {args}");
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
    color: Color32,
}

impl Server {
    fn new(mut child: Child) -> Self {
        Server {
            stdin: child.stdin.take().unwrap(),
            stdout: BufReader::new(child.stdout.take().unwrap()),
            child,
            color: Color32::WHITE,
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
                // Gather input lines
                let mut input_lines = Vec::new();
                // Window size
                input_lines.push(format!(
                    "window_size {} {}",
                    ui.available_width(),
                    ui.available_height()
                ));
                // Events
                for event in &ui.input().events {
                    match event {
                        Event::Key {
                            key,
                            pressed,
                            modifiers,
                        } => {
                            input_lines.push(format!(
                                "key {key:?} {pressed} {} {} {}",
                                modifiers.ctrl, modifiers.shift, modifiers.alt
                            ));
                        }
                        Event::PointerMoved(pos) => {
                            input_lines.push(format!("mouse_moved {} {}", pos.x, pos.y));
                        }
                        _ => {}
                    }
                }
                // End
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
                            let (command, args) = line.split_once(' ').unwrap_or((&line, ""));
                            let split_args: Vec<&str> = args.split_whitespace().collect();
                            match (command, split_args.as_slice()) {
                                ("clear", _) => {
                                    ui.painter().rect_filled(
                                        Rect::from_min_size(Pos2::ZERO, ui.available_size()),
                                        Rounding::default(),
                                        server.color,
                                    );
                                }
                                ("color", [r, g, b, a]) => {
                                    let [r, g, b, a] = parse_floats([r, g, b, a], 1.0);
                                    server.color = Rgba::from_rgba_premultiplied(r, g, b, a).into();
                                }
                                ("color", [r, g, b]) => {
                                    let [r, g, b] = parse_floats([r, g, b], 1.0);
                                    server.color =
                                        Rgba::from_rgba_premultiplied(r, g, b, 1.0).into();
                                }
                                ("color", _) => {
                                    if let Ok(color) = csscolorparser::parse(args) {
                                        let (r, g, b, a) = color.to_linear_rgba_u8();
                                        server.color = Color32::from_rgba_premultiplied(r, g, b, a);
                                    } else {
                                        eprintln!("Invalid color: {}", args);
                                    }
                                }
                                ("rectangle", [x, y, width, height]) => {
                                    let [x, y, width, height] =
                                        parse_floats([x, y, width, height], 0.0);
                                    ui.painter().rect_filled(
                                        Rect::from_min_size(pos2(x, y), vec2(width, height)),
                                        Rounding::default(),
                                        server.color,
                                    );
                                }
                                ("circle", [x, y, radius]) => {
                                    let [x, y, radius] = parse_floats([x, y, radius], 0.0);
                                    ui.painter().circle_filled(pos2(x, y), radius, server.color);
                                }
                                ("end_frame", _) => break,
                                _ => {
                                    eprintln!("Invalid frame command: {command} {args}");
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

fn parse_floats<const N: usize>(args: [&&str; N], default: f32) -> [f32; N] {
    args.map(|s| s.parse().unwrap_or(default))
}