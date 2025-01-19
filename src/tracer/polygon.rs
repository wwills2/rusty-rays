use std::fmt;

use uuid::Uuid;

use crate::tracer::coords::Coords;
use crate::tracer::misc_types::{Entity, Intersection, Ray, Surface};
use crate::tracer::plane_coords::PlaneCoords;
use crate::tracer::polygon::PolygonError::FailedToInitializePolygon;

pub static TYPE_NAME: &str = "polygon";

#[derive(Debug)]
pub struct Polygon {
    pub uuid: Uuid,
    pub surface: Surface,
    pub normal_vector: Coords,
    pub basis_vectors: (Coords, Coords),
    pub plane_projected_vertices: Vec<PlaneCoords>,
    pub point_in_polygon_inf_test_vector: PlaneCoords,
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
        let mut largest_axis_magnitude = 0.0;
        let mut plane_projected_vertices: Vec<PlaneCoords> = vec![];
        for vertex in &vertices {
            let projection = project_to_plane(vertex, &projection_origin, basis_vectors);

            if projection.x > largest_axis_magnitude {
                largest_axis_magnitude = projection.x;
            }
            if projection.y > largest_axis_magnitude {
                largest_axis_magnitude = projection.y;
            }

            plane_projected_vertices.push(projection);
        }

        // need to cast a ray to effectively infinity without getting into f64 precision errors
        // multiply by the largest projected x or y by 10 billion for pseudo infinity
        let point_in_polygon_inf_test_vector = PlaneCoords {
            x: largest_axis_magnitude * 10_000_000_000_f64,
            y: largest_axis_magnitude * 10_000_000_000_f64,
        };

        Ok(Polygon {
            uuid: Uuid::new_v4(),
            surface,
            normal_vector,
            basis_vectors,
            plane_projected_vertices,
            point_in_polygon_inf_test_vector,
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
            point_in_polygon_inf_test_vector: self.point_in_polygon_inf_test_vector.clone(),
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

    fn get_surface(&self) -> &Surface {
        &self.surface
    }

    fn calculate_intersection(&self, ray: &Ray) -> Option<Intersection> {
        let denominator = &self.normal_vector * &ray.direction;
        if f64::abs(denominator) < 10e-10 {
            return None;
        }

        let distance = (self.normal_vector * (self.vertices[0] - ray.origin)) / denominator;
        let plane_intersection_point = ray.origin + (&ray.direction * distance);

        // assume ray is cast from here to (inf, inf)
        let projected_intersection = project_to_plane(
            &plane_intersection_point,
            &self.projection_origin,
            self.basis_vectors,
        );

        let mut projected_edge_intersection_count = 0;
        let num_plane_projected_vertices = self.plane_projected_vertices.len();
        for edge_p1_index in 0..num_plane_projected_vertices {
            let edge_p2_index = (edge_p1_index + 1) % num_plane_projected_vertices;
            let edge_p1 = self.plane_projected_vertices[edge_p1_index];
            let edge_p2 = self.plane_projected_vertices[edge_p2_index];

            let a_matrix = edge_p2.x - edge_p1.x;
            let b_matrix = -self.point_in_polygon_inf_test_vector.x;
            let c_matrix = edge_p2.y - edge_p1.y;
            let d_matrix = -self.point_in_polygon_inf_test_vector.y;

            let determinant = (a_matrix * d_matrix) - (c_matrix * b_matrix);
            if f64::abs(determinant) < 10e-10 {
                // matrix is not invertible, no intersection with edge and test vector
                continue;
            }

            let inverse_multiplicand = 1.0 / determinant;
            let a_inv_matrix = d_matrix * inverse_multiplicand;
            let b_inv_matrix = -b_matrix * inverse_multiplicand;
            let c_inv_matrix = -c_matrix * inverse_multiplicand;
            let d_inv_matrix = a_matrix * inverse_multiplicand;

            let solution_multiplicand = PlaneCoords {
                x: projected_intersection.x - edge_p1.x,
                y: projected_intersection.y - edge_p1.y,
            };

            let solution = PlaneCoords {
                x: a_inv_matrix * solution_multiplicand.x + b_inv_matrix * solution_multiplicand.y,
                y: c_inv_matrix * solution_multiplicand.x + d_inv_matrix * solution_multiplicand.y,
            };

            if solution.y >= 0.0 && solution.x >= 0.0 && solution.x <= 1.0 {
                projected_edge_intersection_count = projected_edge_intersection_count + 1;
            }
        }

        if projected_edge_intersection_count % 2 != 0 {
            Some(Intersection {
                location: plane_intersection_point,
                distance_along_ray: distance,
                normal_vector: self.normal_vector,
                uuid: self.uuid,
            })
        } else {
            None
        }
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
        return Err("all vertices of the polygon are the same. invalid polygon".to_string());
    }

    let edge_vector_basis_1 = (vertex_2 - vertex_1).calc_normalized_vector();
    let basis_2 = normal.cross(&edge_vector_basis_1).calc_normalized_vector();

    Ok((edge_vector_basis_1, basis_2))
}

pub fn project_to_plane(
    coordinates: &Coords,
    projection_origin: &Coords,
    plane_basis_vectors: (Coords, Coords),
) -> PlaneCoords {
    let diff_vector = *coordinates - projection_origin;
    let u = plane_basis_vectors.0 * diff_vector;
    let w = plane_basis_vectors.1 * diff_vector;

    PlaneCoords { x: u, y: w }
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
