use crate::tracer::camera::Camera;
use crate::utils::Config;
use crate::utils::logger::{LOG, debug, error, info, trace, warn};
use bvh::Bvh;
pub use coords::Coords;
pub use misc_types::{CancellationToken, RenderEvent};
pub use misc_types::{Fov, Screen, Surface};
pub use model::Model;
pub use plane_coords_2d::PlaneCoords2D;
pub use primitives::Cone;
pub use primitives::Plane;
pub use primitives::Polygon;
use primitives::Primitive;
pub use primitives::Sphere;
pub use primitives::Triangle;
pub use shader::Color;
use std::sync::{Arc, Mutex, atomic, mpsc};
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
    camera: Camera,
}

impl Clone for Tracer {
    fn clone(&self) -> Self {
        Tracer {
            model: self.model.clone(),
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

        // Collect all primitives into a vector for Bvh construction
        let primitives_for_bvh: Vec<Box<dyn Primitive>> =
            model.get_all_primitives().values().cloned().collect();
        let mut bvh = Bvh::new();
        bvh.build(primitives_for_bvh);

        Self { model, bvh, camera }
    }

    /// Optional cancellation token, optional event sender, optional progress blocks.
    ///
    /// `progress_blocks` controls how many progress events are emitted across the whole render:
    /// - Some(10) => ~10% increments
    /// - Some(100) => ~1% increments
    /// - Some(0) or None => disabled (no progress events; logging still occurs only for other messages)
    pub fn render(
        &self,
        cancel: Option<CancellationToken>,
        events_tx: Option<mpsc::Sender<RenderEvent>>,
        num_progress_blocks: Option<usize>,
    ) -> Result<Vec<Vec<Color>>, RenderError> {
        info!(LOG, "rendering model");
        let self_arc = Arc::new(self.clone());
        Self::_render(self_arc, cancel, events_tx, num_progress_blocks)
    }

    pub fn get_intersected_uuid_by_pixel_pos(
        &self,
        x: usize,
        y: usize,
    ) -> Option<(uuid::Uuid, String)> {
        // i & j refer to 2d array indices -> transpose of x & y pixel positions
        let ray = self.camera.calc_ray_definition(y, x, &self.model);
        let maybe_intersection = self.bvh.intersect(&ray);
        if let Some(intersection) = maybe_intersection {
            Some((
                intersection.intersected_primitive_uuid,
                intersection.primitive_type,
            ))
        } else {
            None
        }
    }

    fn _render(
        self: Arc<Self>,
        cancel: Option<CancellationToken>,
        event_tx: Option<mpsc::Sender<RenderEvent>>,
        num_progress_blocks: Option<usize>,
    ) -> Result<Vec<Vec<Color>>, RenderError> {
        let max_threads = Config::get().max_render_threads;
        let num_physical_cores = num_cpus::get_physical();
        let num_threads = max_threads.min(num_physical_cores).max(1);
        let mut thread_handles = vec![];

        let total_pixels = self.model.screen.width * self.model.screen.height;
        let ray_counter_arc = Arc::new(atomic::AtomicUsize::new(0));

        // Progress configuration (optional)
        let _num_progress_blocks = num_progress_blocks.unwrap_or(10);
        let block_size = (total_pixels / _num_progress_blocks).max(1);

        // Counts how many blocks have been emitted (so only one thread emits each block once).
        let progress_block_counter_arc = Arc::new(atomic::AtomicUsize::new(0));

        let image_data_arc = Arc::new(Mutex::new(vec![
            vec![Color::new(); self.model.screen.width];
            self.model.screen.height
        ]));
        let self_arc = Arc::new(self.clone());

        // If caller didn't provide a token, use a default "never canceled" token.
        let cancel = cancel.unwrap_or_default();

        let start_time = SystemTime::now();
        for thread_num in 0..num_threads {
            // these arc clones do not clone the underlying data
            let _image_data_arc_clone = Arc::clone(&image_data_arc);
            let _self_arc_clone = Arc::clone(&self_arc);
            let _ray_counter_arc_clone = Arc::clone(&ray_counter_arc);
            let _progress_block_counter_arc_clone = Arc::clone(&progress_block_counter_arc);

            let cancel_clone = cancel.clone();
            let events_tx_clone = event_tx.clone();

            let handle = thread::spawn(move || {
                info!(LOG, "starting render thread #{}", thread_num);
                let mut thread_rendered_pixels: Vec<((usize, usize), Color)> = vec![];

                loop {
                    if cancel_clone.is_canceled() {
                        break;
                    }

                    let ray_index = _ray_counter_arc_clone.fetch_add(1, atomic::Ordering::Relaxed);
                    trace!(LOG, "thread {} rendering ray #{}", thread_num, ray_index);

                    if ray_index >= total_pixels {
                        break;
                    }

                    // Progress emission based on configured blocks.
                    if ray_index.is_multiple_of(block_size) {
                        let block_idx = _progress_block_counter_arc_clone
                            .fetch_add(1, atomic::Ordering::Relaxed)
                            + 1;

                        // Clamp to avoid emitting past 100% due to rounding.
                        if block_idx <= _num_progress_blocks {
                            let percent = ((block_idx * 100) / _num_progress_blocks).min(100) as u8;

                            info!(LOG, "rendering {}% complete", percent);

                            if let Some(tx) = &events_tx_clone {
                                let _ = tx.send(RenderEvent::Progress { percent });
                            }
                        }
                    }

                    let i = ray_index % _self_arc_clone.model.screen.width;
                    let j = ray_index / _self_arc_clone.model.screen.width;

                    if cancel_clone.is_canceled() {
                        break;
                    }

                    let ray =
                        &_self_arc_clone
                            .camera
                            .calc_ray_definition(i, j, &_self_arc_clone.model);

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
                        if let Some(tx) = &events_tx_clone {
                            let _ = tx.send(RenderEvent::Error(
                                "poisoned mutex while writing image data".to_string(),
                            ));
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

        let stop_time = SystemTime::now();
        let elapsed_millis = stop_time
            .duration_since(start_time)
            .map(|d| d.as_millis())
            .unwrap_or(0);

        if thread_error {
            if let Some(tx) = &event_tx {
                let _ = tx.send(RenderEvent::Error(
                    "render threads did not exit properly".to_string(),
                ));
            }
            return Err(RenderError(
                "render threads did not exit properly".to_string(),
            ));
        }

        if cancel.is_canceled()
            && let Some(tx) = &event_tx
        {
            let _ = tx.send(RenderEvent::Canceled {
                millis: elapsed_millis,
            });
        }

        if let Some(tx) = &event_tx {
            let _ = tx.send(RenderEvent::Finished {
                millis: elapsed_millis,
            });
        }

        match Arc::try_unwrap(image_data_arc) {
            Ok(image_data_mutex) => match image_data_mutex.into_inner() {
                Ok(raw_image_data) => {
                    info!(
                        LOG,
                        "render completed in {} seconds",
                        elapsed_millis as f64 / 1000.0
                    );
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
}

#[derive(Debug)]
pub struct RenderError(pub String);

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to render model. Error: {}", self.0)
    }
}

impl std::error::Error for RenderError {}
