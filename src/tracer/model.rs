use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use uuid::Uuid;

use crate::tracer::bvh::BVH;
use crate::tracer::coords::Coords;
use crate::tracer::misc_types::{Fov, Screen};
use crate::tracer::primitives::cone::Cone;
use crate::tracer::primitives::polygon::Polygon;
use crate::tracer::primitives::sphere::Sphere;
use crate::tracer::primitives::Primitive;
use crate::tracer::shader::color::Color;
use crate::tracer::shader::light::Light;

#[derive(Debug)]
pub struct Model {
    pub background: Color,
    pub eyep: Coords,
    pub lookp: Coords,
    pub up: Coords,
    pub fov: Fov,
    pub screen: Screen,
    pub light_sources: Vec<Light>,
    pub spheres: HashMap<Uuid, Sphere>,
    pub cones: HashMap<Uuid, Cone>,
    pub polygons: HashMap<Uuid, Polygon>,
    pub all_primitives: HashMap<Uuid, Box<dyn Primitive>>,
    pub bvh: BVH,
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
    pub fn new(input_file_path: &Path) -> Result<Self, ModelError> {
        let open_file_result = match File::open(input_file_path) {
            Ok(input_file) => input_file,
            Err(error) => {
                return Err(ModelError::FailedToOpenInputFile(error.to_string()));
            }
        };

        let file_reader = BufReader::new(open_file_result);
        crate::tracer::parse(file_reader)
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
              "spheres": [
            {}
              ]
            }}"#,
            self.background,
            self.eyep,
            self.lookp,
            self.up.x,
            self.up.y,
            self.up.z,
            self.fov,
            self.screen,
            self.spheres
                .values()
                .map(|sphere| format!("    {}", sphere))
                .collect::<Vec<String>>()
                .join(",\n")
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
            all_primitives: self.all_primitives.clone(),
            bvh: self.bvh.clone(),
        }
    }
}
