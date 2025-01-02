use std::fmt;

use crate::tracer::color::Color;
use crate::tracer::coords::Coords;
use crate::tracer::sphere::TYPE_NAME;
use crate::tracer::types::{Entity, Surface};
use crate::utils::logger;
use crate::utils::logger::LOG;
use slog::{debug, Logger};
use uuid::Uuid;

pub static NAME: &str = "polygon";

#[derive(Debug)]
pub struct Polygon {
    pub uuid: Uuid,
    pub surface: Surface,
    pub vertices: Vec<Coords>,
}

impl Clone for Polygon {
    fn clone(&self) -> Polygon {
        Polygon {
            uuid: self.uuid,
            surface: self.surface.clone(),
            vertices: self.vertices.clone(),
        }
    }
}

impl Entity for Polygon {
    fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    fn get_type(&self) -> String {
        TYPE_NAME.to_string()
    }

    fn calculate_intersection_distances(
        &self,
        ray_direction: &Coords,
        ray_origin: &Coords,
    ) -> Option<Vec<f64>> {
        debug!(
            LOG,
            "calculating intersections between ray {} and polygon {}", ray_direction, self.uuid
        );
        todo!()
    }

    fn calculate_color(&self, intersection_point: &Coords) -> &Color {
        debug!(
            LOG,
            "calculating color of polygon {} at position {}", self.uuid, intersection_point
        );
        todo!()
    }
}
