use std::ops::{Add, Div, Mul, Sub};

#[derive(PartialEq, Debug)]
pub struct PlaneCoords2D {
    pub x: f64,
    pub y: f64,
}

impl PlaneCoords2D {
    pub fn new() -> Self {
        PlaneCoords2D { x: 0.0, y: 0.0 }
    }

    /// in-place
    pub fn normalize_vector(&mut self) {
        let len = self.calc_vector_length();
        self.x /= len;
        self.y /= len;
    }

    /// returns new normalized vector
    pub fn calc_normalized_vector(&self) -> PlaneCoords2D {
        let len = match self.calc_vector_length() {
            0.0 => 1.0,
            length => length,
        };
        PlaneCoords2D {
            x: self.x / len,
            y: self.y / len,
        }
    }

    pub fn calc_vector_length(&self) -> f64 {
        f64::sqrt(self.x.powi(2) + self.y.powi(2))
    }
}

impl Default for PlaneCoords2D {
    fn default() -> Self {
        PlaneCoords2D::new()
    }
}

impl Clone for PlaneCoords2D {
    fn clone(&self) -> Self {
        PlaneCoords2D {
            x: self.x,
            y: self.y,
        }
    }
}

impl Add for PlaneCoords2D {
    type Output = PlaneCoords2D;
    fn add(self, rhs: PlaneCoords2D) -> PlaneCoords2D {
        PlaneCoords2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add for &PlaneCoords2D {
    type Output = PlaneCoords2D;
    fn add(self, rhs: &PlaneCoords2D) -> PlaneCoords2D {
        PlaneCoords2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for PlaneCoords2D {
    type Output = PlaneCoords2D;
    fn sub(self, rhs: PlaneCoords2D) -> PlaneCoords2D {
        PlaneCoords2D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub<&PlaneCoords2D> for PlaneCoords2D {
    type Output = PlaneCoords2D;
    fn sub(self, rhs: &PlaneCoords2D) -> PlaneCoords2D {
        PlaneCoords2D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul for PlaneCoords2D {
    type Output = f64;
    fn mul(self, rhs: PlaneCoords2D) -> f64 {
        self.x * rhs.x + self.y * rhs.y
    }
}

impl Mul<&PlaneCoords2D> for &PlaneCoords2D {
    type Output = f64;
    fn mul(self, rhs: &PlaneCoords2D) -> f64 {
        self.x * rhs.x + self.y * rhs.y
    }
}

impl Mul<f64> for &PlaneCoords2D {
    type Output = PlaneCoords2D;
    fn mul(self, rhs: f64) -> PlaneCoords2D {
        PlaneCoords2D {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Div for &PlaneCoords2D {
    type Output = PlaneCoords2D;
    fn div(self, rhs: &PlaneCoords2D) -> PlaneCoords2D {
        PlaneCoords2D {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}
