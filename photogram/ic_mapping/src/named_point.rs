//a Imports
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::hash_set::Difference,
};

use geo_nd::{Quaternion, Vector};
use ic_camera::{AdjustableCameraProjection, CameraLensProjection, CameraProjection};
use serde::{Deserialize, Serialize};

use ic_base::{ModelData, Point3D, Quat, Tag, TagData};
use ic_image::Color8;

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
    color: RefCell<Color8>,
    /// The 3D model coordinate this point corresponds to, radius of uncertainty, etc
    #[serde(deserialize_with = "ModelData::deserialize_refcell")]
    model: RefCell<ModelData>,
    /// Preferred CIP name for generating its patch
    #[serde(default)]
    preferred_cip: String,
}

//ip TagData for NamedPoint {
impl TagData for NamedPoint {
    fn tag(&self) -> &RefCell<Tag> {
        &self.name
    }
}

impl std::fmt::Display for NamedPoint {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            fmt,
            "{} {} {}",
            self.name.borrow(),
            self.color.borrow(),
            self.model.borrow()
        )
    }
}

//ip NamedPoint
impl NamedPoint {
    /// Create a new NamedPoint, within a NamedPointSet
    ///
    /// The Tag must thus be Owned or Shared
    pub fn new(name: Tag, color: Color8) -> Self {
        let name = name.into();
        let color = color.into();
        let model = ModelData::default().into();
        let preferred_cip = "".into();
        Self {
            name,
            color,
            model,
            preferred_cip,
        }
    }

    /// Create a NamedPoint that is an unresolved reference (for deserializing a
    /// PointMapping); this will be resolved later into a named point defined in
    /// a set
    pub fn reference<S: Into<String>>(name: S) -> Self {
        Self::new(Tag::make_unresolved(name).into(), Color8::black().into())
    }

    #[inline]
    pub fn is_unmapped(&self) -> bool {
        self.model.borrow().is_unmapped()
    }

    #[inline]
    pub fn is_mapped(&self) -> bool {
        self.model.borrow().is_mapped()
    }

    #[inline]
    pub fn model_is_direction(&self) -> bool {
        self.model.borrow().model_is_direction()
    }

    #[inline]
    pub fn model_pt(&self) -> Point3D {
        self.model.borrow().model_pt()
    }

    #[inline]
    pub fn model_uncertainty(&self) -> f64 {
        self.model.borrow().model_uncertainty()
    }

    /// Calculate the direction to the model data from the given location
    ///
    /// If the model data is at infinity then the location is ignored
    pub fn model_direction_from(&self, location: &Point3D) -> Point3D {
        self.model.borrow().model_direction_from(location)
    }

    /// Get the *world* direction to the named model point
    #[inline]
    pub fn model_world_direction<C: CameraProjection>(&self, camera: &C) -> Point3D {
        let model = self.model.borrow();
        if model.model_is_direction() {
            model.model_pt()
        } else {
            camera.world_xyz_to_world_dir(model.model_pt())
        }
    }

    #[inline]
    pub fn color(&self) -> Color8 {
        *self.color.borrow()
    }

    /// Set the model position or direction to a given value, with an error value
    ///
    /// If the bool is true the point is 'at infinity' and the vector is a direction
    #[inline]
    pub fn model_mut<'a>(&'a self) -> RefMut<'a, ModelData> {
        self.model.borrow_mut()
    }
    pub fn model<'a>(&'a self) -> Ref<'a, ModelData> {
        self.model.borrow()
    }

    #[inline]
    pub fn set_color(&self, color: Color8) {
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

    /// Set the camera orientation (an potentially position) so that it points to the named point, with the named point's surface 'tangent' as 'Up'
    ///
    /// Set the position if the named point is not a placed point; otherwise ignore the distance
    #[inline]
    pub fn set_camera_for_facet<C: AdjustableCameraProjection>(
        &self,
        camera: &mut C,
        mm_distance_to_point: f64,
    ) {
        let model = self.model.borrow().tidy_patch_data();
        let direction: Point3D;
        let up = model.tangent();
        if model.model_is_direction() {
            direction = model.model_pt();
        } else {
            direction = model.normal();
        }
        let direction = direction.normalize();
        let up = up
            .cross_product(direction)
            .cross_product(direction)
            .normalize();
        if up.length_sq() < 0.9 {
            panic!("up not perpendiculr to direction")
        };
        camera.set_orientation(&Quat::look_at(&direction, &up));
        if !model.model_is_direction() {
            camera.set_position(&(model.model_pt() - direction * mm_distance_to_point));
        }
    }

    /// Get the tan of half of the field of view required for the facet, given its angle (if a direction) or its distance from the camera, plus the facet size
    ///
    /// In the case of a model point (not a direction) the result is the facet width divide by 2, then divided by the distance from the camera to the model point
    pub fn facet_tan_hfov(&self, mm_distance_to_point: f64) -> f64 {
        let model = self.model.borrow();
        if model.model_is_direction() {
            model.facet_size().tan() / 2.0
        } else {
            model.facet_size() / 2.0 / mm_distance_to_point
        }
    }
}
