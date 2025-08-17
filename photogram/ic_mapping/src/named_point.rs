//a Imports
use std::cell::RefCell;

use serde::{Deserialize, Serialize};

use ic_base::{Point3D, Tag, TagData, TagSet};
use ic_image::Color;

//a NamedPoint
//tp NamedPoint
/// A point in model space, with a name
///
/// This does not support Clone, as it should always be used as an Rc
#[derive(Debug, Serialize, Deserialize)]
pub struct NamedPoint {
    /// Name of the point
    name: Tag,
    /// Color of the point in calibration images
    color: Color,
    /// The 3D model coordinate this point corresponds to and the radius of uncertainty
    ///
    /// This is known for a calibration point, with 0 uncertainty!
    ///
    /// The units are mm (as that is what cameras focal lengths are in)
    // #[serde(deserialize_with = "deserialize_model")]
    model: RefCell<Option<(Point3D, f64)>>,
}

//ip TagData for NamedPoint {
impl TagData for NamedPoint {
    fn tag(&self) -> &Tag {
        &self.name
    }
    fn tag_mut(&mut self) -> &mut Tag {
        &mut self.name
    }
}

//fi deserialize_model
#[allow(dead_code)]
fn deserialize_model<'de, D>(
    deserializer: D,
) -> std::result::Result<RefCell<Option<(Point3D, f64)>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // let model = <Option<(Point3D, f64)>>::deserialize(deserializer)?;
    let model = <Option<Point3D>>::deserialize(deserializer)?;
    let model = model.map(|a| (a, 0.));
    Ok(model.into())
}

//ip Display for NamedPoint
impl std::fmt::Display for NamedPoint {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        if let Some(position) = self.opt_model() {
            write!(
                fmt,
                "{} {} @[{:.2}, {:.2}, {:.2}] +- {:.2}",
                self.name, self.color, position.0[0], position.0[1], position.0[2], position.1
            )
        } else {
            write!(fmt, "{} {} unmapped", self.name, self.color,)
        }
    }
}

//ip NamedPoint
impl NamedPoint {
    pub fn new<S: Into<Tag>>(name: S, color: Color, model: Option<(Point3D, f64)>) -> Self {
        let name = name.into();
        let model = model.into();
        Self { name, color, model }
    }

    pub fn reference<S: Into<String>>(name: S) -> Self {
        let name = Tag::reference(name);
        let color = Color::black();
        let model = None.into();
        Self { name, color, model }
    }

    #[inline]
    pub fn is_unmapped(&self) -> bool {
        self.model.borrow().is_none()
    }
    #[inline]
    pub fn is_mapped(&self) -> bool {
        self.model.borrow().is_some()
    }
    #[inline]
    pub fn model(&self) -> (Point3D, f64) {
        (*self.model.borrow()).unwrap_or_default()
    }
    #[inline]
    pub fn opt_model(&self) -> Option<(Point3D, f64)> {
        *self.model.borrow()
    }
    #[inline]
    pub fn color(&self) -> &Color {
        &self.color
    }
    #[inline]
    pub fn set_model(&self, model: Option<(Point3D, f64)>) {
        *self.model.borrow_mut() = model;
    }
    #[inline]
    pub fn name(&self) -> &Tag {
        &self.name
    }
}
