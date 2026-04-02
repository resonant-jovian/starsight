use crate::primitives::geom::Rect;

// -------------------------------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    pub const fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
        }
    }
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255 };
    pub const RED: Self = Self { r: 255, g: 0, b: 0 };
    pub const GREEN: Self = Self { r: 0, g: 255, b: 0 };
    pub const BLUE: Self = Self { r: 0, g: 0, b: 255 };

    pub const fn to_f32(self) -> (f32, f32, f32) {
        (self.r as f32 / 255.0, self.g as f32 / 255.0, self.b as f32 / 255.0)
    }

    pub fn from_f32(r: f32, g: f32, b: f32) -> Self {
        Self {
            r: (r.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            g: (g.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            b: (b.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        }
    }

    pub fn to_tiny_skia(self) -> tiny_skia::Color {
        tiny_skia::Color::from_rgba8(self.r, self.g, self.b, 255)
    }

    pub fn with_alpha(self, a: u8) -> ColorAlpha {
        ColorAlpha { r: self.r, g: self.g, b: self.b, a }
    }
    pub fn from_css_hex(s: &str) -> Option<Self> {
        let hex = s.strip_prefix('#').unwrap_or(s);
        match hex.len() {
            6 => {
                let val = u32::from_str_radix(hex, 16).ok()?;
                Some(Self::from_hex(val))
            }
            3 => {
                let mut chars = hex.chars();
                let r = chars.next().and_then(|c| c.to_digit(16))? as u8;
                let g = chars.next().and_then(|c| c.to_digit(16))? as u8;
                let b = chars.next().and_then(|c| c.to_digit(16))? as u8;
                Some(Self { r: r << 4 | r, g: g << 4 | g, b: b << 4 | b })
            }
            _ => None,
        }
    }

    pub fn to_css_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}
impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}
// -------------------------------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorAlpha {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8 }
impl ColorAlpha {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    pub const fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
            a: ((hex >> 24) & 0xFF) as u8,
        }
    }
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };
    pub const RED: Self = Self { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Self = Self { r: 0, g: 255, b: 0, a: 255 };
    pub const BLUE: Self = Self { r: 0, g: 0, b: 255, a: 255 };

    pub const fn to_f32(self) -> (f32, f32, f32) {
        (self.r as f32 / 255.0, self.g as f32 / 255.0, self.b as f32 / 255.0)
    }

    pub fn from_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: (r.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            g: (g.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            b: (b.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            a: (a.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        }
    }

    pub fn to_tiny_skia(self) -> tiny_skia::Color {
        tiny_skia::Color::from_rgba8(self.r, self.g, self.b, self.a)
    }

    pub fn without_alpha(self) -> Color {
        Color { r: self.r, g: self.g, b: self.b }
    }
}
// -------------------------------------------------------------------------------------------------
