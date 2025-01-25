use std::fmt;

use uuid::Uuid;

use crate::tracer::coords::Coords;
use crate::tracer::misc_types::{Entity, Intersection, Ray, Surface};

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

    fn get_surface(&self) -> &Surface {
        &self.surface
    }

    fn calculate_intersection(&self, ray: &Ray) -> Option<Intersection> {
        let a = ray.direction.x.powi(2) + ray.direction.y.powi(2) + ray.direction.z.powi(2);

        let b = (2.0 * ray.direction.x * (ray.origin.x - self.position.x))
            + (2.0 * ray.direction.y * (ray.origin.y - self.position.y))
            + (2.0 * ray.direction.z * (ray.origin.z - self.position.z));

        let c = (ray.origin.x - self.position.x).powi(2)
            + (ray.origin.y - self.position.y).powi(2)
            + (ray.origin.z - self.position.z).powi(2)
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

        let location = &ray.origin + &(&ray.direction * distance);
        let normal = (&location - &self.position).calc_normalized_vector();

        Some(Intersection {
            distance_along_ray: distance,
            ray: ray.clone(),
            position: location.clone(),
            surface_normal_at_intersection: normal,
            uuid: self.uuid,
        })
    }

    fn entity_clone(&self) -> Box<dyn Entity> {
        Box::new((*self).clone())
    }
}
