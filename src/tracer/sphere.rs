use std::fmt;

use uuid::Uuid;

use crate::tracer::color::Color;
use crate::tracer::coords::Coords;
use crate::tracer::types::{Entity, Intersection, Surface};

pub static TYPE_NAME: &str = "sphere";

#[derive(Debug)]
pub struct Sphere {
    pub uuid: Uuid,
    pub surface: Surface,
    pub radius: f64,
    pub position: Coords,
}

impl Clone for Sphere {
    fn clone(&self) -> Sphere {
        Sphere {
            uuid: self.uuid,
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
    fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    fn get_type(&self) -> String {
        TYPE_NAME.to_string()
    }

    fn calculate_intersection(
        &self,
        ray_direction: &Coords,
        ray_origin: &Coords,
    ) -> Option<Intersection> {
        let a = ray_direction.x.powi(2) + ray_direction.y.powi(2) + ray_direction.z.powi(2);

        let b = (2.0 * ray_direction.x * (ray_origin.x - self.position.x))
            + (2.0 * ray_direction.y * (ray_origin.y - self.position.y))
            + (2.0 * ray_direction.z * (ray_origin.z - self.position.z));

        let c = (ray_origin.x - self.position.x).powi(2)
            + (ray_origin.y - self.position.y).powi(2)
            + (ray_origin.z - self.position.z).powi(2)
            - self.radius.powi(2);

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None;
        }

        let distance = match discriminant {
            0.0 => -b / (2.0 * a),
            _ => {
                let dist_t1 = (-b - discriminant.sqrt()) / (2.0 * a);
                let dist_t2 = (-b + discriminant.sqrt()) / (2.0 * a);

                if dist_t1 < dist_t2 {
                    dist_t1
                } else {
                    dist_t2
                }
            }
        };

        if distance < 0.0 {
            return None;
        }

        let location = ray_origin + &(ray_direction * distance);

        Some(Intersection {
            distance_along_ray: distance,
            location,
        })
    }

    fn calculate_color(&self, intersection_point: &Coords) -> &Color {
        // todo intersection point will be important for reflection angles
        &self.surface.diffuse
    }

    fn entity_clone(&self) -> Box<dyn Entity> {
        Box::new((*self).clone())
    }
}
