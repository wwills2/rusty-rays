use crate::tracer;
use crate::tracer::misc_types::{Intersection, Ray, Surface};
use crate::tracer::model::Model;
use crate::tracer::shader::color::Color;
use crate::tracer::shader::light::Light;

pub mod color;
pub mod light;

pub fn calculate_color(
    model: &Model,
    surface: &Surface,
    starting_intersection: &Intersection,
) -> Color {
    if model.light_sources.len() == 0 {
        return surface.ambient.clone();
    }

    let mut exposed_light_sources: Vec<&Light> = vec![];

    for light_source in &model.light_sources {
        let mut shadow_ray_direction = light_source.position - starting_intersection.position;
        let shadow_ray_length = shadow_ray_direction.calc_vector_length();
        shadow_ray_direction.normalize_vector();

        /*  move the origin just off the intersected object along the shadow ray to protect against
           intersection with the originally intersected object
        */
        let offset_shadow_ray_origin =
            starting_intersection.position + (&shadow_ray_direction * 10e-5);

        let shadow_ray = Ray {
            origin: offset_shadow_ray_origin,
            direction: shadow_ray_direction,
            i: 0,
            j: 0,
        };

        let shadow_ray_intersection =
            match tracer::calculate_ray_first_intersection(&shadow_ray, model) {
                Some(intersection) => intersection,
                None => {
                    exposed_light_sources.push(light_source);
                    continue;
                }
            };

        if shadow_ray_intersection.distance_along_ray > shadow_ray_length {
            exposed_light_sources.push(light_source);
        }
    }

    match exposed_light_sources.len() {
        0 => surface.ambient.clone(),
        _ => surface.diffuse.clone(),
    }
}
