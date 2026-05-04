//! Radar / spider chart — starsight 0.3.0 polar showcase #31.
//!
//! Three players' competence profile across 8 dimensions (passing,
//! shooting, dribbling, defense, stamina, speed, vision, leadership) on a
//! shared polar grid. Each player overlays as a translucent-fill polyline
//! so all three can be compared at once. Pair with
//! `Figure::polar_axes(theta, r)` and `polar_angular_categorical(8)` so
//! every dimension lands at the band-center angle.

use starsight::axes::Axis;
use starsight::prelude::*;
use starsight::ticks::polar_ticks_categorical;

fn main() -> Result<()> {
    let dims: Vec<String> = [
        "pass", "shoot", "drib", "def", "stam", "speed", "vis", "lead",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect();
    let thetas: Vec<f64> = (0..dims.len() as u32).map(f64::from).collect();
    let player_a = vec![85.0, 70.0, 90.0, 50.0, 80.0, 78.0, 92.0, 75.0];
    let player_b = vec![60.0, 95.0, 75.0, 55.0, 85.0, 88.0, 70.0, 50.0];
    let player_c = vec![70.0, 65.0, 70.0, 92.0, 75.0, 65.0, 80.0, 88.0];

    let mut theta_axis = Axis::polar_angular_categorical(dims.len());
    let (theta_pos, theta_lab) = polar_ticks_categorical(&dims);
    theta_axis.tick_positions = theta_pos;
    theta_axis.tick_labels = theta_lab;
    let mut r_axis = Axis::polar_radial(0.0, 100.0);
    r_axis.tick_positions = vec![25.0, 50.0, 75.0, 100.0];
    r_axis.tick_labels = vec!["25".into(), "50".into(), "75".into(), "100".into()];

    Figure::new(800, 800)
        .title("Player profiles — 8 dimensions")
        .polar_axes(theta_axis, r_axis)
        .add(
            RadarMark::new(thetas.clone(), player_a)
                .color(Color::from_hex(0x004C_72B0))
                .label("Player A"),
        )
        .add(
            RadarMark::new(thetas.clone(), player_b)
                .color(Color::from_hex(0x00C4_4E52))
                .label("Player B"),
        )
        .add(
            RadarMark::new(thetas, player_c)
                .color(Color::from_hex(0x0055_A868))
                .label("Player C"),
        )
        .save("examples/scientific/radar_spider.png")
}
