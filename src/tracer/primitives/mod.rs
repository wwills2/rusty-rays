use std::fmt;

use uuid::Uuid;

use crate::tracer::misc_types::{Intersection, Ray, Surface};

pub mod cone;
pub mod cylinder;
pub mod plane;
pub mod polygon;
pub mod sphere;
pub mod triangle;

// primitive trait and methods
pub trait Primitive: Send + Sync {
    fn get_uuid(&self) -> Uuid;
    fn get_type(&self) -> String;
    fn get_surface(&self) -> &Surface;
    fn calculate_intersection(&self, ray: &Ray) -> Option<Intersection>;
    fn primitive_clone(&self) -> Box<dyn Primitive>;
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
