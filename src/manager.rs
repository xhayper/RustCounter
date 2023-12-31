use crate::utility;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::ops::Add;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub struct ThemeImageData<'a> {
    pub width: u64,
    pub height: u64,
    pub data: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct SvgGenerateOptions<'a> {
    pub count: u64,
    pub theme: &'a str,
    pub pixelated: bool,
    pub length: u8,
}

#[derive(Debug)]
pub struct ThemeManager<'a> {
    pub themes: HashMap<String, HashMap<u8, ThemeImageData<'a>>>,
}

impl ThemeManager<'_> {
    pub fn new() -> ThemeManager<'static> {
        ThemeManager {
            themes: HashMap::new(),
        }
    }

    pub fn load(&mut self) {
        if !Path::new("./static/assets/theme").exists() {
            return;
        };

        let read_result = fs::read_dir(Path::new("./static/assets/theme"));
        if read_result.is_err() {
            return;
        };

        let read_result = read_result.unwrap();

        for entry in read_result {
            if entry.is_err() {
                continue;
            };

            let entry = entry.unwrap();

            if !entry.file_type().unwrap().is_dir() {
                continue;
            };

            let theme_name = entry.file_name().into_string().unwrap();

            self.themes.insert(theme_name.clone(), HashMap::new());

            for n in 0..10 {
                let image_path = format!("./static/assets/theme/{}/{}", theme_name, n);

                let is_gif = Path::new(&format!("{}.gif", image_path.clone())).exists();
                let file_extension = if is_gif { "gif" } else { "png" };

                let image_path = format!("{}.{}", image_path, file_extension);
                let image_path = Path::new("").join(image_path);

                if !image_path.exists() {
                    continue;
                };

                let mut image_data = vec![];

                let file = File::open(&image_path);
                if file.is_err() {
                    eprintln!("Failed to open file {}", image_path.display());
                    continue;
                }

                let mut file = file.unwrap();
                if file.read_to_end(&mut image_data).is_err() {
                    eprintln!("Failed to read file {}", image_path.display());
                    continue;
                }

                let encoded_data = utility::file_to_base64(&image_data);

                let mut image_data = ThemeImageData {
                    width: 0,
                    height: 0,
                    // How do we resolve this??
                    data: encoded_data.leak(),
                };

                if is_gif {
                    let mut options = gif::DecodeOptions::new();
                    options.set_color_output(gif::ColorOutput::RGBA);

                    let decoder = options.read_info(File::open(image_path.clone()).unwrap());
                    if decoder.is_err() {
                        eprintln!("{} | Failed to decode {}", theme_name, image_path.display());
                        continue;
                    }

                    let mut decoder = decoder.unwrap();
                    let first_frame = decoder.read_next_frame();
                    if first_frame.is_err() {
                        eprintln!("{} | Failed to decode {}", theme_name, image_path.display());
                        continue;
                    }

                    let first_frame = first_frame.unwrap();
                    if first_frame.is_none() {
                        eprintln!("{} | {} has no frames", theme_name, image_path.display());
                        continue;
                    }

                    let first_frame = first_frame.unwrap();
                    image_data.width = first_frame.width as u64;
                    image_data.height = first_frame.height as u64;
                } else {
                    let decoder = png::Decoder::new(File::open(image_path.clone()).unwrap());
                    let reader = decoder.read_info();
                    if reader.is_err() {
                        eprintln!("{} | Can't read {}", theme_name, image_path.display());
                        continue;
                    }

                    let reader = reader.unwrap();
                    let info = reader.info();
                    image_data.width = info.width as u64;
                    image_data.height = info.height as u64;
                }

                // This will never be None
                self.themes
                    .get_mut(&theme_name)
                    .unwrap()
                    .insert(n, image_data);
            }
        }
    }

    // TODO: Throw error instead of using Options by using Result<>
    pub fn generate_svg(&self, options: &SvgGenerateOptions) -> Option<String> {
        if self.themes.is_empty() {
            return None;
        };

        let theme = self.themes.get(options.theme);
        theme?;

        let theme = theme.unwrap();

        let mut width = 0;
        let mut height = 0;

        let mut image_parts = String::new();

        let mut padded = String::new();
        for _ in 0..((options.count.to_string().len() as u8).max(options.length)
            - options.count.to_string().len() as u8)
        {
            padded = padded.add("0");
        }
        padded = padded.add(&options.count.to_string());

        for num in padded.chars() {
            let num = num.to_digit(10).unwrap_or(0) as u8;
            let image_data = theme.get(&num);
            image_data?;

            let image_data = image_data.unwrap();
            let image_width = image_data.width;
            let image_height = image_data.height;
            let data = image_data.data;

            image_parts = image_parts.add(&format!("<image x=\"{width}\" y=\"0\" width=\"{image_width}\" height=\"{image_height}\" href=\"{data}\" />\n"));

            width += image_width;
            height = image_height.max(height);
        }

        let mut svg = String::new()
        .add("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n")
        .add(&format!("<svg width=\"{width}\" height=\"{height}\" version=\"1.1\" xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\""));

        if options.pixelated {
            svg = svg.clone().add(" style='image-rendering: pixelated;'");
        }

        svg = svg
            .clone()
            .add(">\n")
            .add(&format!("<title>{}</title>\n", options.count))
            .add(&format!("<g>{image_parts}</g>\n"))
            .add("</svg>");

        Some(svg)
    }
}
