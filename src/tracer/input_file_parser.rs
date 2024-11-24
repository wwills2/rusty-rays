use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Lines};
use std::iter::{Enumerate, Peekable};
use std::str::SplitWhitespace;

use slog::warn;

use crate::tracer::model::{Model, ModelError};
use crate::tracer::sphere::Sphere;
use crate::tracer::types::{Color, Fov, Point, Screen, Surface};
use crate::utils::logger::LOGGER;

static SCENE_META_DATA_KEYWORDS: [&str; 7] = [
    "background",
    "eyep",
    "lookp",
    "up",
    "fov",
    "screen",
    "diffuse",
];

type FileIterator = Peekable<Lines<BufReader<File>>>;
type DetermineNextLineClosure<'a> = Box<dyn FnMut() -> DetermineNextLineResult<'a>>;
type DetermineNextLineResult<'a> = Result<Option<String>, ModelError>;

pub fn iterate_input_data(mut file_iterator: FileIterator) -> Result<Model, ModelError> {
    let mut background = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
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

    let mut line_number = 1;
    let mut process_file = true;

    /// closure which handles error and edge cases, returns a peekable iterator of the next line's
    /// content, and sets the while loop condition to false if need be.
    /// returns None when there are not more lines in the file.
    let mut determine_next_line_iter: DetermineNextLineClosure = Box::new(|| {
        let maybe_line_read_result = file_iterator.next();
        if maybe_line_read_result.is_none() && line_number == 1 {
            return Err(ModelError::ErrorParsingInputFile(
                line_number,
                "input file is empty".to_string(),
            ));
        } else if maybe_line_read_result.is_none() {
            process_file = false;
            return Ok(None);
        }

        let mut line_read_result = maybe_line_read_result.unwrap();
        if line_read_result.is_err() {
            return Err(ModelError::ErrorParsingInputFile(
                line_number,
                format!(
                    "failed to read input file. Error: {}",
                    line_read_result.err().unwrap()
                ),
            ));
        }

        let input_file_line = line_read_result.unwrap();

        Ok(Some(input_file_line))
    });

    while process_file {
        let line_words_result = determine_next_line_iter();
        if line_words_result.is_err() {
            return Err(line_words_result.err().unwrap());
        }

        let mut maybe_line_words = line_words_result.unwrap();
        if maybe_line_words.is_none() {
            // next line is none, entire file has been processed
            process_file = false;
            continue;
        }

        let line_words = maybe_line_words.unwrap();
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
                    &mut determine_next_line_iter,
                    &mut line_words_iter,
                    &mut surfaces,
                    &mut line_number,
                ) {
                    Err(error) => return Err(error),
                    _ => {}
                }
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

/// expects that the keyword has already been consumed
fn process_surface(
    determine_next_line_iter: &mut DetermineNextLineClosure,
    keyword_line_iter: &mut Peekable<SplitWhitespace>,
    surfaces: &mut HashMap<String, Surface>,
    line_number: &mut usize,
) -> Result<(), ModelError> {
    //advance past surface keyword
    keyword_line_iter.next();

    // caller determines that at least the first word exits
    let name = match keyword_line_iter.next() {
        Some(name) => name.to_string(),
        None => {
            return Err(ModelError::ErrorParsingInputFile(
                *line_number,
                "dangling \"surface\" keyword".to_string(),
            ))
        }
    };

    let line_words_iter_result = determine_next_line_iter();
    if line_words_iter_result.is_err() {
        return Err(line_words_iter_result.err().unwrap());
    }

    let mut maybe_next_line = line_words_iter_result.unwrap();
    if maybe_next_line.is_none() {
        // next line is none, entire file has been processed
        return Ok(());
    }

    let binding = maybe_next_line.unwrap();
    let mut line_words_iter = binding.split_whitespace();
    let maybe_next_word = line_words_iter.next();

    while maybe_next_word.is_some() {
        let next_word = maybe_next_word.unwrap();
        match next_word {
            "diffuse" => {}
            _ => warn!(
                LOGGER,
                "key word {} on input file line {} is not supported", next_word, line_number
            ),
        }
    }

    todo!()
}
