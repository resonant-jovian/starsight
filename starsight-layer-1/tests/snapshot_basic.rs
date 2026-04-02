use starsight_layer_1::backend::DrawBackend;
use starsight_layer_1::backend::skia::raster::SkiaBackend;
use starsight_layer_1::primitives::{color::Color, geom::Rect};

#[test]
fn blue_rect_on_white() {
    let mut backend = SkiaBackend::new(200, 100).unwrap();
    backend.fill(Color::WHITE);
    backend
        .fill_rect(Rect::from_xywh(10.0, 10.0, 180.0, 80.0), Color::BLUE)
        .unwrap();
    let bytes = backend.png_bytes().unwrap();
    insta::assert_binary_snapshot!(".png", bytes);
}
