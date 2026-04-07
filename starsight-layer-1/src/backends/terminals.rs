//! Terminal backends: render charts inline in modern terminals.
//!
//! Status: stub. Implementations land in 0.4.0–0.8.0 behind the `terminal` feature.
//! Each variant targets a specific protocol so users can pick the highest-fidelity
//! one their terminal supports.

// ── KittyBackend ─────────────────────────────────────────────────────────────────────────────────
// TODO(0.4.0): pub struct KittyBackend { pixmap: Pixmap }
//              -- Kitty graphics protocol: PNG payload + escape sequence
//              -- highest fidelity, supports kitty + ghostty + wezterm

// ── SixelBackend ─────────────────────────────────────────────────────────────────────────────────
// TODO(0.4.0): pub struct SixelBackend { pixmap: Pixmap }
//              -- DEC Sixel protocol: 6-pixel-tall column stripes
//              -- supports xterm, mintty, mlterm

// ── ITerm2Backend ────────────────────────────────────────────────────────────────────────────────
// TODO(0.4.0): pub struct ITerm2Backend { pixmap: Pixmap }
//              -- iTerm2 inline image protocol: base64-encoded PNG

// ── HalfBlockBackend ─────────────────────────────────────────────────────────────────────────────
// TODO(0.6.0): pub struct HalfBlockBackend { ... }
//              -- Unicode upper-half-block + bg/fg color: 2 pixels per character cell

// ── BrailleBackend ───────────────────────────────────────────────────────────────────────────────
// TODO(0.8.0): pub struct BrailleBackend { ... }
//              -- Unicode braille dots: 2×4 pixels per character cell, monochrome
