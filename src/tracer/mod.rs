use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

use slog::{info, Logger};

use crate::tracer::model::{Model, ModelError};
use crate::tracer::sphere::Sphere;
use crate::tracer::types::{Color, Entity, Fov, Point, Screen, Surface};
use crate::utils::logger;

pub mod model;
mod sphere;
mod types;

pub fn render(model: &Model) {
    let logger = &logger::LOGGER;
    info!(logger, "Rendering model from input file");
}

pub fn write() {}

fn parse(input_file_buf_reader: BufReader<File>) -> Result<Model, ModelError> {
    let mut background = Color { r: 0, g: 0, b: 0 };
    let mut eyep = Point {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let mut lookp = Point {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let mut up = (0u8, 0u8, 0u8);
    let mut fov = Fov { horz: 0, vert: 0 };
    let screen = Screen {
        width: 0,
        height: 0,
    };
    let mut spheres: Vec<Sphere> = Vec::new();
    let mut surfaces: HashMap<String, Surface> = HashMap::new();

    for (line_number, line_read_result) in input_file_buf_reader.lines().enumerate() {
        if line_read_result.is_err() {
            return Err(ModelError::ErrorParsingInputFile(format!(
                "error reading input file at line {}. Error: {}",
                line_number,
                line_read_result.err().unwrap()
            )));
        }

        let line_iter = line_read_result.unwrap();
        let mut line_split_iter = line_iter.split(' ');
        let maybe_keyword = line_split_iter.next();

        // line has no content, got to next line
        if maybe_keyword.is_none() {
            continue;
        }

        let keyword = maybe_keyword.unwrap();

        println!("keyword is {}", keyword);
        let mut line_parts = String::new();
        let mut next_word = line_split_iter.next();
        while next_word.is_some() {
            line_parts.push_str(next_word.unwrap());

            next_word = line_split_iter.next();

            if next_word.is_some() {
                line_parts.push_str(" | ")
            }
        }
        println!("rest of the line is {}", line_parts)
    }

    return Ok(Model {
        background,
        eyep,
        lookp,
        up,
        fov,
        screen,
        spheres,
    });
}
