use std::{
    io::{self, BufRead, BufReader, Read, Write},
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
        Vertex, WHITE_UV,
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
    let child = Command::new(command)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;
    let child = Arc::new(Mutex::new(child));
    *RUNNING_CHILD.lock() = Some(Arc::clone(&child));
    let mut server = Server::new(Arc::clone(&child));
    let mut native_options = eframe::NativeOptions::default();
    let mut window_title = "My Game".to_owned();
    let closed = server.handle_io(
        || (),
        |server| {
            for line in server.stdout.by_ref().lines() {
                let line = line?;
                let (command, args) = line.split_once(char::is_whitespace).unwrap_or((&line, ""));
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
    if closed.is_some() {
        return Ok(());
    }
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
        let now = Instant::now();
        let dt = (now - self.last_instant).as_secs_f32();
        self.last_instant = now;

        let resp = CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
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

impl Server {
    fn handle_frame_lines(&mut self, ui: &mut Ui) -> io::Result<()> {
        let mut line = String::new();
        loop {
            line.clear();
            self.stdout.read_line(&mut line)?;
            if let Some(line) = line.strip_prefix('/') {
                println!("{line}");
                continue;
            }
            let (command, args) = line.split_once(char::is_whitespace).unwrap_or((&line, ""));
            if command.is_empty() {
                continue;
            }
            let split_args: Vec<&str> = args.split_whitespace().collect();
            match (command, split_args.as_slice()) {
                ("clear", _) => self.clear_color = self.color,
                ("color", [r, g, b, a]) => {
                    let [r, g, b, a] =
                        parse_floats([r, g, b, a], 1.0).map(|c| (c * 255.0).round() as u8);
                    self.color = Color32::from_rgba_premultiplied(r, g, b, a);
                }
                ("color", [r, g, b]) => {
                    let [r, g, b] = parse_floats([r, g, b], 1.0).map(|c| (c * 255.0).round() as u8);
                    self.color = Color32::from_rgba_premultiplied(r, g, b, 255);
                }
                ("color", _) => {
                    if let Ok(color) = csscolorparser::parse(args) {
                        let (r, g, b, a) = color.to_linear_rgba_u8();
                        self.color = Color32::from_rgba_premultiplied(r, g, b, a);
                    } else {
                        eprintln!("Invalid color: {}", args);
                    }
                }
                ("rectangle", [x, y, width, height]) => {
                    let [x, y, width, height] = parse_floats([x, y, width, height], 0.0);
                    let rect = Rect::from_min_size(pos2(x, y), vec2(width, height));
                    let rect = self.anchor.anchor_rect(rect);
                    ui.painter()
                        .rect_filled(rect, Rounding::default(), self.color);
                }
                ("circle", [x, y, radius]) => {
                    let [x, y, radius] = parse_floats([x, y, radius], 0.0);
                    let rect = Rect::from_center_size(
                        pos2(x + radius, y + radius),
                        Vec2::splat(radius * 2.0),
                    );
                    let rect = self.anchor.anchor_rect(rect);
                    ui.painter()
                        .circle_filled(rect.center(), radius, self.color);
                }
                ("font_size", [size]) => {
                    self.font_size = size.parse().unwrap_or(16.0);
                }
                ("text", [x, y, ..]) => {
                    let text = args.splitn(3, char::is_whitespace).nth(2).unwrap_or("");
                    let [x, y] = parse_floats([x, y], 0.0);
                    let font_id = FontId {
                        size: self.font_size,
                        family: FontFamily::Proportional,
                    };
                    ui.painter()
                        .text(pos2(x, y), self.anchor, text, font_id, self.color);
                }
                ("anchor", [h, v]) => {
                    let h = match *h {
                        "left" => Align::LEFT,
                        "center" => Align::Center,
                        "right" => Align::RIGHT,
                        _ => {
                            eprintln!("Invalid anchor: {v}");
                            self.anchor[0]
                        }
                    };
                    let v = match *v {
                        "top" => Align::TOP,
                        "center" => Align::Center,
                        "bottom" => Align::BOTTOM,
                        _ => {
                            eprintln!("Invalid anchor: {v}");
                            self.anchor[1]
                        }
                    };
                    self.anchor = Align2([h, v]);
                }
                ("anchor", ["center"]) => self.anchor = Align2::CENTER_CENTER,
                ("anchor", _) => {
                    eprintln!("Invalid anchor: {args}");
                }
                ("polygon", points) => {
                    let mut vertices = Vec::new();
                    for point in points.chunks_exact(2) {
                        let [x, y] = parse_floats([&point[0], &point[1]], 0.0);
                        vertices.push(Vertex {
                            pos: pos2(x, y),
                            uv: WHITE_UV,
                            color: self.color,
                        });
                    }
                    let mut indices: Vec<u32> = (0..vertices.len() as u32).collect();
                    indices.push(0);
                    ui.painter().add(Mesh {
                        vertices,
                        indices,
                        texture_id: TextureId::Managed(1),
                    });
                }
                ("show_cursor", [show]) => self.show_cursor = *show != "false",
                ("image", [path, rest @ ..]) => {
                    let Some(texture) = self.texture(path, ui.ctx()) else {
                        continue;
                    };
                    let size = texture.size_vec2();
                    let mut values = [0.0, 0.0, size.x, size.y, 0.0, 0.0, 1.0, 1.0];
                    for (val, s) in values.iter_mut().zip(rest) {
                        if let Ok(v) = s.parse() {
                            *val = v;
                        }
                    }
                    let [x, y, width, height, uv_x, uv_y, uv_width, uv_height] = values;
                    let rect = Rect::from_min_size(pos2(x, y), vec2(width, height));
                    let rect = self.anchor.anchor_rect(rect);
                    let uv_min = pos2(uv_x, uv_y);
                    let uv_size = vec2(uv_width, uv_height);
                    let uv = Rect::from_min_max(uv_min, uv_min + uv_size);
                    ui.painter().image(texture.id(), rect, uv, Color32::WHITE);
                }
                ("get_texture_size", [path]) => {
                    let Some(texture) = self.texture(path, ui.ctx()) else {
                        continue;
                    };
                    let line = format!("{} {}\n", texture.size()[0], texture.size()[1]);
                    self.stdin.write_all(line.as_bytes())?;
                }
                ("end_frame", _) => break,
                _ => {
                    eprintln!("Invalid frame command: {command} {args}");
                }
            }
        }
        Ok(())
    }
    fn texture(&mut self, path: &str, ctx: &Context) -> Option<TextureHandle> {
        if let Some(handle) = self.textures.get(path) {
            return handle.clone();
        }
        let image = match image::open(path) {
            Ok(image) => image,
            Err(e) => {
                println!("Error loading texture {path:?}: {e}");
                self.textures.insert(path.into(), None);
                return None;
            }
        };
        let image = image.into_rgba8();
        let image = ColorImage {
            size: [image.width() as usize, image.height() as usize],
            pixels: image
                .pixels()
                .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
                .collect(),
        };
        let handle = ctx.load_texture(path, image, TextureOptions::default());
        println!("loaded texture: {path}");
        self.textures.insert(path.into(), Some(handle.clone()));
        Some(handle)
    }
}

fn parse_floats<const N: usize>(args: [&&str; N], default: f32) -> [f32; N] {
    args.map(|s| s.parse().unwrap_or(default))
}
