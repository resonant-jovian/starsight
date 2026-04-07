use starsight_layer_1::primitives::color::Color;
use starsight_layer_3::mark::{LineMark, PointMark};
use starsight_layer_5::Figure;

#[test]
fn snapshot_line_basic() {
    let fig = Figure::new(800, 600).add(LineMark::new(
        vec![0.0, 1.0, 2.0, 3.0],
        vec![0.0, 1.0, 0.5, 2.0],
    ));
    let bytes = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", bytes);
}

#[test]
fn snapshot_line_nan_gaps() {
    let fig = Figure::new(800, 600).add(LineMark::new(
        vec![0.0, 1.0, 2.0, 3.0, 4.0],
        vec![0.0, 1.0, f64::NAN, 0.5, 2.0],
    ));
    let bytes = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", bytes);
}

#[test]
fn snapshot_line_multi() {
    let fig = Figure::new(800, 600)
        .add(LineMark::new(vec![0.0, 1.0, 2.0], vec![0.0, 1.0, 2.0]).color(Color::BLUE))
        .add(LineMark::new(vec![0.0, 1.0, 2.0], vec![2.0, 1.0, 0.0]).color(Color::RED));
    let bytes = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", bytes);
}

#[test]
fn snapshot_scatter_basic() {
    let fig = Figure::new(800, 600).add(PointMark::new(vec![0.5, 1.5, 2.5], vec![1.0, 3.0, 2.0]));
    let bytes = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", bytes);
}

#[test]
fn snapshot_scatter_sizes() {
    let fig = Figure::new(800, 600)
        .add(PointMark::new(vec![1.0, 2.0, 3.0], vec![1.0, 2.0, 3.0]).radius(8.0))
        .add(PointMark::new(vec![1.0, 2.0, 3.0], vec![3.0, 2.0, 1.0]).radius(3.0));
    let bytes = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", bytes);
}
