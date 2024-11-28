use std::fmt;

use slog::warn;

use PointError::FailedToParseFromVec;

use crate::utils::logger::LOG;

pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\n{{\n  x: {}, \n  y: {}, \n  z: {},\n}}",
            self.x, self.y, self.z
        )
    }
}

impl Point {
    pub fn new_from_str_vec(rgba_vec: Vec<&str>) -> Result<Self, PointError> {
        let mut point = Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };

        if rgba_vec.len() != 3 {
            return Err(FailedToParseFromVec(
                "3d point values should be defined by 3 numerical values".to_string(),
            ));
        }
        for (i, maybe_point_dimension) in rgba_vec.iter().enumerate() {
            let point_result = match maybe_point_dimension.parse::<f64>() {
                Ok(point) => Ok(point),
                Err(_) => Err(format!(
                    "failed to parse point. {} is not a number",
                    maybe_point_dimension
                )),
            };

            match point_result {
                Ok(channel_val) => match i {
                    0 => {
                        point.x = channel_val;
                    }
                    1 => {
                        point.y = channel_val;
                    }
                    2 => {
                        point.z = channel_val;
                    }
                    _ => {
                        warn!(LOG, "abnormality while parsing point")
                    }
                },
                Err(error_message) => return Err(FailedToParseFromVec(error_message)),
            }
        }

        return Ok(point);
    }
}

impl Clone for Point {
    fn clone(&self) -> Self {
        Point {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

#[derive(Debug)]
pub enum PointError {
    FailedToParseFromVec(String),
}

impl fmt::Display for PointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FailedToParseFromVec(error_message) => {
                write!(f, "Failed to parse point from &str vec: {}", error_message)
            }
        }
    }
}

impl std::error::Error for PointError {}
