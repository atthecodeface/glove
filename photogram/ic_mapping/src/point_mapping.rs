//a Imports
use std::rc::Rc;

use geo_nd::Vector;
use serde::{Deserialize, Serialize};

use ic_base::{Point2D, Point3D, Ray};
use ic_camera::CameraProjection;
use ic_image::Color;

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
}

//ip PartialEq for PointMapping
impl PartialEq for PointMapping {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.named_point, &other.named_point)
    }
}

//ip Eq for PointMapping
impl Eq for PointMapping {}

//ip Ord for PointMapping
impl Ord for PointMapping {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.named_point.cmp(&other.named_point)
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
        seq.serialize_element(self.named_point.name())?;
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
        let named_point = Rc::new(NamedPoint::new(model_name, Color::black(), None));
        Ok(Self {
            named_point,
            screen,
            error,
        })
    }
}

//ip PointMapping constructors
impl PointMapping {
    //fp new_npt
    pub fn new_npt(named_point: Rc<NamedPoint>, screen: &Point2D, error: f64) -> Self {
        PointMapping {
            named_point,
            screen: *screen,
            error,
        }
    }

    //mp set_np
    pub fn set_np(&mut self, named_point: Rc<NamedPoint>) {
        self.named_point = named_point;
    }
}

//ip PointMapping accessors
impl PointMapping {
    //ap is_unmapped
    #[inline]
    pub fn is_unmapped(&self) -> bool {
        self.named_point.is_unmapped()
    }

    //ap is_mapped
    #[inline]
    pub fn is_mapped(&self) -> bool {
        !self.named_point.is_unmapped()
    }

    //mp model
    #[inline]
    pub fn model(&self) -> Point3D {
        self.named_point.model().0
    }

    //mp model_error
    #[inline]
    pub fn model_error(&self) -> f64 {
        self.named_point.model().1
    }

    //ap screen
    #[inline]
    pub fn screen(&self) -> &Point2D {
        &self.screen
    }

    //ap error
    #[inline]
    pub fn error(&self) -> f64 {
        self.error
    }

    //ap name
    pub fn name(&self) -> &str {
        self.named_point.name()
    }

    //ap named_point
    pub fn named_point(&self) -> &Rc<NamedPoint> {
        &self.named_point
    }

    //zz All done
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
    /// used by model line set
    pub fn get_mapped_unit_vector<C: CameraProjection>(&self, camera: &C) -> Point3D {
        camera
            .px_abs_xy_to_camera_txty(&self.screen)
            .to_unit_vector()
    }

    //mp get_mapped_world_dir
    // was get_pm_direction
    //
    // used by reorient_using_rays_from_model

    /// Get the direction vector for the frame point of a mapping in
    /// the world (post-orientation of camera)
    pub fn get_mapped_world_dir<C: CameraProjection>(&self, camera: &C) -> Point3D {
        -camera.camera_txty_to_world_dir(&camera.px_abs_xy_to_camera_txty(&self.screen))
    }

    //mp get_mapped_ray
    // was get_pm_as_ray
    //
    // used by get_rays, project derive_nps_location
    pub fn get_mapped_ray<C: CameraProjection>(&self, camera: &C, from_camera: bool) -> Ray {
        // Can calculate 4 vectors for pm.screen() +- pm.error()
        //
        // Calculate dots with the actual vector - cos of angles
        //
        // tan^2 angle = sec^2 - 1
        let world_pm_direction_vec = self.get_mapped_world_dir(camera);

        let mut min_cos = 1.0;
        for e in [
            [-self.error, 0.],
            [self.error, 0.],
            [0., -self.error],
            [0., self.error],
        ] {
            let e: Point2D = e.into();
            let err_s_xy = self.screen + e;

            let err_c_txty = camera.px_abs_xy_to_camera_txty(&err_s_xy);
            let world_err_vec = -camera.camera_txty_to_world_dir(&err_c_txty);

            let dot = world_pm_direction_vec.dot(&world_err_vec);
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

    //fp get_mapped_dpxy
    // was get_pm_dxdy
    //
    // used by show_pm_error
    #[inline]
    fn get_mapped_dpxy<C: CameraProjection>(&self, camera: &C) -> Option<Point2D> {
        if self.is_unmapped() {
            return None;
        }
        Some(self.screen - camera.world_xyz_to_px_abs_xy(&self.model()))
    }

    //fp get_mapped_dpxy_error2
    // was get_pm_sq_error
    //
    // used for total_error and find_worst_error and show_pm_error
    #[inline]
    pub fn get_mapped_dpxy_error2<C: CameraProjection>(&self, camera: &C) -> f64 {
        if let Some(dpxy) = self.get_mapped_dpxy(camera) {
            let esq = dpxy.length_sq();
            esq * esq / (esq + self.error.powi(2))
        } else {
            0.0
        }
    }

    //fp get_mapped_model_error
    fn get_mapped_model_error<C: CameraProjection>(
        &self,
        camera: &C,
    ) -> (f64, Point3D, f64, Point3D) {
        let model_rel_xyz = camera.world_xyz_to_camera_xyz(&self.model());
        let model_dist = model_rel_xyz.length();
        let model_vec = camera
            .world_xyz_to_camera_txty(&self.model())
            .to_unit_vector();
        let screen_vec = camera
            .px_abs_xy_to_camera_txty(self.screen())
            .to_unit_vector();
        let dxdy = camera.camera_xyz_to_world_xyz(&((-screen_vec) * model_dist)) - self.model();
        let axis = model_vec.cross_product(&screen_vec);
        let sin_sep = axis.length();
        let error = sin_sep * model_dist;
        let angle = sin_sep.asin().to_degrees();
        let axis = axis.normalize();
        if error < 0. {
            (-error, dxdy, -angle, -axis)
        } else {
            (error, dxdy, angle, axis)
        }
    }

    //fp show_mapped_error
    pub fn show_mapped_error<C: CameraProjection>(&self, camera: &C) {
        if self.is_unmapped() {
            return;
        }
        let camera_scr_xy = camera.world_xyz_to_px_abs_xy(&self.model());
        let (model_error, model_dxdy, model_angle, model_axis) =
            self.get_mapped_model_error(camera);
        let dxdy = self.get_mapped_dpxy(camera).unwrap();
        let esq = self.get_mapped_dpxy_error2(camera);
        eprintln!(
            "esq {esq:.2} {} {} <> {:.2}: Maps to {camera_scr_xy:.2}, dxdy {dxdy:.2}: model rot {model_axis:.2} by {model_angle:.2} dxdydz {model_dxdy:.2} dist {model_error:.3}  ",
            self.name(),
            self.model(),
            self.screen(),
        );
    }

    //zz All done
}
