//a Imports
use super::Quat;

use geo_nd::quat;

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
