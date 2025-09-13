use std::collections::HashMap;
use std::iter::Peekable;
use std::str::SplitWhitespace;

use crate::tracer::misc_types::Surface;
use crate::tracer::model::ModelError;
use crate::tracer::model::ModelError::FailedToParseInputFile;
use crate::tracer::primitives::Polygon;
use crate::tracer::rayshade4_file_parser::{
    GetNextLineClosure, NextIfClosure, NextLine, SCENE_DATA_KEYWORDS,
};
use crate::tracer::Coords;
use crate::utils::logger::{debug, LOG};

pub fn process_polygon(
    determine_next_line_iter: &mut GetNextLineClosure,
    keyword_line_iter: &mut Peekable<SplitWhitespace>,
    surfaces: &HashMap<String, Surface>,
    starting_line_number: usize,
) -> Result<Polygon, ModelError> {
    debug!(LOG, "processing polygon");

    let line_number = starting_line_number;

    // lines associated with polygons following the keyword should start with floats
    let is_matching_line: NextIfClosure =
        Box::new(|line: &String| match line.split_whitespace().next() {
            Some(word) => SCENE_DATA_KEYWORDS.get(word).is_none() && word.parse::<f64>().is_ok(),
            None => false,
        });

    // advance past polygon keyword
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
                "polygon declaration missing surface".to_string(),
            ));
        }
    };

    let mut vertices: Vec<Coords> = vec![];
    let vertices_string = keyword_line_iter.collect::<Vec<_>>().join(" ");

    iterate_and_update_single_line_vertices(
        &NextLine {
            line_value: vertices_string,
            line_number,
        },
        &mut vertices,
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
                "process_polygon() received None for next line. instantiating polygon"
            );

            let polygon = match Polygon::new(vertices, surface.clone()) {
                Ok(polygon) => polygon,
                Err(error) => return Err(FailedToParseInputFile(line_number, error.to_string())),
            };

            debug!(LOG, "appending polygon and returning");
            return Ok(polygon);
        }

        let next_line = maybe_next_line.unwrap();
        iterate_and_update_single_line_vertices(&next_line, &mut vertices)?
    }
}

fn iterate_and_update_single_line_vertices(
    next_line: &NextLine,
    vertices: &mut Vec<Coords>,
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
            Ok(coords) => vertices.push(coords),
            Err(error) => {
                return Err(FailedToParseInputFile(
                    next_line.line_number,
                    format!("{}", error),
                ));
            }
        }
    }
}
