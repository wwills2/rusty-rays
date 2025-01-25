use std::ops::{Add, Div, Mul, Sub};

#[derive(PartialEq, Debug)]
pub struct PlaneCoords {
    pub x: f64,
    pub y: f64,
}

impl PlaneCoords {
    pub fn new() -> Self {
        PlaneCoords { x: 0.0, y: 0.0 }
    }

    /// in-place
    pub fn normalize_vector(&mut self) {
        let len = self.calc_vector_length();
        self.x /= len;
        self.y /= len;
    }

    /// returns new normalized vector
    pub fn calc_normalized_vector(&self) -> PlaneCoords {
        let len = match self.calc_vector_length() {
            length if length == 0.0 => 1.0,
            length => length,
        };
        PlaneCoords {
            x: self.x / len,
            y: self.y / len,
        }
    }

    pub fn calc_vector_length(&self) -> f64 {
        f64::sqrt(self.x.powi(2) + self.y.powi(2))
    }
}

impl Default for PlaneCoords {
    fn default() -> Self {
        PlaneCoords::new()
    }
}

impl Clone for PlaneCoords {
    fn clone(&self) -> Self {
        PlaneCoords {
            x: self.x,
            y: self.y,
        }
    }
}

impl Add for PlaneCoords {
    type Output = PlaneCoords;
    fn add(self, rhs: PlaneCoords) -> PlaneCoords {
        PlaneCoords {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add for &PlaneCoords {
    type Output = PlaneCoords;
    fn add(self, rhs: &PlaneCoords) -> PlaneCoords {
        PlaneCoords {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for PlaneCoords {
    type Output = PlaneCoords;
    fn sub(self, rhs: PlaneCoords) -> PlaneCoords {
        PlaneCoords {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub<&PlaneCoords> for PlaneCoords {
    type Output = PlaneCoords;
    fn sub(self, rhs: &PlaneCoords) -> PlaneCoords {
        PlaneCoords {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul for PlaneCoords {
    type Output = f64;
    fn mul(self, rhs: PlaneCoords) -> f64 {
        self.x * rhs.x + self.y * rhs.y
    }
}

impl Mul<&PlaneCoords> for &PlaneCoords {
    type Output = f64;
    fn mul(self, rhs: &PlaneCoords) -> f64 {
        self.x * rhs.x + self.y * rhs.y
    }
}

impl Mul<f64> for &PlaneCoords {
    type Output = PlaneCoords;
    fn mul(self, rhs: f64) -> PlaneCoords {
        PlaneCoords {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Div for &PlaneCoords {
    type Output = PlaneCoords;
    fn div(self, rhs: &PlaneCoords) -> PlaneCoords {
        PlaneCoords {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}
