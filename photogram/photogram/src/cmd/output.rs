//a Imports
use std::cell::Ref;
use std::io::Write;
use std::rc::Rc;

use clap::{Arg, ArgAction, ArgMatches, Command};
use star_catalog::{Catalog, StarFilter};
use thunderclap::{ArgCount, CommandArgs, CommandBuilder};

use ic_base::{json, Ray, Rrc};
use ic_base::{Error, Result};
use ic_camera::CameraInstance;
use ic_camera::{CalibrationMapping, CameraDatabase, CameraProjection, LensPolys};
use ic_image::{Color, Image, ImagePt, ImageRgb8};
use ic_mapping::{CameraPtMapping, PointMapping};
use ic_mapping::{NamedPointSet, PointMappingSet};
use ic_project::{Cip, Project};
use ic_stars::StarMapping;

use super::{CmdArgs, CmdResult};

//a CmdArgs output methods
//ip CmdArgs output methods
impl CmdArgs {
    //mp write_outputs
    pub fn write_outputs(&self) -> Result<()> {
        if let Some(filename) = &self.write_camera {
            let s = self.camera.to_json()?;
            let mut f = std::fs::File::create(filename)?;
            f.write_all(s.as_bytes())?;
        }
        if let Some(filename) = &self.write_polys {
            let s = self.camera.lens().polys().to_json()?;
            let mut f = std::fs::File::create(filename)?;
            f.write_all(s.as_bytes())?;
        }
        if let Some(filename) = &self.write_calibration_mapping {
            let s = self.calibration_mapping.clone().to_json()?;
            let mut f = std::fs::File::create(filename)?;
            f.write_all(s.as_bytes())?;
        }
        if let Some(filename) = &self.write_star_mapping {
            let s = self.star_mapping.clone().to_json()?;
            let mut f = std::fs::File::create(filename)?;
            f.write_all(s.as_bytes())?;
        }
        Ok(())
    }

    //mp output_camera
    pub fn output_camera(&self) -> CmdResult {
        let s = self.camera.to_json()?;
        Ok(s.to_string())
    }

    //mp output_calibration_mapping
    pub fn output_calibration_mapping(&self) -> CmdResult {
        let s = self.calibration_mapping.clone().to_json()?;
        Ok(s.to_string())
    }

    //mp output_star_mapping
    pub fn output_star_mapping(&self) -> CmdResult {
        let s = self.star_mapping.clone().to_json()?;
        Ok(s.to_string())
    }

    //mp output_polynomials
    pub fn output_polynomials(&self) -> CmdResult {
        let s = self.camera.lens().polys().to_json()?;
        Ok(s.to_string())
    }
}
