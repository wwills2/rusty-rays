use std::fmt;

use uuid::Uuid;

use crate::tracer::Coords;
use crate::tracer::bvh::Aabb;
use crate::tracer::misc_types::{Intersection, Ray, Surface};
use crate::tracer::primitives::Plane;
use crate::tracer::primitives::Primitive;
use crate::tracer::primitives::triangle::TriangleError::FailedToInitializeTriangle;

pub static TYPE_NAME: &str = "triangle";

#[derive(Debug)]
pub struct Triangle {
    pub uuid: Uuid,
    pub surface: Surface,
    pub plane: Plane,
    pub vertex_1: Coords,
    pub vertex_2: Coords,
    pub vertex_3: Coords,
    pub edge_1: Coords,
    pub edge_2: Coords,
    pub edge_3: Coords,
    pub v1_normal: Coords,
    pub v2_normal: Coords,
    pub v3_normal: Coords,
    pub flat_shaded: bool,
    pub total_area: f64,
}

impl Triangle {
    pub fn new(
        vertex_1: Coords,
        vertex_2: Coords,
        vertex_3: Coords,
        maybe_v1_normal: Option<Coords>,
        maybe_v2_normal: Option<Coords>,
        maybe_v3_normal: Option<Coords>,
        surface: Surface,
    ) -> Result<Self, TriangleError> {
        let edge_1 = &vertex_2 - &vertex_1;
        let edge_2 = &vertex_3 - &vertex_2;
        let edge_3 = &vertex_1 - &vertex_3;

        let total_area = &edge_1.cross(&(&vertex_3 - &vertex_1)).calc_vector_length() * 0.5;
        let plane = match Plane::new(&[vertex_1.clone(), vertex_2.clone(), vertex_3.clone()]) {
            Ok(plane) => plane,
            Err(error) => return Err(FailedToInitializeTriangle(error.to_string())),
        };

        let flat_shaded =
            maybe_v1_normal.is_none() && maybe_v2_normal.is_none() && maybe_v3_normal.is_none();

        Ok(Triangle {
            uuid: Uuid::new_v4(),
            vertex_1,
            vertex_2,
            vertex_3,
            v1_normal: maybe_v1_normal.unwrap_or_default(),
            v2_normal: maybe_v2_normal.unwrap_or_default(),
            v3_normal: maybe_v3_normal.unwrap_or_default(),
            edge_1,
            edge_2,
            edge_3,
            plane,
            surface,
            flat_shaded,
            total_area,
        })
    }

    fn interpolate_normal_at_intersection(&self, intersection_point: &Coords) -> Coords {
        // area for v2 and v3
        let alpha_area = 0.5
            * (&self.vertex_2 - intersection_point)
                .cross(&(&self.vertex_3 - intersection_point))
                .calc_vector_length();
        // area for v3 and v1
        let beta_area = 0.5
            * (&self.vertex_3 - intersection_point)
                .cross(&(&self.vertex_1 - intersection_point))
                .calc_vector_length();
        // area for v1 and v2
        let gamma_area = 0.5
            * (&self.vertex_1 - intersection_point)
                .cross(&(&self.vertex_2 - intersection_point))
                .calc_vector_length();

        let alpha = alpha_area / self.total_area;
        let beta = beta_area / self.total_area;
        let gamma = gamma_area / self.total_area;

        let interpolated_normal =
            (&self.v1_normal * alpha) + (&self.v2_normal * beta) + (&self.v3_normal * gamma);
        interpolated_normal.calc_normalized_vector()
    }
}

impl Clone for Triangle {
    fn clone(&self) -> Self {
        Triangle {
            uuid: self.uuid,
            surface: self.surface.clone(),
            plane: self.plane.clone(),
            vertex_1: self.vertex_1.clone(),
            vertex_2: self.vertex_2.clone(),
            vertex_3: self.vertex_3.clone(),
            edge_1: self.edge_1.clone(),
            edge_2: self.edge_2.clone(),
            edge_3: self.edge_3.clone(),
            v1_normal: self.v1_normal.clone(),
            v2_normal: self.v2_normal.clone(),
            v3_normal: self.v3_normal.clone(),
            flat_shaded: self.flat_shaded,
            total_area: self.total_area,
        }
    }
}

impl Primitive for Triangle {
    #[inline]
    fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    #[inline]
    fn get_type(&self) -> String {
        TYPE_NAME.to_string()
    }

    #[inline]
    fn get_surface(&self) -> &Surface {
        &self.surface
    }

    fn calculate_intersection(&self, ray: &Ray) -> Option<Intersection> {
        let (plane_intersection_point, distance_along_ray) =
            self.plane.calculate_intersection_coords_only(ray)?;
        let lambda_1 = &plane_intersection_point - &self.vertex_1;
        let lambda_2 = &plane_intersection_point - &self.vertex_2;
        let lambda_3 = &plane_intersection_point - &self.vertex_3;

        let lambda_normal_1 = self.edge_1.cross(&lambda_1);
        let lambda_normal_2 = self.edge_2.cross(&lambda_2);
        let lambda_normal_3 = self.edge_3.cross(&lambda_3);

        let lambda_angle_1 = &lambda_normal_1 * &self.plane.normal;
        let lambda_angle_2 = &lambda_normal_2 * &self.plane.normal;
        let lambda_angle_3 = &lambda_normal_3 * &self.plane.normal;

        if (lambda_angle_1 >= 0.0 && lambda_angle_2 >= 0.0 && lambda_angle_3 >= 0.0)
            || (lambda_angle_1 < 0.0 && lambda_angle_2 < 0.0 && lambda_angle_3 < 0.0)
        {
            let surface_normal_at_intersection = if self.flat_shaded {
                self.plane.normal.clone()
            } else {
                self.interpolate_normal_at_intersection(&plane_intersection_point)
            };

            Some(Intersection {
                surface_normal_at_intersection,
                distance_along_ray,
                intersected_primitive_uuid: self.uuid,
                position: plane_intersection_point,
                ray: ray.clone(),
            })
        } else {
            None
        }
    }

    fn primitive_clone(&self) -> Box<dyn Primitive> {
        Box::new((*self).clone())
    }

    fn compute_bounding_box(&self) -> Aabb {
        // For a triangle, the bounding box is determined by the min and max coordinates of its vertices
        let points = vec![
            self.vertex_1.clone(),
            self.vertex_2.clone(),
            self.vertex_3.clone(),
        ];
        Aabb::from_points(&points)
    }

    fn compute_centroid(&self) -> Coords {
        // The centroid of a triangle is the average of its three vertices
        let sum = &(&self.vertex_1 + &self.vertex_2) + &self.vertex_3;
        Coords {
            x: sum.x / 3.0,
            y: sum.y / 3.0,
            z: sum.z / 3.0,
        }
    }
}

#[derive(Debug)]
pub enum TriangleError {
    FailedToInitializeTriangle(String),
}

impl fmt::Display for TriangleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FailedToInitializeTriangle(error_message) => {
                write!(f, "Failed to initialize triangle: {}", error_message)
            }
        }
    }
}

impl std::error::Error for TriangleError {}
