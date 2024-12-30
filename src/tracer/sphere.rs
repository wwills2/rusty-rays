use std::fmt;

use crate::tracer::color::Color;
use crate::tracer::coords::Coords;
use crate::tracer::types::{Entity, Surface};

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
    fn calculate_intersection_distances(
        &self,
        direction_vector: &Coords,
        ray_origin: &Coords,
    ) -> Option<Vec<f64>> {
        let a =
            direction_vector.x.powi(2) + direction_vector.y.powi(2) + direction_vector.z.powi(2);

        let b = (2.0 * direction_vector.x * (ray_origin.x - self.position.x))
            + (2.0 * direction_vector.y * (ray_origin.y - self.position.y))
            + (2.0 * direction_vector.z * (ray_origin.z - self.position.z));

        let c = (ray_origin.x - self.position.x).powi(2)
            + (ray_origin.y - self.position.y).powi(2)
            + (ray_origin.z - self.position.z).powi(2)
            - self.radius.powi(2);

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None;
        }

        if discriminant == 0.0 {
            return Some(vec![-b / (2.0 * a)]);
        }

        let dist_t1 = (-b - discriminant.sqrt()) / (2.0 * a);
        let dist_t2 = (-b + discriminant.sqrt()) / (2.0 * a);

        Some(vec![dist_t1, dist_t2])
    }

    fn calculate_color(&self, intersection_point: &Coords) -> &Color {
        // todo intersection point will be important for reflection angles
        &self.surface.diffuse
    }
}
