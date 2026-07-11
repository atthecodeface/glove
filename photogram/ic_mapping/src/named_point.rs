//a Imports
use std::cell::{Ref, RefCell, RefMut};

use serde::{Deserialize, Serialize};

use ic_base::{Point3D, Tag, TagData};
use ic_image::Color;

//a NamedPoint
//tp NamedPoint
/// A point in model space, with a name
///
/// This does not support Clone, as it should always be used as an Rc
#[derive(Debug, Serialize, Deserialize)]
pub struct NamedPoint {
    /// Name of the point
    ///
    /// Can this by a RefCell? If so, it cannot be a TagData
    name: RefCell<Tag>,
    /// Color of the point in calibration images
    color: RefCell<Color>,
    /// The 3D model coordinate this point corresponds to and the radius of uncertainty
    ///
    /// The bool is 'at_infinity' - i.e this is a known direction (with no uncertainty), not a 3D position
    ///
    /// This is known for a calibration point, with 0 uncertainty!
    ///
    /// The units for a model position are mm (as that is what cameras focal lengths are in)
    // #[serde(deserialize_with = "deserialize_model")]
    model: RefCell<Option<(bool, Point3D, f64)>>,
}

//ip TagData for NamedPoint {
impl TagData for NamedPoint {
    fn tag(&self) -> &RefCell<Tag> {
        &self.name
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
            let (at_infinity, xyz, error) = position;
            if at_infinity {
                write!(
                    fmt,
                    "{} {} -> [{:.2}, {:.2}, {:.2}]",
                    self.name.borrow(),
                    self.color.borrow(),
                    xyz[0],
                    xyz[1],
                    xyz[2]
                )
            } else {
                write!(
                    fmt,
                    "{} {} @[{:.2}, {:.2}, {:.2}] +- {:.2}",
                    self.name.borrow(),
                    self.color.borrow(),
                    xyz[0],
                    xyz[1],
                    xyz[2],
                    error
                )
            }
        } else {
            write!(
                fmt,
                "{} {} unmapped",
                self.name.borrow(),
                self.color.borrow(),
            )
        }
    }
}

//ip NamedPoint
impl NamedPoint {
    //cp new
    /// Create a new NamedPoint, within a NamedPointSet
    ///
    /// The Tag must thus be Owned or Shared
    pub fn new(name: Tag, color: Color, model: Option<(bool, Point3D, f64)>) -> Self {
        let model = model.into();
        let name = name.into();
        let color = color.into();
        Self { name, color, model }
    }

    pub fn reference<S: Into<String>>(name: S) -> Self {
        let name = Tag::make_unresolved(name).into();
        let color = Color::black().into();
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
    pub fn model(&self) -> (bool, Point3D, f64) {
        (*self.model.borrow()).unwrap_or_default()
    }

    #[inline]
    pub fn model_is_direction(&self) -> bool {
        (*self.model.borrow()).unwrap_or_default().0
    }

    #[inline]
    pub fn model_pt(&self) -> Point3D {
        (*self.model.borrow()).unwrap_or_default().1
    }

    #[inline]
    pub fn model_uncertainty(&self) -> f64 {
        (*self.model.borrow()).unwrap_or_default().2
    }

    #[inline]
    pub fn opt_model(&self) -> Option<(bool, Point3D, f64)> {
        *self.model.borrow()
    }

    #[inline]
    pub fn color(&self) -> Color {
        *self.color.borrow()
    }

    #[inline]
    pub fn set_model(&self, model: Option<(bool, Point3D, f64)>) {
        *self.model.borrow_mut() = model;
    }

    #[inline]
    pub fn set_color(&self, color: Color) {
        *self.color.borrow_mut() = color;
    }

    #[inline]
    pub fn ref_tag<'a>(&'a self) -> Ref<'a, Tag> {
        self.name.borrow()
    }

    pub fn cmp_np_name(&self, np: &NamedPoint) -> std::cmp::Ordering {
        self.name.borrow().as_str().cmp(np.name.borrow().as_str())
    }

    pub fn has_name(&self, name: &str) -> bool {
        self.name.borrow().as_str() == name
    }
}
