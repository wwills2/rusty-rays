use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

use crate::utils::logger::{LOG, warn};
use CoordsError::FailedToParseFromVec;

#[derive(PartialEq, Debug)]
pub struct Coords {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug)]
pub enum CoordsError {
    FailedToParseFromVec(String),
}

impl fmt::Display for CoordsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FailedToParseFromVec(error_message) => {
                write!(f, "Failed to parse coords from &str vec: {}", error_message)
            }
        }
    }
}

impl std::error::Error for CoordsError {}

impl Coords {
    pub fn new() -> Self {
        Coords {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn new_from_str_vec(xyz_vec: Vec<&str>) -> Result<Self, CoordsError> {
        let mut coords = Self::new();

        if xyz_vec.len() != 3 {
            return Err(FailedToParseFromVec(
                "3d coordinate values should be defined by 3 numerical values".to_string(),
            ));
        }
        for (i, maybe_coords_dimension) in xyz_vec.iter().enumerate() {
            let value_result = match maybe_coords_dimension.parse::<f64>() {
                Ok(value) => Ok(value),
                Err(_) => Err(format!(
                    "failed to parse coords. {} is not a number",
                    maybe_coords_dimension
                )),
            };

            match value_result {
                Ok(channel_val) => match i {
                    0 => {
                        coords.x = channel_val;
                    }
                    1 => {
                        coords.y = channel_val;
                    }
                    2 => {
                        coords.z = channel_val;
                    }
                    _ => {
                        warn!(LOG, "abnormality while parsing coords")
                    }
                },
                Err(error_message) => return Err(FailedToParseFromVec(error_message)),
            }
        }

        Ok(coords)
    }

    /// in-place
    pub fn normalize_vector(&mut self) {
        let len = self.calc_vector_length();
        self.x /= len;
        self.y /= len;
        self.z /= len;
    }

    /// returns new normalized vector
    pub fn calc_normalized_vector(&self) -> Coords {
        let len = match self.calc_vector_length() {
            0.0 => 1.0,
            length => length,
        };
        Coords {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
        }
    }

    pub fn calc_vector_length(&self) -> f64 {
        f64::sqrt(self.x.powi(2) + self.y.powi(2) + self.z.powi(2))
    }

    pub fn cross(&self, rhs: &Coords) -> Coords {
        let x = self.y * rhs.z - self.z * rhs.y;
        let y = self.z * rhs.x - self.x * rhs.z;
        let z = self.x * rhs.y - self.y * rhs.x;

        Coords { x, y, z }
    }

    /// Constructs two orthonormal tangent vectors perpendicular to this direction vector.
    pub fn build_orthonormal_basis(&self) -> (Coords, Coords) {
        let mut tangent1 = if self.x.abs() > 0.9 {
            Coords {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            }
        } else {
            Coords {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            }
        };

        let dot_product = &tangent1 * self;
        let scaled_main_dir = self * dot_product;
        tangent1 = (&tangent1 - &scaled_main_dir).calc_normalized_vector();

        let tangent2 = self.cross(&tangent1).calc_normalized_vector();

        (tangent1, tangent2)
    }
}

impl Default for Coords {
    fn default() -> Self {
        Coords::new()
    }
}

impl fmt::Display for Coords {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\n{{\n  x: {}, \n  y: {}, \n  z: {},\n}}",
            self.x, self.y, self.z
        )
    }
}

impl Clone for Coords {
    fn clone(&self) -> Self {
        Coords {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl Add for Coords {
    type Output = Coords;
    fn add(self, rhs: Coords) -> Coords {
        Coords {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Add for &Coords {
    type Output = Coords;
    fn add(self, rhs: &Coords) -> Coords {
        Coords {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Coords {
    type Output = Coords;
    fn sub(self, rhs: Coords) -> Coords {
        Coords {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Sub<&Coords> for Coords {
    type Output = Coords;
    fn sub(self, rhs: &Coords) -> Coords {
        Coords {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Sub<&Coords> for &Coords {
    type Output = Coords;
    fn sub(self, rhs: &Coords) -> Coords {
        Coords {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul for Coords {
    type Output = f64;
    fn mul(self, rhs: Coords) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

impl Mul<&Coords> for &Coords {
    type Output = f64;
    fn mul(self, rhs: &Coords) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

impl Mul<f64> for &Coords {
    type Output = Coords;
    fn mul(self, rhs: f64) -> Coords {
        Coords {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Div for &Coords {
    type Output = Coords;
    fn div(self, rhs: &Coords) -> Coords {
        Coords {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_orthonormal_basis() {
        let directions = [
            Coords {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            Coords {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            Coords {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            Coords {
                x: -1.0,
                y: 0.0,
                z: 0.0,
            },
            Coords {
                x: 0.577,
                y: 0.577,
                z: 0.577,
            }
            .calc_normalized_vector(),
            Coords {
                x: 0.95,
                y: 0.1,
                z: 0.0,
            }
            .calc_normalized_vector(),
        ];

        for dir in directions.iter() {
            let (t1, t2) = dir.build_orthonormal_basis();

            // Lengths should be ~1
            assert!((t1.calc_vector_length() - 1.0).abs() < 1e-6);
            assert!((t2.calc_vector_length() - 1.0).abs() < 1e-6);

            // Dot products with each other and dir should be ~0
            assert!((&t1 * dir).abs() < 1e-6);
            assert!((&t2 * dir).abs() < 1e-6);
            assert!((&t1 * &t2).abs() < 1e-6);
        }
    }
}
