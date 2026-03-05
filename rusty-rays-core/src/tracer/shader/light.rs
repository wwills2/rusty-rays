use std::fmt;
use std::str::FromStr;

use crate::tracer::Coords;

#[derive(Debug)]
pub struct Light {
    pub position: Coords,
    pub intensity: f64,
    pub source_type: LightSourceType,
    pub radius: f64, // Radius for area lights (used for soft shadows)
}

impl Clone for Light {
    fn clone(&self) -> Light {
        Light {
            position: self.position.clone(),
            intensity: self.intensity,
            source_type: self.source_type.clone(),
            radius: self.radius,
        }
    }
}

#[derive(Debug)]
pub enum LightSourceType {
    Point,
    Directional,
    Ambient,
    Extended,
}

impl Clone for LightSourceType {
    fn clone(&self) -> LightSourceType {
        match self {
            LightSourceType::Point => LightSourceType::Point,
            LightSourceType::Directional => LightSourceType::Directional,
            LightSourceType::Ambient => LightSourceType::Ambient,
            LightSourceType::Extended => LightSourceType::Extended,
        }
    }
}

impl FromStr for LightSourceType {
    type Err = String;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        match str {
            "point" => Ok(LightSourceType::Point),
            "directional" => Ok(LightSourceType::Directional),
            "ambient" => Ok(LightSourceType::Ambient),
            "extended" => Ok(LightSourceType::Extended),
            _ => Err(format!("'{}' is not a valid light source type", str)),
        }
    }
}

impl fmt::Display for LightSourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LightSourceType::Point => write!(f, "point"),
            LightSourceType::Directional => write!(f, "directional"),
            LightSourceType::Ambient => write!(f, "ambient"),
            LightSourceType::Extended => write!(f, "extended"),
        }
    }
}
