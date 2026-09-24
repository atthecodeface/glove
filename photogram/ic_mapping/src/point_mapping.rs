//a Imports
use std::rc::Rc;

use geo_nd::Vector;
use serde::{Deserialize, Serialize};

use ic_base::{Point2D, Point3D, Ray};
use ic_camera::{CameraLensProjection, CameraProjection};

use crate::NamedPoint;

//a PointMapping
//tp PointMapping
#[derive(Debug, Clone)]
pub struct PointMapping {
    /// The 3D model coordinate this point corresponds to
    ///
    /// This is known for a calibration point!
    named_point: Rc<NamedPoint>,
    /// Screen coordinate
    screen: Point2D,
    /// Error in pixels
    error: f64,
    /// Whether to use for initial orientation or not
    usage: u64,
}

//ip Serialize for PointMapping
impl Serialize for PointMapping {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeTuple;
        let mut seq = serializer.serialize_tuple(4)?;
        seq.serialize_element(self.named_point.ref_tag().as_str())?;
        seq.serialize_element(&self.screen)?;
        seq.serialize_element(&self.error)?;
        seq.serialize_element(&self.usage)?;
        seq.end()
    }
}

//ip Deserialize for PointMapping
impl<'de> Deserialize<'de> for PointMapping {
    fn deserialize<DE>(deserializer: DE) -> std::result::Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        let (model_name, screen, error, usage) =
            <(String, Point2D, f64, u64)>::deserialize(deserializer)?;
        let named_point = Rc::new(NamedPoint::reference(model_name));
        Ok(Self {
            named_point,
            screen,
            error,
            usage,
        })
    }
}

impl PointMapping {
    //fp new_npt
    pub fn new_npt(named_point: Rc<NamedPoint>, screen: &Point2D, error: f64) -> Self {
        PointMapping {
            named_point,
            screen: *screen,
            error,
            usage: 0,
        }
    }

    pub fn set_np(&mut self, named_point: Rc<NamedPoint>) {
        self.named_point = named_point;
    }
}

//ip PointMapping accessors
impl PointMapping {
    /// Return true if the named point is not mapped
    #[inline]
    pub fn is_unmapped(&self) -> bool {
        self.named_point.is_unmapped()
    }

    /// Return true if the named point is mapped to either a point or a direction (point at infinity, essentially)
    #[inline]
    pub fn is_mapped(&self) -> bool {
        !self.named_point.is_unmapped()
    }

    /// Return true if the named point (assuming it is mapped) is a direction, not a position
    pub fn model_is_direction(&self) -> bool {
        self.named_point.model_is_direction()
    }

    /// Get the named point model position / direction (the latter if
    /// model_is_direction() returns true)
    #[inline]
    pub fn model(&self) -> Point3D {
        self.named_point.model_pt()
    }

    /// Get the *world* direction to the named model point
    #[inline]
    pub fn model_world_direction<C: CameraProjection>(&self, camera: &C) -> Point3D {
        let world = self.named_point.model_pt();
        if self.named_point().model_is_direction() {
            world
        } else {
            camera.world_xyz_to_world_dir(world)
        }
    }

    /// Get the uncertainty in the named point's model data
    ///
    /// If the named point is not mapped, then this is invalid
    ///
    /// If the named point is mapped as a direction (point at infinity) then this is the tan of the angle of uncertainty
    ///
    /// If the named point is mapped as a point then this is the error radius in mm
    #[inline]
    pub fn model_uncertainty(&self) -> f64 {
        self.named_point.model_uncertainty()
    }

    /// Calculate the direction to the model data from the given location
    ///
    /// If the model data is at infinity then the location is ignored
    #[inline]
    pub fn model_direction_from(&self, location: &Point3D) -> Point3D {
        self.named_point.model_direction_from(location)
    }

    #[inline]
    pub fn screen(&self) -> Point2D {
        self.screen
    }

    #[inline]
    pub fn error(&self) -> f64 {
        self.error
    }

    pub fn named_point(&self) -> &Rc<NamedPoint> {
        &self.named_point
    }

    pub fn usage(&self) -> u64 {
        self.usage
    }

    pub fn set_screen(&mut self, screen: Point2D) {
        self.screen = screen;
    }

    pub fn set_error(&mut self, error: f64) {
        self.error = error;
    }

    pub fn set_usage(&mut self, usage: u64) {
        self.usage = usage;
    }
    pub fn is_mapping_of_np(&self, np: &Rc<NamedPoint>) -> bool {
        Rc::ptr_eq(np, &self.named_point)
    }
}

//ip PointMapping camera operations
impl PointMapping {
    //mp get_mapped_unit_vector
    //
    // was get_pm_unit_vector
    /// Get the direction vector for the frame point of a mapping
    ///
    /// This does not apply the camera orientation
    ///
    /// This does apply the lens mapping
    pub fn sensor_as_unit_camera_dir<C: CameraLensProjection>(&self, camera: &C) -> Point3D {
        camera.sensor_px_abs_xy_to_camera_dir(self.screen)
    }

    /// Get the direction vector for the frame point of a mapping in
    /// the world (post-orientation of camera)
    pub fn sensor_as_unit_world_dir<C: CameraLensProjection>(&self, camera: &C) -> Point3D {
        camera.sensor_px_abs_xy_to_world_dir(self.screen)
    }

    //mp get_mapped_ray
    // was get_pm_as_ray
    //
    // used by get_rays, project derive_nps_location
    pub fn get_mapped_ray<C: CameraLensProjection>(&self, camera: &C, from_camera: bool) -> Ray {
        // Can calculate 4 vectors for pm.screen() +- pm.error()
        //
        // Calculate dots with the actual vector - cos of angles
        //
        // tan^2 angle = sec^2 - 1
        let world_pm_direction_vec = self.sensor_as_unit_world_dir(camera);

        let mut min_cos = 1.0;
        for e in [
            [-self.error, 0.],
            [self.error, 0.],
            [0., -self.error],
            [0., self.error],
        ] {
            let e: Point2D = e.into();
            let err_s_xy = self.screen + e;

            let world_err_vec = -camera.sensor_px_abs_xy_to_world_dir(err_s_xy);

            let dot = world_pm_direction_vec.dot(world_err_vec);
            if dot < min_cos {
                min_cos = dot;
            }
        }
        let tan_error_sq = 1.0 / (min_cos * min_cos) - 1.0;
        let tan_error = tan_error_sq.sqrt();

        if from_camera {
            Ray::default()
                .set_start(camera.position())
                .set_direction(world_pm_direction_vec)
                .set_tan_error(tan_error)
        } else {
            Ray::default()
                .set_start(self.model())
                .set_direction(-world_pm_direction_vec)
                .set_tan_error(tan_error)
        }
    }

    /// Calculate the offset from this mappings specified sensor postition to
    /// the derived sensor position (given the camera) of the mapping
    ///
    /// Return None if the mapping is unmapped or if the model point maps behind the camera
    #[inline]
    pub fn get_mapped_dpxy<C: CameraLensProjection>(&self, camera: &C) -> Option<Point2D> {
        self.is_mapped()
            .then(|| {
                camera
                    .world_dir_to_opt_sensor_px_abs_xy(camera.world_xyz_to_world_dir(self.model()))
                    .map(|pxy| self.screen - pxy)
            })
            .flatten()
    }

    /// Get the total dpxy squared error
    #[inline]
    pub fn get_mapped_dpxy_error2<C: CameraLensProjection>(&self, camera: &C) -> f64 {
        if let Some(dpxy) = self.get_mapped_dpxy(camera) {
            let esq = dpxy.length_sq();
            esq * esq / (esq + self.error.powi(2))
        } else {
            0.0
        }
    }

    /// Get the errors in mapping this *point*; does not work for a point mapped to a *direction* named point
    ///
    /// Calculates the direction vector from the camera of the sensor XY of the PM, and of the model point
    ///
    /// For a named point that is a position, it returns the angular error, the
    /// axis of rotation required to rotate the camera by the error angle to
    /// remove the error, and the *world* dxyz assuming the model point is the
    /// specified distance from the camera
    ///
    /// For a named point that is a direction, it returns the angular error, the
    /// axis of rotation required to rotate the camera by the error angle to
    /// remove the error, and an arbitrary length vector
    fn get_mapped_model_error<C: CameraLensProjection>(
        &self,
        camera: &C,
    ) -> (f64, Point3D, Point3D) {
        let model_world_dir = self.model_world_direction(camera);
        let model_dist = model_world_dir.length();
        let model_camera_dir = camera.world_dir_to_camera_dir(model_world_dir).normalize();
        let screen_camera_dir = camera.sensor_px_abs_xy_to_camera_dir(self.screen());

        let axis = model_camera_dir.cross_product(screen_camera_dir);
        let error_angle = axis.length().asin();
        let axis = axis.normalize();

        let dxyz =
            camera.camera_dir_to_world_dir(screen_camera_dir - model_camera_dir) * model_dist;
        if error_angle < 0. {
            (-error_angle, -axis, dxyz)
        } else {
            (error_angle, axis, dxyz)
        }
    }

    //fp show_mapped_error
    pub fn show_mapped_error<C: CameraLensProjection>(&self, camera: &C) {
        if self.is_unmapped() {
            return;
        }
        let Some(camera_scr_xy) =
            camera.world_dir_to_opt_sensor_px_abs_xy(self.model_world_direction(camera))
        else {
            return;
        };
        let (model_angle_error, model_axis, model_dxdy) = self.get_mapped_model_error(camera);
        let dxdy = self.get_mapped_dpxy(camera).unwrap();
        let esq = self.get_mapped_dpxy_error2(camera);
        eprintln!(
            "esq {esq:.2} {} {} <> {:.2}: Maps to {camera_scr_xy:.2}, dxdy {dxdy:.2}: model rot {model_axis:.2} by {:.2} dxdydz {model_dxdy:.2} dist {:.3}  ",
            self.named_point().ref_tag(),
            self.model(),
            self.screen(),
            model_angle_error.to_degrees(),
            model_dxdy.length()
        );
    }

    //zz All done
}
