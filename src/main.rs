mod frame;
mod input;

use std::{
    io::{self, BufRead, BufReader, Read},
    process::{exit, Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::Arc,
    time::Instant,
};

use clap::Parser;
use eframe::{
    egui::*,
    epaint::{
        ahash::{HashMap, HashSet},
        mutex::Mutex,
    },
};
use once_cell::sync::Lazy;

static RUNNING_CHILD: Lazy<Mutex<Option<Arc<Mutex<Child>>>>> = Lazy::new(|| Mutex::new(None));

fn main() {
    let _ = ctrlc::set_handler(|| {
        if let Some(child) = RUNNING_CHILD.lock().take() {
            let _ = child.lock().kill();
        }
        exit(0);
    });
    let app = App::parse();
    run_command(app.command, app.args);
}

#[derive(Parser)]
#[command(author, version, about = "print_game: a game engine for any language")]
struct App {
    #[arg(help = "the command to invoke the backend")]
    command: String,
    #[arg(help = "the arguments to pass to the backend command")]
    args: Vec<String>,
}

struct Server {
    child: Arc<Mutex<Child>>,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    color: Color32,
    clear_color: Color32,
    font_size: f32,
    anchor: Align2,
    show_cursor: bool,
    keys_down: HashSet<Key>,
    last_instant: Instant,
    last_window_size: Vec2,
    textures: HashMap<String, Option<TextureHandle>>,
}

impl Server {
    fn new(child: Arc<Mutex<Child>>) -> Self {
        let stdin = child.lock().stdin.take().unwrap();
        let stdout = BufReader::new(child.lock().stdout.take().unwrap());
        Server {
            stdin,
            stdout,
            child,
            clear_color: Color32::BLACK,
            color: Color32::WHITE,
            font_size: 16.0,
            anchor: Align2::LEFT_TOP,
            show_cursor: true,
            keys_down: HashSet::default(),
            last_instant: Instant::now(),
            last_window_size: Vec2::ZERO,
            textures: HashMap::default(),
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
                let _ = self.child.lock().kill();
                return Some(on_close());
            }
        }
        None
    }
}

fn parse_line(line: &str) -> Option<(&str, &str, Vec<&str>)> {
    let (command, args) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
    if let Some(command) = command.strip_prefix('/') {
        let split_args: Vec<&str> = args.split_whitespace().collect();
        Some((command, args, split_args))
    } else {
        if !line.trim().is_empty() {
            println!("{line}");
        }
        None
    }
}

fn run_command(command: String, args: Vec<String>) {
    match run_command_impl(&command, args) {
        Ok(()) => (),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            eprintln!("Command not found: {}", command)
        }
        Err(e) => {
            eprintln!("Error running command: {}", e)
        }
    }
}

fn run_command_impl(command: &str, args: Vec<String>) -> io::Result<()> {
    // Start the child process
    let child = Command::new(command)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;
    let child = Arc::new(Mutex::new(child));
    *RUNNING_CHILD.lock() = Some(Arc::clone(&child));
    let mut server = Server::new(Arc::clone(&child));
    // Collect window options
    let mut native_options = eframe::NativeOptions::default();
    let mut window_title = "My Game".to_owned();
    let closed = server.handle_io(
        || (),
        |server| {
            for line in server.stdout.by_ref().lines() {
                let line = line?;
                let Some((command, args, split_args)) = parse_line(&line) else {
                    continue;
                };
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
                    ("vsync", &[on]) => native_options.vsync = on == "true",
                    ("end_init", _) => break,
                    _ => {
                        eprintln!("Invalid init command: {command} {args}");
                    }
                }
            }
            Ok(())
        },
    );
    // Return if the child process has already exited
    if closed.is_some() {
        return Ok(());
    }
    // Run the game
    eframe::run_native(
        &window_title,
        Default::default(),
        Box::new(|cc| {
            cc.egui_ctx.tex_manager().write().alloc(
                "white".into(),
                ImageData::Color(ColorImage {
                    size: [1, 1],
                    pixels: vec![Color32::WHITE],
                }),
                TextureOptions::default(),
            );
            Box::new(server)
        }),
    );
    Ok(())
}

impl eframe::App for Server {
    fn clear_color(&self, _visuals: &Visuals) -> Rgba {
        Rgba::from_rgba_premultiplied(
            self.clear_color.r() as f32 / 255.0,
            self.clear_color.g() as f32 / 255.0,
            self.clear_color.b() as f32 / 255.0,
            self.clear_color.b() as f32 / 255.0,
        )
    }
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        let resp = CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                self.handle_io(|| frame.close(), |server| server.send_input_lines(ui));
                self.handle_io(|| frame.close(), |server| server.handle_frame_lines(ui));
            })
            .response;
        resp.on_hover_cursor(if self.show_cursor {
            CursorIcon::Default
        } else {
            CursorIcon::None
        });
        ctx.request_repaint();
    }
    fn on_close_event(&mut self) -> bool {
        _ = self.child.lock().kill();
        true
    }
}
