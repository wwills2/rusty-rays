use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Lines};
use std::iter::{Enumerate, Peekable};
use std::str::SplitWhitespace;

use once_cell::sync::Lazy;
use slog::{debug, warn};

use crate::tracer::color::Color;
use crate::tracer::coords::Coords;
use crate::tracer::model::{Model, ModelError};
use crate::tracer::model::ModelError::FailedToParseInputFile;
use crate::tracer::sphere::Sphere;
use crate::tracer::types::{Fov, Screen, Surface};
use crate::utils::logger::LOG;

static SCENE_META_DATA_KEYWORDS: [&str; 6] = ["background", "eyep", "lookp", "up", "fov", "screen"];

static SURFACE_KEY_WORDS: [&str; 5] = ["diffuse", "ambient", "specular", "specpow", "reflect"];

struct NextLine {
    line_value: String,
    line_number: usize,
}

type FileIterator = Peekable<Lines<BufReader<File>>>;
type GetNextLineClosure<'a> = Box<dyn FnMut(Option<&NextIfClosure>) -> GetNextLineResult<'a>>;
type GetNextLineResult<'a> = Result<Option<NextLine>, ModelError>;
type NextIfClosure = Box<dyn Fn(&String) -> bool>;

pub fn iterate_input_data(mut file_iterator: FileIterator) -> Result<Model, ModelError> {
    let mut background = Color::new();
    let mut eyep = Coords::new();
    let mut lookp = Coords::new();
    let mut up = Coords::new();
    let mut fov = Fov {
        horz: 0.0,
        vert: 0.0,
    };
    let mut screen = Screen {
        width: 0,
        height: 0,
    };
    let mut spheres: Vec<Sphere> = Vec::new();
    let mut surfaces: HashMap<String, Surface> = HashMap::new();

    let mut line_number = 0;

    /// closure which handles error and edge cases, returns a peekable iterator of the next line's
    /// content, and sets the while loop condition to false if need be.
    /// returns None when there are not more lines in the file.
    let mut get_next_line: GetNextLineClosure = Box::new(move |maybe_next_eq_fn| {
        line_number += 1;

        debug!(
            LOG,
            "attempting to retrieve input file line number {}", line_number
        );

        let maybe_line_read_result = match maybe_next_eq_fn {
            Some(ref next_eq_fn) => file_iterator.next_if(|line_result| match line_result {
                Ok(line) => next_eq_fn(line),
                Err(_) => return false,
            }),
            None => file_iterator.next(),
        };

        if maybe_line_read_result.is_none() && line_number == 1 {
            return Err(FailedToParseInputFile(
                line_number,
                "input file is empty".to_string(),
            ));
        } else if maybe_line_read_result.is_none() && maybe_next_eq_fn.is_some() {
            debug!(LOG, "file iterator conditionally returned none.");
            return Ok(None);
        } else if maybe_line_read_result.is_none() {
            debug!(LOG, "file iterator returned none. reached the end of file");
            return Ok(None);
        }

        let mut line_read_result = maybe_line_read_result.unwrap();
        if line_read_result.is_err() {
            return Err(FailedToParseInputFile(
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
        let line_words_result = get_next_line(None);
        if line_words_result.is_err() {
            return Err(line_words_result.err().unwrap());
        }

        let mut maybe_next_line = line_words_result.unwrap();
        if maybe_next_line.is_none() {
            break;
        }

        let next_line_struct = maybe_next_line.unwrap();
        let line_number = next_line_struct.line_number;
        let line_words = next_line_struct.line_value;

        let mut line_words_iter = line_words.split_whitespace().peekable();
        let maybe_peeked_line_word = line_words_iter.peek();
        if maybe_peeked_line_word.is_none() {
            // blank line, skip
            continue;
        }

        let peeked_line_word = *maybe_peeked_line_word.unwrap();
        match peeked_line_word {
            "background" => {
                // iterate past peeked keyword
                line_words_iter.next();

                let rgba_vec: Vec<&str> = line_words_iter.by_ref().take(4).collect();
                match Color::new_from_str_vec(rgba_vec) {
                    Ok(color) => background = color,
                    Err(error) => {
                        return Err(FailedToParseInputFile(line_number, error.to_string()))
                    }
                }

                let invalid_value = line_words_iter.next();
                if invalid_value.is_some() {
                    return Err(FailedToParseInputFile(
                        line_number,
                        format!("value {} should be on a new line", invalid_value.unwrap()),
                    ));
                }
            }
            "eyep" => {
                // iterate past peeked keyword
                line_words_iter.next();

                let xyz_vec: Vec<&str> = line_words_iter.by_ref().take(3).collect();
                match Coords::new_from_str_vec(xyz_vec) {
                    Ok(position) => eyep = position,
                    Err(error) => {
                        return Err(FailedToParseInputFile(line_number, error.to_string()))
                    }
                }

                let invalid_value = line_words_iter.next();
                if invalid_value.is_some() {
                    return Err(FailedToParseInputFile(
                        line_number,
                        format!("value {} should be on a new line", invalid_value.unwrap()),
                    ));
                }
            }
            "lookp" => {
                // iterate past peeked keyword
                line_words_iter.next();

                let xyz_vec: Vec<&str> = line_words_iter.by_ref().take(3).collect();
                match Coords::new_from_str_vec(xyz_vec) {
                    Ok(position) => lookp = position,
                    Err(error) => {
                        return Err(FailedToParseInputFile(line_number, error.to_string()))
                    }
                }

                let invalid_value = line_words_iter.next();
                if invalid_value.is_some() {
                    return Err(FailedToParseInputFile(
                        line_number,
                        format!("value {} should be on a new line", invalid_value.unwrap()),
                    ));
                }
            }
            "up" => {
                // iterate past peeked keyword
                line_words_iter.next();

                let xyz_vec: Vec<&str> = line_words_iter.by_ref().take(3).collect();
                match Coords::new_from_str_vec(xyz_vec) {
                    Ok(position) => up = position,
                    Err(error) => {
                        return Err(FailedToParseInputFile(line_number, error.to_string()))
                    }
                }

                let invalid_value = line_words_iter.next();
                if invalid_value.is_some() {
                    return Err(FailedToParseInputFile(
                        line_number,
                        format!("value {} should be on a new line", invalid_value.unwrap()),
                    ));
                }
            }
            "fov" => {
                // iterate past peeked keyword
                line_words_iter.next();

                let h_fov: f64 = match line_words_iter.next() {
                    Some(h_fov_str) => match h_fov_str.parse::<f64>() {
                        Ok(h_fov) => h_fov,
                        Err(error) => {
                            return Err(FailedToParseInputFile(
                                line_number,
                                format!("invalid horizontal fov value: {}", h_fov_str),
                            ))
                        }
                    },
                    None => {
                        return Err(FailedToParseInputFile(
                            line_number,
                            "missing horizontal fov value".to_string(),
                        ))
                    }
                };

                let v_fov: f64 = match line_words_iter.next() {
                    Some(h_fov_str) => match h_fov_str.parse::<f64>() {
                        Ok(h_fov) => h_fov,
                        Err(error) => {
                            return Err(FailedToParseInputFile(
                                line_number,
                                format!("invalid vertical fov value: {}", h_fov_str),
                            ))
                        }
                    },
                    None => {
                        return Err(FailedToParseInputFile(
                            line_number,
                            "missing vertical fov value".to_string(),
                        ))
                    }
                };

                fov = Fov {
                    horz: h_fov,
                    vert: v_fov,
                };

                let invalid_value = line_words_iter.next();
                if invalid_value.is_some() {
                    return Err(FailedToParseInputFile(
                        line_number,
                        format!("value {} should be on a new line", invalid_value.unwrap()),
                    ));
                }
            }
            "screen" => {
                // iterate past peeked keyword
                line_words_iter.next();

                let h_screen_px = match line_words_iter.next() {
                    Some(h_screen_px_str) => match h_screen_px_str.parse::<usize>() {
                        Ok(h_fov) => h_fov,
                        Err(error) => {
                            return Err(FailedToParseInputFile(
                                line_number,
                                format!("invalid horizontal screen size: {}", h_screen_px_str),
                            ))
                        }
                    },
                    None => {
                        return Err(FailedToParseInputFile(
                            line_number,
                            "missing horizontal screen size value".to_string(),
                        ))
                    }
                };

                let v_screen_px = match line_words_iter.next() {
                    Some(v_screen_px_str) => match v_screen_px_str.parse::<usize>() {
                        Ok(h_fov) => h_fov,
                        Err(error) => {
                            return Err(FailedToParseInputFile(
                                line_number,
                                format!("invalid vertical screen size: {}", v_screen_px_str),
                            ))
                        }
                    },
                    None => {
                        return Err(FailedToParseInputFile(
                            line_number,
                            "missing horizontal screen size value".to_string(),
                        ))
                    }
                };

                screen = Screen {
                    height: h_screen_px,
                    width: v_screen_px,
                };

                let invalid_value = line_words_iter.next();
                if invalid_value.is_some() {
                    return Err(FailedToParseInputFile(
                        line_number,
                        format!("value {} should be on a new line", invalid_value.unwrap()),
                    ));
                }
            }
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
                debug!(LOG, "processed sphere {}", sphere);
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
        surface: surface.clone(),
        radius,
        position,
    })
}

/// expects that the keyword has already been consumed
fn process_surface(
    determine_next_line_iter: &mut GetNextLineClosure,
    keyword_line_iter: &mut Peekable<SplitWhitespace>,
    surfaces: &mut HashMap<String, Surface>,
    starting_line_number: usize,
) -> Result<(), ModelError> {
    debug!(LOG, "processing surface");

    let is_matching_line: NextIfClosure =
        Box::new(|line: &String| match line.split_whitespace().next() {
            Some(word) => SURFACE_KEY_WORDS.contains(&word),
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
        let line_read_result = determine_next_line_iter(Some(&is_matching_line));
        if line_read_result.is_err() {
            return Err(line_read_result.err().unwrap());
        }

        let mut maybe_next_line = line_read_result.unwrap();
        if maybe_next_line.is_none() {
            // next line is none, entire file has been processed
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
                        return Err(FailedToParseInputFile(
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
