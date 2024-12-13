use std::fmt;

use slog::{debug, Logger};

use crate::tracer::color::Color;
use crate::tracer::coords::Coords;
use crate::tracer::types::{Entity, Surface};
use crate::utils::logger;
use crate::utils::logger::LOG;

pub static NAME: &str = "sphere";

#[derive(Debug)]
pub struct Sphere {
    pub surface: Surface,
    pub radius: f64,
    pub position: Coords,
}

impl Clone for Sphere {
    fn clone(&self) -> Sphere {
        Sphere {
            surface: self.surface.clone(),
            radius: self.radius,
            position: self.position.clone(),
        }
    }
}

impl fmt::Display for Sphere {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.position)
    }
}

impl Entity for Sphere {
    fn calculate_intersections(&self, ray: &Coords) -> Vec<Coords> {
        debug!(
            LOG,
            "calculating intersections between ray {} and sphere {}", ray, self
        );
        todo!()
    }

    fn calculate_color(&self, intersection_point: &Coords) -> Color {
        debug!(
            LOG,
            "calculating color of sphere {} at position {}", self, intersection_point
        );
        todo!()
    }
}
