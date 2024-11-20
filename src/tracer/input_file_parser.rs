use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Lines};
use std::iter::{Enumerate, Peekable};
use std::str::SplitWhitespace;

use crate::tracer::model::{Model, ModelError};
use crate::tracer::sphere::Sphere;
use crate::tracer::types::{Color, Fov, Point, Screen, Surface};

static SCENE_META_DATA_KEYWORDS: [&str; 7] = [
    "background",
    "eyep",
    "lookp",
    "up",
    "fov",
    "screen",
    "diffuse",
];

pub fn iterate_input_data(
    file_enumerator: Enumerate<Peekable<Lines<BufReader<File>>>>,
) -> Result<Model, ModelError> {
    let mut background = Color { r: 0, g: 0, b: 0 };
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

    for (line_number, line_read_result) in file_enumerator {
        if line_read_result.is_err() {
            return Err(ModelError::ErrorParsingInputFile(
                line_number,
                format!(
                    "Failed to read input file. Error: {}",
                    line_read_result.err().unwrap()
                ),
            ));
        }

        let line = line_read_result.unwrap();
        let mut line_words_iter = line.split_whitespace().peekable();

        let mut maybe_line_word = line_words_iter.next();
        if maybe_line_word.is_none() {
            continue;
        }

        while maybe_line_word.is_some() {
            let line_word = maybe_line_word.unwrap();
            match line_word {
                "surface" => {
                    match process_surface(&mut surfaces, &mut line_words_iter, line_number) {
                        Err(error) => return Err(error),
                        _ => {}
                    }
                }
                _ => {}
            }
            maybe_line_word = line_words_iter.next();
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

/// expects that the keyword has already been consumed
fn process_surface(
    surfaces: &mut HashMap<String, Surface>,
    line_iter: &mut Peekable<SplitWhitespace>,
    line_number: usize,
) -> Result<(), ModelError> {
    let name = match line_iter.next() {
        Some(name) => name,
        None => {
            return Err(ModelError::ErrorParsingInputFile(
                line_number,
                "dangling \"surface\" keyword".to_string(),
            ))
        }
    };

    todo!()
}
