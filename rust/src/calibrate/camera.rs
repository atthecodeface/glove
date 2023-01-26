//a Imports
use super::{Point2D, Point3D, PointMapping, Projection, Quat, Rotations};

use geo_nd::{quat, vector};

//a Constants
const MIN_ERROR: f64 = 0.5;

//a LCamera
//tp LCamera
#[derive(Debug, Clone, Copy)]
pub struct LCamera {
    position: Point3D,
    direction: Quat,
}

//ip Projection for LCamera
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
    //
    // The spec diagonal FOV is probably 55.0 degrees
    //
    // This yields an X FOV of 2.0 * atan(4/5 * tan(55.0/2.0)) = 45.2 degrees
    //
    // Hence 22.6 would seem to be the correct number
    fn tan_fov_x(&self) -> f64 {
        // With MIN_ERROR = 2.0
        // 20.9 Lowest WE 85 27.96 Camera @[-191.72,-247.43,472.45] yaw -18.19 pitch -19.85 + [-0.29,-0.34,0.89]
        // 21.9 Lowest WE 10 26.74 Camera @[-180.39,-208.51,469.58] yaw -17.10 pitch -16.33 + [-0.28,-0.28,0.92]
        // 22.9 Lowest WE 17 9.51 Camera @[-177.09,-202.00,441.55] yaw -17.65 pitch -16.40 + [-0.29,-0.28,0.91]
        // 23.9 Lowest WE 88 5.36 Camera @[-183.55,-190.09,409.24] yaw -19.55 pitch -16.17 + [-0.32,-0.28,0.91]
        // 24.9 Lowest WE 235 6.36 Camera @[-173.57,-175.53,395.57] yaw -18.95 pitch -14.92 + [-0.31,-0.26,0.91]
        // 25.9 Lowest WE 247 7.25 Camera @[-165.02,-173.48,376.42] yaw -18.66 pitch -15.36 + [-0.31,-0.26,0.91]
        // 26.9 Lowest WE 297 64.51 Camera @[-121.16,-187.45,367.38] yaw -13.56 pitch -17.81 + [-0.22,-0.31,0.93]
        // 27,6 WE 74.49 Camera @[-118.03,-134.71,404.21] yaw -11.56 pitch -9.87 + [-0.20,-0.17,0.97]
        // 28.6 WE 82.28 Camera @[-122.58,-123.63,388.92] yaw -12.39 pitch -8.26 + [-0.21,-0.14,0.97]
        // 29.1 WE 83.41 Camera @[-103.61,-132.34,374.19] yaw -10.41 pitch -10.21 + [-0.18,-0.18,0.97]
        // 29.6 WE 68.79 Camera @[-110.52,-137.75,353.28] yaw -11.92 pitch -11.80 + [-0.20,-0.20,0.96]

        // With MIN_ERROR = 0.5
        // 22.9 Lowest WE 77 4.20 Camera @[-190.81,-194.42,434.13] yaw -19.44 pitch -15.79 + [-0.32,-0.27,0.91]
        // 23.4 Lowest WE 74 6.08 Camera @[-180.90,-186.53,431.35] yaw -18.37 pitch -15.04 + [-0.30,-0.26,0.92]
        // 23.5 Lowest WE 57 11.82 Camera @[-173.70,-202.87,424.04] yaw -17.78 pitch -17.10 + [-0.29,-0.29,0.91]
        // 23.6 Lowest WE 15 12.30 Camera @[-168.33,-193.25,428.91] yaw -17.15 pitch -15.95 + [-0.28,-0.27,0.92]
        // 23.7 Lowest WE 56 5.81 Camera @[-182.86,-183.68,420.27] yaw -19.07 pitch -15.03 + [-0.32,-0.26,0.91]
        // 23.9 Lowest WE 92 4.77 Camera @[-182.12,-185.26,414.33] yaw -19.18 pitch -15.42 + [-0.32,-0.27,0.91]
        // 24.9 Lowest WE 251 16.39 Camera @[-173.77,-186.47,396.40] yaw -18.77 pitch -16.02 + [-0.31,-0.28,0.91]

        // With MIN_ERROR = 0.5, pos in out adj by 0.01
        // 22.5  Lowest WE 54 10.98 38.10 Camera @[-184.62,-210.27,440.32] yaw -18.49 pitch -17.35 + [-0.30,-0.30,0.91]
        // 22.55 Lowest WE 83 3.69 15.41 Camera @[-195.99,-196.84,442.77] yaw -19.65 pitch -15.73 + [-0.32,-0.27,0.91]
        // 22.57 Lowest WE 83 4.57 23.53 Camera @[-199.16,-201.65,435.92] yaw -20.21 pitch -16.46 + [-0.33,-0.28,0.90]
        // 22.58 Lowest WE 122 19.63 87.42 Camera @[-175.07,-223.47,436.55] yaw -17.63 pitch -18.90 + [-0.29,-0.32,0.90]
        // 22.59 Lowest WE 53 8.44 28.08 Camera @[-185.74,-209.59,439.91] yaw -18.68 pitch -17.23 + [-0.31,-0.30,0.90]
        // 22.6  Lowest WE 92 4.02 17.16 Camera @[-195.55,-199.31,439.53] yaw -19.69 pitch -16.11 + [-0.32,-0.28,0.90]
        // 22.61 Lowest WE 77 3.66 16.26 Camera @[-196.20,-200.80,435.82] yaw -19.93 pitch -16.39 + [-0.33,-0.28,0.90]
        // 22.62 Lowest WE 65 3.63 15.87 Camera @[-195.54,-198.09,439.14] yaw -19.75 pitch -15.99 + [-0.32,-0.28,0.90]
        // 22.65 Lowest WE 124 4.91 19.63 Camera @[-195.62,-205.03,432.49] yaw -19.98 pitch -16.94 + [-0.33,-0.29,0.90]
        // 23.3  Lowest WE 152 4.92 23.04 Camera @[-190.61,-193.61,421.99] yaw -19.82 pitch -16.09 + [-0.33,-0.28,0.90]
        // 23.35 Lowest WE 117 5.39 24.82 Camera @[-189.70,-198.25,418.95] yaw -19.87 pitch -16.67 + [-0.33,-0.29,0.90]
        // 23.4  Lowest WE 120 4.35 22.65 Camera @[-190.19,-194.27,417.75] yaw -19.97 pitch -16.29 + [-0.33,-0.28,0.90]
        // 23.45 Lowest WE 100 4.62 19.28 Camera @[-187.44,-188.69,423.31] yaw -19.44 pitch -15.50 + [-0.32,-0.27,0.91]
        // 23.5  Lowest WE 84 3.96 20.11 Camera @[-187.12,-186.96,421.74] yaw -19.47 pitch -15.38 + [-0.32,-0.27,0.91]
        // 23.51 Lowest WE 102 5.50 26.89 Camera @[-186.52,-198.07,413.55] yaw -19.69 pitch -16.90 + [-0.32,-0.29,0.90]
        // 23.52 Lowest WE 81 4.80 19.55 Camera @[-187.01,-186.60,421.22] yaw -19.48 pitch -15.32 + [-0.32,-0.26,0.91]
        // 23.53 Lowest WE 69 4.62 19.32 Camera @[-185.16,-185.59,423.40] yaw -19.20 pitch -15.15 + [-0.32,-0.26,0.91]
        // 23.54 Lowest WE 101 6.20 29.70 Camera @[-185.01,-195.70,414.41] yaw -19.48 pitch -16.67 + [-0.32,-0.29,0.90]
        // 23.55 Lowest WE 145 4.68 23.16 Camera @[-188.06,-188.81,418.34] yaw -19.68 pitch -15.69 + [-0.32,-0.27,0.91]
        // 23.6  Lowest WE 104 4.55 22.78 Camera @[-187.47,-188.88,417.79] yaw -19.63 pitch -15.69 + [-0.32,-0.27,0.91]
        // 23.65 Lowest WE 66 4.25 21.91 Camera @[-185.98,-185.40,418.98] yaw -19.42 pitch -15.28 + [-0.32,-0.26,0.91]
        // 23.7  Lowest WE 87 4.73 22.97 Camera @[-186.76,-185.65,415.75] yaw -19.65 pitch -15.40 + [-0.32,-0.27,0.91]
        // 23.75 Lowest WE 41 11.28 54.03 Camera @[-169.76,-202.04,419.17] yaw -17.58 pitch -17.20 + [-0.29,-0.30,0.91]
        // 23.8  Lowest WE 113 4.95 Camera @[-183.80,-184.33,415.98] yaw -19.32 pitch -15.27 + [-0.32,-0.26,0.91]
        // 23.85 Lowest WE 133 5.57 Camera @[-185.01,-194.11,408.40] yaw -19.75 pitch -16.60 + [-0.32,-0.29,0.90]
        // 23.95 Lowest WE 96 5.05 Camera @[-183.71,-183.40,412.44] yaw -19.42 pitch -15.30 + [-0.32,-0.26,0.91]
        // 24.05 Lowest WE 246 6.71 Camera @[-185.75,-179.94,408.13] yaw -19.85 pitch -15.00 + [-0.33,-0.26,0.91]
        // 27.05 Lowest WE 251 9.70 Camera @[-157.94,-162.90,358.31] yaw -18.47 pitch -14.75 + [-0.31,-0.25,0.92]
        // 28.05 Lowest WE 175 12.26 Camera @[-147.95,-157.90,346.69] yaw -17.63 pitch -14.44 + [-0.29,-0.25,0.92]
        // 29.05 Lowest WE 213 14.73 66.67 Camera @[-142.12,-150.93,332.64] yaw -17.39 pitch -14.02 + [-0.29,-0.24,0.93]

        // With new rotation adjustment to 'worst case of all' by spinnning aroud all of them one by one
        // 22.57 WE 5.18 Camera @[-195.94,-203.95,434.83] yaw -20.00 pitch -16.72 + [-0.33,-0.29,0.90]
        // 22.58 Lowest WE 2 4.77 18.39 Camera @[-195.95,-203.95,434.86] yaw -19.97 pitch -16.73 + [-0.33,-0.29,0.90]
        // 22.59 Lowest WE 1 4.65 17.13 Camera @[-195.98,-203.99,434.95] yaw -19.96 pitch -16.74 + [-0.33,-0.29,0.90]
        // 22.6  Lowest WE 1 4.55 20.35 Camera @[-196.02,-204.02,435.05] yaw -19.91 pitch -16.76 + [-0.33,-0.29,0.90]
        // 22.61 Lowest WE 11 4.57 20.24 Camera @[-195.94,-203.95,434.85] yaw -19.91 pitch -16.76 + [-0.33,-0.29,0.90]
        // 22.62 Lowest WE 1 4.92 20.56 Camera @[-196.02,-204.01,435.04] yaw -19.97 pitch -16.74 + [-0.33,-0.29,0.90]
        // 22.63 Lowest WE 3 4.87 19.99 Camera @[-195.96,-203.97,434.89] yaw -19.93 pitch -16.74 + [-0.33,-0.29,0.90]
        // 22.64 Lowest WE 2 4.97 21.15 Camera @[-195.95,-203.96,434.86] yaw -19.91 pitch -16.74 + [-0.33,-0.29,0.90]
        // 22.65 Lowest WE 3 5.16 20.74 Camera @[-195.98,-203.98,434.94] yaw -19.92 pitch -16.73 + [-0.33,-0.29,0.90]

        // CDATA 1
        // 22.62 Lowest WE 6 12.06 73.79 Camera @[54.87,-29.79,781.31] yaw 3.00 pitch -6.00 + [0.05,-0.10,0.99]
        // 22.7  Lowest WE 5 11.76 75.93 Camera @[54.27,-29.57,777.99] yaw 2.96 pitch -6.00 + [0.05,-0.10,0.99]
        22.62_f64.to_radians().tan()
    }
}

//ip Display for LCamera
impl std::fmt::Display for LCamera {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let dxyz = quat::apply3(&quat::conjugate(self.direction.as_ref()), &[0., 0., 1.]);
        // First rotation around Y axis (yaw)
        let yaw = dxyz[0].atan2(dxyz[2]).to_degrees();
        // Then rotation around X axis (elevation)
        let pitch = dxyz[1]
            .atan2((dxyz[0] * dxyz[0] + dxyz[2] * dxyz[2]).sqrt())
            .to_degrees();
        write!(
            fmt,
            "@[{:.2},{:.2},{:.2}] yaw {:.2} pitch {:.2} + [{:.2},{:.2},{:.2}]",
            self.position[0],
            self.position[1],
            self.position[2],
            yaw,
            pitch,
            dxyz[0],
            dxyz[1],
            dxyz[2]
        )
    }
}

//ip LCamera
impl LCamera {
    //fp new
    pub fn new(position: Point3D, direction: Quat) -> Self {
        Self {
            position,
            direction,
        }
    }

    //fp moved_by
    pub fn moved_by(&self, dp: [f64; 3]) -> Self {
        let position = self.position + Point3D::from(dp);
        Self {
            position,
            direction: self.direction,
        }
    }

    //fp position
    pub fn position(&self) -> &[f64; 3] {
        self.position.as_ref()
    }

    //fp rotation_matrix
    pub fn rotation_matrix(&self) -> [f64; 9] {
        let mut rot = [0.; 9];
        quat::to_rotation3(self.direction.as_ref(), &mut rot);
        rot
    }

    //fp to_camera_space
    pub fn to_camera_space(&self, model_xyz: &Point3D) -> Point3D {
        let camera_relative_xyz = *model_xyz - self.position;
        quat::apply3(self.direction.as_ref(), camera_relative_xyz.as_ref()).into()
    }

    //fp to_sph_xy
    pub fn to_sph_xy(&self, model_xyz: &Point3D) -> Point2D {
        let camera_xyz = self.to_camera_space(model_xyz);
        let camera_as_sph_x = camera_xyz[0] / camera_xyz[2];
        let camera_as_sph_y = camera_xyz[1] / camera_xyz[2];
        [camera_as_sph_x, camera_as_sph_y].into()
    }

    //fp to_scr_xy
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
        if camera_xyz[2].abs() < 0.00001 {
            return [0., 0.].into();
        }
        [
            centre[0] + camera_as_sph_x * wh[0] / 2.0 / self.tan_fov_x(),
            centre[1] - camera_as_sph_y * wh[1] / 2.0 / self.tan_fov_y(),
        ]
        .into()
    }

    //fp apply_quat_to_get_min_sq_error
    pub fn apply_quat_to_get_min_sq_error(
        &self,
        steps_per_rot: usize,
        pm: &PointMapping,
        q: &Quat,
    ) -> (Self, f64) {
        let mut c = *self;
        let mut tc = c;
        let mut e = pm.get_sq_error(&c);
        for _ in 0..steps_per_rot {
            tc.direction = c.direction * *q;
            let ne = pm.get_sq_error(&tc);
            if ne > e {
                break;
            }
            c = tc;
            e = ne;
        }
        return (c, e);
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
            (c, e) = c.apply_quat_to_get_min_sq_error(steps_per_rot, pm, &q);
        }
        (c, e)
    }

    //fp error_with_quat
    #[inline]
    fn error_with_quat(&self, pm: &PointMapping, quat: &Quat) -> f64 {
        let mut c = self.clone();
        c.direction = self.direction * *quat;
        pm.get_sq_error(&c)
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
        let mut c = *self;
        let mut tc = c;
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
            tc.direction = c.direction * Quat::from(q);
            let ne = f(&tc, mappings, test_pm);
            if ne > e {
                // dbg!("Adjusted", i, e, ne, k_x_k_x_t);
                *c.direction.as_mut() = quat::normalize(*c.direction.as_ref());
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
        return (c, e);
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
        let keep_v = self.to_camera_space(mappings[keep_pm].model());
        let mut rot: Quat = quat::of_axis_angle(keep_v.as_ref(), da).into();
        let mut c = *self;
        let mut tc = c;
        let mut e = f(&c, mappings, test_pm);
        for _sc in 0..2 {
            // dbg!("Preadjusted", e);
            for _i in 0..100_000 {
                tc.direction = rot * c.direction;
                let ne = f(&tc, mappings, test_pm);
                if ne > e {
                    // dbg!("Adjusted", i, e, ne);
                    *c.direction.as_mut() = quat::normalize(*c.direction.as_ref());
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
        return (c, e);
    }

    //fp find_best_error
    pub fn find_best_error(&self, mappings: &[PointMapping]) -> (usize, f64) {
        let mut n = 0;
        let mut best_e = 1000_000_000.0;
        for (i, pm) in mappings.iter().enumerate() {
            let e = pm.get_sq_error(self);
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
            let e = pm.get_sq_error(self);
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
            let e = pm.get_sq_error(self);
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
        let mut cam = *self;
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
            cam = cn
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
        let [dx, dy, dz] = quat::apply3(&quat::conjugate(self.direction.as_ref()), &[0., 0., 1.]);
        dbg!(dx, dy, dz);
        let mut cam = *self;
        let mut e = f(&cam, mappings);
        for sc in [1., -1., 0.01, -0.01] {
            for _ in 0..10_000 {
                let cn = cam.moved_by([dx * sc, dy * sc, dz * sc]);
                let en = f(&cn, mappings);
                if en > e {
                    break;
                }
                e = en;
                cam = cn
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
        dbg!("Find coarse position", self, scales, n);
        let coarse_rotations = Rotations::new(1.0_f64.to_radians());
        let fine_rotations = Rotations::new(0.1_f64.to_radians());
        let mut worst_data = (1_000_000.0, 0, *self, 0.);
        let num = mappings.len();
        let map = |i, sc| ((i as f64) - (n as f64) / 2.0) * sc / (n as f64);
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
                        &|c, _m, _n| c.worst_error(&mappings),
                        0.2_f64.to_radians(),
                        mappings,
                        i,
                        0,
                    )
                    .0;
            }
            /*
            let mut last_n = cam.find_worst_error(mappings).0;
            for i in 0..30 {
                let n = cam.find_worst_error(mappings).0;
                dbg!(i, n, last_n);
                if n == last_n {
                    last_n = (last_n + 1 + (i % (num - 1))) % num;
                }
                cam = cam
                    .adjust_direction_while_keeping_one_okay(
                        1000,
                        0.5_f64.to_radians(),
                        &fine_rotations,
                        &|c, m, n| m[n].get_sq_error(c),
                        mappings,
                        last_n,
                        n,
                    )
                    .0;
                last_n = n;
            }
             */
            let te = cam.total_error(mappings);
            let we = cam.worst_error(mappings);
            if we < worst_data.0 {
                worst_data = (we, i, cam, te);
                // dbg!(worst_data);
            }
        }
        dbg!(worst_data);
        worst_data.2
    }

    //zz All done
}
