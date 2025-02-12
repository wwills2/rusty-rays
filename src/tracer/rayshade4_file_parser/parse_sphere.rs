use std::collections::HashMap;
use std::iter::Peekable;
use std::str::SplitWhitespace;

use slog::debug;
use uuid::Uuid;

use crate::tracer::coords::Coords;
use crate::tracer::misc_types::Surface;
use crate::tracer::model::ModelError;
use crate::tracer::model::ModelError::FailedToParseInputFile;
use crate::tracer::primitives::sphere::Sphere;
use crate::utils::logger::LOG;

pub fn process_sphere(
    keyword_line_iter: &mut Peekable<SplitWhitespace>,
    surfaces: &HashMap<String, Surface>,
    line_number: usize,
) -> Result<Sphere, ModelError> {
    debug!(LOG, "processing sphere");

    // advance past sphere keyword
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
                    ))
                }
            }
        }
        None => {
            return Err(FailedToParseInputFile(
                line_number,
                "sphere declaration missing surface".to_string(),
            ))
        }
    };

    let maybe_radius_str = keyword_line_iter.next();
    let radius = match maybe_radius_str {
        Some(radius) => match radius.parse::<f64>() {
            Ok(radius) => radius,
            Err(_) => {
                return Err(FailedToParseInputFile(
                    line_number,
                    "invalid radius value".to_string(),
                ))
            }
        },
        None => {
            return Err(FailedToParseInputFile(
                line_number,
                "sphere missing radius".to_string(),
            ))
        }
    };

    let xyz_vec: Vec<&str> = keyword_line_iter.take(3).collect();
    let position_result = Coords::new_from_str_vec(xyz_vec);
    let position = match position_result {
        Ok(position) => position,
        Err(error) => return Err(FailedToParseInputFile(line_number, error.to_string())),
    };

    let invalid_value = keyword_line_iter.next();
    if invalid_value.is_some() {
        return Err(FailedToParseInputFile(
            line_number,
            format!("value {} should be on a new line", invalid_value.unwrap()),
        ));
    }

    Ok(Sphere {
        uuid: Uuid::new_v4(),
        surface: surface.clone(),
        radius,
        position,
    })
}
