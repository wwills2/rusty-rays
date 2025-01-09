use std::fmt;

use uuid::Uuid;

use crate::tracer::color::Color;
use crate::tracer::coords::Coords;
use crate::tracer::polygon::PolygonError::FailedToInitializePolygon;
use crate::tracer::sphere::TYPE_NAME;
use crate::tracer::types::{Entity, Intersection, Surface};

pub static NAME: &str = "polygon";

#[derive(Debug)]
pub struct Polygon {
    pub uuid: Uuid,
    pub surface: Surface,
    pub normal_vector: Coords,
    pub basis_vectors: (Coords, Coords),
    pub plane_projected_vertices: Vec<(f64, f64)>,
    pub projection_origin: Coords,
    pub vertices: Vec<Coords>,
}

impl Polygon {
    pub fn new(vertices: Vec<Coords>, surface: Surface) -> Result<Self, PolygonError> {
        let normal_vector = match calculate_plane_normal_vector(&vertices) {
            Ok(normal) => normal,
            Err(error_message) => {
                return Err(FailedToInitializePolygon(error_message));
            }
        };

        let basis_vectors = match calculate_basis_vectors(&vertices, &normal_vector) {
            Ok(basis) => basis,
            Err(error_message) => {
                return Err(FailedToInitializePolygon(error_message));
            }
        };

        let projection_origin = vertices[0];
        let mut plane_projected_vertices: Vec<(f64, f64)> = vec![];
        for vertex in &vertices {
            let projection = project_to_plane(vertex, &projection_origin, basis_vectors);
            plane_projected_vertices.push(projection);
        }

        Ok(Polygon {
            uuid: Uuid::new_v4(),
            surface,
            normal_vector,
            basis_vectors,
            plane_projected_vertices,
            projection_origin,
            vertices,
        })
    }
}

impl Clone for Polygon {
    fn clone(&self) -> Polygon {
        Polygon {
            uuid: self.uuid,
            normal_vector: self.normal_vector,
            surface: self.surface.clone(),
            basis_vectors: self.basis_vectors,
            plane_projected_vertices: self.plane_projected_vertices.clone(),
            projection_origin: self.projection_origin,
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

    fn calculate_intersection(
        &self,
        ray_direction: &Coords,
        ray_origin: &Coords,
    ) -> Option<Intersection> {
        let denominator = &self.normal_vector * ray_direction;
        if f64::abs(denominator) < 10e-10 {
            return None;
        }

        let distance = self.normal_vector * (self.vertices[0] - ray_origin);
        let plane_intersection_point = ray_origin + &(ray_direction * distance);

        let projected_intersection = project_to_plane(
            &plane_intersection_point,
            &self.projection_origin,
            self.basis_vectors,
        );

        todo!("need to calculate edges and projected polygon intersection");
    }

    fn calculate_color(&self, intersection_point: &Coords) -> &Color {
        // todo intersection point will be important for reflection angles
        &self.surface.diffuse
    }

    fn entity_clone(&self) -> Box<dyn Entity> {
        Box::new((*self).clone())
    }
}

pub fn calculate_plane_normal_vector(vertices: &Vec<Coords>) -> Result<Coords, String> {
    if vertices.len() < 3 {
        return Err("a polygon must have atleast 3 vertices".to_string());
    }

    let start = 0;
    let vertex_1 = vertices[start];
    let mut vertex_2 = vertices[start + 1];
    let mut vertex_3 = vertices[start + 2];

    let mut plane_edge_1 = vertex_2 - vertex_1;
    let mut plane_edge_2 = vertex_3 - vertex_1;

    plane_edge_1.normalize_vector();
    plane_edge_2.normalize_vector();
    let mut is_collinear = plane_edge_1 == plane_edge_2;

    if is_collinear {
        for i in 2..vertices.len() - 1 {
            vertex_2 = vertices[i];
            vertex_3 = vertices[i + 1];
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
        return Err("failed to find two non-collinear edges on the polygon plane".to_string());
    }

    let normal_vector = plane_edge_1.cross(&plane_edge_2).calc_normalized_vector();

    Ok(normal_vector)
}

pub fn calculate_basis_vectors(
    vertices: &Vec<Coords>,
    normal: &Coords,
) -> Result<(Coords, Coords), String> {
    let mut index = 1;
    let vertex_1 = vertices[0];
    let mut vertex_2 = vertices[index];

    while vertex_1 == vertex_2 && index < vertices.len() {
        index += 1;
        vertex_2 = vertices[index];
    }

    if vertex_1 == vertex_2 {
        return Err("all points on the polygon are the same. invalid polygon".to_string());
    }

    let edge_vector_basis_1 = (vertex_2 - vertex_1).calc_normalized_vector();
    let mut basis_2 = normal.cross(&edge_vector_basis_1).calc_normalized_vector();

    Ok((edge_vector_basis_1, basis_2))
}

pub fn project_to_plane(
    coordinates: &Coords,
    projection_origin: &Coords,
    plane_basis_vectors: (Coords, Coords),
) -> (f64, f64) {
    let diff_vector = *coordinates - projection_origin;
    let u = plane_basis_vectors.0 * diff_vector;
    let w = plane_basis_vectors.1 * diff_vector;

    (u, w)
}

#[derive(Debug)]
pub enum PolygonError {
    FailedToInitializePolygon(String),
}

impl fmt::Display for PolygonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FailedToInitializePolygon(error_message) => {
                write!(f, "Failed to initialize polygon: {}", error_message)
            }
        }
    }
}

impl std::error::Error for PolygonError {}
