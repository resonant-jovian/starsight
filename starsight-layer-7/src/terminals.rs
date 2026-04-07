//! Terminal inline export: render a figure straight into the user's terminal.
//!
//! Wraps the layer-1 terminal backends in a one-shot "save to stdout" API.
//!
//! Status: stub. Implementation lands in 0.4.0–0.8.0 behind the `terminal` feature.

// ── TerminalExport ───────────────────────────────────────────────────────────────────────────────
// TODO(0.4.0): pub struct TerminalExport { protocol: TerminalProtocol, dimensions: Option<(u32, u32)> }

// ── TerminalProtocol ─────────────────────────────────────────────────────────────────────────────
// TODO(0.4.0): pub enum TerminalProtocol { Auto, Kitty, Sixel, ITerm2, HalfBlock, Braille }
