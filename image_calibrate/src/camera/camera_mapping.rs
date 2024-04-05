//a Imports
use geo_nd::{quat, Quaternion, Vector, Vector3};

use crate::camera::{BestMapping, CameraView};
use crate::{
    CameraPolynomial, ModelLineSet, NamedPointSet, Point2D, Point3D, PointMapping, PointMappingSet,
    Quat, Ray,
};

//a CameraPtMapping
//tp CameraPtMapping
pub trait CameraPtMapping {
    fn get_pm_dxdy(&self, pm: &PointMapping) -> Option<Point2D>;
    fn get_pm_sq_error(&self, pm: &PointMapping) -> f64;
    fn get_pm_model_error(&self, pm: &PointMapping) -> (f64, Point3D, f64, Point3D);
    fn get_pm_unit_vector(&self, pm: &PointMapping) -> Point3D;
    fn get_pm_direction(&self, pm: &PointMapping) -> Point3D;
    fn get_pm_as_ray(&self, pm: &PointMapping, from_camera: bool) -> Ray;
    fn get_rays(&self, mappings: &[PointMapping], from_camera: bool) -> Vec<(String, Ray)> {
        let mut r = Vec::new();
        for pm in mappings {
            r.push((pm.name().into(), self.get_pm_as_ray(pm, from_camera)));
        }
        r
    }
    fn find_worst_error(&self, mappings: &[PointMapping]) -> (usize, f64);
    fn total_error(&self, mappings: &[PointMapping]) -> f64;
    fn worst_error(&self, mappings: &[PointMapping]) -> f64;
    fn get_quats_for_mappings_given_one(&self, mappings: &[PointMapping], n: usize) -> Vec<Quat>;
}

//ip CameraPtMapping for CameraPolynomial
impl CameraPtMapping for CameraPolynomial {
    //fp get_pm_dxdy
    #[inline]
    fn get_pm_dxdy(&self, pm: &PointMapping) -> Option<Point2D> {
        if pm.is_unmapped() {
            return None;
        }
        let camera_scr_xy = self.world_xyz_to_px_abs_xy(pm.model());
        let dx = pm.screen[0] - camera_scr_xy[0];
        let dy = pm.screen[1] - camera_scr_xy[1];
        Some([dx, dy].into())
    }

    //fp get_pm_sq_error
    #[inline]
    fn get_pm_sq_error(&self, pm: &PointMapping) -> f64 {
        if pm.is_unmapped() {
            0.0
        } else {
            let esq = self.get_pm_dxdy(pm).unwrap().length_sq();
            esq * esq / (esq + pm.error() * pm.error())
        }
    }

    //fp get_pm_model_error
    fn get_pm_model_error(&self, pm: &PointMapping) -> (f64, Point3D, f64, Point3D) {
        let model_rel_xyz = self.world_xyz_to_camera_xyz(pm.model());
        let model_dist = model_rel_xyz.length();
        let model_vec = self.world_xyz_to_camera_txty(pm.model()).to_unit_vector();
        let screen_vec = self.px_abs_xy_to_camera_txty(pm.screen()).to_unit_vector();
        let dxdy = self.camera_xyz_to_world_xyz((-screen_vec) * model_dist) - pm.model();
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

    //mp get_pm_unit_vector
    /// Get the direction vector for the frame point of a mapping
    fn get_pm_unit_vector(&self, pm: &PointMapping) -> Point3D {
        let screen_xy = pm.screen();
        self.px_abs_xy_to_camera_txty(screen_xy).to_unit_vector()
    }

    //mp get_pm_direction
    /// Get the direction vector for the frame point of a mapping in
    /// the world (post-orientation of camera)
    fn get_pm_direction(&self, pm: &PointMapping) -> Point3D {
        // Can calculate 4 vectors for pm.screen() +- pm.error()
        //
        // Calculate dots with the actual vector - cos of angles
        //
        // tan^2 angle = sec^2 - 1
        let screen_xy = pm.screen();
        let camera_pm_txty = self.px_abs_xy_to_camera_txty(screen_xy);
        -self.camera_txty_to_world_dir(&camera_pm_txty)
    }

    //mp get_pm_as_ray
    fn get_pm_as_ray(&self, pm: &PointMapping, from_camera: bool) -> Ray {
        // Can calculate 4 vectors for pm.screen() +- pm.error()
        //
        // Calculate dots with the actual vector - cos of angles
        //
        // tan^2 angle = sec^2 - 1
        let screen_xy = pm.screen();
        let world_pm_direction_vec = self.get_pm_direction(pm);

        let mut min_cos = 1.0;
        let error = pm.error();
        for e in [(-1., 0.), (1., 0.), (0., -1.), (0., 1.)] {
            let err_s_xy = [screen_xy[0] + e.0 * error, screen_xy[1] + e.1 * error];
            let err_c_txty = self.px_abs_xy_to_camera_txty(err_s_xy.into());
            let world_err_vec = -self.camera_txty_to_world_dir(&err_c_txty);
            let dot = world_pm_direction_vec.dot(&world_err_vec);
            if dot < min_cos {
                min_cos = dot;
            }
        }
        let tan_error_sq = 1.0 / (min_cos * min_cos) - 1.0;
        let tan_error = tan_error_sq.sqrt();

        if from_camera {
            Ray::default()
                .set_start(self.position())
                .set_direction(world_pm_direction_vec)
                .set_tan_error(tan_error)
        } else {
            Ray::default()
                .set_start(pm.model())
                .set_direction(-world_pm_direction_vec)
                .set_tan_error(tan_error)
        }
    }

    //fp find_worst_error
    fn find_worst_error(&self, mappings: &[PointMapping]) -> (usize, f64) {
        let mut n = 0;
        let mut worst_e = 0.;
        for (i, pm) in mappings.iter().enumerate() {
            let e = self.get_pm_sq_error(pm);
            if e > worst_e {
                n = i;
                worst_e = e;
            }
        }
        (n, worst_e)
    }

    //fp total_error
    fn total_error(&self, mappings: &[PointMapping]) -> f64 {
        let mut sum_e = 0.;
        for pm in mappings.iter() {
            let e = self.get_pm_sq_error(pm);
            sum_e += e;
        }
        sum_e
    }

    //fp worst_error
    fn worst_error(&self, mappings: &[PointMapping]) -> f64 {
        self.find_worst_error(mappings).1
    }

    //fp get_quats_for_mappings_given_one
    fn get_quats_for_mappings_given_one(&self, mappings: &[PointMapping], n: usize) -> Vec<Quat> {
        let pivot_scr_vec = self
            .px_abs_xy_to_camera_txty(mappings[n].screen())
            .to_unit_vector();
        let pivot_model_vec = (self.position() - mappings[n].model()).normalize();
        let q_s2z = Quat::rotation_of_vec_to_vec(&pivot_scr_vec, &[0., 0., 1.].into());
        let q_m2s = Quat::rotation_of_vec_to_vec(&pivot_model_vec, &pivot_scr_vec);
        let q_m2z = q_s2z * q_m2s;
        let mut result = Vec::new();
        for (i, pm) in mappings.iter().enumerate() {
            if i == n {
                continue;
            }
            let pm_scr_vec = self.px_abs_xy_to_camera_txty(pm.screen()).to_unit_vector();
            let pm_model_vec = (self.position() - pm.model()).normalize();
            let m_mapped = q_m2z.apply3(&pm_model_vec);
            let scr_mapped = q_s2z.apply3(&pm_scr_vec);
            let m_mapped = [m_mapped[0] / m_mapped[2], m_mapped[1] / m_mapped[2]];
            let scr_mapped = [scr_mapped[0] / scr_mapped[2], scr_mapped[1] / scr_mapped[2]];
            let m_angle = m_mapped[1].atan2(m_mapped[0]);
            let scr_angle = scr_mapped[1].atan2(scr_mapped[0]);
            let qp5 = Quat::of_axis_angle(&pivot_scr_vec, -m_angle + scr_angle);
            let q = qp5 * q_m2s;
            result.push(q);
        }
        result
    }

    //zz All done
}

//a CameraShowMapping
pub trait CameraShowMapping {
    fn show_point_set(&self, nps: &NamedPointSet);
    fn show_pm_error(&self, pm: &PointMapping);
    fn show_mappings(&self, mappings: &[PointMapping]);
}

//ip CameraShowMapping for CameraPolynomial
impl CameraShowMapping for CameraPolynomial {
    //fp show_point_set
    fn show_point_set(&self, nps: &NamedPointSet) {
        for (name, model) in nps.iter() {
            if model.is_unmapped() {
                continue;
            }
            let camera_scr_xy = self.world_xyz_to_px_abs_xy(model.model().0);
            eprintln!(
                "model {} : {}+-{} maps to {}",
                name,
                model.model().0,
                model.model().1,
                camera_scr_xy,
            );
        }
    }

    //fp show_pm_error
    fn show_pm_error(&self, pm: &PointMapping) {
        if pm.is_unmapped() {
            return;
        }
        let camera_scr_xy = self.world_xyz_to_px_abs_xy(pm.model());
        let (model_error, model_dxdy, model_angle, model_axis) = self.get_pm_model_error(pm);
        let dxdy = self.get_pm_dxdy(pm).unwrap();
        let esq = self.get_pm_sq_error(pm);
        eprintln!(
            "esq {:.2} {} {} <> {:.2}: Maps to {:.2}, dxdy {:.2}: model rot {:.2} by {:.2} dxdydz {:.2} dist {:.3}  ",
            esq,
            pm.name(),
            pm.model(),
            pm.screen,
            camera_scr_xy,
            dxdy,
            model_axis,
            model_angle,
            model_dxdy,
            model_error
        );
    }

    //fp show_mappings
    fn show_mappings(&self, mappings: &[PointMapping]) {
        for pm in mappings {
            self.show_pm_error(pm);
        }
    }

    //zz All done
}

//a CameraAdjustMapping
pub trait CameraAdjustMapping: std::fmt::Debug + std::fmt::Display + Clone {
    // Used internally
    fn locate_using_model_lines(&mut self, pms: &PointMappingSet, max_np_error: f64) -> f64;
    fn get_location_given_direction(&self, mappings: &[PointMapping]) -> Point3D;
    fn get_best_location(&self, mappings: &[PointMapping], steps: usize) -> BestMapping<Self>;
    fn orient_using_rays_from_model(&mut self, mappings: &[PointMapping]) -> f64;
    fn reorient_using_rays_from_model(&mut self, mappings: &[PointMapping]) -> f64;
}

//ip CameraAdjustMapping for CameraPolynomial
impl CameraAdjustMapping for CameraPolynomial {
    //mp locate_using_model_lines
    fn locate_using_model_lines(&mut self, pms: &PointMappingSet, max_np_error: f64) -> f64 {
        let f = |p: &PointMapping| p.model_error() <= max_np_error;
        let mut mls = ModelLineSet::new(self);
        let mappings = pms.mappings();
        for (i, j) in pms.get_good_screen_pairs(&f) {
            mls.add_line((&mappings[i], &mappings[j]));
        }
        let (location, err) = mls.find_best_min_err_location(30, 500);
        self.set_position(location);
        err
    }

    //fp orient_using_rays_from_model
    fn orient_using_rays_from_model(&mut self, mappings: &[PointMapping]) -> f64 {
        let n = mappings.len();
        assert!(n > 2);
        let mut qs = vec![];
        // eprintln!("Befpre orient {self:?}");
        for i in 0..n {
            let pm = &mappings[i];
            if pm.is_unmapped() {
                continue;
            }
            let screen_xy = pm.screen();
            let camera_pm_txty = self.px_abs_xy_to_camera_txty(screen_xy);
            let di_c = camera_pm_txty.to_unit_vector();
            let di_m = (self.position() - pm.model()).normalize();
            let z_axis: Point3D = [0., 0., 1.].into();
            let qi_c: Quat = quat::rotation_of_vec_to_vec(&di_c.into(), &z_axis.into()).into();

            for j in 0..n {
                let pm = &mappings[j];
                if pm.is_unmapped() {
                    continue;
                }
                if i == j {
                    continue;
                }

                let screen_xy = pm.screen();
                let camera_pm_txty = self.px_abs_xy_to_camera_txty(screen_xy);
                let dj_c = camera_pm_txty.to_unit_vector();
                let dj_m = (self.position() - pm.model()).normalize();

                let qi_m: Quat = quat::rotation_of_vec_to_vec(&di_m.into(), &z_axis.into()).into();
                let dj_c_rotated: Point3D = quat::apply3(qi_c.as_ref(), dj_c.as_ref()).into();
                let dj_m_rotated: Point3D = quat::apply3(qi_m.as_ref(), dj_m.as_ref()).into();

                let theta_dj_m = dj_m_rotated[0].atan2(dj_m_rotated[1]);
                let theta_dj_c = dj_c_rotated[0].atan2(dj_c_rotated[1]);
                let theta = theta_dj_m - theta_dj_c;
                let theta_div_2 = theta / 2.0;
                let cos_2theta = theta_div_2.cos();
                let sin_2theta = theta_div_2.sin();
                let q_z = Quat::of_rijk(cos_2theta, 0.0, 0.0, sin_2theta);

                // At this point, qi_m * di_m = (0,0,1)
                //
                // At this point, q_z.conj * qi_m * di_m = (0,0,1)
                //                q_z.conj * qi_m * dj_m = dj_c_rotated
                //
                let q = qi_c.conjugate() * q_z * qi_m;

                // dc_i === quat::apply3(q.as_ref(), di_m.as_ref()).into();
                // dc_j === quat::apply3(q.as_ref(), dj_m.as_ref()).into();
                // eprintln!("{di_c} ==? {:?}", quat::apply3(q.as_ref(), di_m.as_ref()));
                // eprintln!("{dj_c} ==? {:?}", quat::apply3(q.as_ref(), dj_m.as_ref()));
                self.set_orientation(q);
                // let _te = self.total_error(mappings);
                // eprintln!("total error {_te} : {q} :\n   {self}");

                qs.push((1., q.into()));
            }
        }

        let qr = quat::weighted_average_many(qs.into_iter()).into();
        self.set_orientation(qr);
        let te = self.total_error(mappings);
        eprintln!("total error {te} QR: {qr}");
        te
    }

    //fp reorient_using_rays_from_model
    fn reorient_using_rays_from_model(&mut self, mappings: &[PointMapping]) -> f64 {
        let mut last_te = self.total_error(mappings);
        loop {
            // Find directions to each named point as given by camera (on frame) and by model (model point - camera location)
            let mut qs = vec![];
            let n = mappings.len();
            let initial_orientation = self.orientation();
            qs.push((10. * (n as f64), [0., 0., 0., 1.]));
            for m in mappings {
                if m.is_unmapped() {
                    continue;
                }
                let d_c = self.get_pm_direction(m);
                let d_m = (m.model() - self.position()).normalize();
                let q = quat::rotation_of_vec_to_vec(&d_c.into(), &d_m.into());
                qs.push((1., q));
            }
            let qr: Quat = quat::weighted_average_many(qs.into_iter()).into();

            self.set_orientation(qr * initial_orientation);
            let te = self.total_error(mappings);
            if te > last_te {
                self.set_orientation(initial_orientation);
                break;
            }
            last_te = te;
        }
        last_te
    }

    //fp get_location_given_direction
    fn get_location_given_direction(&self, mappings: &[PointMapping]) -> Point3D {
        let named_rays = self.get_rays(mappings, false);
        let mut ray_list = Vec::new();
        for (_name, ray) in named_rays {
            ray_list.push(ray);
        }
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
    fn get_best_location(&self, mappings: &[PointMapping], steps: usize) -> BestMapping<Self> {
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
                    let te = cp.total_error(mappings);
                    let we = cp.worst_error(mappings);
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
        eprintln!("=> {}", cp_best_mapping);
        cp_best_mapping
    }
}
