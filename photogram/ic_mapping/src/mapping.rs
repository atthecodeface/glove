//a Imports
use geo_nd::{quat, Quaternion, Vector, Vector3};

use ic_base::{utils, Point3D, Quat, Ray};
use ic_camera::{CameraInstance, CameraProjection};

use crate::{BestMapping, ModelLineSet, PointMapping, PointMappingSet};
//a CameraAdjustMapping
pub trait CameraAdjustMapping: std::fmt::Debug + std::fmt::Display + Clone {
    // Used internally
    fn get_location_given_direction(&self, mappings: &PointMappingSet) -> Point3D;
    fn get_best_location(&self, mappings: &PointMappingSet, steps: usize) -> BestMapping<Self>;
    fn orient_using_rays_from_model(&mut self, mappings: &PointMappingSet) -> f64;
    fn reorient_using_rays_from_model(&mut self, mappings: &PointMappingSet) -> f64;
}

//ip CameraAdjustMapping for CameraInstance
impl CameraAdjustMapping for CameraInstance {
    //fp orient_using_rays_from_model
    #[track_caller]
    fn orient_using_rays_from_model(&mut self, mappings: &PointMappingSet) -> f64 {
        let n = mappings.len();
        assert!(n > 2, "To orient using rays, must have at least 3 mappings");
        let mut qs = vec![];

        for (i, pm) in mappings.mappings().iter().enumerate() {
            if pm.is_unmapped() {
                continue;
            }
            let screen_xy = pm.screen();
            let camera_pm_txty = self.px_abs_xy_to_camera_txty(screen_xy);
            let di_c = -camera_pm_txty.to_unit_vector();
            let di_m = (pm.model() - self.position()).normalize();

            for (j, pm) in mappings.mappings().iter().enumerate() {
                if pm.is_unmapped() {
                    continue;
                }
                if i == j {
                    continue;
                }

                let screen_xy = pm.screen();
                let camera_pm_txty = self.px_abs_xy_to_camera_txty(screen_xy);
                let dj_c = -camera_pm_txty.to_unit_vector();
                let dj_m = (pm.model() - self.position()).normalize();

                qs.push((
                    1.0,
                    utils::orientation_mapping_vpair_to_ppair(
                        di_m.as_ref(),
                        dj_m.as_ref(),
                        &di_c,
                        &dj_c,
                    )
                    .into(),
                ));
            }
        }

        let (qr, e) = utils::weighted_average_many_with_err(&qs);

        let di_c: Point3D = [0., 0., 1.].into();
        let di_m = quat::apply3(&quat::conjugate(qr.as_ref()), &[0., 0., 1.]);

        let qr_c = qr.conjugate();

        for pm in mappings.mappings() {
            if pm.is_unmapped() {
                continue;
            }
            let screen_xy = pm.screen();
            let camera_pm_txty = self.px_abs_xy_to_camera_txty(screen_xy);
            let dj_c = -camera_pm_txty.to_unit_vector();
            let dj_m = (pm.model() - self.position()).normalize();
            let qd = utils::orientation_mapping_vpair_to_ppair(dj_m.as_ref(), &di_m, &dj_c, &di_c);
            let q = qr_c * qd;
            let r = q.as_rijk().0.abs();
            let _err2 = {
                if r < 1.0 {
                    1.0 - r
                } else {
                    0.0
                }
            };
            // eprintln!("{j} {err2:.4e} {}", pm.name());
        }

        self.set_orientation(qr);
        for pm in mappings.mappings() {
            if pm.is_unmapped() {
                continue;
            }
            let _mapped_pxy = self.world_xyz_to_px_abs_xy(pm.model());
            // eprintln!("{j} {mapped_pxy} {}", pm.screen());
        }
        let te = mappings.total_error(self);
        eprintln!("Error in qr's {e} total error {te} QR: {qr}");
        te
    }

    //fp reorient_using_rays_from_model
    fn reorient_using_rays_from_model(&mut self, mappings: &PointMappingSet) -> f64 {
        let mut last_te = mappings.total_error(self);
        loop {
            // Find directions to each named point as given by camera (on frame) and by model (model point - camera location)
            let mut qs = vec![];
            let n = mappings.len();
            let initial_orientation = self.orientation();
            qs.push((10. * (n as f64), [0., 0., 0., 1.]));
            for m in mappings.mappings() {
                if m.is_unmapped() {
                    continue;
                }
                let d_c = m.get_mapped_world_dir(self);
                let d_m = (m.model() - self.position()).normalize();
                let q = quat::rotation_of_vec_to_vec(&d_c.into(), &d_m.into());
                qs.push((1., q));
            }
            let qr: Quat = quat::weighted_average_many(qs.into_iter()).into();

            self.set_orientation(qr * initial_orientation);
            let te = mappings.total_error(self);
            if te > last_te {
                self.set_orientation(initial_orientation);
                break;
            }
            last_te = te;
        }
        last_te
    }

    //fp get_location_given_direction
    fn get_location_given_direction(&self, mappings: &PointMappingSet) -> Point3D {
        // Get list of rays from model to camera
        let ray_list: Vec<_> = mappings
            .mappings()
            .iter()
            .map(|pm| pm.get_mapped_ray(self, false))
            .collect();
        Ray::closest_point(&ray_list, &|r| 1.0 / r.tan_error()).unwrap()
    }

    //fp get_best_location
    /// Get the best location simply, by trying many orientations of
    /// the model and getting the point of intersection for rays
    /// for the point mappings and placing cameras there.
    ///
    /// For each orientation of the model with respect to the camera
    /// this yields a 'best location' for the camera, and placing the
    /// camera there allows the error in mappings for that orientation
    /// to be determined.
    ///
    /// It is important to try a wide range of orientations - so a
    /// uniform mapping of points in the unit sphere to [0,0,1] each
    /// with many rotations around the Z axis is good.
    fn get_best_location(&self, mappings: &PointMappingSet, steps: usize) -> BestMapping<Self> {
        let initial_placement = (self.position(), self.orientation());
        let mut cp = self.clone();
        let mut best_mapping = BestMapping::new(false, initial_placement); // use total error
        for xy in 0..steps * steps {
            let x = xy % steps;
            let y = (xy / steps) % steps;
            let dirn = Point3D::uniform_dist_sphere3(
                [y as f64 / (steps as f64), x as f64 / (steps as f64)],
                true,
            );
            // Note: this may be overkill as it is effectively mapping
            // a uniform xyz to a similarly uniform ijk?
            //
            // qxy places [0., 0., 1.] at dirn. Can choose rijk to have r=0
            //
            // (qxy * (0,0,0,1)) * qxy' = (0, dx, dy, dz)
            // ((0,i,j,k) * (0,0,0,1)) * (0,-i,-j,-k) = (0, dx, dy, dz)
            // (-k, j, -i, 0) * (0,-i,-j,-k) = (0, dx, dy, dz)
            //
            // ik + ik = dx => i = dx / 2k
            // jk + jk = dy => j = dy / 2k
            // k^2 - j^2 -i^2 = dz => k^2 = (dz +1) / 2
            //
            // i^2 + j^2 + k^2 = (dx^2 + dy^2) / 4k^2 + dz/2 + 1/2
            // i^2 + j^2 + k^2 = (1-dz^2) / 2(1+dz) + dz/2 + 1/2
            // i^2 + j^2 + k^2 = (1-dz) / 2 + dz/2 + 1/2 = 1
            let k = ((dirn[2] + 1.0) / 2.0).sqrt();
            let (i, j) = if k < 1.0E-6 {
                // this implies dz=-1 i.e. go for 180 around X
                (1., 0.)
            } else {
                (dirn[0] / 2.0 / k, dirn[1] / 2.0 / k)
            };

            let qxy = Quat::of_rijk(0., -i, -j, -k);

            // let qxy: Quat = quat::of_rijk(0., dirn[0], dirn[1], dirn[2]).into();
            // Map our 'unit sphere direction' to [0,0,1]
            let mut cam = initial_placement;
            cam.1 = qxy * cam.1;
            // Find best rotation around Z axis for this basic orientation
            let mut angle_range = 6.282;
            let mut best_of_axis = BestMapping::new(false, initial_placement);
            for _ in 0..6 {
                for z in 0..(steps * 2 + 1) {
                    let zf = (z as f64) / (steps as f64) - 1.0;
                    let qz = Quat::of_axis_angle(&[0., 0., 1.].into(), zf * angle_range);
                    let mut tc = cam;
                    tc.1 = qz * tc.1;

                    cp.set_orientation(tc.1);
                    tc.0 = self.get_location_given_direction(mappings);
                    cp.set_position(tc.0);
                    let te = mappings.total_error(&cp);
                    let we = mappings.find_worst_error(&cp).1;
                    best_of_axis.update_best(we, te, &tc);
                }
                cam = *best_of_axis.data();
                angle_range /= steps as f64;
                if angle_range < 1.0E-4 {
                    break;
                }
            }
            best_mapping = best_mapping.best_of_both(best_of_axis);
        }
        cp.set_position(best_mapping.data().0);
        cp.set_orientation(best_mapping.data().1);
        let mut cp_best_mapping = BestMapping::new(false, cp.clone());
        cp_best_mapping.update_best(best_mapping.we(), best_mapping.te(), &cp);
        eprintln!("=> {cp_best_mapping}");
        cp_best_mapping
    }
}
