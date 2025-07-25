//a Imports
use std::cell::{Ref, RefMut};

use serde::{Deserialize, Serialize};

use ic_base::{json, Point3D, Ray, Result, Rrc};
use ic_camera::CameraDatabase;
use ic_mapping::{CameraPtMapping, NamedPointSet};

use crate::{Cip, CipDesc};

//a Project
//tp ProjectDesc
/// A project description is a deserializable that can be stored in a
/// JSON file
#[derive(Debug, Default, Serialize, Deserialize)]
struct ProjectDesc {
    cdb: CameraDatabase,
    nps: Rrc<NamedPointSet>,
    /// A list of CameraInstanceDesc and PointMappingSet, and an image name
    cips: Vec<CipDesc>,
}

//tp Project
/// This encompasses a complete project
///
/// It holds the camera database and a single named point set for a
/// set of images / model
///
/// It then also holds a camera/point mapping set and an image name
/// for each mapped image in the project
///
/// This can be serialized into a ProjectDesc.
///
/// The nps is in an Rrc to enable the Wasm (for example) to 'borrow'
/// it to add points, move points, etc without having to have such
/// methods on the project itself.
#[derive(Debug, Default, Serialize)]
pub struct Project {
    cdb: Rrc<CameraDatabase>,
    nps: Rrc<NamedPointSet>,
    cips: Vec<Rrc<Cip>>,
}

//ip Deserialize for Project
impl<'de> Deserialize<'de> for Project {
    fn deserialize<DE>(deserializer: DE) -> std::result::Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        let mut project_desc = <ProjectDesc>::deserialize(deserializer)?;
        project_desc.cdb.derive();
        let cdb = project_desc.cdb.into();
        let nps = project_desc.nps;
        let cips = project_desc.cips;
        let mut project = Self {
            cdb,
            nps,
            cips: vec![],
        };
        for cip_desc in cips {
            use serde::de::Error;
            let (cip, _warnings) = Cip::from_desc(&project, cip_desc)
                .map_err(|e| DE::Error::custom(format!("bad CIP desc: {e}")))?;
            project.cips.push(cip.into());
        }
        Ok(project)
    }
}

//ip Project
impl Project {
    //ap cdb
    pub fn cdb(&self) -> &Rrc<CameraDatabase> {
        &self.cdb
    }

    //ap nps
    pub fn nps(&self) -> &Rrc<NamedPointSet> {
        &self.nps
    }

    //ap cdb_ref
    pub fn cdb_ref(&self) -> Ref<CameraDatabase> {
        self.cdb.borrow()
    }

    //ap nps_ref
    /// Get a borrowed reference to the NamedPointSet
    pub fn nps_ref(&self) -> Ref<NamedPointSet> {
        self.nps.borrow()
    }

    //ap nps_mut
    /// Get a mutable borrowed reference to the NamedPointSet
    pub fn nps_mut(&self) -> RefMut<NamedPointSet> {
        self.nps.borrow_mut()
    }

    //ap ncips
    pub fn ncips(&self) -> usize {
        self.cips.len()
    }

    //ap cip
    pub fn cip(&self, n: usize) -> &Rrc<Cip> {
        &self.cips[n]
    }

    //cp from_json
    pub fn from_json(json: &str) -> Result<Self> {
        json::from_json("project", json)
    }

    //mp set_cdb
    /// Set the CameraDatabase for the [Project]
    pub fn set_cdb(&mut self, cdb: Rrc<CameraDatabase>) {
        self.cdb = cdb;
    }

    //mp set_nps
    /// Set the NamedPointSet for the [Project]
    pub fn set_nps(&mut self, nps: Rrc<NamedPointSet>) {
        self.nps = nps;
    }

    //mp add_cip
    pub fn add_cip(&mut self, cip: Rrc<Cip>) -> usize {
        let n = self.cips.len();
        self.cips.push(cip);
        n
    }

    //mp to_json
    pub fn to_json(&self, pretty: bool) -> Result<String> {
        if pretty {
            Ok(serde_json::to_string_pretty(self)?)
        } else {
            Ok(serde_json::to_string(self)?)
        }
    }

    //mp locate_all
    pub fn locate_all(&self, max_np_error: f64) {
        for cip in &self.cips {
            cip.borrow().locate(max_np_error);
        }
    }

    //mp derive_nps_location
    pub fn derive_nps_location(&self, name: &str) -> Option<(Point3D, f64)> {
        let mut rays = vec![];
        for cip in &self.cips {
            let cip = cip.borrow();
            for m in cip.pms_ref().mappings() {
                if m.name() == name {
                    rays.push(cip.camera_ref().get_pm_as_ray(m, true));
                }
            }
        }
        if rays.len() > 1 {
            if let Some(pt) = Ray::closest_point(&rays, &|_r| 1.0) {
                let e_sq = rays
                    .iter()
                    .fold(f64::MAX, |acc, r| acc.min(r.distances(&pt).1));
                Some((pt, e_sq.sqrt()))
            } else {
                None
            }
        } else {
            None
        }
    }

    //zz All done
}
