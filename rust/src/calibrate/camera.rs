use super::{Point2D, Point3D, PointMapping, Projection, Quat};

use geo_nd::{quat, vector};

const MIN_ERROR: f64 = 4.0;

pub struct Rotations {
    pub quats: [Quat; 6],
}

impl Rotations {
    pub fn new(da: f64) -> Self {
        let q = quat::new();
        let rot_dx_n = quat::rotate_x(&q, -da).into();
        let rot_dx_p = quat::rotate_x(&q, da).into();
        let rot_dy_n = quat::rotate_y(&q, -da).into();
        let rot_dy_p = quat::rotate_y(&q, da).into();
        let rot_dz_n = quat::rotate_z(&q, -da).into();
        let rot_dz_p = quat::rotate_z(&q, da).into();
        let quats = [rot_dx_n, rot_dx_p, rot_dy_n, rot_dy_p, rot_dz_n, rot_dz_p];
        Self { quats }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LCamera {
    position: Point3D,
    direction: Quat,
}

impl Projection for LCamera {
    fn centre_xy(&self) -> Point2D {
        [320., 240.].into()
    }
    fn screen_size(&self) -> Point2D {
        [640., 480.].into()
    }
    fn aspect_ratio(&self) -> f64 {
        640. / 480.
    }
    // tan_fov_x is the x to z ratio that makes a right-most pixel map to the camera space for the edge of the camera view
    // So it is tan of *half* the full camera FOV width
    fn tan_fov_x(&self) -> f64 {
        // Got best values with 60.6, maps to about -120, -180, 660
        // 60.6_f64.to_radians().tan()
        35.2_f64.to_radians().tan()
    }
}

impl LCamera {
    pub fn new(position: Point3D, direction: Quat) -> Self {
        Self {
            position,
            direction,
        }
    }
    pub fn to_camera_space(&self, model_xyz: &Point3D) -> Point3D {
        let camera_relative_xyz = *model_xyz - self.position;
        quat::apply3(self.direction.as_ref(), camera_relative_xyz.as_ref()).into()
    }
    pub fn to_sph_xy(&self, model_xyz: &Point3D) -> Point2D {
        let camera_xyz = self.to_camera_space(model_xyz);
        let camera_as_sph_x = camera_xyz[0] / camera_xyz[2];
        let camera_as_sph_y = camera_xyz[1] / camera_xyz[2];
        [camera_as_sph_x, camera_as_sph_y].into()
    }
    pub fn to_scr_xy(&self, model_xyz: &Point3D) -> Point2D {
        // If x is about 300, and z about 540, and a FOV of 60 degress across
        // then this should map to the right-hand edge (i.e. about 640)
        // hence 320 + 640/2 * 300/540 / tan(fov/2)
        //
        // If the FOV is smaller (telephoto) then tan(fov) is smaller, and scr_x should
        // be largerfor the same model x
        let wh = self.screen_size();
        let centre = self.centre_xy();
        let camera_xyz = self.to_camera_space(model_xyz);
        let camera_as_sph_x = camera_xyz[0] / camera_xyz[2];
        let camera_as_sph_y = camera_xyz[1] / camera_xyz[2];
        [
            centre[0] + camera_as_sph_x * wh[0] / 2.0 / self.tan_fov_x(),
            centre[1] - camera_as_sph_y * wh[1] / 2.0 / self.tan_fov_y(),
        ]
        .into()
    }
    pub fn position(&self) -> &[f64; 3] {
        self.position.as_ref()
    }
    pub fn rotation_matrix(&self) -> [f64; 9] {
        let mut rot = [0.; 9];
        quat::to_rotation3(self.direction.as_ref(), &mut rot);
        rot
    }
    pub fn moved_by(&self, dp: [f64; 3]) -> Self {
        let position = self.position + Point3D::from(dp);
        Self {
            position,
            direction: self.direction,
        }
    }
    pub fn apply_quat_to_get_min_sq_error(&self, pm: &PointMapping, q: &Quat) -> (Self, f64) {
        let mut c = self.clone();
        let mut tc = c.clone();
        let mut e = pm.get_sq_error(&c);
        for _ in 0..10000 {
            tc.direction = c.direction * *q;
            let ne = pm.get_sq_error(&tc);
            if ne > e {
                return (c, e);
            }
            c = tc;
            e = ne;
        }
        panic!("Should not get here as the loop should cover all rotations");
    }
    pub fn get_best_direction(&self, rotations: &Rotations, pm: &PointMapping) -> (Self, f64) {
        let mut c = self.clone();
        let mut e = 0.;
        for q in rotations.quats.iter() {
            (c, e) = c.apply_quat_to_get_min_sq_error(pm, &q);
        }
        (c, e)
    }
    #[inline]
    fn error_with_quat(&self, pm: &PointMapping, quat: &Quat) -> f64 {
        let mut c = self.clone();
        c.direction = self.direction * *quat;
        pm.get_sq_error(&c)
    }
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
    pub fn adjust_direction_while_keeping_one_okay(
        &self,
        rotations: &Rotations,
        keep_pm: &PointMapping,
        test_pm: &PointMapping,
    ) -> (Self, f64) {
        // We first find the surface normal for the error field for the keep_pm; moving at all
        // in that direction changes esq for that point. So only move perpendicular to it.
        //
        // We can thus move in any direction along the surface that reduces the error in the
        // test_pm. We can find the surface for test_pm, which has normal n_pm.
        //
        // We want to move in direction - n x (n_pm x n)
        let mut c = self.clone();
        let mut tc = c.clone();
        let mut e = test_pm.get_sq_error(&c);
        let da = 0.02_f64.to_radians();
        for i in 0..100_000 {
            let keep_pm_n = c.error_surface_normal(keep_pm, rotations);
            let test_pm_n = c.error_surface_normal(test_pm, rotations);
            let k_x_t = vector::cross_product3(&keep_pm_n, &test_pm_n);
            let k_x_k_x_t = vector::cross_product3(&keep_pm_n, &k_x_t);
            let k_x_k_x_t = vector::normalize(k_x_k_x_t);
            // This is always in the correct 'direction' to reduce error if possible
            let q = quat::rotate_x(&quat::new(), da * k_x_k_x_t[0]);
            let q = quat::rotate_y(&q, da * k_x_k_x_t[1]);
            let q = quat::rotate_z(&q, da * k_x_k_x_t[2]);
            tc.direction = c.direction * Quat::from(q);
            let ne = test_pm.get_sq_error(&tc);
            if ne > e {
                dbg!("Adjusted", i, e);
                *c.direction.as_mut() = quat::normalize(*c.direction.as_ref());
                return (c, e);
            }
            if e < MIN_ERROR {
                dbg!("Adjusted to MIN ERROR", i, e);
                return (c, e);
            }
            c = tc;
            e = ne;
        }
        dbg!("Adjusted BUT TOO MUCH!", e);
        return (c, e);
    }
    pub fn find_worst_error(&self, mappings: &[PointMapping]) -> usize {
        let mut n = 0;
        let mut worst_e = 0.;
        for (i, pm) in mappings.iter().enumerate() {
            let e = pm.get_sq_error(self);
            if e > worst_e {
                n = i;
                worst_e = e;
            }
        }
        n
    }
    pub fn total_error(&self, mappings: &[PointMapping]) -> f64 {
        let mut sum_e = 0.;
        for pm in mappings.iter() {
            let e = pm.get_sq_error(self);
            sum_e += e;
        }
        sum_e
    }
    pub fn adjust_position(&self, mappings: &[PointMapping]) -> (Self, f64) {
        let mut cam = *self;
        let mut e = cam.total_error(mappings);
        for _ in 0..10_000 {
            let cx = cam.moved_by([1., 0., 0.]);
            let de_cx = cx.total_error(mappings) - e;
            let cy = cam.moved_by([0., 1., 0.]);
            let de_cy = cy.total_error(mappings) - e;
            let cz = cam.moved_by([0., 0., 1.]);
            let de_cz = cz.total_error(mappings) - e;
            let sqe = de_cx * de_cx + de_cy * de_cy + de_cz * de_cz;
            let rsqe = sqe.sqrt();
            let dx = de_cx / rsqe * 0.25;
            let dy = de_cy / rsqe * 0.25;
            let dz = de_cz / rsqe * 0.25;
            let cn = cam.moved_by([-dx, -dy, -dz]);
            let en = cn.total_error(mappings);
            if en > e {
                return (cam, e);
            }
            e = en;
            cam = cn
        }
        (cam, e)
    }
}
