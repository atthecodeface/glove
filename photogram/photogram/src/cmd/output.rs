//a Imports
use std::io::Write;

use ic_photogram::{Image, Result};

use thunderclap::json;

use super::{CmdArgs, CmdResult};

impl CmdArgs {
    pub fn write_outputs(&self) -> Result<()> {
        if let Some(filename) = &self.write_project {
            let s = self.project.to_json(true)?;
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
        if let Some(filename) = &self.write_patches {
            self.project()
                .np_images_ref()
                .image()
                .borrow()
                .write(filename)?;
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
