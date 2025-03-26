use std::fmt;

use uuid::Uuid;

use crate::tracer::coords::Coords;
use crate::tracer::misc_types::{Intersection, Ray, Surface};
use crate::tracer::primitives::Primitive;

pub static TYPE_NAME: &str = "cylinder";

#[derive(Debug)]
pub struct Cylinder {
    pub uuid: Uuid,
    pub surface: Surface,
    pub radius: f64,
    pub base: Coords,
    pub axis: Coords,
    pub height: f64,
}

impl Clone for Cylinder {
    fn clone(&self) -> Cylinder {
        Cylinder {
            uuid: self.uuid,
            surface: self.surface.clone(),
            radius: self.radius,
            base: self.base.clone(),
            axis: self.axis.clone(),
            height: self.height,
        }
    }
}

impl fmt::Display for Cylinder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cylinder at base {} with axis {} and radius {}", self.base, self.axis, self.radius)
    }
}

impl Primitive for Cylinder {
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
        // Normalize the axis direction
        let axis = self.axis.calc_normalized_vector();

        // Vector from ray origin to cylinder base
        let oc = &ray.origin - &self.base;

        // Calculate coefficients for the quadratic equation
        let a = ray.direction.x.powi(2) + ray.direction.y.powi(2) + ray.direction.z.powi(2) - 
                (ray.direction.clone() * axis.clone()).powi(2);

        let b = 2.0 * (oc.x * ray.direction.x + oc.y * ray.direction.y + oc.z * ray.direction.z - 
                (oc.clone() * axis.clone()) * (ray.direction.clone() * axis.clone()));

        let c = oc.x.powi(2) + oc.y.powi(2) + oc.z.powi(2) - 
                (oc.clone() * axis.clone()).powi(2) - self.radius.powi(2);

        // Solve the quadratic equation
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None;
        }

        // Calculate the two potential intersection points
        let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
        let t2 = (-b + discriminant.sqrt()) / (2.0 * a);

        // Choose the closest positive intersection
        let t = if t1 < 0.0 {
            if t2 < 0.0 {
                return None;
            }
            t2
        } else if t2 < 0.0 || t1 < t2 {
            t1
        } else {
            t2
        };

        // Calculate the intersection point
        let intersection_point = &ray.origin + &(&ray.direction * t);

        // Check if the intersection is within the cylinder's height
        let v = &intersection_point - &self.base;
        let d = v * axis.clone();

        if d < 0.0 || d > self.height {
            return None;
        }

        // Calculate the surface normal at the intersection point
        let axis_point = &self.base + &(&axis * d);
        let normal = (&intersection_point - &axis_point).calc_normalized_vector();

        Some(Intersection {
            distance_along_ray: t,
            ray: ray.clone(),
            position: intersection_point,
            surface_normal_at_intersection: normal,
            intersected_primitive_uuid: self.uuid,
        })
    }

    fn primitive_clone(&self) -> Box<dyn Primitive> {
        Box::new((*self).clone())
    }
}
