//a Imports
use std::rc::Rc;

use ic_photogram::CacheRef;
use ic_photogram::CameraSensor;
use ic_photogram::NamedPoint;
use ic_photogram::Result;
use ic_photogram::{Image, ImagePt, ImageRgb8};

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
        let nps = self.nps().borrow();
        let r = nps.select(self.np.iter().map(|s| s.as_str()))?;
        if r.is_empty() {
            return Ok(nps.iter().cloned().collect());
        }
        Ok(r)
    }

    pub fn get_pms_indices_of_nps(&self) -> Result<Vec<usize>> {
        let mut pms = vec![];
        let nps = self.get_nps()?;
        let Some(cip_pms) = self.pms() else {
            return Ok(vec![]);
        };
        for (i, m) in cip_pms.borrow().mappings().iter().enumerate() {
            for n in &nps {
                if Rc::ptr_eq(n, m.named_point()) {
                    pms.push(i);
                }
            }
        }
        Ok(pms)
    }

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

    pub fn get_read_image(&mut self, n: usize) -> Result<CacheRef> {
        let Some(read_filename) = self.read_img.get(n) else {
            return Err(format!("Required at least {} read images to be specified", n + 1).into());
        };
        let Some(read_filename) = self.path_set.find_file(read_filename) else {
            return Err(format!("could not finde image file {read_filename}").into());
        };
        self.image_cache.src_image(read_filename)
    }

    pub fn get_cip_image(&mut self) -> Result<CacheRef> {
        let Some(cip) = self.cip() else {
            return Err("No CIP to get image for".into());
        };
        let cip = cip.borrow();
        let cip_image_filename = cip.image_filename();
        let Some(read_filename) = self.path_set.find_file(cip_image_filename) else {
            return Err(format!("could not finde image file {cip_image_filename}").into());
        };
        drop(cip);
        self.image_cache.src_image(read_filename)
    }

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
