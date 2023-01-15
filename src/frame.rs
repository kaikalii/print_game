use std::io::{self, BufRead, Write};

use eframe::{
    egui::*,
    epaint::{Vertex, WHITE_UV},
};

use crate::{parse_line, Server};

impl Server {
    pub fn handle_frame_lines(&mut self, ui: &mut Ui) -> io::Result<()> {
        let mut line = String::new();
        loop {
            line.clear();
            self.stdout.read_line(&mut line)?;
            line = line.trim().into();
            let Some((command, args, split_args)) = parse_line(&line) else {
                continue;
            };
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
