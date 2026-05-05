//! Eclipse mark — E03 total eclipse, monochrome.
//!
//! 100×100 viewBox. Rendered as SVG inner (no `<svg>` wrapper) so it can be
//! placed inside the wordmark or rasterized into the hero PNG.

use super::palette::Palette;

const RAY_COUNT: usize = 72;
const RAY_INNER: f64 = 33.0; // start radius
const RAY_OUTER: f64 = 92.0; // base end radius (jittered)
const DISC_R: f64 = 18.0; // central black disc

pub fn svg_inner(p: &Palette) -> String {
    let mut out = String::new();

    // Soft outer glow ring (replaces the orange radial gradient)
    out.push_str(&format!(
        r#"    <circle cx="50" cy="50" r="50" fill="none" stroke="{c}" stroke-width="0.5" opacity="0.6"/>
"#,
        c = p.border
    ));

    // 72 corona rays — tiny, deterministic length jitter for organic feel.
    out.push_str(&format!(
        r#"    <g stroke="{c}" stroke-width="0.4" stroke-linecap="round" opacity="0.55">
"#,
        c = p.muted
    ));
    for i in 0..RAY_COUNT {
        let theta = (i as f64) * std::f64::consts::TAU / (RAY_COUNT as f64);
        // Cheap deterministic jitter: 4-period sinusoid.
        let jitter = ((i as f64) * 1.319).sin() * 4.5;
        let r1 = RAY_INNER;
        let r2 = RAY_OUTER + jitter;
        let x1 = 50.0 + r1 * theta.cos();
        let y1 = 50.0 + r1 * theta.sin();
        let x2 = 50.0 + r2 * theta.cos();
        let y2 = 50.0 + r2 * theta.sin();
        out.push_str(&format!(
            r#"      <line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}"/>
"#
        ));
    }
    out.push_str("    </g>\n");

    // Central disc
    out.push_str(&format!(
        r#"    <circle cx="50" cy="50" r="{r}" fill="{c}"/>
"#,
        r = DISC_R,
        c = p.text
    ));

    out
}
