use std::fmt;

use uuid::Uuid;

use crate::tracer::coords::Coords;
use crate::tracer::shader::color::Color;

// fov type and methods
#[derive(Debug)]
pub struct Fov {
    pub horz: f64,
    pub vert: f64,
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
#[derive(Debug, Copy)]
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
    pub distance_along_ray: f64,
    pub location: Coords,
    pub normal_vector: Coords,
    pub uuid: Uuid,
}

pub struct SphereSurfaceNormalKey {
    pub intersection_point: Coords,
    pub uuid: Uuid,
}

// entity trait and methods
pub trait Entity: Send + Sync {
    fn get_uuid(&self) -> Uuid;
    fn get_type(&self) -> String;
    fn get_surface(&self) -> &Surface;
    fn calculate_intersection(&self, ray: &Ray) -> Option<Intersection>;
    fn entity_clone(&self) -> Box<dyn Entity>;
}

impl fmt::Debug for dyn Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "entity of type {} with uuid {}",
            self.get_type(),
            self.get_uuid()
        )
    }
}

impl Clone for Box<dyn Entity> {
    fn clone(&self) -> Self {
        self.entity_clone()
    }
}
