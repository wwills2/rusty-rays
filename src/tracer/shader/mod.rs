use crate::tracer;
use crate::tracer::coords::Coords;
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

    let mut point_color = surface.ambient.clone();

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
                    adjust_color_for_light(
                        &mut point_color,
                        light_source,
                        &shadow_ray.direction,
                        &starting_intersection.surface_normal_at_intersection,
                        surface,
                    );
                    continue;
                }
            };

        if shadow_ray_intersection.distance_along_ray > shadow_ray_length {
            adjust_color_for_light(
                &mut point_color,
                light_source,
                &shadow_ray.direction,
                &starting_intersection.surface_normal_at_intersection,
                surface,
            )
        }
    }

    point_color
}

fn adjust_color_for_light(
    point_color: &mut Color,
    light: &Light,
    shadow_ray_direction: &Coords,
    surface_normal: &Coords,
    surface: &Surface,
) {
    let surface_normal_relative_angle = shadow_ray_direction * surface_normal;
    if surface_normal_relative_angle > 0.0 {
        let diffuse_intensity = light.intensity * surface_normal_relative_angle;
        let diffuse_color_contribution = &surface.diffuse.scale(diffuse_intensity);
        point_color.adjust_by(diffuse_color_contribution);
    }
}
