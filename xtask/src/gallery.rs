//! Gallery command: render every example to `target/gallery/<name>.png`.
//!
//! Used by CI's `gallery.yml` workflow and by humans before a release to
//! visually verify nothing has regressed.
//!
//! Status: stub. Implementation lands in 0.2.0.

// ── run ──────────────────────────────────────────────────────────────────────────────────────────
// TODO(0.2.0): pub fn run() -> std::io::Result<()> {
//     for example in walk_examples() {
//         let output = render_example(example)?;
//         write_to(format!("target/gallery/{}.png", example.name), output)?;
//     }
//     Ok(())
// }
