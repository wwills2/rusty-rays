use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Lines};
use std::iter::Peekable;
use std::str::{FromStr, SplitWhitespace};

use once_cell::sync::Lazy;
use slog::{debug, warn};
use uuid::Uuid;

use crate::tracer::coords::Coords;
use crate::tracer::misc_types::{Entity, Fov, Screen, Surface};
use crate::tracer::model::ModelError::FailedToParseInputFile;
use crate::tracer::model::{Model, ModelError};
use crate::tracer::polygon::Polygon;
use crate::tracer::shader::color::Color;
use crate::tracer::shader::light::{Light, LightSourceType};
use crate::tracer::sphere::Sphere;
use crate::utils::logger::LOG;

static SCENE_DATA_KEYWORDS: Lazy<HashMap<&'static str, String>> = Lazy::new(|| {
    let map = HashMap::from([
        ("background", "background".to_string()),
        ("eyep", "eyep".to_string()),
        ("lookp", "lookp".to_string()),
        ("fov", "fov".to_string()),
        ("screen", "screen".to_string()),
        ("sphere", "sphere".to_string()),
        ("polygon", "polygon".to_string()),
        ("up", "up".to_string()),
        ("surface", "surface".to_string()),
        ("light", "light".to_string()),
    ]);

    map
});

static SURFACE_KEYWORDS: Lazy<HashMap<&'static str, String>> = Lazy::new(|| {
    let map = HashMap::from([
        ("diffuse", "diffuse".to_string()),
        ("ambient", "ambient".to_string()),
        ("specular", "specular".to_string()),
        ("specpow", "specpow".to_string()),
        ("reflect", "reflect".to_string()),
    ]);

    map
});

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
    let mut light_sources: Vec<Light> = Vec::new();
    let mut spheres: HashMap<Uuid, Sphere> = HashMap::new();
    let mut polygons: HashMap<Uuid, Polygon> = HashMap::new();
    let mut surfaces: HashMap<String, Surface> = HashMap::new();

    let mut line_number = 1;

    /*  closure which handles error and edge cases, returns a peekable iterator of the next line's
       content, and sets the while loop condition to false if need be.
       returns None when there are not more lines in the file.
    */
    let mut get_next_line: GetNextLineClosure = Box::new(move |maybe_next_eq_fn| {
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

        let line_read_result = maybe_line_read_result.unwrap();
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
            "read \"{}\" from input file line {}", input_file_line, line_number
        );

        line_number += 1;

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

        let maybe_next_line = line_words_result?;
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

        // process line comment
        let peeked_line_word = *maybe_peeked_line_word.unwrap();
        if peeked_line_word.to_string().eq("//") {
            continue;
        }

        if SCENE_DATA_KEYWORDS
            .get("background")
            .unwrap()
            .eq(peeked_line_word)
        {
            // iterate past peeked keyword
            line_words_iter.next();

            let rgba_vec: Vec<&str> = line_words_iter.by_ref().take(4).collect();
            match Color::new_from_str_vec(rgba_vec) {
                Ok(color) => background = color,
                Err(error) => return Err(FailedToParseInputFile(line_number, error.to_string())),
            }

            let invalid_value = line_words_iter.next();
            if invalid_value.is_some() {
                return Err(FailedToParseInputFile(
                    line_number,
                    format!("value {} should be on a new line", invalid_value.unwrap()),
                ));
            }
        } else if SCENE_DATA_KEYWORDS
            .get("light")
            .unwrap()
            .eq(peeked_line_word)
        {
            // iterate past peeked keyword
            line_words_iter.next();

            let intensity = match line_words_iter.next() {
                Some(intensity_str) => match intensity_str.parse::<f64>() {
                    Ok(intensity) => intensity,
                    Err(error) => {
                        return Err(FailedToParseInputFile(
                            line_number,
                            "light intensity must be a valid decimal value".to_string(),
                        ))
                    }
                },
                None => {
                    return Err(FailedToParseInputFile(
                        line_number,
                        "missing light source intensity value".to_string(),
                    ))
                }
            };

            let light_source_type = match line_words_iter.next() {
                Some(source_type) => match LightSourceType::from_str(source_type) {
                    Ok(light_source_type) => light_source_type,
                    Err(error) => {
                        return Err(FailedToParseInputFile(line_number, error.to_string()))
                    }
                },
                None => {
                    return Err(FailedToParseInputFile(
                        line_number,
                        "missing light source type".to_string(),
                    ))
                }
            };

            let xyz_vec: Vec<&str> = line_words_iter.by_ref().take(4).collect();
            let position = match Coords::new_from_str_vec(xyz_vec) {
                Ok(position) => position,
                Err(error) => return Err(FailedToParseInputFile(line_number, error.to_string())),
            };

            let invalid_value = line_words_iter.next();
            if invalid_value.is_some() {
                return Err(FailedToParseInputFile(
                    line_number,
                    format!("value {} should be on a new line", invalid_value.unwrap()),
                ));
            }

            light_sources.push(Light {
                position,
                intensity,
                source_type: light_source_type,
            })
        } else if SCENE_DATA_KEYWORDS
            .get("eyep")
            .unwrap()
            .eq(peeked_line_word)
        {
            // iterate past peeked keyword
            line_words_iter.next();

            let xyz_vec: Vec<&str> = line_words_iter.by_ref().take(3).collect();
            match Coords::new_from_str_vec(xyz_vec) {
                Ok(position) => eyep = position,
                Err(error) => return Err(FailedToParseInputFile(line_number, error.to_string())),
            }

            let invalid_value = line_words_iter.next();
            if invalid_value.is_some() {
                return Err(FailedToParseInputFile(
                    line_number,
                    format!("value {} should be on a new line", invalid_value.unwrap()),
                ));
            }
        } else if SCENE_DATA_KEYWORDS
            .get("lookp")
            .unwrap()
            .eq(peeked_line_word)
        {
            // iterate past peeked keyword
            line_words_iter.next();

            let xyz_vec: Vec<&str> = line_words_iter.by_ref().take(3).collect();
            match Coords::new_from_str_vec(xyz_vec) {
                Ok(position) => lookp = position,
                Err(error) => return Err(FailedToParseInputFile(line_number, error.to_string())),
            }

            let invalid_value = line_words_iter.next();
            if invalid_value.is_some() {
                return Err(FailedToParseInputFile(
                    line_number,
                    format!("value {} should be on a new line", invalid_value.unwrap()),
                ));
            }
        } else if SCENE_DATA_KEYWORDS.get("up").unwrap().eq(peeked_line_word) {
            // iterate past peeked keyword
            line_words_iter.next();

            let xyz_vec: Vec<&str> = line_words_iter.by_ref().take(3).collect();
            match Coords::new_from_str_vec(xyz_vec) {
                Ok(position) => up = position,
                Err(error) => return Err(FailedToParseInputFile(line_number, error.to_string())),
            }

            let invalid_value = line_words_iter.next();
            if invalid_value.is_some() {
                return Err(FailedToParseInputFile(
                    line_number,
                    format!("value {} should be on a new line", invalid_value.unwrap()),
                ));
            }
        } else if SCENE_DATA_KEYWORDS.get("fov").unwrap().eq(peeked_line_word) {
            // iterate past peeked keyword
            line_words_iter.next();

            let h_fov: f64 = match line_words_iter.next() {
                Some(h_fov_str) => match h_fov_str.parse::<f64>() {
                    Ok(h_fov) => h_fov,
                    Err(_) => {
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
                    Err(_) => {
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
        } else if SCENE_DATA_KEYWORDS
            .get("screen")
            .unwrap()
            .eq(peeked_line_word)
        {
            // iterate past peeked keyword
            line_words_iter.next();

            let h_screen_px = match line_words_iter.next() {
                Some(h_screen_px_str) => match h_screen_px_str.parse::<usize>() {
                    Ok(h_fov) => h_fov,
                    Err(_) => {
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
                    Err(_) => {
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
        } else if SCENE_DATA_KEYWORDS
            .get("surface")
            .unwrap()
            .eq(peeked_line_word)
        {
            if let Err(result) = process_surface(
                &mut get_next_line,
                &mut line_words_iter,
                &mut surfaces,
                line_number,
            ) {
                return Err(result);
            }
        } else if SCENE_DATA_KEYWORDS
            .get("sphere")
            .unwrap()
            .eq(peeked_line_word)
        {
            let sphere = match process_sphere(&mut line_words_iter, &surfaces, line_number) {
                Ok(sphere) => sphere,
                Err(error) => return Err(error),
            };
            debug!(LOG, "processed sphere {}", sphere);
            spheres.insert(sphere.uuid, sphere);
        } else if SCENE_DATA_KEYWORDS
            .get("polygon")
            .unwrap()
            .eq(peeked_line_word)
        {
            let polygon = match process_polygon(
                &mut get_next_line,
                &mut line_words_iter,
                &surfaces,
                line_number,
            ) {
                Ok(polygon) => polygon,
                Err(error) => return Err(error),
            };
            debug!(LOG, "processed polygon");
            polygons.insert(polygon.uuid, polygon);
        } else {
            warn!(
                LOG,
                "unhandled key word \"{}\". line number {}", peeked_line_word, line_number
            );
        }
    }

    let mut all_entities: HashMap<Uuid, Box<dyn Entity>> = HashMap::new();
    for (uuid, sphere) in &spheres {
        all_entities.insert(uuid.clone(), Box::new(sphere.clone()));
    }

    for (uuid, polygon) in &polygons {
        all_entities.insert(uuid.clone(), Box::new(polygon.clone()));
    }

    Ok(Model {
        background,
        eyep,
        lookp,
        up,
        fov,
        screen,
        light_sources,
        spheres,
        polygons,
        all_entities,
    })
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
        uuid: Uuid::new_v4(),
        surface: surface.clone(),
        radius,
        position,
    })
}

fn process_polygon(
    determine_next_line_iter: &mut GetNextLineClosure,
    keyword_line_iter: &mut Peekable<SplitWhitespace>,
    surfaces: &HashMap<String, Surface>,
    starting_line_number: usize,
) -> Result<Polygon, ModelError> {
    debug!(LOG, "processing polygon");

    let mut line_number = starting_line_number;

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
                    ))
                }
            }
        }
        None => {
            return Err(FailedToParseInputFile(
                starting_line_number,
                "polygon declaration missing surface".to_string(),
            ))
        }
    };

    let invalid_value = keyword_line_iter.next();
    if invalid_value.is_some() {
        return Err(FailedToParseInputFile(
            line_number,
            format!("value {} should be on a new line", invalid_value.unwrap()),
        ));
    }

    let mut vertices: Vec<Coords> = vec![];

    loop {
        let line_read_result = determine_next_line_iter(Some(&is_matching_line));
        if line_read_result.is_err() {
            return Err(line_read_result.err().unwrap());
        }

        let maybe_next_line = line_read_result?;
        if maybe_next_line.is_none() {
            debug!(
                LOG,
                "process_polygon() received None for next line. calculating plane normal before appending"
            );

            let polygon = match Polygon::new(vertices, surface.clone()) {
                Ok(polygon) => polygon,
                Err(error) => return Err(FailedToParseInputFile(line_number, error.to_string())),
            };

            debug!(LOG, "appending polygon and returning");
            return Ok(polygon);
        }

        let next_line = maybe_next_line.unwrap();
        let next_line_value = next_line.line_value;
        line_number = next_line.line_number;
        let mut line_words_iter = next_line_value.split_whitespace();

        loop {
            let mut xyz_str_vec = vec![];
            for _ in 0..3 {
                if let Some(next) = line_words_iter.next() {
                    xyz_str_vec.push(next);
                }
            }

            if xyz_str_vec.len() == 0 {
                // could take no more from line. go to next line
                break;
            }

            match Coords::new_from_str_vec(xyz_str_vec) {
                Ok(coords) => vertices.push(coords),
                Err(error) => {
                    return Err(FailedToParseInputFile(
                        next_line.line_number,
                        format!("{}", error),
                    ))
                }
            }
        }
    }
}

fn process_surface(
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
                    surface.diffuse = specular_color;
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
