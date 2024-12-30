use crate::tracer::color::Color;
use crate::tracer::coords::Coords;
use crate::tracer::model::{Model, ModelError};
use crate::tracer::types::Entity;
use crate::utils::logger::LOG;
use image::{ImageBuffer, RgbImage};
use num_cpus;
use slog::{debug, error, info, trace, warn};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{fmt, sync, thread};

mod color;
mod coords;
mod input_file_parser;
pub mod model;
mod polygon;
mod sphere;
mod types;

static ENTITY_NAMES: [&str; 1] = [sphere::NAME];

#[derive(Debug, Copy)]
struct _Ray {
    i: usize,
    j: usize,
    coords: Coords,
}

impl Clone for _Ray {
    fn clone(&self) -> Self {
        _Ray {
            i: self.i,
            j: self.j,
            coords: self.coords.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Tracer {
    model: Model,
    primary_rays: Vec<_Ray>,
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
        let num_cores = num_cpus::get_physical();
        let max_thread_num = num_cores - 1;
        let rays_per_thread: usize = self.primary_rays.len() / num_cores;
        let surplus_rays = self.primary_rays.len() - (rays_per_thread * num_cores);
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

        for thread_num in 0..num_cores {
            // these arc clones do not clone the underlying data
            let _image_data_arc_clone = Arc::clone(&image_data_arc);
            let _self_arc_clone = Arc::clone(&self_arc);
            let _counter_mutex_arc_clone = Arc::clone(&counter_mutex_arc);
            let _total_work_arc_clone = Arc::clone(&total_work_arc);
            let _ten_percent_arc_clone = Arc::clone(&ten_percent_arc);
            let _progress_block_mutex_arc_clone = Arc::clone(&progress_block_mutex_arc);

            let start_index = thread_num * rays_per_thread;
            let end_index = if thread_num == num_cores - 1 {
                start_index + rays_per_thread + surplus_rays
            } else {
                start_index + rays_per_thread
            };

            let handle = thread::spawn(move || {
                info!(LOG, "starting render thread #{}", thread_num);

                for ray_index in start_index..end_index {
                    let ray = &_self_arc_clone.primary_rays[ray_index];

                    // block to release counter guard ASAP
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

                    let pixel_color = _self_arc_clone.calculate_primary_ray_color(ray).clone();
                    match _image_data_arc_clone.lock() {
                        Ok(mut mutex_guard) => {
                            mutex_guard[ray.i][ray.j] = pixel_color;
                        }
                        Err(error) => {
                            warn!(LOG, "thread {} encountered poisoned data error from mutex when writing color for pixel ({}, {}). proceeding with render", thread_num, ray.i, ray.j);
                        }
                    }
                }
            });

            thread_handles.push(handle);
        }

        let mut thread_error = false;
        for handle in thread_handles {
            match handle.join() {
                Ok(_) => {}
                Err(_) => {
                    error!(LOG, "failed to join renderer thread");
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

    fn calculate_primary_ray_color(&self, ray: &_Ray) -> &Color {
        trace!(
            LOG,
            "Calculating primary ray color for pixel ({}, {})",
            ray.i,
            ray.j
        );

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
            }
            None => &self.model.background,
        }
    }

    fn calculate_primary_rays(model: &Model) -> Vec<_Ray> {
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

        let mut rays: Vec<_Ray> = Vec::new();

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
                rays.push(_Ray { i, j, coords });
            }
        }

        rays
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
