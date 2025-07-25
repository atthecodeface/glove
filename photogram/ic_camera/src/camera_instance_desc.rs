//a Imports
use serde::{Deserialize, Serialize};

use geo_nd::quat;

use ic_base::json;
use ic_base::{Point3D, Quat, Result};

use crate::utils;

//a CameraInstanceDesc
//tp CameraInstanceDesc
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CameraInstanceDesc {
    /// Name of the camera body
    body: String,
    /// The spherical lens mapping polynomial
    lens: String,
    /// The distance the lens if focussed on - make it 1E6*mm_focal_length  for infinity
    mm_focus_distance: f64,
    /// Position in world coordinates of the camera
    position: Point3D,
    /// Orientation to be applied to camera-relative world coordinates
    /// to convert to camera-space coordinates
    orientation: Quat,
}

//ip Display for CameraInstanceDesc
impl std::fmt::Display for CameraInstanceDesc {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            fmt,
            "CameraInstanceDesc[{} + {} @ {}mm] at {}",
            self.body,
            self.lens,
            self.mm_focus_distance(),
            utils::show_pos_orient(&self.position, &self.orientation)
        )
    }
}

//ip CameraInstanceDesc - Accessors
impl CameraInstanceDesc {
    //ap lens
    pub fn lens(&self) -> &str {
        &self.lens
    }

    //ap body
    pub fn body(&self) -> &str {
        &self.body
    }

    //ap position
    pub fn position(&self) -> &Point3D {
        &self.position
    }

    //ap orientation
    pub fn orientation(&self) -> &Quat {
        &self.orientation
    }

    //ap mm_focus_distance
    pub fn mm_focus_distance(&self) -> f64 {
        self.mm_focus_distance
    }

    //ap direction
    pub fn direction(&self) -> Point3D {
        quat::apply3(&quat::conjugate(self.orientation.as_ref()), &[0., 0., 1.]).into()
    }

    //ap centred_on
    pub fn centred_on(&self) -> Point3D {
        let dxyz = self.direction();
        self.position + dxyz * self.mm_focus_distance
    }
}

//ip CameraInstanceDesc - Constructors/desctructors
impl CameraInstanceDesc {
    //cp new
    pub fn new(
        body: String,
        lens: String,
        mm_focus_distance: f64,
        position: Point3D,
        orientation: Quat,
    ) -> Self {
        Self {
            body,
            lens,
            mm_focus_distance,
            position,
            orientation,
        }
    }

    //cp from_json`
    pub fn from_json(json: &str) -> Result<Self> {
        json::from_json("camera instance descriptor", json)
    }

    //dp to_json
    pub fn to_json(self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self)?)
    }
}

//ip CameraInstanceDesc - Modifiers
impl CameraInstanceDesc {
    //mp set_position
    pub fn set_position(&mut self, position: Point3D) {
        self.position = position;
    }

    //mp set_orientation
    pub fn set_orientation(&mut self, orientation: Quat) {
        self.orientation = orientation;
    }

    //mp set_mm_focus_distance
    pub fn set_mm_focus_distance(&mut self, mm_focus_distance: f64) {
        self.mm_focus_distance = mm_focus_distance;
    }
}
