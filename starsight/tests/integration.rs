use starsight::prelude::*;

#[test]
fn quickstart_produces_png() {
    let fig = plot!(&[1.0_f64, 2.0, 3.0], &[4.0_f64, 5.0, 6.0]);
    let bytes = fig.render_png().unwrap();
    // PNG magic bytes
    assert_eq!(&bytes[..4], &[0x89, 0x50, 0x4E, 0x47]);
    assert!(bytes.len() > 100, "PNG should have meaningful content");
}

#[test]
fn save_writes_file() {
    let fig = plot!(&[1.0_f64, 2.0, 3.0], &[4.0_f64, 5.0, 6.0]);
    let tmp = std::env::temp_dir().join("starsight_integration_test.png");
    fig.save(&tmp).unwrap();
    assert!(tmp.exists());
    assert!(std::fs::metadata(&tmp).unwrap().len() > 0);
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn figure_builder_api() {
    let fig = Figure::new(400, 300)
        .title("Test")
        .x_label("X")
        .y_label("Y")
        .add(LineMark::new(vec![0.0, 1.0, 2.0], vec![0.0, 1.0, 0.5]));
    let bytes = fig.render_png().unwrap();
    assert_eq!(&bytes[..4], &[0x89, 0x50, 0x4E, 0x47]);
}
