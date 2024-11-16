pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct Position {
    pub x: u64,
    pub y: u64,
    pub z: u64,
}

pub struct Fov {
    pub horz: u8,
    pub vert: u8,
}

pub struct Screen {
    pub width: u64,
    pub height: u64,
}

pub struct Surface {
    pub name: String,
    pub ambient: Color,
    pub diffuse: Color,
    pub specular: Color,
    pub specpow: f64,
    pub reflect: f64,
}

pub struct Sphere {
    pub surface: Surface,
    pub radius: f64,
    pub position: Position,
}

pub enum Entity {
    Sphere(Sphere),
}
