use std::fmt;

use uuid::Uuid;

use crate::tracer::coords::Coords;
use crate::tracer::misc_types::{Intersection, Ray, Surface};
use crate::tracer::plane_coords_2d::PlaneCoords2D;
use crate::tracer::primitives::plane::PlaneError::FailedToInitializePlane;
use crate::tracer::primitives::Primitive;
use crate::tracer::shader::color::Color;

pub static TYPE_NAME: &str = "plane";

#[derive(Debug)]
pub struct Plane {
    pub uuid: Uuid,
    pub basis_vectors: (Coords, Coords),
    pub sample_point: Coords,
    pub normal: Coords,
    pub surface: Surface,
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
            surface: placeholder_surface,
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
            "failed to find two non-collinear edges on the polygon plane".to_string(),
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
            "all vertices of the plane are the same. invalid plane".to_string(),
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
