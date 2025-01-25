use std::{f64, fmt, thread};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::{Arc, Mutex};

use image::{ImageBuffer, RgbImage};
use slog::{debug, error, info, trace, warn};

use shader::color::Color;

use crate::tracer::coords::Coords;
use crate::tracer::misc_types::{Intersection, Ray};
use crate::tracer::model::{Model, ModelError};
use crate::utils::config::CONFIG;
use crate::utils::logger::LOG;

mod coords;
mod misc_types;
pub mod model;
mod plane_coords;
mod polygon;
mod rayshade4_file_parser;
mod shader;
mod sphere;

#[derive(Debug)]
pub struct Tracer {
    model: Model,
    primary_rays: Vec<Ray>,
}

impl Clone for Tracer {
    fn clone(&self) -> Self {
        Tracer {
            model: self.model.clone(),
            primary_rays: self.primary_rays.clone(),
        }
    }
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
        let self_arc = Arc::new(self.clone());
        Self::_render(self_arc)
    }

    fn _render(self: Arc<Self>) -> Result<Vec<Vec<Color>>, RenderError> {
        let num_threads = CONFIG.max_render_threads;
        let max_thread_num = num_threads - 1;
        let rays_per_thread: usize = self.primary_rays.len() / num_threads;
        let surplus_rays = self.primary_rays.len() - (rays_per_thread * num_threads);
        let mut thread_handles = vec![];

        let counter_mutex_arc = Arc::new(Mutex::new(0usize));
        let total_work_arc = Arc::new(self.primary_rays.len());
        let ten_percent_arc = Arc::new(self.primary_rays.len() / 10);
        let progress_block_mutex_arc = Arc::new(Mutex::new(1usize));

        let image_data_arc = Arc::new(Mutex::new(vec![
            vec![Color::new(); self.model.screen.width];
            self.model.screen.height
        ]));
        let self_arc = Arc::new(self);

        for thread_num in 0..num_threads {
            // these arc clones do not clone the underlying data
            let _image_data_arc_clone = Arc::clone(&image_data_arc);
            let _self_arc_clone = Arc::clone(&self_arc);
            let _counter_mutex_arc_clone = Arc::clone(&counter_mutex_arc);
            let _total_work_arc_clone = Arc::clone(&total_work_arc);
            let _ten_percent_arc_clone = Arc::clone(&ten_percent_arc);
            let _progress_block_mutex_arc_clone = Arc::clone(&progress_block_mutex_arc);

            let start_index = thread_num * rays_per_thread;
            let end_index = if thread_num == max_thread_num {
                start_index + rays_per_thread + surplus_rays
            } else {
                start_index + rays_per_thread
            };

            let handle = thread::spawn(move || {
                info!(LOG, "starting render thread #{}", thread_num);

                for ray_index in start_index..end_index {
                    let ray = &_self_arc_clone.primary_rays[ray_index];

                    // code block to release counter guard ASAP
                    {
                        match _counter_mutex_arc_clone.lock() {
                            Ok(mut counter_mutex_guard) => {
                                match _progress_block_mutex_arc_clone.lock() {
                                    Ok(mut progress_block_mutex_guard) => {
                                        if *counter_mutex_guard != 0
                                            && *counter_mutex_guard % *_ten_percent_arc_clone == 0
                                        {
                                            info!(
                                                LOG,
                                                "rendering {}% complete",
                                                *progress_block_mutex_guard * 10
                                            );
                                            *progress_block_mutex_guard += 1;
                                        }
                                        *counter_mutex_guard += 1;
                                    }
                                    Err(_) => {
                                        warn!(LOG, "thread {} encountered poisoned data error from mutex when updating counter. proceeding with render", thread_num);
                                    }
                                }
                            }
                            Err(_) => {
                                warn!(LOG, "thread {} encountered poisoned data error from mutex when updating counter. proceeding with render", thread_num);
                            }
                        }
                    }

                    let pixel_color = shader::process_ray(0, ray, &_self_arc_clone.model);

                    match _image_data_arc_clone.lock() {
                        Ok(mut mutex_guard) => {
                            mutex_guard[ray.i][ray.j] = pixel_color;
                        }
                        Err(_) => {
                            warn!(LOG, "thread {} encountered poisoned data error from mutex when writing color for pixel ({}, {}). proceeding with render", thread_num, ray.i, ray.j);
                        }
                    }
                }
            });

            thread_handles.push(handle);
        }

        let mut thread_error = false;
        for (joined_thread_num, handle) in thread_handles.into_iter().enumerate() {
            match handle.join() {
                Ok(_) => {
                    info!(LOG, "render thread #{} finished", joined_thread_num);
                }
                Err(_) => {
                    error!(LOG, "failed to join renderer thread #{}", joined_thread_num);
                    thread_error = true;
                }
            }
        }

        if thread_error {
            return Err(RenderError(
                "render threads did not exit properly".to_string(),
            ));
        }

        match Arc::try_unwrap(image_data_arc) {
            Ok(image_data_mutex) => match image_data_mutex.into_inner() {
                Ok(raw_image_data) => Ok(raw_image_data),
                Err(error) => Err(RenderError(format!(
                    "failed to get data from mutex. {}",
                    error
                ))),
            },
            Err(_) => Err(RenderError(
                "an asynchronous resource sharing error occurred".to_string(),
            )),
        }
    }

    fn calculate_primary_rays(model: &Model) -> Vec<Ray> {
        let direction = &model.lookp - &model.eyep;
        let forward = direction.calc_normalized_vector();
        let right = forward.cross(&model.up).calc_normalized_vector();
        let true_up = right.cross(&forward);

        let focal_len = direction.calc_vector_length();
        let screen_plane_width =
            2.0 * focal_len * f64::tan((model.fov.horz / 2.0) * (f64::consts::PI / 180.0));
        let screen_plane_height =
            2.0 * focal_len * f64::tan((model.fov.vert / 2.0) * (f64::consts::PI / 180.0));

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
            screen_plane_height
        );

        let mut rays: Vec<Ray> = Vec::new();

        let calc_ray_definition = |i, j| -> Coords {
            let horz_pos = ((j as f64 + 0.5) / model.screen.width as f64) - 0.5;
            let vert_pos = 0.5 - ((i as f64 + 0.5) / model.screen.height as f64);

            let pixel_pos = &model.lookp
                + &(&right * (screen_plane_width * horz_pos))
                + (&true_up * (vert_pos * screen_plane_height));
            trace!(
                LOG,
                "position of image plane pixel (i: {}, j: {}); {}",
                i,
                j,
                pixel_pos
            );

            (pixel_pos - &model.eyep).calc_normalized_vector()
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
                rays.push(Ray {
                    i,
                    j,
                    direction: coords,
                    origin: model.eyep.clone(),
                });
            }
        }

        rays
    }
}

fn calculate_ray_closest_intersection(ray: &Ray, model: &Model) -> Option<Intersection> {
    trace!(
        LOG,
        "Calculating primary ray color for pixel ({}, {})",
        ray.i,
        ray.j
    );

    let mut closest_intersection: Option<Intersection> = None;

    for entity in model.all_entities.values() {
        if let Some(intersection) = entity.calculate_intersection(ray) {
            match closest_intersection {
                Some(ref current_intersection)
                    if intersection.distance_along_ray
                        < current_intersection.distance_along_ray =>
                {
                    closest_intersection = Some(intersection);
                }
                None => {
                    closest_intersection = Some(intersection);
                }
                _ => {}
            }
        }
    }

    closest_intersection
}

fn calculate_ray_first_intersection(ray: &Ray, model: &Model) -> Option<Intersection> {
    for entity in model.all_entities.values() {
        if let Some(intersection) = entity.calculate_intersection(ray) {
            return Some(intersection);
        }
    }

    None
}

pub fn write(output_file_path: &Path, raw_image_data: &Vec<Vec<Color>>) -> Result<(), WriteError> {
    let height = raw_image_data.len();
    let width = raw_image_data[0].len();
    let normalized_image_data = normalize_and_flatten_to_u8_rgb(raw_image_data);

    let maybe_image: Option<RgbImage> =
        ImageBuffer::from_raw(width as u32, height as u32, normalized_image_data);
    let image = match maybe_image {
        Some(image) => image,
        None => {
            return Err(WriteError(
                "cannot write raw render data into image format".to_string(),
            ))
        }
    };

    match image.save(output_file_path) {
        Ok(_) => Ok(()),
        Err(error) => Err(WriteError(format!(
            "cannot write image data to {}. Error: {}",
            output_file_path.display(),
            error
        ))),
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
    let model =
        rayshade4_file_parser::iterate_input_data(input_file_buf_reader.lines().peekable())?;
    trace!(LOG, "parsed model:\n{}", model);
    Ok(model)
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
