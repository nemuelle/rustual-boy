//! Some color utilities that are useful for implementing color transforms,
//! anaglyph modes, etc.

use std::ops::Add;

/// Represents a color
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl From<(f32, f32, f32)> for Color {
    /// Convert a tuple of RGB in the [0, 1] range into a color
    fn from((r, g, b): (f32, f32, f32)) -> Color {
        let r = (r * 255.0) as u8;
        let g = (g * 255.0) as u8;
        let b = (b * 255.0) as u8;
        Color { r: r, g: g, b: b }
    }
}

impl From<u32> for Color {
    /// Convert a packed integer into a color, where the compnents are RGB
    /// from most significant to least significant byte
    fn from(i: u32) -> Color {
        let r = ((i >> 16) & 0xFF) as u8;
        let g = ((i >> 8) & 0xFF) as u8;
        let b = (i & 0xFF) as u8;

        Color { r: r, g: g, b: b }
    }
}

impl Into<u32> for Color {
    /// Convert a color into RGB packed format, where the compnents are RGB
    /// from most significant to least significant byte
    fn into(self) -> u32 {
        let r = self.r as u32;
        let g = self.g as u32;
        let b = self.b as u32;

        (r << 16) | (g << 8) | b
    }
}

impl<'a> Into<u32> for &'a Color {
    /// Convert a color reference into RGB packed format
    fn into(self) -> u32 {
        let r = self.r as u32;
        let g = self.g as u32;
        let b = self.b as u32;

        (r << 16) | (g << 8) | b
    }
}

impl From<(u8, u8, u8)> for Color {
    /// Convert a tuple of u8's (R, G, B) into a color
    fn from((r, g, b): (u8, u8, u8)) -> Color {
        Color { r: r, g: g, b: b }
    }
}

impl Into<(u8, u8, u8)> for Color {
    /// Convert a color into a tuple of u8's (R, G, B)
    fn into(self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}

impl<'a> Into<(u8, u8, u8)> for &'a Color {
    /// Convert a color into a tuple of u8's (R, G, B)
    fn into(self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}

impl Color {
    /// Scale a color by a uniform constant factor
    pub fn scale_by(&self, u: u8) -> Color {
        let s = u as u32;
        let r = self.r as u32;
        let g = self.g as u32;
        let b = self.b as u32;
        Color {
            r: (s * r / 255) as u8,
            g: (s * g / 255) as u8,
            b: (s * b / 255) as u8,
        }
    }
}

impl Add for Color {
    type Output = Color;

    fn add(self, other: Color) -> Color {
        Color {
            r: self.r.saturating_add(other.r),
            g: self.g.saturating_add(other.g),
            b: self.b.saturating_add(other.b),
        }
    }
}
