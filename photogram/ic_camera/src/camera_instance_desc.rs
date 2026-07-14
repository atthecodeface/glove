use serde::{Deserialize, Serialize};

use geo_nd::Quaternion;

use ic_base::{JsonParsable, Point3D, Quat, Result};

use crate::{CameraDatabase, CameraInstance};

use crate::utils;

/// This structure is an abstracted description of a camera instance, using
/// names for the camera body and lens
///
/// To create a camera instance, a camera database of lenses and bodies is
/// required to convert the names to actual camera body and lens structures
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CameraInstanceDesc {
    /// Name of the camera body
    body: String,

    /// The lens attached
    lens: String,

    /// The distance the lens if focussed on
    ///
    /// For infinity this can be 1E6 times the focal length (for example), as at
    /// that level there is minimal difference.
    mm_focus_distance: f64,

    /// Position in world coordinates of the camera
    position: Point3D,

    /// Orientation to be applied to camera-relative world coordinates
    /// to convert to camera-space coordinates
    orientation: Quat,
}

impl JsonParsable for CameraInstanceDesc {
    fn reason() -> &'static str {
        "camera instance descriptor"
    }
    type PostParseArg = CameraDatabase;
    type PostParseResult = CameraInstance;
    fn post_parse(self, cdb: &CameraDatabase) -> Result<CameraInstance> {
        CameraInstance::from_desc(cdb, self)
    }
}

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

impl CameraInstanceDesc {
    /// Get the name of the lens
    pub fn lens(&self) -> &str {
        &self.lens
    }

    /// Get the name of the body
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Get the position of the camera in the world
    pub fn position(&self) -> &Point3D {
        &self.position
    }

    /// Get the orientation of the camera, which maps world direction to
    /// camera-relative direction
    pub fn orientation(&self) -> &Quat {
        &self.orientation
    }

    /// Get the focusing distance for this camera instance
    pub fn mm_focus_distance(&self) -> f64 {
        self.mm_focus_distance
    }

    /// Get the direction the camera was pointing
    ///
    /// The centre of the image has relative XY of (0,0), and hence a camera
    /// space direction
    pub fn world_direction(&self) -> Point3D {
        self.orientation
            .conjugate()
            .apply3_arr(&[0., 0., -1.])
            .into()
    }

    /// Find the world position that the camera was focussed on
    pub fn posn_focussed_on(&self) -> Point3D {
        let dxyz = self.world_direction();
        self.position + dxyz * self.mm_focus_distance
    }
}

impl CameraInstanceDesc {
    /// Create a new [CameratInstanceDesc] given the data
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

    /// Convert the structure to Json
    pub fn to_json(self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self)?)
    }

    /// Set the world position of the camera instance
    pub fn set_position(&mut self, position: Point3D) {
        self.position = position;
    }

    /// Set the world-to-camera orientation quaterion for the camera instance
    pub fn set_orientation(&mut self, orientation: Quat) {
        self.orientation = orientation;
    }

    /// Set the distance of focus for the image
    pub fn set_mm_focus_distance(&mut self, mm_focus_distance: f64) {
        self.mm_focus_distance = mm_focus_distance;
    }
}
