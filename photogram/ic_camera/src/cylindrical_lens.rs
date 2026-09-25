use ic_base::{Result, TanXTanY};

use crate::{CylindricalProjection, LensProjection};

/// A cylindrical lens that can be used for projections
#[derive(Debug)]
pub struct CylindricalLens {
    projection: Box<dyn CylindricalProjection>,
}

impl std::default::Default for CylindricalLens {
    fn default() -> Self {
        Self {
            projection: Box::new(CylindricalCentral::default()),
        }
    }
}

impl Clone for CylindricalLens {
    fn clone(&self) -> Self {
        Self {
            projection: self.projection.boxed_clone(),
        }
    }
}

impl LensProjection for CylindricalLens {
    // TanXTanY operates in the sensor/optical domain using x and y as 0 on
    // axis, and then a relative value (unitless) that in theory is mm distance
    // (in X or Y) divided by mm distance from sensor to 'lens'
    //
    // Hence y goes to tan(phi), where y is actually optical_txty.tany
    fn optical_txty_to_camera_txty(&self, optical_txty: TanXTanY) -> TanXTanY {
        let camera_ty = self.tan_phi_of_y(optical_txty.tany());
        let camera_tx = optical_txty.tanx();
        TanXTanY::of_tx_ty(camera_tx, camera_ty)
    }
    fn camera_txty_to_optical_txty(&self, camera_txty: TanXTanY) -> TanXTanY {
        let optical_ty = self.y_of_phi(camera_txty.tany().atan());
        let optical_tx = camera_txty.tanx();
        TanXTanY::of_tx_ty(optical_tx, optical_ty)
    }
}

impl CylindricalLens {
    pub fn set_projection(&mut self, projection: &str) -> Result<()> {
        self.projection = {
            match projection {
                "equirectangular" => Box::new(CylindricalEquirectangular::default()),
                "lambert" => Box::new(CylindricalLambert::default()),
                "central" => Box::new(CylindricalCentral::default()),
                "stereographic" => Box::new(CylindricalStereographic::default()),
                _ => Box::new(CylindricalCentral::default()),
            }
        };
        Ok(())
    }
}

impl CylindricalProjection for CylindricalLens {
    fn name(&self) -> &str {
        self.projection.name()
    }
    fn set_vfov(&mut self, fov_v: f64, v_ofs: f64) {
        self.projection.set_vfov(fov_v, v_ofs)
    }
    fn phi_of_y(&self, y_relative: f64) -> f64 {
        self.projection.phi_of_y(y_relative)
    }
    fn tan_phi_of_y(&self, y_relative: f64) -> f64 {
        self.projection.tan_phi_of_y(y_relative)
    }
    fn y_of_phi(&self, phi: f64) -> f64 {
        self.projection.y_of_phi(phi)
    }
    fn boxed_clone(&self) -> Box<dyn CylindricalProjection> {
        Box::new(self.clone())
    }
}

#[derive(Default, Debug, Clone)]
struct CylindricalEquirectangular {
    range_y: f64,
    max_y: f64,
}

impl CylindricalProjection for CylindricalEquirectangular {
    fn name(&self) -> &str {
        "equirectangular"
    }
    fn set_vfov(&mut self, fov_v: f64, v_ofs: f64) {
        self.max_y = v_ofs + fov_v / 2.0;
        self.range_y = fov_v;
    }
    fn phi_of_y(&self, y_relative: f64) -> f64 {
        self.max_y - y_relative * self.range_y
    }
    fn y_of_phi(&self, phi: f64) -> f64 {
        (self.max_y - phi) / self.range_y
    }
    fn boxed_clone(&self) -> Box<dyn CylindricalProjection> {
        Box::new(self.clone())
    }
}

#[derive(Default, Debug, Clone)]
struct CylindricalLambert {
    max_minus_min_y: f64,
    max_y: f64,
}

impl CylindricalProjection for CylindricalLambert {
    fn name(&self) -> &str {
        "lambert"
    }
    fn set_vfov(&mut self, fov_v: f64, v_ofs: f64) {
        let min_y = (v_ofs - fov_v / 2.0).sin();
        let max_y = (v_ofs + fov_v / 2.0).sin();
        self.max_y = max_y;
        self.max_minus_min_y = max_y - min_y;
    }
    fn phi_of_y(&self, y_relative: f64) -> f64 {
        let y_angle = self.max_y - y_relative * self.max_minus_min_y;
        y_angle.asin()
    }
    fn y_of_phi(&self, phi: f64) -> f64 {
        let y_angle = phi.sin();
        (self.max_y - y_angle) / self.max_minus_min_y
    }
    fn boxed_clone(&self) -> Box<dyn CylindricalProjection> {
        Box::new(self.clone())
    }
}

#[derive(Default, Debug, Clone)]
struct CylindricalCentral {
    max_minus_min_y: f64,
    max_y: f64,
}
impl CylindricalProjection for CylindricalCentral {
    fn name(&self) -> &str {
        "central"
    }
    fn set_vfov(&mut self, fov_v: f64, v_ofs: f64) {
        let min_y = (v_ofs - fov_v / 2.0).tan();
        let max_y = (v_ofs + fov_v / 2.0).tan();
        self.max_y = max_y;
        self.max_minus_min_y = max_y - min_y;
    }
    fn phi_of_y(&self, y_relative: f64) -> f64 {
        self.tan_phi_of_y(y_relative).atan()
    }
    fn tan_phi_of_y(&self, y_relative: f64) -> f64 {
        self.max_y - y_relative * self.max_minus_min_y
    }
    fn y_of_phi(&self, phi: f64) -> f64 {
        let y_angle = phi.tan();
        (self.max_y - y_angle) / self.max_minus_min_y
    }
    fn boxed_clone(&self) -> Box<dyn CylindricalProjection> {
        Box::new(self.clone())
    }
}

#[derive(Default, Debug, Clone)]
struct CylindricalStereographic {
    max_minus_min_y: f64,
    max_y: f64,
}

impl CylindricalProjection for CylindricalStereographic {
    fn name(&self) -> &str {
        "stereographic"
    }
    fn set_vfov(&mut self, fov_v: f64, v_ofs: f64) {
        let min_y = ((v_ofs - fov_v / 2.0) / 2.0).tan();
        let max_y = ((v_ofs + fov_v / 2.0) / 2.0).tan();
        self.max_y = max_y;
        self.max_minus_min_y = max_y - min_y;
    }
    fn phi_of_y(&self, y_relative: f64) -> f64 {
        let y_angle = self.max_y - y_relative * self.max_minus_min_y;
        2.0 * y_angle.atan()
    }
    fn y_of_phi(&self, phi: f64) -> f64 {
        let y_angle = (phi / 2.0).tan();
        (self.max_y - y_angle) / self.max_minus_min_y
    }
    fn boxed_clone(&self) -> Box<dyn CylindricalProjection> {
        Box::new(self.clone())
    }
}
