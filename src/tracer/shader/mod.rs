use crate::tracer;
use crate::tracer::calculate_ray_closest_intersection;
use crate::tracer::coords::Coords;
use crate::tracer::misc_types::{Intersection, Ray, Surface};
use crate::tracer::model::Model;
use crate::tracer::shader::color::Color;
use crate::tracer::shader::light::Light;

pub mod color;
pub mod light;

static MAX_REFLECTIONS: u8 = 5;

// !!!
// DO NOT USE model.eyep anywhere in this file
// !!!

pub fn process_ray(trace_depth: u8, ray: &Ray, model: &Model) -> Color {
    match calculate_ray_closest_intersection(ray, model) {
        Some(intersection) => {
            let intersected_entity = model.all_entities.get(&intersection.uuid).unwrap();
            calculate_color(
                trace_depth,
                model,
                intersected_entity.get_surface(),
                &intersection,
            )
        }
        None => model.background.clone(),
    }
}

fn calculate_color(
    trace_depth: u8,
    model: &Model,
    surface: &Surface,
    starting_intersection: &Intersection,
) -> Color {
    if model.light_sources.is_empty() {
        return surface.ambient.clone();
    }

    let ray_from_intersection_back_to_source = (&starting_intersection.ray.origin
        - &starting_intersection.position)
        .calc_normalized_vector();

    let mut point_color = surface.ambient.clone();

    for light_source in &model.light_sources {
        let mut shadow_ray_direction = &light_source.position - &starting_intersection.position;
        let shadow_ray_length = shadow_ray_direction.calc_vector_length();
        shadow_ray_direction.normalize_vector();

        /*  move the origin just off the intersected object along the shadow ray to protect against
          intersection with the originally intersected object
        */
        let offset_shadow_ray_origin =
            &starting_intersection.position + &(&shadow_ray_direction * 10e-5);

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
                    adjust_color_for_diffuse_and_specular(
                        &mut point_color,
                        light_source,
                        &shadow_ray.direction,
                        &starting_intersection.surface_normal_at_intersection,
                        &ray_from_intersection_back_to_source,
                        surface,
                    );
                    continue;
                }
            };

        if shadow_ray_intersection.distance_along_ray > shadow_ray_length {
            adjust_color_for_diffuse_and_specular(
                &mut point_color,
                light_source,
                &shadow_ray.direction,
                &starting_intersection.surface_normal_at_intersection,
                &ray_from_intersection_back_to_source,
                surface,
            )
        }
    }

    if trace_depth < MAX_REFLECTIONS && surface.reflect > 0.0 {
        let angle_normal_to_source_ray = &starting_intersection.ray.direction
            * &starting_intersection.surface_normal_at_intersection;

        let scaled_normal_vector = &starting_intersection.surface_normal_at_intersection
            * (2.0 * angle_normal_to_source_ray);
        let reflection_ray_direction =
            (&starting_intersection.ray.direction - &scaled_normal_vector).calc_normalized_vector();

        /* move the origin just off the intersected object along the shadow ray to protect against
          intersection with the originally intersected object
        */
        let offset_reflection_ray_origin =
            &starting_intersection.position + &(&reflection_ray_direction * 10e-5);

        let reflection_ray = Ray {
            direction: reflection_ray_direction,
            origin: offset_reflection_ray_origin,
            i: 0,
            j: 0,
        };
        let reflection_color = process_ray(trace_depth + 1, &reflection_ray, model);
        let reflect_attenuation = 1.0 / (1.0 + trace_depth as f64);
        let reflection_contribution = reflection_color.scale(surface.reflect * reflect_attenuation);

        point_color.adjust_by(&reflection_contribution)
    }

    point_color
}

fn adjust_color_for_diffuse_and_specular(
    point_color: &mut Color,
    light: &Light,
    shadow_ray_direction: &Coords,
    surface_normal: &Coords,
    ray_to_eyep: &Coords,
    surface: &Surface,
) {
    let surface_normal_relative_angle: f64 = shadow_ray_direction * surface_normal;
    if surface_normal_relative_angle > 0.0 {
        let diffuse_intensity =
            (1.0 - surface.reflect) * light.intensity * surface_normal_relative_angle;
        let diffuse_color_contribution = &surface.diffuse.scale(diffuse_intensity);
        point_color.adjust_by(diffuse_color_contribution);

        if surface.specpow > 0.0 {
            let specular_reflection_incidence: f64 = 2.0 * (shadow_ray_direction * surface_normal);
            if specular_reflection_incidence > 0.0 {
                let specular_reflection_ray =
                    &(surface_normal - shadow_ray_direction) * specular_reflection_incidence;
                let specular_incidence = &specular_reflection_ray * ray_to_eyep;
                if specular_incidence > 0.0 {
                    let specular_intensity = specular_incidence.powf(surface.specpow);
                    let specular_color_contribution = &surface.specular.scale(specular_intensity);
                    point_color.adjust_by(specular_color_contribution)
                }
            }
        }
    }
}
