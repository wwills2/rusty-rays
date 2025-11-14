use std::fmt;

use uuid::Uuid;

use crate::tracer::bvh::Aabb;
use crate::tracer::misc_types::{Intersection, Ray};
use crate::tracer::plane_coords_2d::PlaneCoords2D;
use crate::tracer::primitives::polygon::PolygonError::FailedToInitializePolygon;
use crate::tracer::primitives::Plane;
use crate::tracer::primitives::Primitive;
use crate::tracer::Coords;

pub static TYPE_NAME: &str = "polygon";

#[derive(Debug)]
pub struct Polygon {
    pub uuid: Uuid,
    pub surface: String,
    pub plane: Plane,
    pub plane_projected_vertices: Vec<PlaneCoords2D>,
    pub point_in_polygon_inf_test_vector: PlaneCoords2D,
    pub projection_origin: Coords,
    pub vertices: Vec<Coords>,
}

impl Polygon {
    pub fn new(vertices: Vec<Coords>, surface: String) -> Result<Self, PolygonError> {
        let plane = match Plane::new(&vertices) {
            Ok(plane) => plane,
            Err(error) => return Err(FailedToInitializePolygon(error.to_string())),
        };

        let projection_origin = &vertices[0];
        let mut largest_axis_magnitude = 0.0;
        let mut plane_projected_vertices: Vec<PlaneCoords2D> = vec![];
        for vertex in &vertices {
            let projection = plane.project_point_to_plane(vertex, projection_origin);

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
        let point_in_polygon_inf_test_vector = PlaneCoords2D {
            x: largest_axis_magnitude * 10_000_000_000_f64,
            y: largest_axis_magnitude * 10_000_000_000_f64,
        };

        Ok(Polygon {
            uuid: Uuid::new_v4(),
            surface,
            plane,
            plane_projected_vertices,
            point_in_polygon_inf_test_vector,
            projection_origin: projection_origin.clone(),
            vertices,
        })
    }
}

impl Clone for Polygon {
    fn clone(&self) -> Polygon {
        Polygon {
            uuid: self.uuid,
            surface: self.surface.clone(),
            plane: self.plane.clone(),
            plane_projected_vertices: self.plane_projected_vertices.clone(),
            point_in_polygon_inf_test_vector: self.point_in_polygon_inf_test_vector.clone(),
            projection_origin: self.projection_origin.clone(),
            vertices: self.vertices.clone(),
        }
    }
}

impl Primitive for Polygon {
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
        let (plane_intersection_point, distance) =
            self.plane.calculate_intersection_coords_only(ray)?;

        // assume ray is cast from here to (inf, inf)
        let projected_intersection = self
            .plane
            .project_point_to_plane(&plane_intersection_point, &self.projection_origin);

        let mut projected_edge_intersection_count = 0;
        let num_plane_projected_vertices = self.plane_projected_vertices.len();
        for edge_p1_index in 0..num_plane_projected_vertices {
            let edge_p2_index = (edge_p1_index + 1) % num_plane_projected_vertices;
            let edge_p1 = &self.plane_projected_vertices[edge_p1_index];
            let edge_p2 = &self.plane_projected_vertices[edge_p2_index];

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

            let solution_multiplicand = PlaneCoords2D {
                x: projected_intersection.x - edge_p1.x,
                y: projected_intersection.y - edge_p1.y,
            };

            let solution = PlaneCoords2D {
                x: a_inv_matrix * solution_multiplicand.x + b_inv_matrix * solution_multiplicand.y,
                y: c_inv_matrix * solution_multiplicand.x + d_inv_matrix * solution_multiplicand.y,
            };

            if solution.y >= 0.0 && solution.x >= 0.0 && solution.x <= 1.0 {
                projected_edge_intersection_count += 1;
            }
        }

        if projected_edge_intersection_count % 2 != 0 {
            Some(Intersection {
                position: plane_intersection_point,
                ray: ray.clone(),
                distance_along_ray: distance,
                surface_normal_at_intersection: self.plane.normal.clone(),
                intersected_primitive_uuid: self.uuid,
            })
        } else {
            None
        }
    }

    fn primitive_clone(&self) -> Box<dyn Primitive> {
        Box::new((*self).clone())
    }

    fn compute_bounding_box(&self) -> Aabb {
        // For a polygon, the bounding box is determined by the min and max coordinates of its vertices
        Aabb::from_points(&self.vertices)
    }

    fn compute_centroid(&self) -> Coords {
        // The centroid of a polygon is the average of its vertices
        if self.vertices.is_empty() {
            return Coords::new();
        }

        let mut sum = Coords::new();
        for vertex in &self.vertices {
            sum.x += vertex.x;
            sum.y += vertex.y;
            sum.z += vertex.z;
        }

        let num_vertices = self.vertices.len() as f64;
        Coords {
            x: sum.x / num_vertices,
            y: sum.y / num_vertices,
            z: sum.z / num_vertices,
        }
    }
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
