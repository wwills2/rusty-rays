use std::fmt;
use std::fs::File;
use std::io::BufReader;

use crate::tracer::color::Color;
use crate::tracer::point::Point;
use crate::tracer::sphere::Sphere;
use crate::tracer::types::{Fov, Screen};

pub struct Model {
    pub background: Color,
    pub eyep: Point,
    pub lookp: Point,
    pub up: Point,
    pub fov: Fov,
    pub screen: Screen,
    pub spheres: Vec<Sphere>,
}

impl Model {
    pub fn new(input_file_path: &str) -> Result<Self, ModelError> {
        let open_file_result = match File::open(input_file_path) {
            Ok(input_file) => input_file,
            Err(error) => {
                return Err(ModelError::FailedToOpenInputFile(error.to_string()));
            }
        };

        let file_reader = BufReader::new(open_file_result);
        return match crate::tracer::parse(file_reader) {
            Ok(model) => Ok(model),
            Err(error) => Err(error),
        };
    }
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\n{{\n  background: {},\n  eyep: {},\n  lookp: {},\n  up: ({}, {}, {}),\n  fov: {},\n  screen: {},\n  spheres: [\n{}  ]\n}}",
            self.background,
            self.eyep,
            self.lookp,
            self.up.x, self.up.y, self.up.x,
            self.fov,
            self.screen,
            self.spheres
                .iter()
                .map(|s| format!("    {},", s))
                .collect::<Vec<String>>()
                .join("\n")               
        )
    }
}

#[derive(Debug)]
pub enum ModelError {
    FailedToOpenInputFile(String),
    ErrorParsingInputFile(usize, String),
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelError::FailedToOpenInputFile(error_message) => {
                write!(f, "Failed to open input file: {}", error_message)
            }
            ModelError::ErrorParsingInputFile(line_number, error_message) => {
                write!(
                    f,
                    "Error parsing input file at line {line_number}. Error: {}",
                    error_message
                )
            }
        }
    }
}

impl std::error::Error for ModelError {}
