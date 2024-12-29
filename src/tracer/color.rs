use std::fmt;

use slog::warn;

use crate::tracer::color::ColorError::FailedToParseFromVec;
use crate::utils::logger::LOG;

#[derive(Debug)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

pub struct NormalizedColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new() -> Color {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }
    }

    pub fn new_from_str_vec(rgba_vec: Vec<&str>) -> Result<Self, ColorError> {
        let mut color = Self::new();

        if rgba_vec.len() != 4 && rgba_vec.len() != 3 {
            return Err(FailedToParseFromVec(
                "colors should be defined by 3 or 4 numerical values".to_string(),
            ));
        }
        for (i, maybe_color_channel) in rgba_vec.iter().enumerate() {
            let color_result = parse_color_channel_from_str(*maybe_color_channel);
            match color_result {
                Ok(channel_val) => match i {
                    0 => {
                        color.r = channel_val;
                    }
                    1 => {
                        color.g = channel_val;
                    }
                    2 => {
                        color.b = channel_val;
                    }
                    3 => {
                        color.a = channel_val;
                    }
                    _ => {
                        warn!(LOG, "abnormality while parsing color")
                    }
                },
                Err(error_message) => return Err(FailedToParseFromVec(error_message)),
            }
        }

        Ok(color)
    }

    pub fn normalize(&self) -> NormalizedColor {
        let normalize = |color: f64| -> u8 {
            (color * 255.0).clamp(0.0, 255.0) as u8
        };

        NormalizedColor {
            r: normalize(self.r),
            g: normalize(self.g),
            b: normalize(self.b),
        }
    }
}

impl Clone for Color {
    fn clone(&self) -> Self {
        Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
}

fn parse_color_channel_from_str(color_str: &str) -> Result<f64, String> {
    match color_str.parse::<f64>() {
        Ok(color) => {
            if color < 0.0 || color > 1.0 {
                return Err(format!(
                    "failed to parse color. value {} not a valid rgba channel value",
                    color_str
                ));
            }
            Ok(color)
        }
        Err(_) => match color_str.parse::<u8>() {
            Ok(color_int) => {
                let color = color_int as f64 / 255.0;
                if color < 0.0 || color > 1.0 {
                    return Err(format!(
                        "failed to parse color. value {} not a valid rgba channel value",
                        color_str
                    ));
                }
                Ok(color)
            }
            Err(_) => Err(format!(
                "failed to parse color. value {} not a valid rgba channel value",
                color_str
            )),
        },
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\n{{\n  r: {}, \n  g: {}, \n  b: {},\n, b: {},\n}}",
            self.r, self.g, self.b, self.a
        )
    }
}

#[derive(Debug)]
pub enum ColorError {
    FailedToParseFromVec(String),
}

impl fmt::Display for ColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FailedToParseFromVec(error_message) => {
                write!(f, "Failed to parse color from &str vec: {}", error_message)
            }
        }
    }
}

impl std::error::Error for ColorError {}
