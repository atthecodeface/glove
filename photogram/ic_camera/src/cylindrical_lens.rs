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
        let camera_ty = self.tan_phi_of_ty(optical_txty.tany());
        let camera_tx = optical_txty.tanx();
        TanXTanY::of_tx_ty(camera_tx, camera_ty)
    }
    fn camera_txty_to_optical_txty(&self, camera_txty: TanXTanY) -> TanXTanY {
        let optical_ty = self.ty_of_tan_phi(camera_txty.tany());
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
    fn set_vfov(&mut self, ty_sc: f64, fov_v: f64, v_ofs: f64) {
        self.projection.set_vfov(ty_sc, fov_v, v_ofs)
    }
    fn phi_of_ty(&self, ty: f64) -> f64 {
        self.projection.phi_of_ty(ty)
    }
    fn tan_phi_of_ty(&self, ty: f64) -> f64 {
        self.projection.tan_phi_of_ty(ty)
    }
    fn ty_of_phi(&self, phi: f64) -> f64 {
        self.projection.ty_of_phi(phi)
    }
    fn ty_of_tan_phi(&self, tan_phi: f64) -> f64 {
        self.projection.ty_of_tan_phi(tan_phi)
    }
    fn boxed_clone(&self) -> Box<dyn CylindricalProjection> {
        Box::new(self.clone())
    }
}

/// Type that helps to map ty in the range +-ty_sc to
/// v_ofs+-fov_v/2
#[derive(Default, Debug, Clone)]
struct TyToYRel {
    ty_sc: f64,
    range_center: f64,
    range_sc: f64,
}
impl TyToYRel {
    fn set(&mut self, ty_sc: f64, min: f64, max: f64) {
        self.ty_sc = ty_sc;
        self.range_sc = (max - min);
        self.range_center = (max + min) / 2.0;
    }
    fn ty_to_range(&self, ty: f64) -> f64 {
        self.range_center + (ty * self.ty_sc) * self.range_sc
    }
    fn range_to_ty(&self, r: f64) -> f64 {
        (r - self.range_center) / self.range_sc / self.ty_sc
    }
}

/// An equirectangular Y projection for a cylindrical lens
///
/// In this kind of projection the tan(lens vertical angle) maps to Y on the sensor
///
/// ty, the tan(lens vertical angle) is scaled (by ty_sc) and then maps linearly to Y
#[derive(Default, Debug, Clone)]
struct CylindricalEquirectangular(TyToYRel);

impl CylindricalProjection for CylindricalEquirectangular {
    fn name(&self) -> &str {
        "equirectangular"
    }
    fn set_vfov(&mut self, ty_sc: f64, fov_v: f64, v_ofs: f64) {
        self.0.set(ty_sc, v_ofs - fov_v / 2.0, v_ofs + fov_v / 2.0);
    }
    fn phi_of_ty(&self, ty: f64) -> f64 {
        self.0.ty_to_range(ty)
    }
    fn ty_of_phi(&self, phi: f64) -> f64 {
        self.0.range_to_ty(phi)
    }
    fn boxed_clone(&self) -> Box<dyn CylindricalProjection> {
        Box::new(self.clone())
    }
}

#[derive(Default, Debug, Clone)]
struct CylindricalLambert(TyToYRel);

impl CylindricalProjection for CylindricalLambert {
    fn name(&self) -> &str {
        "lambert"
    }
    fn set_vfov(&mut self, ty_sc: f64, fov_v: f64, v_ofs: f64) {
        let min_y = (v_ofs - fov_v / 2.0).sin();
        let max_y = (v_ofs + fov_v / 2.0).sin();
        self.0.set(ty_sc, min_y, max_y);
    }
    fn phi_of_ty(&self, ty: f64) -> f64 {
        let y_angle = self.0.ty_to_range(ty);
        y_angle.asin()
    }
    fn ty_of_phi(&self, phi: f64) -> f64 {
        let y_angle = phi.sin();
        self.0.range_to_ty(y_angle)
    }
    fn boxed_clone(&self) -> Box<dyn CylindricalProjection> {
        Box::new(self.clone())
    }
}

#[derive(Default, Debug, Clone)]
struct CylindricalCentral(TyToYRel);

impl CylindricalProjection for CylindricalCentral {
    fn name(&self) -> &str {
        "central"
    }
    fn set_vfov(&mut self, ty_sc: f64, fov_v: f64, v_ofs: f64) {
        let min_y = (v_ofs - fov_v / 2.0).tan();
        let max_y = (v_ofs + fov_v / 2.0).tan();
        self.0.set(ty_sc, min_y, max_y);
    }
    fn phi_of_ty(&self, ty: f64) -> f64 {
        self.tan_phi_of_ty(ty).atan()
    }
    fn tan_phi_of_ty(&self, ty: f64) -> f64 {
        self.0.ty_to_range(ty)
    }
    fn ty_of_tan_phi(&self, tan_phi: f64) -> f64 {
        self.0.range_to_ty(tan_phi)
    }
    fn ty_of_phi(&self, phi: f64) -> f64 {
        self.ty_of_tan_phi(phi.tan())
    }
    fn boxed_clone(&self) -> Box<dyn CylindricalProjection> {
        Box::new(self.clone())
    }
}

#[derive(Default, Debug, Clone)]
struct CylindricalStereographic(TyToYRel);

impl CylindricalProjection for CylindricalStereographic {
    fn name(&self) -> &str {
        "stereographic"
    }
    fn set_vfov(&mut self, ty_sc: f64, fov_v: f64, v_ofs: f64) {
        let min_y = ((v_ofs - fov_v / 2.0) / 2.0).tan();
        let max_y = ((v_ofs + fov_v / 2.0) / 2.0).tan();
        self.0.set(ty_sc, min_y, max_y);
    }
    fn phi_of_ty(&self, ty: f64) -> f64 {
        let y_angle = self.0.ty_to_range(ty);
        2.0 * y_angle.atan()
    }
    fn ty_of_phi(&self, phi: f64) -> f64 {
        let y_angle = (phi / 2.0).tan();
        self.0.range_to_ty(y_angle)
    }
    fn boxed_clone(&self) -> Box<dyn CylindricalProjection> {
        Box::new(self.clone())
    }
}
