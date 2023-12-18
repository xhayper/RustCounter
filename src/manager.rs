use base64::{engine::general_purpose, Engine};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::ops::Add;
use std::path::Path;

pub struct ThemeImageData<'a> {
    pub width: u64,
    pub height: u64,
    pub data: &'a str,
}

pub struct SvgGenerateOptions<'a> {
    pub count: u64,
    pub theme: &'a str,
    pub pixelated: bool,
    pub length: u8,
}

pub struct ThemeManager<'a> {
    pub themes: HashMap<String, HashMap<u8, ThemeImageData<'a>>>,
}

fn encode_file(file: &mut File) -> Option<String> {
    let mut data = vec![];
    file.read_to_end(&mut data).ok();

    let mime = tree_magic::from_u8(&data);

    Some(format!(
        "data:{};charset=utf-8;base64,{}",
        mime,
        general_purpose::STANDARD.encode(data)
    ))
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

                let encoded_data = encode_file(&mut File::open(&image_path).unwrap()).unwrap();

                let mut image_data = ThemeImageData {
                    width: 0,
                    height: 0,
                    // TODO: Handle error
                    data: encoded_data.leak(),
                };

                if is_gif {
                    let mut options = gif::DecodeOptions::new();
                    options.set_color_output(gif::ColorOutput::RGBA);

                    // TODO: Maybe we should also handle error here?
                    let mut decoder = options.read_info(File::open(image_path).unwrap()).unwrap();
                    let first_frame = decoder.read_next_frame().unwrap().unwrap();

                    image_data.width = first_frame.width as u64;
                    image_data.height = first_frame.height as u64;
                } else {
                    let decoder = png::Decoder::new(File::open(image_path).unwrap());
                    // TODO: Maybe we should also handle error here?
                    let reader = decoder.read_info().unwrap();
                    let info = reader.info();
                    image_data.width = info.width as u64;
                    image_data.height = info.height as u64;
                }

                self.themes
                    .get_mut(&theme_name)
                    .unwrap()
                    .insert(n, image_data);
            }
        }
    }

    pub fn generate_svg(&self, options: &SvgGenerateOptions) -> Option<String> {
        if self.themes.len() == 0 {
            return None;
        };

        let theme = self.themes.get(options.theme);
        if theme.is_none() {
            return None;
        };

        let mut width = 0;
        let mut height = 0;

        let mut image_parts = String::new();

        // add padding according to options.count.to_string().chars().max(options.length)
        let mut padded = String::new();
        for _ in 0..((options.count.to_string().len() as u8).max(options.length)
            - options.count.to_string().len() as u8)
        {
            padded = padded.add("0");
        }
        padded = padded.add(&options.count.to_string());

        for num in padded.chars() {
            let num = num.to_digit(10).unwrap() as u8;
            let image_data = theme.unwrap().get(&num);

            if image_data.is_none() {
                return None;
            };

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
