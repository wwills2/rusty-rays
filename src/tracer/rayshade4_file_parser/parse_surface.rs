use std::collections::HashMap;
use std::iter::Peekable;
use std::str::SplitWhitespace;

use slog::debug;

use crate::tracer::misc_types::Surface;
use crate::tracer::model::ModelError;
use crate::tracer::model::ModelError::FailedToParseInputFile;
use crate::tracer::rayshade4_file_parser::{GetNextLineClosure, NextIfClosure, SURFACE_KEYWORDS};
use crate::tracer::shader::color::Color;
use crate::utils::logger::LOG;

pub fn process_surface(
    determine_next_line_iter: &mut GetNextLineClosure,
    keyword_line_iter: &mut Peekable<SplitWhitespace>,
    surfaces: &mut HashMap<String, Surface>,
    starting_line_number: usize,
) -> Result<(), ModelError> {
    debug!(LOG, "processing surface");

    let is_matching_line: NextIfClosure =
        Box::new(|line: &String| match line.split_whitespace().next() {
            Some(word) => SURFACE_KEYWORDS.get(&word).is_some(),
            None => false,
        });

    // advance past surface keyword
    keyword_line_iter.next();

    let name = match keyword_line_iter.next() {
        Some(name) => name.to_string(),
        None => {
            return Err(FailedToParseInputFile(
                starting_line_number,
                "dangling \"surface\" keyword".to_string(),
            ))
        }
    };

    let invalid_value = keyword_line_iter.next();
    if invalid_value.is_some() {
        return Err(FailedToParseInputFile(
            starting_line_number,
            format!("value {} should be on a new line", invalid_value.unwrap()),
        ));
    }

    let mut surface = Surface {
        name: "name".to_string(),
        ambient: Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }, // black ambient, fully opaque
        diffuse: Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }, // black diffuse, fully opaque
        specular: Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        specpow: 0.0,
        reflect: 0.0,
    };

    loop {
        let line_read_result = determine_next_line_iter(Some(&is_matching_line));
        if line_read_result.is_err() {
            return Err(line_read_result.err().unwrap());
        }

        let maybe_next_line = line_read_result?;
        if maybe_next_line.is_none() {
            debug!(
                LOG,
                "process_surface() received None for next line. appending surface and returning"
            );
            surfaces.insert(name, surface);
            return Ok(());
        }

        let next_line = maybe_next_line.unwrap();
        let next_line_value = next_line.line_value;
        let mut line_words_iter = next_line_value.split_whitespace();
        let maybe_next_word = line_words_iter.next();

        if maybe_next_word.is_none() {
            // blank line, skip
            continue;
        }

        let first_word_next_line = maybe_next_word.unwrap();

        if SURFACE_KEYWORDS
            .get("diffuse")
            .unwrap()
            .eq(first_word_next_line)
        {
            debug!(
                LOG,
                "processing diffuse color on line input file line {}", next_line.line_number
            );

            let rgba_vec: Vec<&str> = line_words_iter.take(4).collect();
            let diffuse_color_result = Color::new_from_str_vec(rgba_vec);

            match diffuse_color_result {
                Ok(diffuse_color) => {
                    surface.diffuse = diffuse_color;
                }
                Err(error) => {
                    return Err(FailedToParseInputFile(
                        next_line.line_number,
                        error.to_string(),
                    ))
                }
            }
        } else if SURFACE_KEYWORDS
            .get("ambient")
            .unwrap()
            .eq(first_word_next_line)
        {
            debug!(
                LOG,
                "processing ambient color on line input file line {}", next_line.line_number
            );

            let rgba_vec: Vec<&str> = line_words_iter.take(4).collect();
            let ambient_color_result = Color::new_from_str_vec(rgba_vec);

            match ambient_color_result {
                Ok(ambient_color) => {
                    surface.diffuse = ambient_color;
                }
                Err(error) => {
                    return Err(FailedToParseInputFile(
                        next_line.line_number,
                        error.to_string(),
                    ))
                }
            }
        } else if SURFACE_KEYWORDS
            .get("specular")
            .unwrap()
            .eq(first_word_next_line)
        {
            debug!(
                LOG,
                "processing specular color on line input file line {}", next_line.line_number
            );

            let rgba_vec: Vec<&str> = line_words_iter.take(4).collect();
            let specular_color_result = Color::new_from_str_vec(rgba_vec);

            match specular_color_result {
                Ok(specular_color) => {
                    surface.specular = specular_color;
                }
                Err(error) => {
                    return Err(FailedToParseInputFile(
                        next_line.line_number,
                        error.to_string(),
                    ))
                }
            }
        } else if SURFACE_KEYWORDS
            .get("specpow")
            .unwrap()
            .eq(first_word_next_line)
        {
            let maybe_specpow_str = match line_words_iter.next() {
                Some(specpow_str) => specpow_str,
                None => {
                    return Err(FailedToParseInputFile(
                        next_line.line_number,
                        "missing specpow value".to_string(),
                    ))
                }
            };

            match maybe_specpow_str.parse::<f64>() {
                Ok(specpow) => surface.specpow = specpow,
                Err(_) => {
                    return Err(FailedToParseInputFile(
                        next_line.line_number,
                        "specpow value must be valid decimal value".to_string(),
                    ))
                }
            }
        } else if SURFACE_KEYWORDS
            .get("reflect")
            .unwrap()
            .eq(first_word_next_line)
        {
            let maybe_reflect_str = match line_words_iter.next() {
                Some(reflect_str) => reflect_str,
                None => {
                    return Err(FailedToParseInputFile(
                        next_line.line_number,
                        "missing reflect value".to_string(),
                    ))
                }
            };

            match maybe_reflect_str.parse::<f64>() {
                Ok(reflect) => surface.reflect = reflect,
                Err(_) => {
                    return Err(FailedToParseInputFile(
                        next_line.line_number,
                        "reflect value must be valid decimal value".to_string(),
                    ))
                }
            }
        }
    }
}
