use std::cmp::Ordering;
use std::fmt;
use std::sync::Arc;

use uuid::Uuid;

use crate::tracer::coords::Coords;
use crate::tracer::misc_types::{Intersection, Ray};
use crate::tracer::primitives::Primitive;
use crate::utils::logger::LOG;
use slog::{debug, trace};

// Axis-aligned bounding box
#[derive(Debug, Clone)]
pub struct AABB {
    pub min: Coords,
    pub max: Coords,
}

impl AABB {
    pub fn new(min: Coords, max: Coords) -> Self {
        AABB { min, max }
    }

    pub fn from_points(points: &[Coords]) -> Self {
        let mut min = Coords {
            x: f64::INFINITY,
            y: f64::INFINITY,
            z: f64::INFINITY,
        };
        let mut max = Coords {
            x: f64::NEG_INFINITY,
            y: f64::NEG_INFINITY,
            z: f64::NEG_INFINITY,
        };

        for point in points {
            min.x = min.x.min(point.x);
            min.y = min.y.min(point.y);
            min.z = min.z.min(point.z);

            max.x = max.x.max(point.x);
            max.y = max.y.max(point.y);
            max.z = max.z.max(point.z);
        }

        AABB { min, max }
    }

    pub fn merge(&self, other: &AABB) -> AABB {
        let min = Coords {
            x: self.min.x.min(other.min.x),
            y: self.min.y.min(other.min.y),
            z: self.min.z.min(other.min.z),
        };
        let max = Coords {
            x: self.max.x.max(other.max.x),
            y: self.max.y.max(other.max.y),
            z: self.max.z.max(other.max.z),
        };
        AABB { min, max }
    }

    pub fn intersect(&self, ray: &Ray) -> bool {
        // Ray-AABB intersection using slab method
        let mut tmin = f64::NEG_INFINITY;
        let mut tmax = f64::INFINITY;

        // Check intersection with x planes
        if ray.direction.x.abs() > f64::EPSILON {
            let tx1 = (self.min.x - ray.origin.x) / ray.direction.x;
            let tx2 = (self.max.x - ray.origin.x) / ray.direction.x;
            tmin = tmin.max(tx1.min(tx2));
            tmax = tmax.min(tx1.max(tx2));
        } else if ray.origin.x < self.min.x || ray.origin.x > self.max.x {
            return false;
        }

        // Check intersection with y planes
        if ray.direction.y.abs() > f64::EPSILON {
            let ty1 = (self.min.y - ray.origin.y) / ray.direction.y;
            let ty2 = (self.max.y - ray.origin.y) / ray.direction.y;
            tmin = tmin.max(ty1.min(ty2));
            tmax = tmax.min(ty1.max(ty2));
        } else if ray.origin.y < self.min.y || ray.origin.y > self.max.y {
            return false;
        }

        // Check intersection with z planes
        if ray.direction.z.abs() > f64::EPSILON {
            let tz1 = (self.min.z - ray.origin.z) / ray.direction.z;
            let tz2 = (self.max.z - ray.origin.z) / ray.direction.z;
            tmin = tmin.max(tz1.min(tz2));
            tmax = tmax.min(tz1.max(tz2));
        } else if ray.origin.z < self.min.z || ray.origin.z > self.max.z {
            return false;
        }

        tmax >= tmin && tmax > 0.0
    }

    pub fn get_longest_axis(&self) -> usize {
        let size_x = self.max.x - self.min.x;
        let size_y = self.max.y - self.min.y;
        let size_z = self.max.z - self.min.z;

        if size_x > size_y && size_x > size_z {
            0 // x-axis
        } else if size_y > size_z {
            1 // y-axis
        } else {
            2 // z-axis
        }
    }
}

// BVH Node
#[derive(Debug, Clone)]
pub enum BVHNode {
    Branch {
        aabb: AABB,
        left: Box<BVHNode>,
        right: Box<BVHNode>,
    },
    Leaf {
        aabb: AABB,
        primitive: Arc<Box<dyn Primitive>>,
    },
}

impl BVHNode {
    pub fn get_aabb(&self) -> &AABB {
        match self {
            BVHNode::Branch { aabb, .. } => aabb,
            BVHNode::Leaf { aabb, .. } => aabb,
        }
    }

    pub fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        if !self.get_aabb().intersect(ray) {
            return None;
        }

        match self {
            BVHNode::Branch { left, right, .. } => {
                let left_hit = left.intersect(ray);
                let right_hit = right.intersect(ray);

                match (left_hit, right_hit) {
                    (Some(left_intersection), Some(right_intersection)) => {
                        if left_intersection.distance_along_ray < right_intersection.distance_along_ray {
                            Some(left_intersection)
                        } else {
                            Some(right_intersection)
                        }
                    }
                    (Some(intersection), None) | (None, Some(intersection)) => Some(intersection),
                    (None, None) => None,
                }
            }
            BVHNode::Leaf { primitive, .. } => primitive.calculate_intersection(ray),
        }
    }
}

// BVH Tree
#[derive(Debug, Clone)]
pub struct BVH {
    root: Option<BVHNode>,
}

impl BVH {
    pub fn new() -> Self {
        BVH { root: None }
    }

    pub fn build(&mut self, primitives: Vec<Box<dyn Primitive>>) {
        debug!(LOG, "Building BVH with {} primitives", primitives.len());

        if primitives.is_empty() {
            self.root = None;
            return;
        }

        // Convert primitives to Arc for shared ownership
        let primitives: Vec<Arc<Box<dyn Primitive>>> = primitives
            .into_iter()
            .map(|p| Arc::new(p))
            .collect();

        // Build the BVH tree
        self.root = Some(self.build_node(&primitives));

        debug!(LOG, "BVH construction complete");
    }

    fn build_node(&self, primitives: &[Arc<Box<dyn Primitive>>]) -> BVHNode {
        if primitives.len() == 1 {
            // Create a leaf node for a single primitive
            let primitive = Arc::clone(&primitives[0]);

            // Compute a tight AABB for the primitive
            let aabb = self.compute_primitive_aabb(&primitive);

            return BVHNode::Leaf { aabb, primitive };
        }

        // Compute bounding box for all primitives
        let aabb = self.compute_primitives_aabb(primitives);

        // Find the longest axis of the bounding box
        let axis = aabb.get_longest_axis();

        // Sort primitives along the chosen axis
        let mut sorted_primitives = primitives.to_vec();
        match axis {
            0 => sorted_primitives.sort_by(|a, b| {
                let centroid_a = self.compute_primitive_centroid(&a);
                let centroid_b = self.compute_primitive_centroid(&b);
                centroid_a.x.partial_cmp(&centroid_b.x).unwrap_or(Ordering::Equal)
            }),
            1 => sorted_primitives.sort_by(|a, b| {
                let centroid_a = self.compute_primitive_centroid(&a);
                let centroid_b = self.compute_primitive_centroid(&b);
                centroid_a.y.partial_cmp(&centroid_b.y).unwrap_or(Ordering::Equal)
            }),
            _ => sorted_primitives.sort_by(|a, b| {
                let centroid_a = self.compute_primitive_centroid(&a);
                let centroid_b = self.compute_primitive_centroid(&b);
                centroid_a.z.partial_cmp(&centroid_b.z).unwrap_or(Ordering::Equal)
            }),
        }

        // Split primitives into two groups
        let mid = sorted_primitives.len() / 2;
        let left_primitives = sorted_primitives[..mid].to_vec();
        let right_primitives = sorted_primitives[mid..].to_vec();

        // Recursively build left and right subtrees
        let left = Box::new(self.build_node(&left_primitives));
        let right = Box::new(self.build_node(&right_primitives));

        BVHNode::Branch { aabb, left, right }
    }

    fn compute_primitive_aabb(&self, primitive: &Arc<Box<dyn Primitive>>) -> AABB {
        // Use the primitive's compute_bounding_box method to get a tight AABB
        primitive.compute_bounding_box()
    }

    fn compute_primitive_centroid(&self, primitive: &Arc<Box<dyn Primitive>>) -> Coords {
        // Use the primitive's compute_centroid method to get the actual centroid
        primitive.compute_centroid()
    }

    fn compute_primitives_aabb(&self, primitives: &[Arc<Box<dyn Primitive>>]) -> AABB {
        if primitives.is_empty() {
            return AABB {
                min: Coords { x: 0.0, y: 0.0, z: 0.0 },
                max: Coords { x: 0.0, y: 0.0, z: 0.0 },
            };
        }

        let mut aabb = self.compute_primitive_aabb(&primitives[0]);

        for primitive in primitives.iter().skip(1) {
            let primitive_aabb = self.compute_primitive_aabb(primitive);
            aabb = aabb.merge(&primitive_aabb);
        }

        aabb
    }

    pub fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        match &self.root {
            Some(root) => root.intersect(ray),
            None => None,
        }
    }
}

impl fmt::Display for BVH {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.root {
            Some(_) => write!(f, "BVH with root node"),
            None => write!(f, "Empty BVH"),
        }
    }
}
