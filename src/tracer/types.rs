use std::fmt;

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
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

pub struct Surface {
    pub name: String,
    pub ambient: Color,
    pub diffuse: Color,
    pub specular: Color,
    pub specpow: f64,
    pub reflect: f64,
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

pub trait Entity {
    fn calculate_intersections(&self, ray: &Point) -> Vec<Point>;
    fn calculate_color(&self, intersection_point: &Point) -> Color;
}
