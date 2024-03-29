//a Imports
use std::rc::Rc;

use geo_nd::{Quaternion, Vector, Vector3};
use serde::Serialize;

use crate::{
    CameraInstance, CameraPolynomial, CameraView, NamedPointSet, Point2D, Point3D, PointMapping,
    Quat, Ray, Rotations,
};

//a Constants
const MIN_ERROR: f64 = 0.5;

//a BestMapping
//tp BestMapping
/// A means for tracking the best mapping
#[derive(Debug, Clone)]
pub struct BestMapping<T: std::fmt::Display + std::fmt::Debug + Clone> {
    /// Asserted if the worst error should be used in evaluating error totals
    use_we: bool,
    /// The worst error
    we: f64,
    /// The total error
    te: f64,
    /// Associated data
    data: T,
}

//ip Copy for BestMapping<T>
impl<T> Copy for BestMapping<T> where T: std::fmt::Debug + std::fmt::Display + Copy {}

//ip BestMapping
impl<T> BestMapping<T>
where
    T: std::fmt::Debug + std::fmt::Display + Clone,
{
    //fp new
    /// Create a new best mapping
    pub fn new(use_we: bool, data: T) -> Self {
        Self {
            use_we,
            we: f64::MAX,
            te: f64::MAX,
            data,
        }
    }

    //ap we
    pub fn we(&self) -> f64 {
        self.we
    }

    //ap te
    pub fn te(&self) -> f64 {
        self.te
    }

    //ap data
    pub fn data(&self) -> &T {
        &self.data
    }

    //ap into_data
    pub fn into_data(self) -> T {
        self.data
    }

    //mp update_best
    /// Update the mapping with data if this is better
    pub fn update_best(&mut self, we: f64, te: f64, data: &T) -> bool {
        if self.use_we && we > self.we {
            return false;
        }
        if !self.use_we && te > self.te {
            return false;
        }
        self.we = we;
        self.te = te;
        self.data = data.clone();
        true
    }

    //cp best_of_both
    /// Pick the best of both
    pub fn best_of_both(self, other: Self) -> Self {
        let pick_self = if self.use_we {
            other.we > self.we
        } else {
            other.te > self.te
        };
        if pick_self {
            self
        } else {
            other
        }
    }

    //zz All done
}

//ip Display for Best
impl<T> std::fmt::Display for BestMapping<T>
where
    T: std::fmt::Debug + std::fmt::Display + Clone,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "we: {:.4} te: {:.4} : {}", self.we, self.te, self.data,)
    }
}

//a CameraMapping
//tp CameraMapping
/// A camera that allows mapping a world point to camera relative XYZ,
/// and then it can be mapped to tan(x) / tan(y) to roll/yaw or pixel
/// relative XY (relative to the center of the camera sensor)
#[derive(Debug, Clone, Serialize)]
pub struct CameraMapping {
    #[serde(flatten)]
    camera: CameraInstance,
}

//ip Display for CameraMapping
impl std::fmt::Display for CameraMapping {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.camera.fmt(fmt)
    }
}

//ip Deref for CameraMapping
impl std::ops::Deref for CameraMapping {
    type Target = CameraInstance;
    fn deref(&self) -> &CameraInstance {
        &self.camera
    }
}

//ip DerefMut for CameraMapping
impl std::ops::DerefMut for CameraMapping {
    fn deref_mut(&mut self) -> &mut CameraInstance {
        &mut self.camera
    }
}

//ip CameraMapping
impl CameraMapping {
    //fp new
    pub fn new(projection: Rc<CameraPolynomial>, position: Point3D, direction: Quat) -> Self {
        let camera = CameraInstance::new(projection, position, direction);
        Self { camera }
    }

    //fp of_camera
    pub fn of_camera(camera: CameraInstance) -> Self {
        Self { camera }
    }

    //ap camera
    pub fn camera(&self) -> &CameraInstance {
        &self.camera
    }

    //mp placed_at
    pub fn placed_at(&self, location: Point3D) -> Self {
        Self {
            camera: self.camera.clone().placed_at(location),
        }
    }

    //mp with_direction
    pub fn with_direction(&self, direction: Quat) -> Self {
        Self {
            camera: self.camera.clone().with_direction(direction),
        }
    }

    //mp moved_by
    pub fn moved_by(&self, dp: Point3D) -> Self {
        Self {
            camera: self.camera.clone().moved_by(dp),
        }
    }

    //mp rotated_by
    pub fn rotated_by(&self, q: &Quat) -> Self {
        Self {
            camera: self.camera.clone().rotated_by(q),
        }
    }

    //fp map_model
    /// Map a model coordinate to an absolute XY camera coordinate
    #[inline]
    pub fn map_model(&self, model: Point3D) -> Point2D {
        self.world_xyz_to_px_abs_xy(model)
    }

    //fp get_pm_dxdy
    #[inline]
    pub fn get_pm_dxdy(&self, pm: &PointMapping) -> Point2D {
        let camera_scr_xy = self.world_xyz_to_px_abs_xy(pm.model());
        let dx = pm.screen[0] - camera_scr_xy[0];
        let dy = pm.screen[1] - camera_scr_xy[1];
        [dx, dy].into()
    }

    //fp get_pm_sq_error
    #[inline]
    pub fn get_pm_sq_error(&self, pm: &PointMapping) -> f64 {
        let esq = self.get_pm_dxdy(pm).length_sq();
        esq * esq / (esq + pm.error() * pm.error())
    }

    //fp get_pm_model_error
    pub fn get_pm_model_error(&self, pm: &PointMapping) -> (f64, Point3D, f64, Point3D) {
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

    //mp get_pm_direction
    pub fn get_pm_direction(&self, pm: &PointMapping) -> Point3D {
        // Can calculate 4 vectors for pm.screen() +- pm.error()
        //
        // Calculate dots with the actual vector - cos of angles
        //
        // tan^2 angle = sec^2 - 1
        let screen_xy = pm.screen();
        let camera_pm_txty = self.px_abs_xy_to_camera_txty(screen_xy);
        let world_pm_direction_vec = -self.camera_txty_to_world_dir(&camera_pm_txty);
        world_pm_direction_vec
    }

    //mp get_pm_as_ray
    pub fn get_pm_as_ray(&self, pm: &PointMapping, from_camera: bool) -> Ray {
        // Can calculate 4 vectors for pm.screen() +- pm.error()
        //
        // Calculate dots with the actual vector - cos of angles
        //
        // tan^2 angle = sec^2 - 1
        let screen_xy = pm.screen();
        let camera_pm_txty = self.px_abs_xy_to_camera_txty(screen_xy);
        let world_pm_direction_vec = -self.camera_txty_to_world_dir(&camera_pm_txty);

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
                .set_start(self.camera.location())
                .set_direction(world_pm_direction_vec)
                .set_tan_error(tan_error)
        } else {
            Ray::default()
                .set_start(pm.model())
                .set_direction(-world_pm_direction_vec)
                .set_tan_error(tan_error)
        }
    }

    //fp show_point_set
    pub fn show_point_set(&self, nps: &NamedPointSet) {
        for (name, model) in nps.iter() {
            let camera_scr_xy = self.world_xyz_to_px_abs_xy(model.model());
            eprintln!(
                "model {} : {} maps to {}",
                name,
                model.model(),
                camera_scr_xy,
            );
        }
    }

    //fp show_pm_error
    pub fn show_pm_error(&self, pm: &PointMapping) {
        let camera_scr_xy = self.world_xyz_to_px_abs_xy(pm.model());
        let (model_error, model_dxdy, model_angle, model_axis) = self.get_pm_model_error(pm);
        let dxdy = self.get_pm_dxdy(pm);
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
    pub fn show_mappings(&self, mappings: &[PointMapping]) {
        for pm in mappings {
            self.show_pm_error(pm);
        }
    }

    //fp get_rays
    pub fn get_rays(&self, mappings: &[PointMapping], from_camera: bool) -> Vec<(String, Ray)> {
        let mut r = Vec::new();
        for pm in mappings {
            r.push((pm.name().into(), self.get_pm_as_ray(pm, from_camera)));
        }
        r
    }

    //fp apply_quat_to_get_min_sq_error
    pub fn apply_quat_to_get_min_sq_error(
        &self,
        max_steps: usize,
        pm: &PointMapping,
        q: &Quat,
    ) -> (Self, f64) {
        let mut c = self.clone();
        let mut tc = c.clone();
        let mut e = c.get_pm_sq_error(pm);
        for _ in 0..max_steps {
            tc = tc.rotated_by(q);
            let ne = tc.get_pm_sq_error(pm);
            if ne > e {
                break;
            }
            c.camera = c.camera.with_direction(tc.camera.direction());
            e = ne;
        }
        c.normalize(); // Tidy the direction quaternion
        (c, e)
    }

    //fp find_best_angle
    /// Find the best angle given an axis of rotation for the point mappings
    pub fn find_best_angle(
        &self,
        q_base: &Quat,
        axis: Point3D,
        mappings: &[PointMapping],
    ) -> (f64, f64, Quat) {
        let mut scr_model_vecs = Vec::new();
        for pm in mappings {
            let pm_scr_vec = self.px_abs_xy_to_camera_txty(pm.screen()).to_unit_vector();
            let pm_model_vec = (self.camera.location() - pm.model()).normalize();
            scr_model_vecs.push((pm_scr_vec, pm_model_vec));
        }
        let n = 30;
        let mut best = (1.0E8, 0.);
        let mut base_angle = 0.;
        let mut angle_range = 180.0_f64.to_radians();
        for _ in 0..7 {
            best = (1.0E8, 0.);
            for i in 0..2 * n + 1 {
                let angle = base_angle + angle_range * ((i as f64) - (n as f64)) / (n as f64);
                let q = Quat::of_axis_angle(&axis, angle);
                let q = q * *q_base;
                // let mut tot_e_sq = 0.;
                let mut worst_e_sq = 0.0;
                for (s, m) in scr_model_vecs.iter() {
                    let m = q.apply3(m);
                    let e_sq = m.distance_sq(s);
                    // tot_e_sq += e_sq;
                    if e_sq > worst_e_sq {
                        worst_e_sq = e_sq;
                    }
                }
                if worst_e_sq < best.0 {
                    best = (worst_e_sq, angle);
                }
            }
            angle_range = angle_range * 2.0 / (n as f64);
            base_angle = best.1;
        }
        let (e_sq, angle) = best;
        let q = Quat::of_axis_angle(&axis, angle);
        let q = q * *q_base;
        (e_sq, angle, q)
    }

    //fp get_quats_for_mappings_given_one
    pub fn get_quats_for_mappings_given_one(
        &self,
        mappings: &[PointMapping],
        n: usize,
    ) -> Vec<Quat> {
        let pivot_scr_vec = self
            .px_abs_xy_to_camera_txty(mappings[n].screen())
            .to_unit_vector();
        let pivot_model_vec = (self.location() - mappings[n].model()).normalize();
        let q_s2z = Quat::rotation_of_vec_to_vec(&pivot_scr_vec, &[0., 0., 1.].into());
        let q_m2s = Quat::rotation_of_vec_to_vec(&pivot_model_vec, &pivot_scr_vec);
        let q_m2z = q_s2z * q_m2s;
        let mut result = Vec::new();
        for (i, pm) in mappings.iter().enumerate() {
            if i == n {
                continue;
            }
            let pm_scr_vec = self.px_abs_xy_to_camera_txty(pm.screen()).to_unit_vector();
            let pm_model_vec = (self.location() - pm.model()).normalize();
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

    //fp get_location_given_direction
    pub fn get_location_given_direction(&self, mappings: &[PointMapping]) -> Point3D {
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
    pub fn get_best_location(&self, mappings: &[PointMapping], steps: usize) -> BestMapping<Self> {
        let mut best_mapping = BestMapping::new(false, self.clone()); // use total error
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
            let mut cam = self.rotated_by(&qxy);
            // Find best rotation around Z axis for this basic orientation
            let mut angle_range = 6.282;
            let mut best_of_axis: BestMapping<Self> = BestMapping::new(false, self.clone()); // use total error
            for _ in 0..6 {
                for z in 0..(steps * 2 + 1) {
                    let zf = (z as f64) / (steps as f64) - 1.0;
                    let qz = Quat::of_axis_angle(&[0., 0., 1.].into(), zf * angle_range);
                    let tc = cam.rotated_by(&qz);

                    let location = tc.get_location_given_direction(mappings);
                    let tc = tc.placed_at(location);
                    let te = tc.total_error(mappings);
                    let we = tc.worst_error(mappings);
                    best_of_axis.update_best(we, te, &tc);
                }
                cam = best_of_axis.data().clone();
                angle_range /= steps as f64;
                if angle_range < 1.0E-4 {
                    break;
                }
            }
            best_mapping = best_mapping.best_of_both(best_of_axis);
        }
        eprintln!("=> {}", best_mapping);
        best_mapping
    }

    //fp get_best_direction
    pub fn get_best_direction(
        &self,
        steps_per_rot: usize,
        rotations: &Rotations,
        pm: &PointMapping,
    ) -> (Self, f64) {
        let mut c = self.clone();
        let mut e = 0.;
        for q in rotations.quats.iter() {
            (c, e) = c.apply_quat_to_get_min_sq_error(steps_per_rot, pm, q);
        }
        (c, e)
    }

    //fp error_with_quat
    #[inline]
    fn error_with_quat(&self, pm: &PointMapping, quat: &Quat) -> f64 {
        self.rotated_by(quat).get_pm_sq_error(pm)
    }

    //fp error_surface_normal
    fn error_surface_normal(&self, pm: &PointMapping, rotations: &Rotations) -> Point3D {
        // At the current point xyz there is *probably* a surface such that any adjustment
        // dxyz within the plane as no immpact. This is grad.es. We can call this vector n.
        let quats = &rotations.quats;
        let dx_n = self.error_with_quat(pm, &quats[0]);
        let dx_p = self.error_with_quat(pm, &quats[1]);
        let dy_n = self.error_with_quat(pm, &quats[2]);
        let dy_p = self.error_with_quat(pm, &quats[3]);
        let dz_n = self.error_with_quat(pm, &quats[4]);
        let dz_p = self.error_with_quat(pm, &quats[5]);
        let n: Point3D = [dx_p - dx_n, dy_p - dy_n, dz_p - dz_n].into();
        n.normalize()
    }

    //fp adjust_direction_while_keeping_one_okay
    #[allow(clippy::too_many_arguments)]
    pub fn adjust_direction_while_keeping_one_okay<F: Fn(&Self, &[PointMapping], usize) -> f64>(
        &self,
        max_adj: usize,
        da: f64,
        rotations: &Rotations,
        f: &F,
        mappings: &[PointMapping],
        keep_pm: usize,
        test_pm: usize,
    ) -> (Self, f64) {
        // We first find the surface normal for the error field for the keep_pm; moving at all
        // in that direction changes esq for that point. So only move perpendicular to it.
        //
        // We can thus move in any direction along the surface that reduces the error in the
        // test_pm. We can find the surface for test_pm, which has normal n_pm.
        //
        // We want to move in direction - n x (n_pm x n)
        let mut c = self.clone();
        let mut e = f(&c, mappings, test_pm);
        // dbg!("Preadjusted", e);
        for _i in 0..max_adj {
            let keep_pm_n = c.error_surface_normal(&mappings[keep_pm], rotations);
            let test_pm_n = c.error_surface_normal(&mappings[test_pm], rotations);
            let k_x_t = keep_pm_n.cross_product(&test_pm_n);
            let k_x_k_x_t = keep_pm_n.cross_product(&k_x_t);
            let k_x_k_x_t = k_x_k_x_t.normalize();
            // This is always in the correct 'direction' to reduce error for test_pm if possible
            //
            // This is not necessarily reducing the f() function value
            let q = Quat::unit().rotate_x(da * k_x_k_x_t[0]);
            let q = q.rotate_y(da * k_x_k_x_t[1]);
            let q = q.rotate_z(da * k_x_k_x_t[2]);
            let tc = c.rotated_by(&q);
            let ne = f(&tc, mappings, test_pm);
            if ne > e {
                // dbg!("Adjusted", i, e, ne, k_x_k_x_t);
                c.normalize();
                return (c, e);
            }
            if e < MIN_ERROR {
                // dbg!("Adjusted to MIN ERROR", i, e);
                return (c, e);
            }
            c = tc;
            e = ne;
        }
        dbg!("Adjusted BUT TOO MUCH!", e);
        (c, e)
    }

    //fp adjust_direction_rotating_around_one_point
    pub fn adjust_direction_rotating_around_one_point<
        F: Fn(&Self, &[PointMapping], usize) -> f64,
    >(
        &self,
        f: &F,
        da: f64,
        mappings: &[PointMapping],
        keep_pm: usize,
        test_pm: usize,
    ) -> (Self, f64) {
        // We first the axis to rotate around - the direction in the view of keep_pm
        //
        // We want a da rotation around the axis (direction * model[keep_pm])
        //
        // Then da * (direction * model[keep_pm)]) does not impact the view position
        // of model[keep_pm]
        let keep_v = self.world_xyz_to_camera_xyz(mappings[keep_pm].model());
        let mut rot = Quat::of_axis_angle(&keep_v, da);
        let mut c = self.clone();
        let mut e = f(&c, mappings, test_pm);
        for _sc in 0..2 {
            // dbg!("Preadjusted", e);
            for _i in 0..100_000 {
                let tc = c.rotated_by(&rot);
                // was tc.direction = rot * c.direction;
                let ne = f(&tc, mappings, test_pm);
                if ne > e {
                    // dbg!("Adjusted", i, e, ne);
                    c.normalize();
                    break;
                }
                if e < MIN_ERROR {
                    // dbg!("Adjusted to MIN ERROR", i, e);
                    return (c, e);
                }
                c = tc;
                e = ne;
            }
            rot = rot.conjugate();
        }
        (c, e)
    }

    //fp find_worst_error
    pub fn find_worst_error(&self, mappings: &[PointMapping]) -> (usize, f64) {
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
    pub fn total_error(&self, mappings: &[PointMapping]) -> f64 {
        let mut sum_e = 0.;
        for pm in mappings.iter() {
            let e = self.get_pm_sq_error(pm);
            sum_e += e;
        }
        sum_e
    }

    //fp worst_error
    pub fn worst_error(&self, mappings: &[PointMapping]) -> f64 {
        self.find_worst_error(mappings).1
    }

    //fp adjust_position
    pub fn adjust_position<F: Fn(&Self, &[PointMapping]) -> f64>(
        &self,
        mappings: &[PointMapping],
        f: &F,
    ) -> (Self, f64) {
        let mut cam = self.clone();
        let mut e = f(&cam, mappings);
        for _ in 0..10_000 {
            let cx = cam.moved_by([1., 0., 0.].into());
            let de_cx = f(&cx, mappings) - e;
            let cy = cam.moved_by([0., 1., 0.].into());
            let de_cy = f(&cy, mappings) - e;
            let cz = cam.moved_by([0., 0., 1.].into());
            let de_cz = f(&cz, mappings) - e;
            let sqe = de_cx * de_cx + de_cy * de_cy + de_cz * de_cz;
            let rsqe = sqe.sqrt();
            let dx = de_cx / rsqe * 0.25;
            let dy = de_cy / rsqe * 0.25;
            let dz = de_cz / rsqe * 0.25;
            let cn = cam.moved_by([-dx, -dy, -dz].into());
            let en = f(&cn, mappings);
            if en > e {
                return (cam, e);
            }
            e = en;
            cam = cn.clone()
        }
        (cam, e)
    }

    //fp adjust_position_in_out
    pub fn adjust_position_in_out<F: Fn(&Self, &[PointMapping]) -> f64>(
        &self,
        mappings: &[PointMapping],
        f: &F,
    ) -> (Self, f64) {
        // Map (0,0,1) to view space
        let dxyz = self.direction().conjugate().apply3(&[0., 0., 1.].into());
        let mut cam = self.clone();
        let mut e = f(&cam, mappings);
        for sc in [1., -1., 0.01, -0.01] {
            for _ in 0..10_000 {
                let cn = cam.moved_by(dxyz * sc);
                let en = f(&cn, mappings);
                if en > e {
                    break;
                }
                e = en;
                cam = cn.clone();
            }
        }
        (cam, e)
    }

    //mp find_coarse_position
    pub fn find_coarse_position<F: Fn(&Self, &[PointMapping], usize) -> f64>(
        &self,
        mappings: &[PointMapping],
        f: &F,
        scales: &[f64; 3],
        n: usize,
    ) -> Self {
        // dbg!("Find coarse position", self, scales, n);
        let coarse_rotations = Rotations::new(1.0_f64.to_radians());
        let fine_rotations = Rotations::new(0.1_f64.to_radians());
        let we = self.worst_error(mappings);
        let mut worst_data = (we * 1.01, 0, self.clone(), 0.);
        let num = mappings.len();
        let map = |i, sc| ((i as f64) - ((n - 1) as f64) / 2.0) * sc / ((n - 1) as f64);
        for i in 0..(n * n * n) {
            let x = map(i % n, scales[0]);
            let y = map((i / n) % n, scales[1]);
            let z = map((i / n / n) % n, scales[2]);
            let mut cam = self.moved_by([x, y, z].into());
            for _ in 0..5 {
                cam = cam
                    .get_best_direction(400, &coarse_rotations, &mappings[0])
                    .0;
            }
            for _ in 0..50 {
                cam = cam.get_best_direction(400, &fine_rotations, &mappings[0]).0;
            }
            for i in 0..num {
                cam = cam
                    .adjust_direction_rotating_around_one_point(
                        f,
                        0.2_f64.to_radians(),
                        mappings,
                        i,
                        0,
                    )
                    .0;
            }
            let te = cam.total_error(mappings);
            let we = cam.worst_error(mappings);
            // eprintln!("{},{},{}: {} : {}", x, y, z, we, te);
            if we < worst_data.0 {
                eprintln!("{}: {} : {}", cam, we, te);
                worst_data = (we, i, cam, te);
                // dbg!(worst_data);
            }
        }
        // dbg!(&worst_data);
        worst_data.2
    }

    //zz All done
}
