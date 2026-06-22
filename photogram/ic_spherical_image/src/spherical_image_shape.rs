use serde::{Deserialize, Serialize};

use ic_base::JsonParsable;

use crate::shapes;
use crate::{SphericalData, SphericalImageError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum SphericalImageShape {
    Tetrahedron,
    Octahedron,
    Icosahedron,
}

impl JsonParsable for SphericalImageShape {
    type PostParseArg = ();
    type PostParseResult = SphericalImageShape;
    fn reason() -> &'static str {
        "SphericalImageShape"
    }
    fn post_parse(self, _args: &()) -> ic_base::Result<Self> {
        Ok(self)
    }
}

impl std::fmt::Display for SphericalImageShape {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SphericalImageShape::Tetrahedron => fmt.write_str("tetrahedron"),
            SphericalImageShape::Octahedron => fmt.write_str("octahedron"),
            SphericalImageShape::Icosahedron => fmt.write_str("icosahedron"),
        }
    }
}

impl std::str::FromStr for SphericalImageShape {
    type Err = SphericalImageError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tetrahedron" => Ok(SphericalImageShape::Tetrahedron),
            "octahedron" => Ok(SphericalImageShape::Octahedron),
            "icosahedron" => Ok(SphericalImageShape::Icosahedron),
            _ => Err(SphericalImageError::BadShape(s.into())),
        }
    }
}

impl SphericalImageShape {
    pub fn to_spherical_data(&self) -> Result<SphericalData, SphericalImageError> {
        match self {
            SphericalImageShape::Tetrahedron => Ok(SphericalData::of_shape(
                shapes::TETRA_POINTS,
                shapes::TETRAHEDRON,
            )),
            SphericalImageShape::Icosahedron => Ok(SphericalData::of_shape(
                shapes::ICOS_POINTS,
                shapes::ICOSAHEDRON,
            )),
            SphericalImageShape::Octahedron => Ok(SphericalData::of_shape(
                shapes::OCTA_POINTS,
                shapes::OCTAHEDRON,
            )),
        }
    }
}
