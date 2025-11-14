use std::fmt;

use uuid::Uuid;

use crate::tracer::bvh::Aabb;
use crate::tracer::misc_types::{Intersection, Ray, Surface};
use crate::tracer::plane_coords_2d::PlaneCoords2D;
use crate::tracer::primitives::plane::PlaneError::FailedToInitializePlane;
use crate::tracer::primitives::Primitive;
use crate::tracer::shader::Color;
use crate::tracer::Coords;

pub static TYPE_NAME: &str = "plane";

/// Internal utility primitive. A planes are treated as polygons for rendering.
#[derive(Debug)]
pub struct Plane {
    pub uuid: Uuid,
    pub basis_vectors: (Coords, Coords),
    pub sample_point: Coords,
    pub normal: Coords,
    pub surface: String,
}

impl Plane {
    pub fn new(points_on_plane: &[Coords]) -> Result<Self, PlaneError> {
        let normal = calculate_plane_normal_vector(points_on_plane)?;
        let basis_vectors = calculate_basis_vectors(points_on_plane, &normal)?;

        // todo need to parse plane from file
        let placeholder_surface = Surface {
            name: String::new(),
            ambient: Color::new(),
            diffuse: Color::new(),
            specular: Color::new(),
            specpow: 0.0,
            reflect: 0.0,
        };

        Ok(Plane {
            uuid: Uuid::new_v4(),
            sample_point: points_on_plane[0].clone(),
            normal,
            basis_vectors,
            surface: placeholder_surface.name,
        })
    }

    pub fn project_point_to_plane(
        &self,
        coordinates: &Coords,
        projection_origin: &Coords,
    ) -> PlaneCoords2D {
        let diff_vector = coordinates - projection_origin;
        let u = &self.basis_vectors.0 * &diff_vector;
        let w = &self.basis_vectors.1 * &diff_vector;

        PlaneCoords2D { x: u, y: w }
    }

    pub fn calculate_intersection_coords_only(&self, ray: &Ray) -> Option<(Coords, f64)> {
        let denominator = &self.normal * &ray.direction;
        if f64::abs(denominator) < 10e-10 {
            return None;
        }

        let distance = (&self.normal * &(&self.sample_point - &ray.origin)) / denominator;
        if distance < 0.0 {
            return None;
        }
        let intersection_coords = &ray.origin + &(&ray.direction * distance).clone();

        Some((intersection_coords, distance))
    }
}

impl Primitive for Plane {
    #[inline]
    fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    #[inline]
    fn get_type(&self) -> String {
        TYPE_NAME.to_string()
    }

    #[inline]
    fn get_surface(&self) -> &String {
        &self.surface
    }

    fn calculate_intersection(&self, ray: &Ray) -> Option<Intersection> {
        let (position, distance_along_ray) = self.calculate_intersection_coords_only(ray)?;
        Some(Intersection {
            position,
            intersected_primitive_uuid: self.uuid,
            distance_along_ray,
            ray: ray.clone(),
            surface_normal_at_intersection: self.normal.clone(),
        })
    }

    fn primitive_clone(&self) -> Box<dyn Primitive> {
        Box::new((*self).clone())
    }

    fn compute_bounding_box(&self) -> Aabb {
        // For a plane, we create a very large bounding box centered at the sample point
        // and extending in the direction of the basis vectors
        let large_value = 1000000.0; // A large value to represent "infinity"

        // Create points in the plane at a large distance from the sample point
        let mut points = Vec::new();

        // Add the sample point
        points.push(self.sample_point.clone());

        // Add points in the direction of the basis vectors
        let basis1 = &self.basis_vectors.0;
        let basis2 = &self.basis_vectors.1;

        // Add points in the positive and negative directions of the basis vectors
        points.push(&self.sample_point + &(basis1 * large_value));
        points.push(&self.sample_point - &(basis1 * large_value));
        points.push(&self.sample_point + &(basis2 * large_value));
        points.push(&self.sample_point - &(basis2 * large_value));

        // Add points in the diagonal directions
        points.push(&(&self.sample_point + &(basis1 * large_value)) + &(basis2 * large_value));
        points.push(&(&self.sample_point - &(basis1 * large_value)) + &(basis2 * large_value));
        points.push(&(&self.sample_point + &(basis1 * large_value)) - &(basis2 * large_value));
        points.push(&(&self.sample_point - &(basis1 * large_value)) - &(basis2 * large_value));

        // Create Aabb from all points
        Aabb::from_points(&points)
    }

    fn compute_centroid(&self) -> Coords {
        // The centroid of a plane is its sample point
        self.sample_point.clone()
    }
}

fn calculate_plane_normal_vector(vertices: &[Coords]) -> Result<Coords, PlaneError> {
    if vertices.len() < 3 {
        return Err(FailedToInitializePlane(format!(
            "only {} vertices provided. at least 3 vertices must be provided to define a plane",
            vertices.len()
        )));
    }

    let start = 0;
    let vertex_1 = &vertices[start];
    let mut vertex_2 = &vertices[start + 1];
    let mut vertex_3 = &vertices[start + 2];

    let mut plane_edge_1 = vertex_2 - vertex_1;
    let mut plane_edge_2 = vertex_3 - vertex_1;

    plane_edge_1.normalize_vector();
    plane_edge_2.normalize_vector();
    let mut is_collinear = plane_edge_1 == plane_edge_2;

    if is_collinear {
        for i in 2..vertices.len() - 1 {
            vertex_2 = &vertices[i];
            vertex_3 = &vertices[i + 1];
            plane_edge_1 = vertex_2 - vertex_1;
            plane_edge_2 = vertex_3 - vertex_1;

            plane_edge_1.normalize_vector();
            plane_edge_2.normalize_vector();
            is_collinear = plane_edge_1 == plane_edge_2;

            if !is_collinear {
                break;
            }
        }
    }

    if is_collinear {
        return Err(FailedToInitializePlane(
            "failed to find two non-collinear edges on the polygon plane. invalid plane"
                .to_string(),
        ));
    }

    let normal_vector = plane_edge_1.cross(&plane_edge_2).calc_normalized_vector();

    Ok(normal_vector)
}

impl Clone for Plane {
    fn clone(&self) -> Self {
        Plane {
            uuid: self.uuid,
            surface: self.surface.clone(),
            sample_point: self.sample_point.clone(),
            normal: self.normal.clone(),
            basis_vectors: self.basis_vectors.clone(),
        }
    }
}

fn calculate_basis_vectors(
    vertices: &[Coords],
    normal: &Coords,
) -> Result<(Coords, Coords), PlaneError> {
    let mut index = 1;
    let vertex_1 = &vertices[0];
    let mut vertex_2 = &vertices[index];

    while vertex_1 == vertex_2 && index < vertices.len() {
        index += 1;
        vertex_2 = &vertices[index];
    }

    if vertex_1 == vertex_2 {
        return Err(FailedToInitializePlane(
            "two or more vertices are the same. invalid plane".to_string(),
        ));
    }

    let edge_vector_basis_1 = (vertex_2 - vertex_1).calc_normalized_vector();
    let basis_2 = normal.cross(&edge_vector_basis_1).calc_normalized_vector();

    Ok((edge_vector_basis_1, basis_2))
}

#[derive(Debug)]
pub enum PlaneError {
    FailedToInitializePlane(String),
}

impl fmt::Display for PlaneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FailedToInitializePlane(error_message) => {
                write!(f, "Failed to initialize plane: {}", error_message)
            }
        }
    }
}

impl std::error::Error for PlaneError {}
