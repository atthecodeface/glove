//a Imports
use std::io::Write;

use ic_photogram::Result;

use thunderclap::json;

use super::{CmdArgs, CmdResult};

//a CmdArgs output methods
//ip CmdArgs output methods
impl CmdArgs {
    //mp write_outputs
    pub fn write_outputs(&self) -> Result<()> {
        if let Some(filename) = &self.write_project {
            let s = self.project.to_json(true)?;
            let mut f = std::fs::File::create(filename)?;
            f.write_all(s.as_bytes())?;
        }
        if let Some(filename) = &self.write_named_points {
            let s = self.nps().borrow().to_json(true)?;
            let mut f = std::fs::File::create(filename)?;
            f.write_all(s.as_bytes())?;
        }
        if let Some(filename) = &self.write_point_mapping {
            let s = self.pms().borrow().to_json(true)?;
            let mut f = std::fs::File::create(filename)?;
            f.write_all(s.as_bytes())?;
        }
        if let Some(filename) = &self.write_camera {
            let s = self.camera.to_json(true)?;
            let mut f = std::fs::File::create(filename)?;
            f.write_all(s.as_bytes())?;
        }
        if let Some(filename) = &self.write_polys {
            let s = self.camera.lens().polys().to_json(true)?;
            let mut f = std::fs::File::create(filename)?;
            f.write_all(s.as_bytes())?;
        }
        if let Some(filename) = &self.write_calibration_mapping {
            let s = self.calibration_mapping.to_json(true)?;
            let mut f = std::fs::File::create(filename)?;
            f.write_all(s.as_bytes())?;
        }
        Ok(())
    }

    pub fn output_camera(&self) -> CmdResult {
        Ok(json::to_value(&self.camera)?)
    }

    pub fn output_polynomials(&self) -> CmdResult {
        Ok(json::to_value(self.camera.lens().polys())?)
    }
}
