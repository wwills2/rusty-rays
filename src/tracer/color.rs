use std::fmt;

use slog::warn;

use crate::tracer::color::ColorError::FailedToParseFromVec;
use crate::utils::logger::LOG;

pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    pub fn new_from_str_vec(rgba_vec: Vec<&str>) -> Result<Self, ColorError> {
        let mut color = Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };

        if rgba_vec.len() != 4 {
            return Err(FailedToParseFromVec(
                "colors should be defined by 4 values".to_string(),
            ));
        }
        for (i, maybe_color_channel) in rgba_vec.iter().enumerate() {
            let color_result = parse_color(*maybe_color_channel);
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
                        warn!(LOG, "abnormality while parsing diffuse color")
                    }
                },
                Err(error_message) => return Err(FailedToParseFromVec(error_message)),
            }
        }

        return Ok(color);
    }
}

fn parse_color(color_str: &str) -> Result<f64, String> {
    match color_str.parse::<f64>() {
        Ok(color) => Ok(color),
        Err(_) => match color_str.parse::<u8>() {
            Ok(color_int) => return Ok(color_int as f64 / 255.0),
            Err(_) => Err(format!(
                "failed to parse color. value {} not a value color channel value",
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

impl std::error::Error for crate::tracer::color::ColorError {}
