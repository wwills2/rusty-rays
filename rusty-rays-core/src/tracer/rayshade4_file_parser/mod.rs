use std::collections::HashMap;
use std::io::{BufRead, Lines};
use std::iter::Peekable;
use std::str::FromStr;

use once_cell::sync::Lazy;
use uuid::Uuid;

use crate::tracer::Coords;
use crate::tracer::misc_types::{Fov, Screen, Surface};
use crate::tracer::model::ModelError::FailedToParseInputFile;
use crate::tracer::model::{Model, ModelError};
use crate::tracer::primitives::Cone;
use crate::tracer::primitives::Polygon;
use crate::tracer::primitives::Sphere;
use crate::tracer::primitives::Triangle;
use crate::tracer::shader::Color;
use crate::tracer::shader::light::{Light, LightSourceType};
use crate::utils::logger::{LOG, trace, warn};

mod parse_cone;
mod parse_cylinder_to_cone;
mod parse_polygon;
mod parse_sphere;
mod parse_surface;
mod parse_triangle;

static SCENE_DATA_KEYWORDS: Lazy<HashMap<&'static str, String>> = Lazy::new(|| {
    HashMap::from([
        ("background", "background".to_string()),
        ("eyep", "eyep".to_string()),
        ("lookp", "lookp".to_string()),
        ("fov", "fov".to_string()),
        ("screen", "screen".to_string()),
        ("sphere", "sphere".to_string()),
        ("cylinder", "cylinder".to_string()),
        ("cone", "cone".to_string()),
        ("polygon", "polygon".to_string()),
        ("triangle", "triangle".to_string()),
        ("up", "up".to_string()),
        ("surface", "surface".to_string()),
        ("light", "light".to_string()),
    ])
});

static SURFACE_KEYWORDS: Lazy<HashMap<&'static str, String>> = Lazy::new(|| {
    HashMap::from([
        ("diffuse", "diffuse".to_string()),
        ("ambient", "ambient".to_string()),
        ("specular", "specular".to_string()),
        ("specpow", "specpow".to_string()),
        ("reflect", "reflect".to_string()),
    ])
});

struct NextLine {
    line_value: String,
    line_number: usize,
}

type GetNextLineClosure<'a> = Box<dyn FnMut(Option<&NextIfClosure>) -> GetNextLineResult<'a>>;
type GetNextLineResult<'a> = Result<Option<NextLine>, ModelError>;
type NextIfClosure = Box<dyn Fn(&String) -> bool>;

pub fn iterate_input_data<R: BufRead + 'static>(
    mut file_iterator: Peekable<Lines<R>>,
) -> Result<Model, ModelError> {
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
    let mut cones: HashMap<Uuid, Cone> = HashMap::new();
    let mut polygons: HashMap<Uuid, Polygon> = HashMap::new();
    let mut triangles: HashMap<Uuid, Triangle> = HashMap::new();
    let mut surfaces: HashMap<String, Surface> = HashMap::new();

    let mut line_number = 1;

    /*  closure which handles error and edge cases, returns a peekable iterator of the next line's
       content, and sets the while loop condition to false if need be.
       returns None when there are not more lines in the file.
    */
    let mut get_next_line: GetNextLineClosure = Box::new(move |maybe_next_eq_fn| {
        trace!(
            LOG,
            "attempting to retrieve input file line number {}", line_number
        );

        let maybe_line_read_result = match maybe_next_eq_fn {
            Some(ref next_eq_fn) => file_iterator.next_if(|line_result| match line_result {
                Ok(line) => next_eq_fn(line),
                Err(_) => false,
            }),
            None => file_iterator.next(),
        };

        if maybe_line_read_result.is_none() && line_number == 1 {
            return Err(FailedToParseInputFile(
                line_number,
                "input file is empty".to_string(),
            ));
        } else if maybe_line_read_result.is_none() && maybe_next_eq_fn.is_some() {
            trace!(LOG, "file iterator conditionally returned none.");
            return Ok(None);
        } else if maybe_line_read_result.is_none() {
            trace!(LOG, "file iterator returned none. reached the end of file");
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
        trace!(
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
                    Err(_) => {
                        return Err(FailedToParseInputFile(
                            line_number,
                            "light intensity must be a valid decimal value".to_string(),
                        ));
                    }
                },
                None => {
                    return Err(FailedToParseInputFile(
                        line_number,
                        "missing light source intensity value".to_string(),
                    ));
                }
            };

            let light_source_type = match line_words_iter.next() {
                Some(source_type) => match LightSourceType::from_str(source_type) {
                    Ok(light_source_type) => light_source_type,
                    Err(error) => {
                        return Err(FailedToParseInputFile(line_number, error.to_string()));
                    }
                },
                None => {
                    return Err(FailedToParseInputFile(
                        line_number,
                        "missing light source type".to_string(),
                    ));
                }
            };

            // Handle different light types according to the specification
            let (position, radius) = match light_source_type {
                LightSourceType::Ambient => {
                    // Ambient lights don't have position or radius
                    (Coords::new(), 0.0)
                }
                LightSourceType::Point => {
                    // Point lights have position only (Xpos, Ypos, Zpos)
                    let xyz_vec: Vec<&str> = line_words_iter.by_ref().take(3).collect();
                    let position = match Coords::new_from_str_vec(xyz_vec) {
                        Ok(position) => position,
                        Err(error) => {
                            return Err(FailedToParseInputFile(line_number, error.to_string()));
                        }
                    };
                    (position, 0.0) // No radius for point lights
                }
                LightSourceType::Directional => {
                    // Directional lights have direction (Xdir, Ydir, Zdir)
                    let xyz_vec: Vec<&str> = line_words_iter.by_ref().take(3).collect();
                    let direction = match Coords::new_from_str_vec(xyz_vec) {
                        Ok(direction) => direction,
                        Err(error) => {
                            return Err(FailedToParseInputFile(line_number, error.to_string()));
                        }
                    };
                    (direction, 0.0) // No radius for directional lights
                }
                LightSourceType::Extended => {
                    // Extended lights have radius and position (Radius, Xpos, Ypos, Zpos)
                    let radius = match line_words_iter.next() {
                        Some(radius_str) => match radius_str.parse::<f64>() {
                            Ok(radius) => radius,
                            Err(_) => {
                                return Err(FailedToParseInputFile(
                                    line_number,
                                    "light radius must be a valid decimal value".to_string(),
                                ));
                            }
                        },
                        None => {
                            return Err(FailedToParseInputFile(
                                line_number,
                                "missing radius for extended light".to_string(),
                            ));
                        }
                    };

                    let xyz_vec: Vec<&str> = line_words_iter.by_ref().take(3).collect();
                    let position = match Coords::new_from_str_vec(xyz_vec) {
                        Ok(position) => position,
                        Err(error) => {
                            return Err(FailedToParseInputFile(line_number, error.to_string()));
                        }
                    };

                    (position, radius)
                }
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
                radius,
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
                        ));
                    }
                },
                None => {
                    return Err(FailedToParseInputFile(
                        line_number,
                        "missing horizontal fov value".to_string(),
                    ));
                }
            };

            let v_fov: f64 = match line_words_iter.next() {
                Some(v_fov_str) => match v_fov_str.parse::<f64>() {
                    Ok(v_fov) => v_fov,
                    Err(_) => {
                        return Err(FailedToParseInputFile(
                            line_number,
                            format!("invalid vertical fov value: {}", v_fov_str),
                        ));
                    }
                },
                None => {
                    return Err(FailedToParseInputFile(
                        line_number,
                        "missing vertical fov value".to_string(),
                    ));
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

            let horz_screen_px = match line_words_iter.next() {
                Some(h_screen_px_str) => match h_screen_px_str.parse::<usize>() {
                    Ok(h_screen_px) => h_screen_px,
                    Err(_) => {
                        return Err(FailedToParseInputFile(
                            line_number,
                            format!("invalid horizontal screen size: {}", h_screen_px_str),
                        ));
                    }
                },
                None => {
                    return Err(FailedToParseInputFile(
                        line_number,
                        "missing horizontal screen size value".to_string(),
                    ));
                }
            };

            let vert_screen_px = match line_words_iter.next() {
                Some(v_screen_px_str) => match v_screen_px_str.parse::<usize>() {
                    Ok(v_screen_px) => v_screen_px,
                    Err(_) => {
                        return Err(FailedToParseInputFile(
                            line_number,
                            format!("invalid vertical screen size: {}", v_screen_px_str),
                        ));
                    }
                },
                None => {
                    return Err(FailedToParseInputFile(
                        line_number,
                        "missing horizontal screen size value".to_string(),
                    ));
                }
            };

            screen = Screen {
                height: vert_screen_px,
                width: horz_screen_px,
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
            parse_surface::process_surface(
                &mut get_next_line,
                &mut line_words_iter,
                &mut surfaces,
                line_number,
            )?
        } else if SCENE_DATA_KEYWORDS
            .get("sphere")
            .unwrap()
            .eq(peeked_line_word)
        {
            let sphere =
                parse_sphere::process_sphere(&mut line_words_iter, &surfaces, line_number)?;
            trace!(LOG, "processed sphere {}", sphere);
            spheres.insert(sphere.uuid, sphere);
        } else if SCENE_DATA_KEYWORDS
            .get("cylinder")
            .unwrap()
            .eq(peeked_line_word)
        {
            let cone = parse_cylinder_to_cone::process_cylinder_to_cone(
                &mut line_words_iter,
                &surfaces,
                line_number,
            )?;
            trace!(LOG, "processed cylinder as cone {}", cone);
            cones.insert(cone.uuid, cone);
        } else if SCENE_DATA_KEYWORDS
            .get("cone")
            .unwrap()
            .eq(peeked_line_word)
        {
            let cone = parse_cone::process_cone(&mut line_words_iter, &surfaces, line_number)?;
            trace!(LOG, "processed cone {}", cone);
            cones.insert(cone.uuid, cone);
        } else if SCENE_DATA_KEYWORDS
            .get("polygon")
            .unwrap()
            .eq(peeked_line_word)
        {
            let polygon = parse_polygon::process_polygon(
                &mut get_next_line,
                &mut line_words_iter,
                &surfaces,
                line_number,
            )?;
            trace!(LOG, "processed polygon");
            polygons.insert(polygon.uuid, polygon);
        } else if SCENE_DATA_KEYWORDS
            .get("triangle")
            .unwrap()
            .eq(peeked_line_word)
        {
            let triangle = parse_triangle::process_triangle(
                &mut get_next_line,
                &mut line_words_iter,
                &surfaces,
                line_number,
            )?;
            trace!(LOG, "processed polygon");
            triangles.insert(triangle.uuid, triangle);
        } else {
            warn!(
                LOG,
                "unhandled key word \"{}\". line number {}", peeked_line_word, line_number
            );
        }
    }

    let mut model = Model::default();
    model.background = background;
    model.eyep = eyep;
    model.lookp = lookp;
    model.up = up;
    model.fov = fov;
    model.screen = screen;
    model.light_sources = light_sources;
    model.surfaces = surfaces;

    for (_, sphere) in spheres {
        model.upsert_sphere(sphere);
    }

    // Cylinders are converted to cones, so we don't add any cylinders to all_primitives
    for (_, cone) in cones {
        model.upsert_cone(cone);
    }

    for (_, polygon) in polygons {
        model.upsert_polygon(polygon);
    }

    for (_, triangle) in triangles {
        model.upsert_triangle(triangle);
    }

    Ok(model)
}
