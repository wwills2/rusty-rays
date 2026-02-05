use crate::tracer::camera::Camera;
use crate::utils::logger::{debug, error, info, trace, warn, LOG};
use crate::utils::Config;
use bvh::Bvh;
pub use coords::Coords;
use misc_types::Ray;
pub use misc_types::{Fov, Screen, Surface};
pub use model::Model;
pub use primitives::Cone;
pub use primitives::Plane;
pub use primitives::Polygon;
use primitives::Primitive;
pub use primitives::Sphere;
pub use primitives::Triangle;
pub use shader::Color;
use std::sync::{atomic, Arc, Mutex};
use std::time::SystemTime;
use std::{f64, fmt, thread};

mod bvh;
mod camera;
mod coords;
mod misc_types;
mod model;
mod plane_coords_2d;
mod primitives;
mod rayshade4_file_parser;
mod shader;

#[derive(Debug)]
pub struct Tracer {
    model: Model,
    bvh: Bvh,
    primary_rays: Vec<Ray>,
    camera: Camera,
}

impl Clone for Tracer {
    fn clone(&self) -> Self {
        Tracer {
            model: self.model.clone(),
            primary_rays: self.primary_rays.clone(),
            bvh: self.bvh.clone(),
            camera: self.camera.clone(),
        }
    }
}

impl Tracer {
    pub fn new(model: Model) -> Self {
        debug!(
            LOG,
            "initializing renderer. calculating primary ray definitions and bvh"
        );
        let camera = Camera::new(&model);
        let primary_rays = Self::calculate_primary_rays(&model, &camera);

        // Collect all primitives into a vector for Bvh construction
        let primitives_for_bvh: Vec<Box<dyn Primitive>> =
            model.get_all_primitives().values().cloned().collect();
        let mut bvh = Bvh::new();
        bvh.build(primitives_for_bvh);

        Self {
            model,
            primary_rays,
            bvh,
            camera,
        }
    }

    pub fn render(&self) -> Result<Vec<Vec<Color>>, RenderError> {
        info!(LOG, "rendering model");
        let self_arc = Arc::new(self.clone());
        Self::_render(self_arc)
    }

    pub fn get_intersected_uuid_by_pixel_pos(&self, x: usize, y: usize) -> Option<uuid::Uuid> {
        // i & j refer to 2d array indices -> transpose of x & y pixel positions
        let ray = self.camera.calc_ray_definition(y, x, &self.model);
        let maybe_intersection = self.bvh.intersect(&ray);
        if let Some(intersection) = maybe_intersection {
            Some(intersection.intersected_primitive_uuid)
        } else {
            None
        }
    }

    fn _render(self: Arc<Self>) -> Result<Vec<Vec<Color>>, RenderError> {
        let max_threads = Config::get().max_render_threads;
        let num_physical_cores = num_cpus::get_physical();
        let num_threads = max_threads.min(num_physical_cores).max(1);
        let mut thread_handles = vec![];

        let ray_counter_arc = Arc::new(atomic::AtomicUsize::new(0));
        let ten_percent_arc = Arc::new((self.primary_rays.len() / 10).max(1));
        let progress_block_counter_arc = Arc::new(atomic::AtomicUsize::new(0));

        let image_data_arc = Arc::new(Mutex::new(vec![
            vec![Color::new(); self.model.screen.width];
            self.model.screen.height
        ]));
        let self_arc = Arc::new(self.clone());

        let start_time = SystemTime::now();
        for thread_num in 0..num_threads {
            // these arc clones do not clone the underlying data
            let _image_data_arc_clone = Arc::clone(&image_data_arc);
            let _self_arc_clone = Arc::clone(&self_arc);
            let _ray_counter_arc_clone = Arc::clone(&ray_counter_arc);
            let _ten_percent_arc_clone = Arc::clone(&ten_percent_arc);
            let _progress_block_counter_arc_clone = Arc::clone(&progress_block_counter_arc);

            let handle = thread::spawn(move || {
                info!(LOG, "starting render thread #{}", thread_num);
                let mut thread_rendered_pixels: Vec<((usize, usize), Color)> = vec![];

                loop {
                    let ray_index = _ray_counter_arc_clone.fetch_add(1, atomic::Ordering::Relaxed);
                    trace!(LOG, "thread {} rendering ray #{}", thread_num, ray_index);

                    if ray_index != 0 && ray_index.is_multiple_of(*_ten_percent_arc_clone) {
                        let progress_block = _progress_block_counter_arc_clone
                            .fetch_add(1, atomic::Ordering::Relaxed)
                            + 1; // fetch_add() returns the previous value, not the new sum
                        info!(LOG, "rendering {}% complete", progress_block * 10);
                    }

                    if ray_index >= _self_arc_clone.primary_rays.len() {
                        break;
                    }

                    let ray = &_self_arc_clone.primary_rays[ray_index];
                    let pixel_color =
                        shader::process_ray(0, ray, &_self_arc_clone.model, &_self_arc_clone.bvh);
                    thread_rendered_pixels.push(((ray.i, ray.j), pixel_color));
                }

                match _image_data_arc_clone.lock() {
                    Ok(mut image_data_guard) => {
                        for ((i, j), pixel_color) in thread_rendered_pixels {
                            image_data_guard[i][j] = pixel_color;
                        }
                    }
                    Err(_) => {
                        warn!(
                            LOG,
                            "thread {} encountered poisoned data error when trying to write pixel colors to image data",
                            thread_num,
                        );
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
                Ok(raw_image_data) => {
                    let stop_time = SystemTime::now();
                    let maybe_duration = stop_time.duration_since(start_time);
                    if let Ok(elapsed_time) = maybe_duration {
                        info!(
                            LOG,
                            "render completed in {} seconds",
                            elapsed_time.as_millis() as f64 / 1000.0
                        )
                    }

                    Ok(raw_image_data)
                }
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

    fn calculate_primary_rays(model: &Model, camera: &Camera) -> Vec<Ray> {
        let mut rays: Vec<Ray> = Vec::new();

        for i in 0..model.screen.height {
            for j in 0..model.screen.width {
                let ray = camera.calc_ray_definition(i, j, model);
                rays.push(ray);
            }
        }

        rays
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
