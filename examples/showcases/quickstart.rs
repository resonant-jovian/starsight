//! Quickstart — starsight 0.2.0 showcase
//!
//! The absolute simplest path: the `plot!` macro, three values, one save call.
//! This is what gets advertised at the top of `starsight/src/lib.rs`.

use starsight::prelude::*;

fn main() -> Result<()> {
    plot!(
        &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
        &[10.0, 20.0, 15.0, 25.0, 18.0, 30.0, 24.0, 35.0]
    )
    .save("examples/showcases/quickstart.png")
}
