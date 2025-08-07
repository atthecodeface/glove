//a Imports
use ic_base::Result;
use ic_image::{Image, ImagePt, ImageRgb8};
use ic_mapping::{NamedPointSet, PointMappingSet};

use super::CmdArgs;

//a CmdArgs accessors
//ip CmdArgs - Operations
impl CmdArgs {
    //mp pms_map
    pub fn pms_map<M, T>(&self, map: M) -> ic_base::Result<T>
    where
        M: FnOnce(&PointMappingSet) -> ic_base::Result<T>,
    {
        let cip = self.cip.borrow();
        let pms = cip.pms_ref();
        map(&*pms)
    }

    //mp calibration_mapping_to_pms
    pub fn calibration_mapping_to_pms(&self) -> PointMappingSet {
        let v = self.calibration_mapping.get_xyz_pairings();
        let mut nps = NamedPointSet::default();
        let mut pms = PointMappingSet::default();

        //cb Add calibrations to NamedPointSet and PointMappingSet
        for (n, (model_xyz, pxy_abs)) in v.into_iter().enumerate() {
            let name = n.to_string();
            let color = [255, 255, 255, 255].into();
            nps.add_pt(name.clone(), color, Some(model_xyz), 0.);
            pms.add_mapping(&nps, &name, &pxy_abs, 0.);
        }
        pms
    }

    //mp draw_image
    pub fn draw_image(&self, pts: &[ImagePt]) -> Result<()> {
        if self.read_img.is_empty() || self.write_img.is_none() {
            return Ok(());
        }
        let mut img = ImageRgb8::read_image(&self.read_img[0])?;
        for p in pts {
            p.draw(&mut img);
        }
        img.write(self.write_img.as_ref().unwrap())?;
        Ok(())
    }

    //mp show_step
    pub fn show_step<S>(&self, s: S)
    where
        S: std::fmt::Display,
    {
        if self.verbose {
            eprintln!("\n{s}");
        }
    }

    //mp update_star_mappings
    pub fn update_star_mappings(&mut self) -> (usize, f64) {
        self.star_mapping.update_star_mappings(
            self.star_catalog.as_ref().unwrap(),
            &self.camera,
            self.closeness,
            self.yaw_error,
            self.yaw_min,
            self.yaw_max,
        )
    }

    //mp if_verbose
    pub fn if_verbose<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        if self.verbose {
            f()
        }
    }
}
