use std::fmt;

use crate::tracer::color::Color;
use crate::tracer::point::Point;

// fov type and methods
pub struct Fov {
    pub horz: u8,
    pub vert: u8,
}

impl fmt::Display for Fov {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\n{{\n  horz: {}, \n  vert: {},\n}}",
            self.horz, self.vert
        )
    }
}

// screen type and methods
pub struct Screen {
    pub width: u64,
    pub height: u64,
}

impl fmt::Display for Screen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\n{{\n  width: {}, \n  height: {},\n}}",
            self.width, self.height
        )
    }
}

// surface type and methods
pub struct Surface {
    pub name: String,
    pub ambient: Color,
    pub diffuse: Color,
    pub specular: Color,
    pub specpow: f64,
    pub reflect: f64,
}

impl Clone for Surface {
    fn clone(&self) -> Self {
        Surface {
            name: self.name.clone(),
            ambient: self.ambient.clone(),
            diffuse: self.diffuse.clone(),
            specular: self.specular.clone(),
            specpow: self.specpow,
            reflect: self.reflect,
        }
    }
}

impl fmt::Display for Surface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\n{{\n  name: \"{}\",\n  ambient: {},\n  diffuse: {},\n  specular: {},\n  specpow: {},\n  reflect: {},\n}}",
            self.name, self.ambient, self.diffuse, self.specular, self.specpow, self.reflect
        )
    }
}

// entity trait and methods
pub trait Entity {
    fn calculate_intersections(&self, ray: &Point) -> Vec<Point>;
    fn calculate_color(&self, intersection_point: &Point) -> Color;
}
