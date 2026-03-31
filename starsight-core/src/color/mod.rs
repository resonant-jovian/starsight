//!

///
pub mod mapping;
///
pub mod theme;

///
pub struct Color {
    ///
    r: u8,
    ///
    g: u8,
    ///
    b: u8,
}

///
impl Color {
    ///
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    ///
    pub const fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
        }
    }
}
