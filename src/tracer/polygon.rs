use std::fmt;

use slog::{debug, Logger};

use crate::tracer::color::Color;
use crate::tracer::coords::Coords;
use crate::tracer::types::{Entity, Surface};
use crate::utils::logger;
use crate::utils::logger::LOG;

pub static NAME: &str = "polygon";

#[derive(Debug)]
pub struct Polygon {
    pub surface: Surface,
    pub position: Coords,
}

impl Clone for Polygon {
    fn clone(&self) -> Polygon {
        Polygon {
            surface: self.surface.clone(),
            position: self.position.clone(),
        }
    }
}

impl fmt::Display for Polygon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.position)
    }
}

impl Entity for Polygon {
    fn calculate_intersection_distances(
        &self,
        ray_direction: &Coords,
        ray_origin: &Coords,
    ) -> Option<Vec<f64>> {
        debug!(
            LOG,
            "calculating intersections between ray {} and polygon {}", ray_direction, self
        );
        todo!()
    }

    fn calculate_color(&self, intersection_point: &Coords) -> &Color {
        debug!(
            LOG,
            "calculating color of polygon {} at position {}", self, intersection_point
        );
        todo!()
    }
}
