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

// TODO: Throw error instead of using Options by using Result<>
pub fn svg_to_png(data: &[u8], pixelated: bool) -> Option<Vec<u8>> {
    let opt = usvg::Options {
        image_rendering: if pixelated {
            usvg::ImageRendering::OptimizeSpeed
        } else {
            usvg::ImageRendering::OptimizeQuality
        },
        ..Default::default()
    };

    let usvg_tree = match usvg::Tree::from_data(data, &opt) {
        Ok(tree) => tree,
        Err(_) => return None,
    };

    let pixmap_size = usvg_tree.size.to_int_size();

    let resvg_tree = resvg::Tree::from_usvg(&usvg_tree);

    let mut pixmap = match tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()) {
        Some(pixmap) => pixmap,
        None => return None,
    };

    resvg_tree.render(tiny_skia::Transform::default(), &mut pixmap.as_mut());

    let result = pixmap.encode_png();

    if result.is_err() {
        eprintln!("Failed to encode PNG: {:?}", result.err());
        return None;
    };

    Some(result.unwrap())
}
