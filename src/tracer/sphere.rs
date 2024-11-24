use std::fmt;

use slog::{debug, Logger};

use crate::tracer::types::{Color, Entity, Point, Surface};
use crate::utils::logger;
use crate::utils::logger::LOG;

pub static NAME: &str = "sphere";

pub struct Sphere {
    pub surface: Surface,
    pub radius: f64,
    pub position: Point,
}

impl fmt::Display for Sphere {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.position)
    }
}

impl Entity for Sphere {
    fn calculate_intersections(&self, ray: &Point) -> Vec<Point> {
        debug!(
            LOG,
            "calculating intersections between ray {} and sphere {}", ray, self
        );
        todo!()
    }

    fn calculate_color(&self, intersection_point: &Point) -> Color {
        debug!(
            LOG,
            "calculating color of sphere {} at point {}", self, intersection_point
        );
        todo!()
    }
}
