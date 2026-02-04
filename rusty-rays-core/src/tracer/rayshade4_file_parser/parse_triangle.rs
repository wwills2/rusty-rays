use std::collections::HashMap;
use std::iter::Peekable;
use std::str::SplitWhitespace;

use crate::tracer::Coords;
use crate::tracer::misc_types::Surface;
use crate::tracer::model::ModelError;
use crate::tracer::model::ModelError::FailedToParseInputFile;
use crate::tracer::primitives::Triangle;
use crate::tracer::rayshade4_file_parser::{
    GetNextLineClosure, NextIfClosure, NextLine, SCENE_DATA_KEYWORDS,
};
use crate::utils::logger::{LOG, debug, trace};

pub fn process_triangle(
    determine_next_line_iter: &mut GetNextLineClosure,
    keyword_line_iter: &mut Peekable<SplitWhitespace>,
    surfaces: &HashMap<String, Surface>,
    starting_line_number: usize,
) -> Result<Triangle, ModelError> {
    trace!(LOG, "processing triangle");

    let line_number = starting_line_number;

    // lines associated with triangles following the keyword should start with floats
    let is_matching_line: NextIfClosure =
        Box::new(|line: &String| match line.split_whitespace().next() {
            Some(word) => SCENE_DATA_KEYWORDS.get(word).is_none() && word.parse::<f64>().is_ok(),
            None => false,
        });

    // advance past triangle keyword
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
                starting_line_number,
                "triangle declaration missing surface".to_string(),
            ));
        }
    };

    let mut defining_coords: Vec<Coords> = vec![];
    let vertices_string = keyword_line_iter.collect::<Vec<_>>().join(" ");

    iterate_and_collect_single_line_def_coords(
        &NextLine {
            line_value: vertices_string,
            line_number,
        },
        &mut defining_coords,
    )?;

    loop {
        let line_read_result = determine_next_line_iter(Some(&is_matching_line));
        if line_read_result.is_err() {
            return Err(line_read_result.err().unwrap());
        }

        let maybe_next_line = line_read_result?;
        if maybe_next_line.is_none() {
            debug!(
                LOG,
                "process_triangle() received None for next line. instantiating triangle"
            );

            let triangle_result = if defining_coords.len() == 3 {
                Triangle::new(
                    defining_coords[0].clone(),
                    defining_coords[1].clone(),
                    defining_coords[2].clone(),
                    None,
                    None,
                    None,
                    surface.name.clone(),
                )
            } else if defining_coords.len() == 6 {
                let v1 = &defining_coords[0];
                let n1 = &defining_coords[1];
                let v2 = &defining_coords[2];
                let n2 = &defining_coords[3];
                let v3 = &defining_coords[4];
                let n3 = &defining_coords[5];

                Triangle::new(
                    v1.clone(),
                    v2.clone(),
                    v3.clone(),
                    Some(n1.clone()),
                    Some(n2.clone()),
                    Some(n3.clone()),
                    surface.name.clone(),
                )
            } else {
                return Err(FailedToParseInputFile(
                    line_number,
                    format!(
                        "a triangle definition should contain a 3 or 6 3D coordinate values. got {}",
                        defining_coords.len()
                    ),
                ));
            };

            return match triangle_result {
                Ok(triangle) => {
                    debug!(LOG, "appending triangle and returning");
                    Ok(triangle)
                }
                Err(error) => Err(FailedToParseInputFile(line_number, error.to_string())),
            };
        }

        let next_line = maybe_next_line.unwrap();
        iterate_and_collect_single_line_def_coords(&next_line, &mut defining_coords)?
    }
}

fn iterate_and_collect_single_line_def_coords(
    next_line: &NextLine,
    defining_coords: &mut Vec<Coords>,
) -> Result<(), ModelError> {
    let mut line_words_iter = next_line.line_value.split_whitespace();

    loop {
        let mut xyz_str_vec = vec![];
        for _ in 0..3 {
            if let Some(next) = line_words_iter.next() {
                xyz_str_vec.push(next);
            }
        }

        if xyz_str_vec.is_empty() {
            // could take no more from line. go to next line
            return Ok(());
        }

        match Coords::new_from_str_vec(xyz_str_vec) {
            Ok(coords) => defining_coords.push(coords),
            Err(error) => {
                return Err(FailedToParseInputFile(
                    next_line.line_number,
                    format!("{}", error),
                ));
            }
        }
    }
}
