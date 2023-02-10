//a Imports
use std::rc::Rc;

use geo_nd::{quat, vector};
use serde::Serialize;

use crate::{
    CameraInstance, CameraPolynomial, CameraView, NamedPointSet, Point2D, Point3D, PointMapping,
    Quat, Ray, Rotations,
};

//a Constants
const MIN_ERROR: f64 = 0.5;

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
    pub fn moved_by(&self, dp: [f64; 3]) -> Self {
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
        let esq = vector::length_sq(self.get_pm_dxdy(pm).as_ref());
        esq * esq / (esq + pm.error() * pm.error())
    }

    //fp get_pm_model_error
    pub fn get_pm_model_error(&self, pm: &PointMapping) -> (f64, Point3D, f64, Point3D) {
        let model_rel_xyz = self.world_xyz_to_camera_xyz(pm.model());
        let model_dist = vector::length(model_rel_xyz.as_ref());
        let model_vec = self.world_xyz_to_camera_txty(pm.model()).to_unit_vector();
        let screen_vec = self.px_abs_xy_to_camera_txty(pm.screen()).to_unit_vector();
        let dxdy = self.camera_xyz_to_world_xyz((-screen_vec) * model_dist) - pm.model();
        let axis = vector::cross_product3(model_vec.as_ref(), screen_vec.as_ref());
        let sin_sep = vector::length(&axis);
        let error = sin_sep * model_dist;
        let angle = sin_sep.asin().to_degrees();
        let axis = vector::normalize(axis);
        if error < 0. {
            (-error, dxdy, -angle, vector::scale(axis, -1.).into())
        } else {
            (error, dxdy, angle, axis.into())
        }
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
            let dot = vector::dot(world_pm_direction_vec.as_ref(), world_err_vec.as_ref());
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
            let pm_model_vec: Point3D =
                vector::normalize(*(self.camera.location() - pm.model()).as_ref()).into();
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
                let q: Quat = quat::of_axis_angle(axis.as_ref(), angle).into();
                let q = q * *q_base;
                // let mut tot_e_sq = 0.;
                let mut worst_e_sq = 0.0;
                for (s, m) in scr_model_vecs.iter() {
                    let m = quat::apply3(q.as_ref(), &(*m).into());
                    let e_sq = vector::distance_sq(&m, &(*s).into());
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
        let q: Quat = quat::of_axis_angle(axis.as_ref(), angle).into();
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
        let pivot_model_vec: Point3D =
            vector::normalize(*(self.location() - mappings[n].model()).as_ref()).into();
        let q_s2z: Quat =
            quat::get_rotation_of_vec_to_vec(pivot_scr_vec.as_ref(), &[0., 0., 1.]).into();
        let q_m2s: Quat =
            quat::get_rotation_of_vec_to_vec(pivot_model_vec.as_ref(), pivot_scr_vec.as_ref())
                .into();
        let q_m2z = q_s2z * q_m2s;
        let mut result = Vec::new();
        for (i, pm) in mappings.iter().enumerate() {
            if i == n {
                continue;
            }
            let pm_scr_vec = self.px_abs_xy_to_camera_txty(pm.screen()).to_unit_vector();
            let pm_model_vec: Point3D =
                vector::normalize(*(self.location() - pm.model()).as_ref()).into();
            let m_mapped = quat::apply3(q_m2z.as_ref(), &pm_model_vec.into());
            let scr_mapped = quat::apply3(q_s2z.as_ref(), &pm_scr_vec.into());
            let m_mapped = [m_mapped[0] / m_mapped[2], m_mapped[1] / m_mapped[2]];
            let scr_mapped = [scr_mapped[0] / scr_mapped[2], scr_mapped[1] / scr_mapped[2]];
            let m_angle = m_mapped[1].atan2(m_mapped[0]);
            let scr_angle = scr_mapped[1].atan2(scr_mapped[0]);
            let qp5: Quat =
                quat::of_axis_angle(pivot_scr_vec.as_ref(), -m_angle + scr_angle).into();
            let q: Quat = qp5 * q_m2s;
            result.push(q.into());
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
    /// the model and getting the point of best intersection for rays
    /// for the point mappings
    pub fn get_best_location(&self, mappings: &[PointMapping], steps: usize) -> Self {
        let mut best_dirn = (1.0E20, 1.0E20, self.clone());
        for i in 0..steps * steps * steps {
            let x = i % steps;
            let y = (i / steps) % steps;
            let z = (i / steps) / steps;
            let qx: Quat =
                quat::of_axis_angle(&[1., 0., 0.], (x as f64) * 6.282 / (steps as f64)).into();
            let qy: Quat =
                quat::of_axis_angle(&[0., 1., 0.], (y as f64) * 6.282 / (steps as f64)).into();
            let qz: Quat =
                quat::of_axis_angle(&[0., 0., 1.], (z as f64) * 6.282 / (steps as f64)).into();
            let q = (qx * qy) * qz;
            let tc = self.rotated_by(&q);
            let named_rays = tc.get_rays(mappings, false);
            let mut ray_list = Vec::new();
            for (_name, ray) in named_rays {
                ray_list.push(ray);
            }
            let location = Ray::closest_point(&ray_list, &|r| 1.0 / r.tan_error()).unwrap();
            let tc = tc.placed_at(location);
            let te = tc.total_error(mappings);
            let we = tc.worst_error(mappings);
            if te < best_dirn.1 {
                best_dirn = (we, te, tc);
            }
        }
        best_dirn.2
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
    fn error_surface_normal(&self, pm: &PointMapping, rotations: &Rotations) -> [f64; 3] {
        // At the current point xyz there is *probably* a surface such that any adjustment
        // dxyz within the plane as no immpact. This is grad.es. We can call this vector n.
        let quats = &rotations.quats;
        let dx_n = self.error_with_quat(pm, &quats[0]);
        let dx_p = self.error_with_quat(pm, &quats[1]);
        let dy_n = self.error_with_quat(pm, &quats[2]);
        let dy_p = self.error_with_quat(pm, &quats[3]);
        let dz_n = self.error_with_quat(pm, &quats[4]);
        let dz_p = self.error_with_quat(pm, &quats[5]);
        vector::normalize([dx_p - dx_n, dy_p - dy_n, dz_p - dz_n])
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
            let k_x_t = vector::cross_product3(&keep_pm_n, &test_pm_n);
            let k_x_k_x_t = vector::cross_product3(&keep_pm_n, &k_x_t);
            let k_x_k_x_t = vector::normalize(k_x_k_x_t);
            // This is always in the correct 'direction' to reduce error for test_pm if possible
            //
            // This is not necessarily reducing the f() function value
            let q = quat::rotate_x(&quat::new(), da * k_x_k_x_t[0]);
            let q = quat::rotate_y(&q, da * k_x_k_x_t[1]);
            let q = quat::rotate_z(&q, da * k_x_k_x_t[2]);
            let tc = c.rotated_by(&q.into());
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
        let mut rot: Quat = quat::of_axis_angle(keep_v.as_ref(), da).into();
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
            rot = quat::conjugate(rot.as_ref()).into();
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
            let cx = cam.moved_by([1., 0., 0.]);
            let de_cx = f(&cx, mappings) - e;
            let cy = cam.moved_by([0., 1., 0.]);
            let de_cy = f(&cy, mappings) - e;
            let cz = cam.moved_by([0., 0., 1.]);
            let de_cz = f(&cz, mappings) - e;
            let sqe = de_cx * de_cx + de_cy * de_cy + de_cz * de_cz;
            let rsqe = sqe.sqrt();
            let dx = de_cx / rsqe * 0.25;
            let dy = de_cy / rsqe * 0.25;
            let dz = de_cz / rsqe * 0.25;
            let cn = cam.moved_by([-dx, -dy, -dz]);
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
        let [dx, dy, dz] = quat::apply3(&quat::conjugate(self.direction().as_ref()), &[0., 0., 1.]);
        // dbg!(dx, dy, dz);
        let mut cam = self.clone();
        let mut e = f(&cam, mappings);
        for sc in [1., -1., 0.01, -0.01] {
            for _ in 0..10_000 {
                let cn = cam.moved_by([dx * sc, dy * sc, dz * sc]);
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
            let mut cam = self.moved_by([x, y, z]);
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
