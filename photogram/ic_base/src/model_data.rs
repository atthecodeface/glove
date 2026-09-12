use std::cell::RefCell;

use geo_nd::Vector;
use serde::{Deserialize, Serialize};

use crate::Point3D;

/// The 3D model coordinate this point corresponds to and the radius of uncertainty
///
/// The bool is 'at_infinity' - i.e this is a known direction (with no uncertainty), not a 3D position
///
/// This is known for a calibration point, with 0 uncertainty!
///
/// The units for a model position are mm (as that is what cameras focal lengths are in)
// #[serde(deserialize_with = "deserialize_model")]
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub enum ModelDataKind {
    #[default]
    None,
    Direction,
    Model,
}
impl ModelDataKind {
    #[inline]
    pub fn is_unmapped(&self) -> bool {
        matches!(self, ModelDataKind::None)
    }

    #[inline]
    pub fn is_mapped(&self) -> bool {
        !matches!(self, ModelDataKind::None)
    }

    #[inline]
    pub fn is_direction(&self) -> bool {
        matches!(self, ModelDataKind::Direction)
    }
}

///
/// The bool is 'at_infinity' - i.e this is a known direction (with no uncertainty), not a 3D position
///
/// This is known for a calibration point, with 0 uncertainty!
///
/// The units for a model position are mm (as that is what cameras focal lengths are in)

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct ModelData {
    data_kind: ModelDataKind,
    /// If a model point, the position in space in mm; if a direction, then the direction vector
    model: Point3D,
    /// Uncertainty in the position (in mm), or in radians if angular
    ///
    /// If 0.0 then this is a *known* model point/direction
    uncertainty: f64,
    /// For model points, the direction the surface faces
    ///
    /// For 'direction' points, this is not required; the 'model' must be the normal
    normal: Point3D,
    /// For model points, a perpendicular to the normal that defines the facet geometry
    ///
    /// For 'direction' points, this is perpendicular to the model direction
    tangent: Point3D,
    /// For model points, the size in mm of the facet
    ///
    /// For directions this is an angle in radians
    facet_size: f64,
}

impl std::fmt::Display for ModelData {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self.data_kind {
            ModelDataKind::None => {
                write!(fmt, "unmapped",)
            }
            ModelDataKind::Direction => {
                write!(fmt, "-> [{:.2}]", self.model)
            }
            ModelDataKind::Model => {
                write!(fmt, "@[{:.2}] +- {:.2}", self.model, self.uncertainty)
            }
        }
    }
}

impl ModelData {
    pub fn deserialize_refcell<'de, D>(
        deserializer: D,
    ) -> std::result::Result<RefCell<Self>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(deserializer.deserialize_any(ModelDataVisitor)?.into())
    }

    pub fn at_infinity(dirn: Point3D) -> Self {
        Self {
            data_kind: ModelDataKind::Direction,
            model: dirn,
            ..Default::default()
        }
    }
    pub fn at_position(model: Point3D) -> Self {
        Self {
            data_kind: ModelDataKind::Model,
            model,
            ..Default::default()
        }
    }
    #[inline]
    pub fn with_uncertainty(mut self, uncertainty: f64) -> Self {
        self.uncertainty = uncertainty;
        self
    }
    #[inline]
    pub fn with_normal(mut self, normal: Point3D) -> Self {
        self.normal = normal;
        self
    }
    #[inline]
    pub fn with_tangent(mut self, tangent: Point3D) -> Self {
        self.tangent = tangent;
        self
    }
    #[inline]
    pub fn with_facet_size(mut self, facet_size: f64) -> Self {
        self.facet_size = facet_size;
        self
    }

    /// Return true if the model data is unmapped
    #[inline]
    pub fn is_unmapped(&self) -> bool {
        self.data_kind.is_unmapped()
    }

    /// Return true if the model data is mapped
    #[inline]
    pub fn is_mapped(&self) -> bool {
        self.data_kind.is_mapped()
    }

    /// Return true if the model data is a direction
    #[inline]
    pub fn model_is_direction(&self) -> bool {
        self.data_kind.is_direction()
    }

    /// Get the model position/direction
    #[inline]
    pub fn model_pt(&self) -> Point3D {
        self.model
    }

    /// Get the uncertainty in the model position/direction
    #[inline]
    pub fn model_uncertainty(&self) -> f64 {
        self.uncertainty
    }

    /// Calculate the direction to the model data from the given location
    ///
    /// If the model data is at infinity then the location is ignored
    pub fn model_direction_from(&self, location: &Point3D) -> Point3D {
        match self.data_kind {
            ModelDataKind::Direction => self.model.normalize(),
            ModelDataKind::Model => (location - self.model).normalize(),
            ModelDataKind::None => *location,
        }
    }
}

struct ModelDataVisitor;

impl<'de> serde::de::Visitor<'de> for ModelDataVisitor {
    type Value = ModelData;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("None, or an old-fashioned (bool,Dirn,err), or a proper ModelData")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ModelData::default())
    }
    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let (at_infinity, dirn, uncertainty) =
            <(bool, Point3D, f64)>::deserialize(serde::de::value::SeqAccessDeserializer::new(seq))?;
        if at_infinity {
            Ok(ModelData {
                data_kind: ModelDataKind::Direction,
                model: dirn,
                uncertainty,
                normal: dirn,
                tangent: Point3D::default(),
                facet_size: 0.0,
            })
        } else {
            Ok(ModelData {
                data_kind: ModelDataKind::Direction,
                model: dirn,
                uncertainty,
                normal: dirn,
                tangent: Point3D::default(),
                facet_size: 0.0,
            })
        }
    }
    fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
    where
        M: serde::de::MapAccess<'de>,
    {
        Deserialize::deserialize(serde::de::value::MapAccessDeserializer::new(map))
    }
}
