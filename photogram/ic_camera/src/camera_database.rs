use serde::{Deserialize, Serialize};

use ic_base::{Error, JsonParsable, Result};

use crate::{CameraBody, CameraLens, CameraSensor};

/// A database of camera bodies and lenses
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CameraDatabase {
    bodies: Vec<CameraBody>,
    lenses: Vec<CameraLens>,
}

impl JsonParsable for CameraDatabase {
    type PostParseArg = ();
    type PostParseResult = Self;
    fn reason() -> &'static str {
        "camera_database"
    }
    fn post_parse(mut self, _args: &Self::PostParseArg) -> Result<Self> {
        self.derive();
        Ok(self)
    }
}

impl std::fmt::Display for CameraDatabase {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        writeln!(fmt, "Bodies:")?;
        for b in self.bodies.iter() {
            writeln!(fmt, "{b}")?;
        }
        writeln!(fmt, "Lenses:")?;
        for l in self.lenses.iter() {
            writeln!(fmt, "{l}")?;
        }
        Ok(())
    }
}

impl CameraDatabase {
    pub fn to_json(&self, pretty: bool) -> Result<String> {
        if pretty {
            Ok(serde_json::to_string_pretty(self)?)
        } else {
            Ok(serde_json::to_string(self)?)
        }
    }

    pub fn derive(&mut self) {
        for b in self.bodies.iter_mut() {
            b.derive();
        }
    }

    pub fn bodies(&self) -> &[CameraBody] {
        &self.bodies
    }

    pub fn lenses(&self) -> &[CameraLens] {
        &self.lenses
    }

    pub fn get_body(&self, name: &str) -> Option<&CameraBody> {
        self.bodies.iter().find(|&b| b.has_name(name))
    }

    pub fn get_body_err(&self, name: &str) -> Result<&CameraBody> {
        self.get_body(name).ok_or(Error::Database(format!(
            "Body '{name}' was not in the database",
        )))
    }

    pub fn add_body(&mut self, body: CameraBody) -> Result<()> {
        if self.get_body(body.name()).is_some() {
            Err(Error::Database(format!(
                "Body {} already in the database",
                body.name()
            )))
        } else {
            self.bodies.push(body);
            Ok(())
        }
    }

    pub fn get_lens(&self, name: &str) -> Option<&CameraLens> {
        self.lenses.iter().find(|&l| l.has_name(name))
    }

    pub fn get_lens_err(&self, name: &str) -> Result<&CameraLens> {
        self.get_lens(name).ok_or(Error::Database(format!(
            "Lens '{name}' was not in the database",
        )))
    }

    pub fn add_lens(&mut self, lens: CameraLens) -> Result<()> {
        if self.get_lens(lens.name()).is_some() {
            Err(Error::Database(format!(
                "Lens {} already in the database",
                lens.name()
            )))
        } else {
            self.lenses.push(lens);
            Ok(())
        }
    }
}
