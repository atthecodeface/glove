//a Imports
use std::rc::Rc;

use super::{
    Camera, CameraProjection, CameraView, NamedPointSet, Point2D, Point3D, PointMapping, Quat,
    Rotations, TanXTanY,
};

use geo_nd::{quat, vector};

//a Constants
const MIN_ERROR: f64 = 0.5;

//a CameraMapping
//tp CameraMapping
/// A camera that allows mapping a world point to camera relative XYZ,
/// and then it can be mapped to tan(x) / tan(y) to roll/yaw or pixel
/// relative XY (relative to the center of the camera sensor)
#[derive(Debug, Clone)]
pub struct CameraMapping {
    camera: Camera,
}

//ip Display for CameraMapping
impl std::fmt::Display for CameraMapping {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.camera.fmt(fmt)
    }
}

//ip Deref for CameraMapping
impl std::ops::Deref for CameraMapping {
    type Target = Camera;
    fn deref(&self) -> &Camera {
        &self.camera
    }
}

//ip DerefMut for CameraMapping
impl std::ops::DerefMut for CameraMapping {
    fn deref_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }
}

//ip CameraMapping
impl CameraMapping {
    //fp new
    pub fn new(projection: Rc<dyn CameraProjection>, position: Point3D, direction: Quat) -> Self {
        let camera = Camera::new(projection, position, direction);
        Self { camera }
    }

    //fp moved_by
    pub fn moved_by(&self, dp: [f64; 3]) -> Self {
        Self {
            camera: self.camera.clone().moved_by(dp),
        }
    }

    //fp rotated_by
    pub fn rotated_by(&self, q: &Quat) -> Self {
        Self {
            camera: self.camera.clone().rotated_by(q),
        }
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
        vector::length_sq(self.get_pm_dxdy(pm).as_ref())
    }

    //fp get_pm_model_error
    pub fn get_pm_model_error(&self, pm: &PointMapping) -> (f64, Point3D, f64, Point3D) {
        let model_rel_xyz = self.world_xyz_to_camera_xyz(pm.model());
        let model_dist = vector::length(model_rel_xyz.as_ref());
        let model_vec = self.world_xyz_to_camera_txty(pm.model()).to_unit_vector();
        let screen_vec = self.px_abs_xy_to_camera_txty(pm.screen()).to_unit_vector();
        let dxdy = self.camera_xyz_to_world_xyz(&((-screen_vec) * model_dist)) - *pm.model();
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
            "{} {} <> {:.2}: Maps to {:.2}, dxdy {:.2} esq {:.2} : model rot {:.2} by {:.2} dxdydz {:.2} dist {:.3}  ",
            pm.name(),
            pm.model(),
            pm.screen,
            camera_scr_xy,
            dxdy,
            esq,
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
                let tc = c.rotated_by(&rot.into());
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

    //fp find_best_error
    pub fn find_best_error(&self, mappings: &[PointMapping]) -> (usize, f64) {
        let mut n = 0;
        let mut best_e = 1_000_000_000.0;
        for (i, pm) in mappings.iter().enumerate() {
            let e = self.get_pm_sq_error(pm);
            if e < best_e {
                n = i;
                best_e = e;
            }
        }
        (n, best_e)
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
    pub fn find_coarse_position(
        &self,
        mappings: &[PointMapping],
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
            for _ in 0..5 {
                cam = cam.get_best_direction(400, &fine_rotations, &mappings[0]).0;
            }
            for i in 0..num {
                cam = cam
                    .adjust_direction_rotating_around_one_point(
                        // &|c, m, n| m[n].get_sq_error(c),
                        // &|c, m, n| c.total_error(&mappings),
                        &|c, _m, _n| c.worst_error(mappings),
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
        dbg!(&worst_data);
        worst_data.2
    }

    //zz All done
}
