use crate::tracer::misc_types::{Fov, Screen};
use crate::tracer::primitives::Sphere;
use crate::tracer::primitives::{Cone, Polygon};
use crate::tracer::primitives::{Plane, Primitive, Triangle};
use crate::tracer::rayshade4_file_parser;
use crate::tracer::shader::light::Light;
use crate::tracer::shader::Color;
use crate::tracer::Coords;
use crate::utils::logger::{trace, LOG};
use std::collections::HashMap;
use std::convert::Into;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug)]
pub struct Model {
    pub background: Color,
    pub eyep: Coords,
    pub lookp: Coords,
    pub up: Coords,
    pub fov: Fov,
    pub screen: Screen,
    pub light_sources: Vec<Light>,
    spheres: HashMap<Uuid, Sphere>,
    cones: HashMap<Uuid, Cone>,
    polygons: HashMap<Uuid, Polygon>,
    triangles: HashMap<Uuid, Triangle>,
    all_primitives: HashMap<Uuid, Box<dyn Primitive>>,
}

#[derive(Debug)]
pub enum ModelError {
    FailedToOpenInputFile(String),
    FailedToParseInputFile(usize, String),
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelError::FailedToOpenInputFile(error_message) => {
                write!(f, "Failed to open input file: {}", error_message)
            }
            ModelError::FailedToParseInputFile(line_number, error_message) => {
                write!(
                    f,
                    "Failed to parse input file at line {line_number}. Error: {}",
                    error_message
                )
            }
        }
    }
}

impl std::error::Error for ModelError {}

impl Model {
    pub fn from_file(input_file: File) -> Result<Self, ModelError> {
        let file_reader = BufReader::new(input_file);
        let model = rayshade4_file_parser::iterate_input_data(file_reader.lines().peekable())?;
        trace!(LOG, "parsed model from file:\n{}", model);
        Ok(model)
    }

    pub fn from_file_path(path_to_input_file: PathBuf) -> Result<Self, ModelError> {
        match File::open(path_to_input_file) {
            Ok(input_file) => Ok(Self::from_file(input_file)?),
            Err(error) => Err(ModelError::FailedToOpenInputFile(error.to_string())),
        }
    }

    pub fn from_string(input_string: String) -> Result<Self, ModelError> {
        let cursor = Cursor::new(input_string);
        let reader = BufReader::new(cursor);
        let model = rayshade4_file_parser::iterate_input_data(reader.lines().peekable())?;
        trace!(LOG, "parsed model from input string:\n{}", model);
        Ok(model)
    }

    #[inline]
    pub fn get_all_primitives(&self) -> &HashMap<Uuid, Box<dyn Primitive>> {
        &self.all_primitives
    }

    #[inline]
    pub fn get_all_spheres(&self) -> &HashMap<Uuid, Sphere> {
        &self.spheres
    }

    #[inline]
    pub fn get_all_polygons(&self) -> &HashMap<Uuid, Polygon> {
        &self.polygons
    }

    #[inline]
    pub fn get_all_triangles(&self) -> &HashMap<Uuid, Triangle> {
        &self.triangles
    }

    #[inline]
    pub fn get_all_cones(&self) -> &HashMap<Uuid, Cone> {
        &self.cones
    }

    #[inline]
    pub fn upsert_sphere(&mut self, sphere: Sphere) -> Option<Sphere> {
        let uuid = sphere.uuid;
        let sphere_clone = sphere.clone();
        self.all_primitives.insert(uuid, Box::new(sphere));
        self.spheres.insert(uuid, sphere_clone)
    }

    #[inline]
    pub fn delete_sphere(&mut self, uuid: Uuid) -> Option<Sphere> {
        self.all_primitives.remove(&uuid);
        self.spheres.remove(&uuid)
    }

    #[inline]
    pub fn upsert_cone(&mut self, cone: Cone) -> Option<Cone> {
        let uuid = cone.uuid;
        let cone_clone = cone.clone();
        self.all_primitives.insert(uuid, Box::new(cone));
        self.cones.insert(uuid, cone_clone)
    }

    #[inline]
    pub fn delete_cone(&mut self, uuid: Uuid) -> Option<Cone> {
        self.all_primitives.remove(&uuid);
        self.cones.remove(&uuid)
    }

    #[inline]
    pub fn upsert_polygon(&mut self, polygon: Polygon) -> Option<Polygon> {
        let uuid = polygon.uuid;
        let polygon_clone = polygon.clone();
        self.all_primitives.insert(uuid, Box::new(polygon));
        self.polygons.insert(uuid, polygon_clone)
    }

    #[inline]
    pub fn delete_polygon(&mut self, uuid: Uuid) -> Option<Polygon> {
        self.all_primitives.remove(&uuid);
        self.polygons.remove(&uuid)
    }

    #[inline]
    pub fn upsert_triangle(&mut self, triangle: Triangle) -> Option<Triangle> {
        let uuid = triangle.uuid;
        let triangle_clone = triangle.clone();
        self.all_primitives.insert(uuid, Box::new(triangle));
        self.triangles.insert(uuid, triangle_clone)
    }

    #[inline]
    pub fn delete_triangle(&mut self, uuid: Uuid) -> Option<Triangle> {
        self.all_primitives.remove(&uuid);
        self.triangles.remove(&uuid)
    }
}

impl Default for Model {
    fn default() -> Self {
        Self {
            background: Color::default(),
            eyep: Coords::default(),
            lookp: Coords::default(),
            up: Coords::default(),
            fov: Fov::default(),
            screen: Screen::default(),
            light_sources: vec![],
            spheres: HashMap::default(),
            cones: HashMap::default(),
            polygons: HashMap::default(),
            triangles: HashMap::default(),
            all_primitives: HashMap::default(),
        }
    }
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"{{
              "background": {},
              "eyep": {},
              "lookp": {},
              "up": {{
                "x": {},
                "y": {},
                "z": {}
              }},
              "fov": {},
              "screen": {},
            }}"#,
            self.background,
            self.eyep,
            self.lookp,
            self.up.x,
            self.up.y,
            self.up.z,
            self.fov,
            self.screen,
        )
    }
}

impl Clone for Model {
    fn clone(&self) -> Self {
        Model {
            background: self.background.clone(),
            eyep: self.eyep.clone(),
            lookp: self.lookp.clone(),
            up: self.up.clone(),
            fov: Fov {
                horz: self.fov.horz,
                vert: self.fov.vert,
            },
            screen: Screen {
                width: self.screen.width,
                height: self.screen.height,
            },
            light_sources: self.light_sources.clone(),
            spheres: self.spheres.clone(),
            cones: self.cones.clone(),
            polygons: self.polygons.clone(),
            triangles: self.triangles.clone(),
            all_primitives: self.all_primitives.clone(),
        }
    }
}
