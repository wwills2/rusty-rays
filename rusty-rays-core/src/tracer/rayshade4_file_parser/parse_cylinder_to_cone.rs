use std::collections::HashMap;
use std::iter::Peekable;
use std::str::SplitWhitespace;

use slog::debug;
use uuid::Uuid;

use crate::tracer::coords::Coords;
use crate::tracer::misc_types::Surface;
use crate::tracer::model::ModelError;
use crate::tracer::model::ModelError::FailedToParseInputFile;
use crate::tracer::primitives::cone::Cone;
use crate::utils::LOG;

pub fn process_cylinder_to_cone(
    keyword_line_iter: &mut Peekable<SplitWhitespace>,
    surfaces: &HashMap<String, Surface>,
    line_number: usize,
) -> Result<Cone, ModelError> {
    debug!(LOG, "processing cylinder as cone");

    // advance past cylinder keyword
    keyword_line_iter.next();

    let maybe_surface_name = keyword_line_iter.next();
    let surface = match maybe_surface_name {
        Some(surface_name) => {
            let maybe_surface = surfaces.get(surface_name);
            match maybe_surface {
                Some(surface) => surface,
                None => {
                    return Err(FailedToParseInputFile(
                        line_number,
                        format!("surface {} referenced before definition", surface_name),
                    ));
                }
            }
        }
        None => {
            return Err(FailedToParseInputFile(
                line_number,
                "cylinder declaration missing surface".to_string(),
            ));
        }
    };

    let maybe_radius_str = keyword_line_iter.next();
    let base_radius = match maybe_radius_str {
        Some(radius) => match radius.parse::<f64>() {
            Ok(radius) => radius,
            Err(_) => {
                return Err(FailedToParseInputFile(
                    line_number,
                    "invalid radius value".to_string(),
                ));
            }
        },
        None => {
            return Err(FailedToParseInputFile(
                line_number,
                "cylinder missing radius".to_string(),
            ));
        }
    };

    // Parse base point (x, y, z)
    let base_xyz_vec: Vec<&str> = keyword_line_iter.take(3).collect();
    let base_result = Coords::new_from_str_vec(base_xyz_vec);
    let base = match base_result {
        Ok(base) => base,
        Err(error) => {
            return Err(FailedToParseInputFile(
                line_number,
                format!("error parsing cylinder base: {}", error),
            ));
        }
    };

    // Parse axis direction (x, y, z)
    let axis_xyz_vec: Vec<&str> = keyword_line_iter.take(3).collect();
    let axis_result = Coords::new_from_str_vec(axis_xyz_vec);
    let axis = match axis_result {
        Ok(axis) => axis,
        Err(error) => {
            return Err(FailedToParseInputFile(
                line_number,
                format!("error parsing cylinder axis: {}", error),
            ));
        }
    };

    // Parse height
    let maybe_height_str = keyword_line_iter.next();
    let height = match maybe_height_str {
        Some(height) => match height.parse::<f64>() {
            Ok(height) => height,
            Err(_) => {
                return Err(FailedToParseInputFile(
                    line_number,
                    "invalid height value".to_string(),
                ));
            }
        },
        None => {
            return Err(FailedToParseInputFile(
                line_number,
                "cylinder missing height".to_string(),
            ));
        }
    };

    let invalid_value = keyword_line_iter.next();
    if invalid_value.is_some() {
        return Err(FailedToParseInputFile(
            line_number,
            format!("value {} should be on a new line", invalid_value.unwrap()),
        ));
    }

    // Calculate apex point from base, axis, and height
    let normalized_axis = axis.calc_normalized_vector();
    let apex = &base + &(&normalized_axis * height);

    // Use the same radius for apex (making it a cylinder-like cone)
    // To make it a true cone, set apex_radius to 0.0
    let apex_radius = 0.0;

    Ok(Cone {
        uuid: Uuid::new_v4(),
        surface: surface.clone(),
        base_radius,
        base,
        apex_radius,
        apex,
    })
}
