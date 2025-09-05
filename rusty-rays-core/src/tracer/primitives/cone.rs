use std::fmt;

use uuid::Uuid;

use crate::tracer::bvh::Aabb;
use crate::tracer::coords::Coords;
use crate::tracer::misc_types::{Intersection, Ray, Surface};
use crate::tracer::primitives::Primitive;

pub static TYPE_NAME: &str = "cone";

#[derive(Debug)]
pub struct Cone {
    pub uuid: Uuid,
    pub surface: Surface,
    pub base_radius: f64,
    pub base: Coords,
    pub apex_radius: f64,
    pub apex: Coords,
}

impl Clone for Cone {
    fn clone(&self) -> Cone {
        Cone {
            uuid: self.uuid,
            surface: self.surface.clone(),
            base_radius: self.base_radius,
            base: self.base.clone(),
            apex_radius: self.apex_radius,
            apex: self.apex.clone(),
        }
    }
}

impl fmt::Display for Cone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cone from base {} with radius {} to apex {} with radius {}",
            self.base, self.base_radius, self.apex, self.apex_radius
        )
    }
}

impl Primitive for Cone {
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
        // Calculate the axis of the cone (from base to apex)
        let axis = (&self.apex - &self.base).calc_normalized_vector();

        // Vector from ray origin to cone base
        let oc = &ray.origin - &self.base;

        // Calculate the height of the cone
        let height = (&self.apex - &self.base).calc_vector_length();

        // Calculate the slope of the cone (change in radius per unit height)
        let tan_theta = (self.base_radius - self.apex_radius) / height;

        // Calculate the dot products we'll need
        let dot_v_axis = &ray.direction * &axis;
        let dot_oc_axis = &oc * &axis;

        // Calculate coefficients for the quadratic equation
        // For a cone, we need to account for the varying radius
        let a = (ray.direction.x.powi(2) + ray.direction.y.powi(2) + ray.direction.z.powi(2))
            - dot_v_axis.powi(2) * (1.0 + tan_theta.powi(2));

        let b = 2.0
            * ((ray.direction.x * oc.x + ray.direction.y * oc.y + ray.direction.z * oc.z)
                - dot_v_axis * dot_oc_axis * (1.0 + tan_theta.powi(2)));

        let c = (oc.x.powi(2) + oc.y.powi(2) + oc.z.powi(2))
            - dot_oc_axis.powi(2) * (1.0 + tan_theta.powi(2))
            - self.base_radius.powi(2);

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

        // Check if the intersection is within the cone's height
        let v = &intersection_point - &self.base;
        let d = v * axis.clone();

        if d < 0.0 || d > height {
            return None;
        }

        // Calculate the radius at the intersection point
        let radius_at_intersection = self.base_radius - tan_theta * d;

        // Calculate the surface normal at the intersection point
        let axis_point = &self.base + &(&axis * d);
        let radial_vector = &intersection_point - &axis_point;

        // The normal for a cone needs to account for the slope
        // We can calculate it as a combination of the radial direction and the axis direction
        let normal = Coords {
            x: radial_vector.x - axis.x * tan_theta * radius_at_intersection,
            y: radial_vector.y - axis.y * tan_theta * radius_at_intersection,
            z: radial_vector.z - axis.z * tan_theta * radius_at_intersection,
        }
        .calc_normalized_vector();

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

    fn compute_bounding_box(&self) -> Aabb {
        // For a cone, we need to consider both the base and apex circles
        // First, calculate points on the base circle in different directions
        let axis = (&self.apex - &self.base).calc_normalized_vector();

        // Find two vectors perpendicular to the axis
        let mut tangent1 = Coords {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        };
        if axis.x.abs() > 0.9 {
            tangent1 = Coords {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            };
        }

        // Make tangent1 perpendicular to axis
        let dot_product = &tangent1 * &axis;
        let scaled_axis = &axis * dot_product;
        let tangent1 = (&tangent1 - &scaled_axis).calc_normalized_vector();
        let tangent2 = axis.cross(&tangent1).calc_normalized_vector();

        // Create points on the base circle
        let mut points = Vec::new();

        // Add base center
        points.push(self.base.clone());

        // Add apex center
        points.push(self.apex.clone());

        // Add points on the base circle
        for i in 0..8 {
            let angle = 2.0 * std::f64::consts::PI * (i as f64 / 8.0);
            let x_offset = self.base_radius * angle.cos();
            let y_offset = self.base_radius * angle.sin();
            let point = &(&self.base + &(&tangent1 * x_offset)) + &(&tangent2 * y_offset);
            points.push(point);
        }

        // Add points on the apex circle (if apex_radius > 0)
        if self.apex_radius > 0.0 {
            for i in 0..8 {
                let angle = 2.0 * std::f64::consts::PI * (i as f64 / 8.0);
                let x_offset = self.apex_radius * angle.cos();
                let y_offset = self.apex_radius * angle.sin();
                let point = &(&self.apex + &(&tangent1 * x_offset)) + &(&tangent2 * y_offset);
                points.push(point);
            }
        }

        // Create Aabb from all points
        Aabb::from_points(&points)
    }

    fn compute_centroid(&self) -> Coords {
        // For a cone, the centroid is at 1/4 of the height from the base
        // if the apex radius is 0 (true cone), or at the center of the
        // frustum if both radii are non-zero
        let axis = &self.apex - &self.base;
        let height = axis.calc_vector_length();
        let normalized_axis = axis.calc_normalized_vector();

        if self.apex_radius == 0.0 {
            // True cone: centroid is at 1/4 of the height from the base
            &self.base + &(&normalized_axis * (height * 0.25))
        } else {
            // Frustum: centroid is at the weighted average of the centers
            let base_weight = self.base_radius.powi(2)
                + self.base_radius * self.apex_radius
                + self.apex_radius.powi(2);
            let apex_weight = self.apex_radius.powi(2);
            let total_weight = base_weight + apex_weight;

            let weighted_base = &self.base * (base_weight / total_weight);
            let weighted_apex = &self.apex * (apex_weight / total_weight);

            &weighted_base + &weighted_apex
        }
    }
}
