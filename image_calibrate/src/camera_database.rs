//a Imports
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::{CameraBody, CameraSensor, SphericalLensPoly};

//a CameraDatabas
//tp CameraDatabase
/// A database of camera bodies and lenses
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CameraDatabase {
    bodies: Vec<Rc<CameraBody>>,
    lenses: Vec<Rc<SphericalLensPoly>>,
}

//ip Display for CameraDatabase
impl std::fmt::Display for CameraDatabase {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(fmt, "Bodies:")?;
        for b in self.bodies.iter() {
            writeln!(fmt, "{}", b)?;
        }
        writeln!(fmt, "Lenses:")?;
        for l in self.lenses.iter() {
            writeln!(fmt, "{}", l)?;
        }
        Ok(())
    }
}

//ip CameraDatabase
impl CameraDatabase {
    //mp derive
    pub fn derive(&mut self) {
        for b in self.bodies.iter_mut() {
            Rc::get_mut(b).unwrap().derive();
        }
    }

    //ap get_body
    pub fn get_body(&self, name: &str) -> Option<Rc<CameraBody>> {
        for b in self.bodies.iter() {
            if b.name() == name {
                return Some(b.clone());
            }
        }
        None
    }

    //mp add_body
    pub fn add_body(&mut self, body: CameraBody) -> Result<(), String> {
        if self.get_body(body.name()).is_some() {
            Err(format!("Body {} already in the database", body.name()))
        } else {
            self.bodies.push(Rc::new(body));
            Ok(())
        }
    }

    //ap get_lens
    pub fn get_lens(&self, name: &str) -> Option<Rc<SphericalLensPoly>> {
        for l in self.lenses.iter() {
            if l.name() == name {
                return Some(l.clone());
            }
        }
        None
    }

    //mp add_lens
    pub fn add_lens(&mut self, lens: SphericalLensPoly) -> Result<(), String> {
        if self.get_lens(lens.name()).is_some() {
            Err(format!("Lens {} already in the database", lens.name()))
        } else {
            self.lenses.push(Rc::new(lens));
            Ok(())
        }
    }

    //zz All done
}
