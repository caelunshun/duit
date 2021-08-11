use glam::{const_vec4, vec4, Vec4};
use palette::Srgba;
use serde::Deserialize;

use std::{hash::Hash, str::FromStr};

/// An sRGB color with alpha.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color(Vec4);

impl Color {
    pub const WHITE: Color = Color(const_vec4!([1.0; 4]));
    pub const BLACK: Color = Color(const_vec4!([0.0, 0.0, 0.0, 1.0]));

    /// Creates a color from sRGB bytes.
    pub fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(vec4(
            r as f32 / 255.,
            g as f32 / 255.,
            b as f32 / 255.,
            a as f32 / 255.,
        ))
    }

    /// Creates a color from sRGB floats.
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(vec4(r, g, b, a))
    }

    pub fn red(self) -> f32 {
        self.0.x
    }

    pub fn green(self) -> f32 {
        self.0.y
    }

    pub fn blue(self) -> f32 {
        self.0.z
    }

    pub fn alpha(self) -> f32 {
        self.0.w
    }

    pub fn red8(self) -> u8 {
        (self.red() * 255.0).round() as u8
    }

    pub fn green8(self) -> u8 {
        (self.green() * 255.0).round() as u8
    }

    pub fn blue8(self) -> u8 {
        (self.blue() * 255.0).round() as u8
    }

    pub fn alpha8(self) -> u8 {
        (self.alpha() * 255.0).round() as u8
    }

    pub fn to_rgba8(self) -> [u8; 4] {
        [self.red8(), self.green8(), self.blue8(), self.alpha8()]
    }
}

impl Eq for Color {}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for Color {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let arr: [f32; 4] = self.0.into();
        arr.iter().for_each(|x| x.to_bits().hash(state));
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ColorParseError {
    #[error("expected parenthesis after color type")]
    MissingParenthesis,
    #[error("unknown color type - expected one of `rgb`, `rgba`")]
    UnknownType,
    #[error(transparent)]
    BadValue(std::num::ParseIntError),
    #[error("expected {expected} color components but found {actual}")]
    ComponentMismatch { expected: usize, actual: usize },
}

impl FromStr for Color {
    type Err = ColorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match *s.as_bytes() {
            [b'r', b'g', b'b', b'a', ..] => parse_rgba(&s[4..]),
            [b'r', b'g', b'b', ..] => parse_rgb(&s[3..]),
            _ => Err(ColorParseError::UnknownType),
        }
    }
}

impl From<Color> for Srgba<u8> {
    fn from(c: Color) -> Self {
        Srgba::new(c.red8(), c.green8(), c.blue8(), c.alpha8())
    }
}

fn parse_rgb(s: &str) -> Result<Color, ColorParseError> {
    let components = parse_components(parenthesized(s)?)?;
    if let [r, g, b] = *components.as_slice() {
        Ok(Color::rgba8(r, g, b, u8::MAX))
    } else {
        Err(ColorParseError::ComponentMismatch {
            expected: 3,
            actual: components.len(),
        })
    }
}

fn parse_rgba(s: &str) -> Result<Color, ColorParseError> {
    let components = parse_components(parenthesized(s)?)?;
    if let [r, g, b, a] = *components.as_slice() {
        Ok(Color::rgba8(r, g, b, a))
    } else {
        Err(ColorParseError::ComponentMismatch {
            expected: 3,
            actual: components.len(),
        })
    }
}

fn parenthesized(s: &str) -> Result<&str, ColorParseError> {
    let s = s.trim();
    match (s.chars().next(), s.chars().last()) {
        (Some('('), Some(')')) => Ok(&s[1..s.len() - 1]),
        _ => Err(ColorParseError::MissingParenthesis),
    }
}

fn parse_components(s: &str) -> Result<Vec<u8>, ColorParseError> {
    let mut result = Vec::new();
    for part in s.split(',') {
        let part = part.trim();
        let component = u8::from_str(part).map_err(ColorParseError::BadValue)?;
        result.push(component);
    }
    Ok(result)
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Color::from_str(&s).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_rgb_from_str() {
        let color = Color::from_str("rgb(0, 0, 0)").unwrap();
        assert_eq!(color, Color::rgba8(0, 0, 0, u8::MAX));
    }

    #[test]
    fn color_rgba_from_str() {
        let color = Color::from_str("rgba ( 10, 11, 12, 90 )").unwrap();
        assert_eq!(color, Color::rgba8(10, 11, 12, 90));
    }
}
