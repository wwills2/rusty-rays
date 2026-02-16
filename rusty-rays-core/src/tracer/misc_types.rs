use std::fmt;

use uuid::Uuid;

use crate::tracer::Color;
use crate::tracer::Coords;

// fov type and methods
#[derive(Debug)]
pub struct Fov {
    pub horz: f64,
    pub vert: f64,
}

impl Default for Fov {
    fn default() -> Self {
        Self {
            horz: 45.0,
            vert: 45.0,
        }
    }
}

impl fmt::Display for Fov {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\n{{\n  horz: {}, \n  vert: {},\n}}",
            self.horz, self.vert
        )
    }
}

// screen type and methods
#[derive(Debug)]
pub struct Screen {
    pub width: usize,
    pub height: usize,
}

impl Default for Screen {
    fn default() -> Self {
        Self {
            width: 500,
            height: 500,
        }
    }
}

impl fmt::Display for Screen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\n{{\n  width: {}, \n  height: {},\n}}",
            self.width, self.height
        )
    }
}

// surface type and methods
#[derive(Debug)]
pub struct Surface {
    pub name: String,
    pub ambient: Color,
    pub diffuse: Color,
    pub specular: Color,
    pub specpow: f64,
    pub reflect: f64,
}

impl Clone for Surface {
    fn clone(&self) -> Self {
        Surface {
            name: self.name.clone(),
            ambient: self.ambient.clone(),
            diffuse: self.diffuse.clone(),
            specular: self.specular.clone(),
            specpow: self.specpow,
            reflect: self.reflect,
        }
    }
}

impl fmt::Display for Surface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\n{{\n  name: \"{}\",\n  ambient: {},\n  diffuse: {},\n  specular: {},\n  specpow: {},\n  reflect: {},\n}}",
            self.name, self.ambient, self.diffuse, self.specular, self.specpow, self.reflect
        )
    }
}

// ray type and methods
#[derive(Debug)]
pub struct Ray {
    pub i: usize,
    pub j: usize,
    pub direction: Coords,
    pub origin: Coords,
}

impl Clone for Ray {
    fn clone(&self) -> Self {
        Ray {
            i: self.i,
            j: self.j,
            direction: self.direction.clone(),
            origin: self.origin.clone(),
        }
    }
}

// intersection type and methods
pub struct Intersection {
    pub primitive_type: String,
    pub distance_along_ray: f64,
    pub ray: Ray,
    pub position: Coords,
    pub surface_normal_at_intersection: Coords,
    pub intersected_primitive_uuid: Uuid,
}
