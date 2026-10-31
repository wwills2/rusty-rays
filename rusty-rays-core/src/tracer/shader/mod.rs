use crate::tracer::Bvh;
use crate::tracer::Coords;
use crate::tracer::Model;
use crate::tracer::misc_types::{Intersection, Ray, Surface};
use crate::tracer::shader::light::{Light, LightSourceType};
use rand::{RngExt, SeedableRng};

mod color;
pub mod light;

pub use color::Color;

pub static NUM_SHADOW_RAYS: u8 = 32;
static MAX_REFLECTIONS: u8 = 5;

// !!!
// DO NOT USE model.eyep anywhere in this file
// !!!

pub fn process_ray(trace_depth: u8, ray: &Ray, model: &Model, bvh: &Bvh) -> Color {
    if let Some(intersection) = bvh.intersect(ray) {
        let maybe_intersected_primitive = model
            .get_all_primitives()
            .get(&intersection.intersected_primitive_uuid);

        if let Some(primitive) = maybe_intersected_primitive
            && let Some(surface) = model.surfaces.get(primitive.get_surface())
        {
            return calculate_color(trace_depth, model, bvh, surface, &intersection);
        }

        model.background.clone()
    } else {
        model.background.clone()
    }
}

fn calculate_color(
    trace_depth: u8,
    model: &Model,
    bvh: &Bvh,
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

    for (light_index, light_source) in model.light_sources.iter().enumerate() {
        match light_source.source_type {
            LightSourceType::Extended => {
                process_extended_light(
                    &mut point_color,
                    light_source,
                    light_index,
                    starting_intersection,
                    &ray_from_intersection_back_to_source,
                    surface,
                    bvh,
                );
            }
            LightSourceType::Ambient => {
                // Ambient light contributes via surface.ambient; skip per-light shading.
            }
            _ => {
                process_point_light(
                    &mut point_color,
                    light_source,
                    starting_intersection,
                    &ray_from_intersection_back_to_source,
                    surface,
                    bvh,
                );
            }
        }
    }

    if trace_depth < MAX_REFLECTIONS && surface.reflect > 0.0 {
        let angle_normal_to_source_ray = &starting_intersection.ray.direction
            * &starting_intersection.surface_normal_at_intersection;

        let scaled_normal_vector = &starting_intersection.surface_normal_at_intersection
            * (2.0 * angle_normal_to_source_ray);
        let reflection_ray_direction =
            (&starting_intersection.ray.direction - &scaled_normal_vector).calc_normalized_vector();

        let offset_reflection_ray_origin =
            &starting_intersection.position + &(&reflection_ray_direction * 10e-5);

        let reflection_ray = Ray {
            direction: reflection_ray_direction,
            origin: offset_reflection_ray_origin,
            i: 0,
            j: 0,
        };

        let reflection_color = process_ray(trace_depth + 1, &reflection_ray, model, bvh);
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
            let specular_reflection_incidence: f64 = 2.0 * (surface_normal * shadow_ray_direction);
            if specular_reflection_incidence > 0.0 {
                let specular_reflection_ray =
                    (surface_normal * specular_reflection_incidence) - shadow_ray_direction;
                let specular_incidence = &specular_reflection_ray * ray_to_eyep;
                if specular_incidence > 0.0 {
                    let mut specular_intensity = specular_incidence.powf(surface.specpow);
                    if let LightSourceType::Extended = light.source_type {
                        specular_intensity /= NUM_SHADOW_RAYS as f64;
                    }

                    let specular_color_contribution = &surface.specular.scale(specular_intensity);
                    point_color.adjust_by(specular_color_contribution)
                }
            }
        }
    }
}

fn process_point_light(
    point_color: &mut Color,
    light_source: &Light,
    starting_intersection: &Intersection,
    ray_from_intersection_back_to_source: &Coords,
    surface: &Surface,
    bvh: &Bvh,
) {
    let mut shadow_ray_direction = &light_source.position - &starting_intersection.position;
    let shadow_ray_length = shadow_ray_direction.calc_vector_length();
    shadow_ray_direction.normalize_vector();

    let offset_shadow_ray_origin =
        &starting_intersection.position + &(&shadow_ray_direction * 10e-5);

    let shadow_ray = Ray {
        origin: offset_shadow_ray_origin,
        direction: shadow_ray_direction,
        i: 0,
        j: 0,
    };

    let light_visible = match bvh.intersect(&shadow_ray) {
        Some(intersection) => intersection.distance_along_ray > shadow_ray_length,
        None => true,
    };

    if light_visible {
        adjust_color_for_diffuse_and_specular(
            point_color,
            light_source,
            &shadow_ray.direction,
            &starting_intersection.surface_normal_at_intersection,
            ray_from_intersection_back_to_source,
            surface,
        );
    }
}

fn process_extended_light(
    point_color: &mut Color,
    light_source: &Light,
    light_index: usize,
    starting_intersection: &Intersection,
    ray_from_intersection_back_to_source: &Coords,
    surface: &Surface,
    bvh: &Bvh,
) {
    let mut main_shadow_ray_direction = &light_source.position - &starting_intersection.position;
    main_shadow_ray_direction.normalize_vector();

    let (tangent1, tangent2) = main_shadow_ray_direction.build_orthonormal_basis();

    let seed = make_extended_light_seed(starting_intersection, light_source, light_index);
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    let rotation_angle = 2.0 * std::f64::consts::PI * rng.random::<f64>();
    let rotation_cos = rotation_angle.cos();
    let rotation_sin = rotation_angle.sin();

    let strata_side = (NUM_SHADOW_RAYS as f64).sqrt().ceil() as usize;
    let target_samples = NUM_SHADOW_RAYS as usize;

    let mut samples_taken = 0usize;

    for sy in 0..strata_side {
        for sx in 0..strata_side {
            if samples_taken >= target_samples {
                break;
            }

            let jx: f64 = rng.random();
            let jy: f64 = rng.random();

            let u1 = (sx as f64 + jx) / strata_side as f64;
            let u2 = (sy as f64 + jy) / strata_side as f64;

            let (mut dx, mut dy) = concentric_sample_disk(u1, u2);

            let rotated_dx = dx * rotation_cos - dy * rotation_sin;
            let rotated_dy = dx * rotation_sin + dy * rotation_cos;
            dx = rotated_dx;
            dy = rotated_dy;

            let offset = &(&tangent1 * (dx * light_source.radius))
                + &(&tangent2 * (dy * light_source.radius));

            let light_sample_position = &light_source.position + &offset;

            let mut shadow_ray_direction = &light_sample_position - &starting_intersection.position;
            let sample_ray_length = shadow_ray_direction.calc_vector_length();
            shadow_ray_direction.normalize_vector();

            let offset_shadow_ray_origin =
                &starting_intersection.position + &(&shadow_ray_direction * 10e-5);

            let shadow_ray = Ray {
                origin: offset_shadow_ray_origin,
                direction: shadow_ray_direction,
                i: 0,
                j: 0,
            };

            let sample_visible = match bvh.intersect(&shadow_ray) {
                Some(intersection) => intersection.distance_along_ray > sample_ray_length,
                None => true,
            };

            if sample_visible {
                adjust_color_for_diffuse_and_specular(
                    point_color,
                    light_source,
                    &shadow_ray.direction,
                    &starting_intersection.surface_normal_at_intersection,
                    ray_from_intersection_back_to_source,
                    surface,
                );
            }

            samples_taken += 1;
        }
    }
}

fn concentric_sample_disk(u1: f64, u2: f64) -> (f64, f64) {
    let sx = 2.0 * u1 - 1.0;
    let sy = 2.0 * u2 - 1.0;

    if sx == 0.0 && sy == 0.0 {
        return (0.0, 0.0);
    }

    let (r, theta) = if sx.abs() > sy.abs() {
        (sx, std::f64::consts::FRAC_PI_4 * (sy / sx))
    } else {
        (
            sy,
            std::f64::consts::FRAC_PI_2 - std::f64::consts::FRAC_PI_4 * (sx / sy),
        )
    };

    (r * theta.cos(), r * theta.sin())
}

fn make_extended_light_seed(
    starting_intersection: &Intersection,
    light_source: &Light,
    light_index: usize,
) -> u64 {
    let mut seed = 0u64;

    seed ^= splitmix64(starting_intersection.ray.i as u64);
    seed ^= splitmix64((starting_intersection.ray.j as u64).rotate_left(17));

    seed ^= splitmix64(starting_intersection.position.x.to_bits());
    seed ^= splitmix64(starting_intersection.position.y.to_bits().rotate_left(11));
    seed ^= splitmix64(starting_intersection.position.z.to_bits().rotate_left(23));

    seed ^= splitmix64(light_source.position.x.to_bits().rotate_left(7));
    seed ^= splitmix64(light_source.position.y.to_bits().rotate_left(19));
    seed ^= splitmix64(light_source.position.z.to_bits().rotate_left(29));

    seed ^= splitmix64((light_index as u64).wrapping_mul(0x9E3779B97F4A7C15));

    seed
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}
