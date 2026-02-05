use crate::logger::{LOG, debug, trace};
use crate::tracer::misc_types::Ray;
use crate::{Coords, Model};
use std::f64;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Camera {
    forward: Coords,
    right: Coords,
    true_up: Coords,
    focal_len: f64,
    viewing_plane_width: f64,
    viewing_plane_height: f64,
}

#[allow(dead_code)]
impl Camera {
    pub fn new(model: &Model) -> Self {
        let direction = &model.lookp - &model.eyep;
        let forward = direction.calc_normalized_vector();
        let right = forward.cross(&model.up).calc_normalized_vector();
        let true_up = right.cross(&forward);

        let focal_len = direction.calc_vector_length();
        let viewing_plane_width =
            2.0 * focal_len * f64::tan((model.fov.horz / 2.0) * (f64::consts::PI / 180.0));
        let viewing_plane_height =
            2.0 * focal_len * f64::tan((model.fov.vert / 2.0) * (f64::consts::PI / 180.0));

        debug!(
            LOG,
            "calculating primary rays. details:
direction vec: {}
forward vec: {}
right vec: {}
true-up vec: {}
focal len: {}
viewing plane width: {}
viewing plane height: {}",
            direction,
            forward,
            right,
            true_up,
            focal_len,
            viewing_plane_width,
            viewing_plane_height
        );

        Self {
            forward,
            right,
            true_up,
            focal_len,
            viewing_plane_width,
            viewing_plane_height,
        }
    }

    pub fn calc_ray_definition(&self, i: usize, j: usize, model: &Model) -> Ray {
        let horz_pos = ((j as f64 + 0.5) / model.screen.width as f64) - 0.5;
        let vert_pos = 0.5 - ((i as f64 + 0.5) / model.screen.height as f64);

        let pixel_pos = &model.lookp
            + &(&self.right * (self.viewing_plane_width * horz_pos))
            + (&self.true_up * (vert_pos * self.viewing_plane_height));
        trace!(
            LOG,
            "position of image plane pixel (i: {}, j: {}); {}", i, j, pixel_pos
        );

        let direction = (pixel_pos - &model.eyep).calc_normalized_vector();
        trace!(
            LOG,
            "calculated direction of ray through image plane pixel position (i: {}, j:{}) to be {} ",
            i,
            j,
            direction
        );

        Ray {
            i,
            j,
            direction,
            origin: model.eyep.clone(),
        }
    }

    #[inline]
    pub fn forward(&self) -> Coords {
        self.forward.clone()
    }

    #[inline]
    pub fn right(&self) -> Coords {
        self.right.clone()
    }

    #[inline]
    pub fn true_up(&self) -> Coords {
        self.true_up.clone()
    }

    #[inline]
    pub fn focal_len(&self) -> f64 {
        self.focal_len
    }

    #[inline]
    pub fn viewing_plane_width(&self) -> f64 {
        self.viewing_plane_width
    }

    #[inline]
    pub fn viewing_plane_height(&self) -> f64 {
        self.viewing_plane_height
    }
}
