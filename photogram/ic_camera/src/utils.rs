//a Imports
use geo_nd::quat;

use ic_base::{Point3D, Quat};

//a Functions
pub fn show_pos_orient(position: &Point3D, orientation: &Quat) -> String {
    let dxyz = quat::apply3(&quat::conjugate(orientation.as_ref()), &[0., 0., 1.]);
    format!(
        "[{:.3},{:.3},{:.3}] in dir [{:.3},{:.3},{:.3}]",
        position[0], position[1], position[2], dxyz[0], dxyz[1], dxyz[2],
    )
}
