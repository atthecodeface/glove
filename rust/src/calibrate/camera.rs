//a Imports
use super::{Point2D, Point3D, PointMapping, Projection, Quat};

use geo_nd::{quat, vector};

//a Constants
const MIN_ERROR: f64 = 0.5;

//a Rotations
//tp Rotations
pub struct Rotations {
    pub quats: [Quat; 6],
}

//ip Rotations
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
        22.61_f64.to_radians().tan()
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
        [
            centre[0] + camera_as_sph_x * wh[0] / 2.0 / self.tan_fov_x(),
            centre[1] - camera_as_sph_y * wh[1] / 2.0 / self.tan_fov_y(),
        ]
        .into()
    }

    //fp apply_quat_to_get_min_sq_error
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

    //fp get_best_direction
    pub fn get_best_direction(&self, rotations: &Rotations, pm: &PointMapping) -> (Self, f64) {
        let mut c = self.clone();
        let mut e = 0.;
        for q in rotations.quats.iter() {
            (c, e) = c.apply_quat_to_get_min_sq_error(pm, &q);
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
        let [dx, dy, dz] = quat::apply3(&quat::conjugate(self.direction.as_ref()), &[0., 0., 1.]);
        // let [dx, dy, dz] = quat::apply3(&self.direction.as_ref(), &[0., 0., 1.]);
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

    //zz All done
}
