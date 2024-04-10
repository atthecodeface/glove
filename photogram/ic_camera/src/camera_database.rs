//a Imports
use serde::{Deserialize, Serialize};

use crate::CameraSensor;
use crate::{CameraBody, CameraLens};

//a CameraDatabase
//tp CameraDatabase
/// A database of camera bodies and lenses
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CameraDatabase {
    bodies: Vec<CameraBody>,
    lenses: Vec<CameraLens>,
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
    //cp from_json
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let mut cdb: Self = serde_json::from_str(json)?;
        cdb.derive();
        Ok(cdb)
    }

    //mp to_json
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("{}", e))
    }

    //mp derive
    pub fn derive(&mut self) {
        for b in self.bodies.iter_mut() {
            b.derive();
        }
    }

    //ap get_body
    pub fn get_body(&self, name: &str) -> Option<&CameraBody> {
        self.bodies.iter().find(|&b| b.has_name(name))
    }

    //ap get_body_err
    pub fn get_body_err(&self, name: &str) -> Result<&CameraBody, String> {
        self.get_body(name)
            .ok_or(format!("Body '{}' was not in the database", name))
    }

    //mp add_body
    pub fn add_body(&mut self, body: CameraBody) -> Result<(), String> {
        if self.get_body(body.name()).is_some() {
            Err(format!("Body {} already in the database", body.name()))
        } else {
            self.bodies.push(body);
            Ok(())
        }
    }

    //ap get_lens
    pub fn get_lens(&self, name: &str) -> Option<&CameraLens> {
        self.lenses.iter().find(|&l| l.has_name(name))
    }

    //ap get_lens_err
    pub fn get_lens_err(&self, name: &str) -> Result<&CameraLens, String> {
        self.get_lens(name)
            .ok_or(format!("Lens '{}' was not in the database", name))
    }

    //mp add_lens
    pub fn add_lens(&mut self, lens: CameraLens) -> Result<(), String> {
        if self.get_lens(lens.name()).is_some() {
            Err(format!("Lens {} already in the database", lens.name()))
        } else {
            self.lenses.push(lens);
            Ok(())
        }
    }

    //zz All done
}
