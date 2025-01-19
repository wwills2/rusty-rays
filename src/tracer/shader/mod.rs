use crate::tracer::misc_types::{Intersection, Surface};
use crate::tracer::shader::color::Color;

pub mod color;
pub mod light;

pub fn calculate_color(surface: &Surface, intersection: &Intersection) -> Color {
    surface.diffuse.clone()
}
