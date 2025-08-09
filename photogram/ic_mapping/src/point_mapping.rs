//a Imports
use std::collections::HashSet;
use std::rc::Rc;

use geo_nd::Vector;
use serde::{Deserialize, Serialize};

use ic_base::{json, Error, Point2D, Point3D, Result};
use ic_image::Color;

use crate::{NamedPoint, NamedPointSet};

//a PointMapping
//tp PointMapping
#[derive(Debug, Clone)]
pub struct PointMapping {
    /// The 3D model coordinate this point corresponds to
    ///
    /// This is known for a calibration point!
    pub(crate) model: Rc<NamedPoint>,
    /// Screen coordinate
    pub(crate) screen: Point2D,
    /// Error in pixels
    pub(crate) error: f64,
}

//ip PartialEq for PointMapping
impl PartialEq for PointMapping {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.model == other.model
    }
}

//ip Eq for PointMapping
impl Eq for PointMapping {}

//ip Ord for PointMapping
impl Ord for PointMapping {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.model.cmp(&other.model)
    }
}

//ip PartialOrd for PointMapping
impl PartialOrd for PointMapping {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

//ip Serialize for PointMapping
impl Serialize for PointMapping {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeTuple;
        let mut seq = serializer.serialize_tuple(3)?;
        seq.serialize_element(self.model.name())?;
        seq.serialize_element(&self.screen)?;
        seq.serialize_element(&self.error)?;
        seq.end()
    }
}

//ip Deserialize for PointMapping
impl<'de> Deserialize<'de> for PointMapping {
    fn deserialize<DE>(deserializer: DE) -> std::result::Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        let (model_name, screen, error) = <(String, Point2D, f64)>::deserialize(deserializer)?;
        let model = Rc::new(NamedPoint::new(model_name, Color::black(), None));
        Ok(Self {
            model,
            screen,
            error,
        })
    }
}

//ip PointMapping
impl PointMapping {
    //fp new_npt
    pub fn new_npt(model: Rc<NamedPoint>, screen: &Point2D, error: f64) -> Self {
        PointMapping {
            model,
            screen: *screen,
            error,
        }
    }

    //fp within_named_point_set
    pub fn within_named_point_set(&mut self, nps: &NamedPointSet) -> bool {
        if let Some(model) = nps.get_pt(self.model.name()) {
            self.model = model;
            true
        } else {
            false
        }
    }

    //ap is_unmapped
    #[inline]
    pub fn is_unmapped(&self) -> bool {
        self.model.is_unmapped()
    }

    //ap is_mapped
    #[inline]
    pub fn is_mapped(&self) -> bool {
        !self.model.is_unmapped()
    }

    //mp model
    #[inline]
    pub fn model(&self) -> Point3D {
        self.model.model().0
    }

    //mp model_error
    #[inline]
    pub fn model_error(&self) -> f64 {
        self.model.model().1
    }

    //ap screen
    #[inline]
    pub fn screen(&self) -> Point2D {
        self.screen
    }

    //ap error
    #[inline]
    pub fn error(&self) -> f64 {
        self.error
    }

    //ap name
    pub fn name(&self) -> &str {
        self.model.name()
    }

    //ap named_point
    pub fn named_point(&self) -> &Rc<NamedPoint> {
        &self.model
    }

    //zz All done
}
