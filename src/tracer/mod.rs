use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

use slog::{info, Logger};

use crate::tracer::model::{Model, ModelError};
use crate::tracer::sphere::Sphere;
use crate::tracer::types::{Color, Entity, Fov, Point, Screen, Surface};
use crate::utils::logger;

mod input_file_parser;
pub mod model;
mod sphere;
mod types;

static ENTITY_NAMES: [&str; 1] = [sphere::NAME];

pub fn render(model: &Model) {
    let logger = &logger::LOGGER;
    info!(logger, "Rendering model from input file");
}

pub fn write() {}

fn parse(input_file_buf_reader: BufReader<File>) -> Result<Model, ModelError> {
    input_file_parser::iterate_input_data(input_file_buf_reader.lines().peekable().enumerate())
}
