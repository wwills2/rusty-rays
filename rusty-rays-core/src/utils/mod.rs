use crate::tracer::Color;
use bincode::config::{BigEndian, Configuration, Varint};
use bincode::error::{DecodeError, EncodeError};
use image::{ImageBuffer, RgbImage};
use std::fmt;
use std::path::PathBuf;

mod config;
pub mod logger;

pub use config::Config;

/// Serializes the 2D vector of pixel Colors into a blob of u8 for lossless binary storage.
/// Deserialize with deserialize_raw_to_blob
/// This serialization does not represent a string of pixel colors.
pub fn serialize_raw_render_to_blob(
    raw_image_data: &Vec<Vec<Color>>,
) -> Result<Vec<u8>, EncodeError> {
    let config: Configuration<BigEndian, Varint> = Configuration::default();
    bincode::serde::encode_to_vec(raw_image_data, config)
}

/// Deserializes the result of serialize_raw_to_blob into a 2D vector of pixel Color
pub fn deserialize_blob_to_raw_render(
    serialized_raw_image_data: &[u8],
) -> Result<Vec<Vec<Color>>, DecodeError> {
    let config: Configuration<BigEndian, Varint> = Configuration::default();
    let (raw_image_data, _) = bincode::serde::decode_from_slice(serialized_raw_image_data, config)?;

    Ok(raw_image_data)
}

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
