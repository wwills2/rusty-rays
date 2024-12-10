use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

use slog::{info, Logger, trace};

use coords::Coords;

use crate::tracer::model::{Model, ModelError};
use crate::tracer::sphere::Sphere;
use crate::tracer::types::{Entity, Fov, Screen, Surface};
use crate::utils::logger;
use crate::utils::logger::LOG;

mod color;
mod coords;
mod input_file_parser;
pub mod model;
mod sphere;
mod types;

static ENTITY_NAMES: [&str; 1] = [sphere::NAME];

pub struct Renderer {
    model: Model,
    image_plane_coords: Vec<Vec<Coords>>,
    rays: Vec<Vec<Coords>>,
}

impl Renderer {
    pub fn new(model: Model) -> Self {
        let image_plane_coords = Self::calculate_image_plane_coords(&model);
        let rays = Self::calculate_rays(&image_plane_coords, &model);

        Renderer {
            model,
            image_plane_coords,
            rays,
        }
    }
    pub fn render(model: Model) -> Result<Vec<Vec<f64>>, RenderError> {
        info!(LOG, "Rendering model");
        todo!()
    }
    fn calculate_image_plane_coords(model: &Model) -> Vec<Vec<Coords>> {
        let direction = model.lookp - model.eyep;
        let forward = direction.calc_normalized_vector();
        let right = (forward * model.up).calc_normalized_vector();
        let true_up = right * forward;

        let focal_len = (model.lookp - model.eyep).calc_vector_length();
        let screen_plane_width = 2.0 * focal_len * f64::tan(model.fov.horz / 2.0);
        let screen_plane_height = 2.0 * focal_len * f64::tan(model.fov.vert / 2.0);

        let mut screen_pixel_coords: Vec<Vec<Coords>> = vec![
            vec![
                Coords {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0
                };
                model.screen.width
            ];
            model.screen.height
        ];

        let calc_pixel_coords = |i, j| -> Coords {
            let horz_pos = ((i as f64 + 0.5) / model.screen.width as f64) - 0.5;
            let vert_pos = 0.5 - ((j as f64 + 0.5) / model.screen.height as f64);

            model.lookp
                + (right * (screen_plane_width * horz_pos))
                + (true_up * (vert_pos * screen_plane_height))
        };

        for i in 0..screen_pixel_coords.len() {
            for j in 0..screen_pixel_coords[i].len() {
                screen_pixel_coords[i][j] = calc_pixel_coords(i, j)
            }
        }

        todo!()
    }

    fn calculate_rays(image_plane: &Vec<Vec<Coords>>, model: &Model) -> Vec<Vec<Coords>> {
        todo!()
    }
}

pub fn write() {}

fn parse(input_file_buf_reader: BufReader<File>) -> Result<Model, ModelError> {
    return match input_file_parser::iterate_input_data(input_file_buf_reader.lines().peekable()) {
        Ok(model) => {
            trace!(LOG, "parsed model:\n{}", model);
            Ok(model)
        }
        Err(error) => Err(error),
    };
}

#[derive(Debug)]
struct RenderError(String);

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to render model. Error: {}", self.0)
    }
}

impl std::error::Error for RenderError {}
