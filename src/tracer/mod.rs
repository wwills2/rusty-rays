use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path};
use image::{ImageBuffer, RgbImage};
use slog::{debug, info, trace};

use coords::Coords;

use crate::tracer::color::Color;
use crate::tracer::model::{Model, ModelError};
use crate::tracer::types::Entity;
use crate::utils::logger::LOG;

mod color;
mod coords;
mod input_file_parser;
pub mod model;
mod polygon;
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
        info!(LOG, "rendering model");
        let mut raw_image_data =
            vec![vec![Color::new(); self.model.screen.width]; self.model.screen.height];

        let ten_percent = &self.primary_rays.len() / 10;
        let mut block = 1;
        for (i ,ray) in self.primary_rays.iter().enumerate() {
            if i != 0 && i % ten_percent == 0 {
                info!(LOG, "rendering {}% complete", block * 10);
                block += 1;
            }
            raw_image_data[ray.i][ray.j] = self.calculate_primary_ray_color(ray).clone();
        }

        Ok(raw_image_data)
    }

    fn calculate_primary_ray_color(&self, ray: &Ray) -> &Color {
        trace!(LOG, "Calculating primary ray color for pixel ({}, {})", ray.i, ray.j);

        let mut closest_entity: Option<&dyn Entity> = None;
        let mut intersection_distance: f64 = f64::INFINITY;

        for entity in self.model.all_entity_iter() {
            let maybe_intersection_distances =
                entity.calculate_intersection_distances(&ray.coords, &self.model.eyep);
            if maybe_intersection_distances.is_none() {
                continue;
            }

            let intersection_distances = maybe_intersection_distances.unwrap();
            let mut set_entity = false;
            for distance in intersection_distances {
                if distance < intersection_distance {
                    intersection_distance = distance;
                    set_entity = true;
                }
            }

            if set_entity {
                closest_entity = Some(entity);
            }
        }

        match closest_entity {
            Some(entity) => {
                let intersection_point = self.model.eyep + ray.coords * intersection_distance;
                entity.calculate_color(&intersection_point)
            },
            None => {
                &self.model.background
            }
        }

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

            let camera_ray_direction = (pixel_pos - model.eyep).calc_normalized_vector();
            return camera_ray_direction;
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

        rays
    }
}

pub fn write(output_file_path: &Path, raw_image_data: &Vec<Vec<Color>>) -> Result<(), WriteError> {
    let height = raw_image_data.len();
    let width = raw_image_data[0].len();
    let normalized_image_data = normalize_and_flatten_to_u8_rgb(raw_image_data);

    let maybe_image: Option<RgbImage> = ImageBuffer::from_raw(width as u32, height as u32, normalized_image_data);
    let image = match maybe_image {
        Some(image) => image,
        None => return Err(WriteError("cannot write raw render data into image format".to_string()))
    };

    match image.save(output_file_path) {
        Ok(_) => Ok(()),
        Err(error) => Err(WriteError(format!("cannot write image data to {}. Error: {}", output_file_path.display(), error)))
    }
}

fn normalize_and_flatten_to_u8_rgb(image_data: &Vec<Vec<Color>>) -> Vec<u8> {
    let mut normalized_image_data: Vec<u8> = Vec::new();
    for row in image_data {
        for color in row {
            let normalized_color = color.normalize();
            normalized_image_data.push(normalized_color.r);
            normalized_image_data.push(normalized_color.g);
            normalized_image_data.push(normalized_color.b);
        }
    }

    normalized_image_data
}

fn parse(input_file_buf_reader: BufReader<File>) -> Result<Model, ModelError> {
    match input_file_parser::iterate_input_data(input_file_buf_reader.lines().peekable()) {
        Ok(model) => {
            trace!(LOG, "parsed model:\n{}", model);
            Ok(model)
        }
        Err(error) => Err(error),
    }
}

#[derive(Debug)]
pub struct RenderError(String);

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to render model. Error: {}", self.0)
    }
}

impl std::error::Error for RenderError {}

#[derive(Debug)]
pub struct WriteError(String);

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to write render data to file. Error: {}", self.0)
    }
}

impl std::error::Error for WriteError {}
