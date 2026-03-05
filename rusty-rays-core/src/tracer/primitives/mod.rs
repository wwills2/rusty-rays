use std::fmt;

use uuid::Uuid;

use crate::tracer::Coords;
use crate::tracer::bvh::Aabb;
use crate::tracer::misc_types::{Intersection, Ray};

mod cone;
mod plane;
mod polygon;
mod sphere;
mod triangle;

pub use cone::Cone;
pub use plane::Plane;
pub use polygon::Polygon;
pub use sphere::Sphere;
pub use triangle::Triangle;

#[allow(unused)]
pub use cone::TYPE_NAME as CONE_TYPE_NAME;
#[allow(unused)]
pub use plane::TYPE_NAME as PLANE_TYPE_NAME;
#[allow(unused)]
pub use polygon::TYPE_NAME as POLYGON_TYPE_NAME;
#[allow(unused)]
pub use sphere::TYPE_NAME as SPHERE_TYPE_NAME;
#[allow(unused)]
pub use triangle::TYPE_NAME as TRIANGLE_TYPE_NAME;

// primitive trait and methods
pub trait Primitive: Send + Sync {
    /// inline
    fn get_uuid(&self) -> Uuid;

    /// inline
    fn get_type(&self) -> String;

    /// inline
    fn get_surface(&self) -> &String;
    fn calculate_intersection(&self, ray: &Ray) -> Option<Intersection>;
    fn primitive_clone(&self) -> Box<dyn Primitive>;
    fn compute_bounding_box(&self) -> Aabb;
    fn compute_centroid(&self) -> Coords;
}

impl fmt::Debug for dyn Primitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "primitive of type {} with uuid {}",
            self.get_type(),
            self.get_uuid()
        )
    }
}

impl Clone for Box<dyn Primitive> {
    fn clone(&self) -> Self {
        self.primitive_clone()
    }
}
