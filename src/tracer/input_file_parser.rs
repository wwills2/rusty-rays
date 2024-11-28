use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Lines};
use std::iter::{Enumerate, Peekable};
use std::str::SplitWhitespace;

use slog::{debug, warn};

use crate::tracer::color::Color;
use crate::tracer::model::{Model, ModelError};
use crate::tracer::model::ModelError::ErrorParsingInputFile;
use crate::tracer::point::Point;
use crate::tracer::sphere::Sphere;
use crate::tracer::types::{Fov, Screen, Surface};
use crate::utils::logger::LOG;

static SCENE_META_DATA_KEYWORDS: [&str; 7] = [
    "background",
    "eyep",
    "lookp",
    "up",
    "fov",
    "screen",
    "diffuse",
];

struct NextLine {
    line_value: String,
    line_number: usize,
}

type FileIterator = Peekable<Lines<BufReader<File>>>;
type DetermineNextLineClosure<'a> = Box<dyn FnMut() -> DetermineNextLineResult<'a>>;
type DetermineNextLineResult<'a> = Result<Option<NextLine>, ModelError>;

pub fn iterate_input_data(mut file_iterator: FileIterator) -> Result<Model, ModelError> {
    let mut background = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    let mut eyep = Point {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let mut lookp = Point {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let mut up = (0u8, 0u8, 0u8);
    let mut fov = Fov { horz: 0, vert: 0 };
    let screen = Screen {
        width: 0,
        height: 0,
    };
    let mut spheres: Vec<Sphere> = Vec::new();
    let mut surfaces: HashMap<String, Surface> = HashMap::new();

    let mut line_number = 0;

    /// closure which handles error and edge cases, returns a peekable iterator of the next line's
    /// content, and sets the while loop condition to false if need be.
    /// returns None when there are not more lines in the file.
    let mut get_next_line: DetermineNextLineClosure = Box::new(move || {
        line_number += 1;

        debug!(
            LOG,
            "attempting to retrieve input file line number {}", line_number
        );

        let maybe_line_read_result = file_iterator.next();
        if maybe_line_read_result.is_none() && line_number == 1 {
            return Err(ErrorParsingInputFile(
                line_number,
                "input file is empty".to_string(),
            ));
        } else if maybe_line_read_result.is_none() {
            debug!(LOG, "file iterator returned none. reached the end of file");
            return Ok(None);
        }

        let mut line_read_result = maybe_line_read_result.unwrap();
        if line_read_result.is_err() {
            return Err(ErrorParsingInputFile(
                line_number,
                format!(
                    "failed to read input file from disk. Error: {}",
                    line_read_result.err().unwrap()
                ),
            ));
        }

        let input_file_line = line_read_result.unwrap();
        debug!(
            LOG,
            "read \"{}\" from input file line{}", input_file_line, line_number
        );

        Ok(Some(NextLine {
            line_number,
            line_value: input_file_line,
        }))
    });

    loop {
        let line_words_result = get_next_line();
        if line_words_result.is_err() {
            return Err(line_words_result.err().unwrap());
        }

        let mut maybe_next_line = line_words_result.unwrap();
        if maybe_next_line.is_none() {
            break;
        }

        let line_words = maybe_next_line.unwrap().line_value;
        let mut line_words_iter = line_words.split_whitespace().peekable();
        let maybe_peeked_line_word = line_words_iter.peek();
        if maybe_peeked_line_word.is_none() {
            // blank line, skip
            continue;
        }

        let peeked_line_word = *maybe_peeked_line_word.unwrap();
        match peeked_line_word {
            "surface" => {
                match process_surface(
                    &mut get_next_line,
                    &mut line_words_iter,
                    &mut surfaces,
                    line_number,
                ) {
                    Err(error) => return Err(error),
                    _ => {}
                }
            }
            "sphere" => {
                let sphere = match process_sphere(&mut line_words_iter, &surfaces, line_number) {
                    Ok(sphere) => sphere,
                    Err(error) => return Err(error),
                };
                spheres.push(sphere);
            }
            _ => {}
        }
    }

    return Ok(Model {
        background,
        eyep,
        lookp,
        up,
        fov,
        screen,
        spheres,
    });
}

fn process_sphere(
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
                    return Err(ErrorParsingInputFile(
                        line_number,
                        format!("surface {} referenced before definition", surface_name),
                    ))
                }
            }
        }
        None => {
            return Err(ErrorParsingInputFile(
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
                return Err(ErrorParsingInputFile(
                    line_number,
                    "invalid radius value".to_string(),
                ))
            }
        },
        None => {
            return Err(ErrorParsingInputFile(
                line_number,
                "sphere missing radius".to_string(),
            ))
        }
    };

    let xyz_vec: Vec<&str> = keyword_line_iter.take(3).collect();
    let point_result = Point::new_from_str_vec(xyz_vec);
    let position = match point_result {
        Ok(point) => point,
        Err(error) => return Err(ErrorParsingInputFile(line_number, error.to_string())),
    };

    let invalid_value = keyword_line_iter.next();
    if invalid_value.is_some() {
        return Err(ErrorParsingInputFile(
            line_number,
            format!("value {} should be on a new line", invalid_value.unwrap()),
        ));
    }

    Ok(Sphere {
        surface: surface.clone(),
        radius,
        position,
    })
}

/// expects that the keyword has already been consumed
fn process_surface(
    determine_next_line_iter: &mut DetermineNextLineClosure,
    keyword_line_iter: &mut Peekable<SplitWhitespace>,
    surfaces: &mut HashMap<String, Surface>,
    starting_line_number: usize,
) -> Result<(), ModelError> {
    debug!(LOG, "processing surface");

    // advance past surface keyword
    keyword_line_iter.next();

    let name = match keyword_line_iter.next() {
        Some(name) => name.to_string(),
        None => {
            return Err(ErrorParsingInputFile(
                starting_line_number,
                "dangling \"surface\" keyword".to_string(),
            ))
        }
    };

    let invalid_value = keyword_line_iter.next();
    if invalid_value.is_some() {
        return Err(ErrorParsingInputFile(
            starting_line_number,
            format!("value {} should be on a new line", invalid_value.unwrap()),
        ));
    }

    let mut surface = Surface {
        name: "name".to_string(),
        ambient: Color {
            r: 0.1,
            g: 0.1,
            b: 0.1,
            a: 1.0,
        }, // Dark gray ambient, fully opaque
        diffuse: Color {
            r: 0.1,
            g: 0.1,
            b: 0.1,
            a: 1.0,
        }, // Light gray diffuse, fully opaque
        specular: Color {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0,
        },
        specpow: 32.0, // Typical phong specular power
        reflect: 0.0,  // Non-reflective
    };

    loop {
        let line_read_result = determine_next_line_iter();
        if line_read_result.is_err() {
            return Err(line_read_result.err().unwrap());
        }

        let mut maybe_next_line = line_read_result.unwrap();
        if maybe_next_line.is_none() {
            // next line is none, entire file has been processed
            debug!(
                LOG,
                "process_surface() reached end of file. appending surface and returning"
            );
            surfaces.insert(name, surface);
            return Ok(());
        }

        let next_line = maybe_next_line.unwrap();
        let next_line_value = next_line.line_value;
        let mut line_words_iter = next_line_value.split_whitespace();
        let mut maybe_next_word = line_words_iter.next();

        if maybe_next_word.is_none() {
            // blank line, skip
            continue;
        }

        let first_word_next_line = maybe_next_word.unwrap();
        match first_word_next_line {
            "diffuse" => {
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
                        return Err(ErrorParsingInputFile(
                            next_line.line_number,
                            error.to_string(),
                        ))
                    }
                }
            }
            "ambient" => warn!(LOG, "ambient is not currently supported"),
            "specular" => warn!(LOG, "specular is not currently supported"),
            "specpow" => warn!(LOG, "specpow is not currently supported"),
            "reflect" => warn!(LOG, "reflect is not currently supported"),
            _ => {
                debug!(
                    LOG,
                    "key word {} on input file line {} is not associated with surfaces. stopping surface processing and appending surface",
                    first_word_next_line,
                    next_line.line_number
                );
                surfaces.insert(name, surface);
                return Ok(());
            }
        }
    }
}
