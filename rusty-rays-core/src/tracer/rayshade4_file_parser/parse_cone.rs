use std::collections::HashMap;
use std::iter::Peekable;
use std::str::SplitWhitespace;
use uuid::Uuid;

use crate::tracer::Coords;
use crate::tracer::misc_types::Surface;
use crate::tracer::model::ModelError;
use crate::tracer::model::ModelError::FailedToParseInputFile;
use crate::tracer::primitives::Cone;
use crate::utils::logger::{LOG, trace};

pub fn process_cone(
    keyword_line_iter: &mut Peekable<SplitWhitespace>,
    surfaces: &HashMap<String, Surface>,
    line_number: usize,
) -> Result<Cone, ModelError> {
    trace!(LOG, "processing cone");

    // advance past cone keyword
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
                "cone declaration missing surface".to_string(),
            ));
        }
    };

    // Parse base radius
    let maybe_base_radius_str = keyword_line_iter.next();
    let base_radius = match maybe_base_radius_str {
        Some(radius) => match radius.parse::<f64>() {
            Ok(radius) => radius,
            Err(_) => {
                return Err(FailedToParseInputFile(
                    line_number,
                    "invalid base radius value".to_string(),
                ));
            }
        },
        None => {
            return Err(FailedToParseInputFile(
                line_number,
                "cone missing base radius".to_string(),
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
                format!("error parsing cone base: {}", error),
            ));
        }
    };

    // Parse apex radius
    let maybe_apex_radius_str = keyword_line_iter.next();
    let apex_radius = match maybe_apex_radius_str {
        Some(radius) => match radius.parse::<f64>() {
            Ok(radius) => radius,
            Err(_) => {
                return Err(FailedToParseInputFile(
                    line_number,
                    "invalid apex radius value".to_string(),
                ));
            }
        },
        None => {
            return Err(FailedToParseInputFile(
                line_number,
                "cone missing apex radius".to_string(),
            ));
        }
    };

    // Parse apex point (x, y, z)
    let apex_xyz_vec: Vec<&str> = keyword_line_iter.take(3).collect();
    let apex_result = Coords::new_from_str_vec(apex_xyz_vec);
    let apex = match apex_result {
        Ok(apex) => apex,
        Err(error) => {
            return Err(FailedToParseInputFile(
                line_number,
                format!("error parsing cone apex: {}", error),
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

    Ok(Cone {
        uuid: Uuid::new_v4(),
        surface: surface.name.clone(),
        base_radius,
        base,
        apex_radius,
        apex,
    })
}
