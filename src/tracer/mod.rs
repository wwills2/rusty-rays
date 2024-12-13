use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

use slog::{debug, info, trace};

use coords::Coords;

use crate::tracer::color::Color;
use crate::tracer::model::{Model, ModelError};
use crate::tracer::sphere::Sphere;
use crate::tracer::types::{Entity, Fov, Screen, Surface};
use crate::utils::logger::LOG;

mod color;
mod coords;
mod input_file_parser;
pub mod model;
mod sphere;
mod types;

static ENTITY_NAMES: [&str; 1] = [sphere::NAME];

pub struct Tracer {
    model: Model,
    primary_rays: Vec<Ray>,
}

struct Ray {
    i: usize,
    j: usize,
    coords: Coords,
}

impl Tracer {
    pub fn new(model: Model) -> Self {
        debug!(
            LOG,
            "initializing renderer and calculating primary ray definitions"
        );
        let primary_rays = Self::calculate_primary_rays(&model);
        Tracer {
            model,
            primary_rays,
        }
    }
    pub fn render(&self) -> Result<Vec<Vec<Color>>, RenderError> {
        info!(LOG, "Rendering model");
        let mut raw_image_data =
            vec![vec![Color::new(); self.model.screen.width]; self.model.screen.height];

        for ray in &self.primary_rays {
            println!("ray for image pixel ({}, {}): {}", ray.i, ray.j, ray.coords);
        }

        Ok(raw_image_data)
    }

    fn calculate_primary_rays(model: &Model) -> Vec<Ray> {
        let direction = model.lookp - model.eyep;
        let forward = direction.calc_normalized_vector();
        let right = forward.cross(&model.up).calc_normalized_vector();
        let true_up = right.cross(&forward);

        let focal_len = (model.lookp - model.eyep).calc_vector_length();
        let screen_plane_width = 2.0 * focal_len * f64::tan(model.fov.horz / 2.0);
        let screen_plane_height = 2.0 * focal_len * f64::tan(model.fov.vert / 2.0);

        debug!(
            LOG,
            "calculating primary rays. details:
direction vec: {}
forward vec: {}
right vec: {}
true-up vec: {}
focal len: {}
screen plane width: {}
screen plane height: {}",
            direction,
            forward,
            right,
            true_up,
            focal_len,
            screen_plane_width,
            screen_plane_width
        );

        let mut rays: Vec<Ray> = Vec::new();

        let calc_ray_definition = |i, j| -> Coords {
            let horz_pos = ((i as f64 + 0.5) / model.screen.width as f64) - 0.5;
            let vert_pos = 0.5 - ((j as f64 + 0.5) / model.screen.height as f64);

            let pixel_pos = model.lookp
                + (right * screen_plane_width * horz_pos)
                + (true_up * vert_pos * screen_plane_height);
            trace!(
                LOG,
                "position of image plane pixel (i: {}, j: {}); {}",
                i,
                j,
                pixel_pos
            );

            pixel_pos.calc_normalized_vector()
        };

        for i in 0..model.screen.height {
            for j in 0..model.screen.width {
                let coords = calc_ray_definition(i, j);
                trace!(
                    LOG,
                    "calculated definition of ray through image plane pixel position (i: {}, j:{}) to be {} ",
                    i,
                    j,
                    coords
                );
                rays.push(Ray { i, j, coords });
            }
        }

        return rays;
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
pub struct RenderError(String);

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to render model. Error: {}", self.0)
    }
}

impl std::error::Error for RenderError {}
