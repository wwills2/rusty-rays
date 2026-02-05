use crate::tracer::Color;
use image::{ImageBuffer, ImageFormat, RgbImage};
use std::fmt;
use std::io::Cursor;
use std::path::PathBuf;

mod config;
pub mod logger;

pub use config::Config;

/// Write the rendered image to a file. clamps pixel values to [0, 255]
pub fn write_render_to_file(
    output_file_path: &PathBuf,
    raw_image_data: &Vec<Vec<Color>>,
) -> Result<(), WriteError> {
    let image = raw_to_image_data(raw_image_data)?;

    if let Some(parent_path) = output_file_path.parent()
        && !parent_path.is_dir()
        && let Err(create_dir_result) = std::fs::create_dir(parent_path)
    {
        return Err(WriteError(create_dir_result.to_string()));
    }

    match image.save(output_file_path) {
        Ok(_) => Ok(()),
        Err(error) => Err(WriteError(format!(
            "cannot write image data to {}. Error: {}",
            output_file_path.display(),
            error
        ))),
    }
}

/// Create the rendered image bytes in-memory (no filesystem I/O).
/// Pixel values are assumed to already be clamped/converted by `raw_to_image_data`.
pub fn write_render_to_image_buffer(
    image_format: String,
    raw_image_data: &Vec<Vec<Color>>,
) -> Result<Vec<u8>, WriteError> {
    let valid_format = match ImageFormat::from_extension(&image_format) {
        Some(format) => format,
        None => {
            return Err(WriteError(format!(
                "unsupported image format: {}",
                image_format
            )));
        }
    };

    let image = raw_to_image_data(raw_image_data)?;

    let mut image_buffer = Cursor::new(Vec::new());
    image
        .write_to(&mut image_buffer, valid_format)
        .map_err(|e| {
            WriteError(format!(
                "cannot encode image to {:?}. Error: {}",
                image_format, e
            ))
        })?;

    Ok(image_buffer.into_inner())
}

/// normalize raw render data to an image type. clamps pixel values to [0, 255]
pub fn raw_to_image_data(raw_image_data: &Vec<Vec<Color>>) -> Result<RgbImage, WriteError> {
    let height = raw_image_data.len();
    let width = raw_image_data[0].len();
    let mut normalized_image_data: Vec<u8> = vec![];

    for row in raw_image_data {
        for color in row {
            let normalized_color = color.normalize();
            normalized_image_data.push(normalized_color.r);
            normalized_image_data.push(normalized_color.g);
            normalized_image_data.push(normalized_color.b);
        }
    }

    let maybe_image: Option<RgbImage> =
        ImageBuffer::from_raw(width as u32, height as u32, normalized_image_data);
    match maybe_image {
        Some(image) => Ok(image),
        None => Err(WriteError(
            "cannot write raw render data into image format".to_string(),
        )),
    }
}

#[derive(Debug)]
pub struct WriteError(String);

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to write render data to file. Error: {}", self.0)
    }
}

impl std::error::Error for WriteError {}
