use crate::tracer::coords::Coords;
use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub struct Light {
    pub position: Coords,
    pub intensity: f64,
    pub source_type: LightSourceType,
}

impl Clone for Light {
    fn clone(&self) -> Light {
        Light {
            position: self.position,
            intensity: self.intensity,
            source_type: self.source_type.clone(),
        }
    }
}

#[derive(Debug)]
pub enum LightSourceType {
    Point,
    Directional,
}

impl Clone for LightSourceType {
    fn clone(&self) -> LightSourceType {
        match self {
            LightSourceType::Point => LightSourceType::Point,
            LightSourceType::Directional => LightSourceType::Directional,
        }
    }
}

impl FromStr for LightSourceType {
    type Err = String;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        match str {
            "point" => Ok(LightSourceType::Point),
            "directional" => Ok(LightSourceType::Directional),
            _ => Err(format!("'{}' is not a valid LightSourceType", str)),
        }
    }
}

impl fmt::Display for LightSourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LightSourceType::Point => write!(f, "point"),
            LightSourceType::Directional => write!(f, "directional"),
        }
    }
}
