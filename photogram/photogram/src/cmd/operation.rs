//a Imports
use std::rc::Rc;

use ic_photogram::CameraProjection;
use ic_photogram::ModelData;
use ic_photogram::Result;
use ic_photogram::{Image, ImagePt, ImageRgb8};
use ic_photogram::{NamedPoint, NamedPointSet, PointMappingSet};

use super::CmdArgs;

//a Is regex
#[allow(dead_code)]
fn is_regex(s: &str) -> bool {
    s.chars().any(|c| "^[*?".contains(c))
}

//a CmdArgs accessors
//ip CmdArgs - Operations
impl CmdArgs {
    //mp get_nps
    pub fn get_nps(&self) -> Result<Vec<Rc<NamedPoint>>> {
        let r = self
            .nps
            .borrow()
            .select(self.np.iter().map(|s| s.as_str()))?;
        if r.is_empty() {
            return Ok(self.nps.borrow().iter().cloned().collect());
        }
        Ok(r)
    }

    //mp get_pms_indices_of_nps
    pub fn get_pms_indices_of_nps(&self) -> Result<Vec<usize>> {
        let mut pms = vec![];
        let nps = self.get_nps()?;
        for (i, m) in self.pms.borrow().mappings().iter().enumerate() {
            for n in &nps {
                if Rc::ptr_eq(n, m.named_point()) {
                    pms.push(i);
                }
            }
        }
        Ok(pms)
    }

    //mp pms_map
    pub fn pms_map<M, T>(&self, map: M) -> Result<T>
    where
        M: FnOnce(&PointMappingSet) -> Result<T>,
    {
        map(&self.pms.borrow())
    }

    /// Create a point mapping set from a calibration mapping
    pub fn calibration_mapping_to_pms(&self) -> PointMappingSet {
        let v = self.calibration_mapping.get_xyz_pairings();
        let mut nps = NamedPointSet::default();
        let mut pms = PointMappingSet::default();

        for (n, (model_xyz, pxy_abs)) in v.into_iter().enumerate() {
            let model = ModelData::at_infinity(model_xyz).with_uncertainty(0.0);
            let name = n.to_string();
            let color = [255, 255, 255, 255].into();
            nps.add_pt(&name, color, model);
            pms.add_mapping(&nps, &name, &pxy_abs, 0.);
        }
        pms
    }

    //mp draw_image
    pub fn draw_image(&self, pts: &[ImagePt]) -> Result<()> {
        if self.read_img.is_empty() || self.write_img.is_none() {
            return Ok(());
        }
        let mut img = ImageRgb8::read(&self.read_img[0])?;
        for p in pts {
            p.draw(&mut img);
        }
        img.write(self.write_img.as_ref().unwrap())?;
        Ok(())
    }

    //mp get_image_read_or_create
    pub fn get_image_read_or_create(&self) -> Result<ImageRgb8> {
        let read_filename = {
            if self.read_img.is_empty() {
                None
            } else {
                let Some(read_filename) = self.path_set.find_file(&self.read_img[0]) else {
                    return Err(format!("could not finde image file {}", self.read_img[0]).into());
                };
                Some(read_filename)
            }
        };
        let img = ImageRgb8::read_or_create_image::<std::path::PathBuf>(
            read_filename,
            Some((
                self.camera.sensor_px_size().0 as u32,
                self.camera.sensor_px_size().1 as u32,
            )),
        )?;
        Ok(img)
    }

    //mp get_read_image
    pub fn get_read_image(&self, n: usize) -> Result<ImageRgb8> {
        let Some(read_filename) = self.read_img.get(n) else {
            return Err(format!("Required at least {} read images to be specified", n + 1).into());
        };
        let Some(read_filename) = self.path_set.find_file(read_filename) else {
            return Err(format!("could not finde image file {read_filename}").into());
        };
        // TDODO: Change to read
        let img =
            ImageRgb8::read(read_filename).map_err(|e| (e, "failed to read image".to_string()))?;
        Ok(img)
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
