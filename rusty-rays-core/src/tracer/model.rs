use crate::tracer::misc_types::{Fov, Screen};
use crate::tracer::primitives::Cone;
use crate::tracer::primitives::Polygon;
use crate::tracer::primitives::Primitive;
use crate::tracer::primitives::Sphere;
use crate::tracer::rayshade4_file_parser;
use crate::tracer::shader::light::Light;
use crate::tracer::shader::Color;
use crate::tracer::Coords;
use crate::utils::logger::{trace, LOG};
use std::collections::HashMap;
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
    pub spheres: HashMap<Uuid, Sphere>,
    pub cones: HashMap<Uuid, Cone>,
    pub polygons: HashMap<Uuid, Polygon>,
    pub all_primitives: HashMap<Uuid, Box<dyn Primitive>>,
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
        }
    }
}
