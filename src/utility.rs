use base64::engine::general_purpose;
use base64::Engine;
use resvg::usvg::TreeParsing;
use resvg::{tiny_skia, usvg};

pub fn file_to_base64(data: &[u8]) -> String {
    let mime = tree_magic::from_u8(data);

    format!(
        "data:{};charset=utf-8;base64,{}",
        mime,
        general_purpose::STANDARD.encode(data)
    )
}

pub fn svg_to_png(data: &[u8], pixelated: bool) -> Vec<u8> {
    let opt = usvg::Options {
        image_rendering: if pixelated {
            usvg::ImageRendering::OptimizeSpeed
        } else {
            usvg::ImageRendering::OptimizeQuality
        },
        ..Default::default()
    };

    let usvg_tree = usvg::Tree::from_data(data, &opt).unwrap();
    let pixmap_size = usvg_tree.size.to_int_size();

    let resvg_tree = resvg::Tree::from_usvg(&usvg_tree);

    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg_tree.render(tiny_skia::Transform::default(), &mut pixmap.as_mut());

    pixmap.encode_png().unwrap()
}
