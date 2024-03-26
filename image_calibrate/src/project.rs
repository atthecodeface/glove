//a Imports
use std::cell::{Ref, RefMut};

use serde::{Deserialize, Serialize};

use crate::utils::Rrc;
use crate::{
    json, CameraDatabase, CameraInstance, CameraInstanceDesc, NamedPointSet, PointMappingSet,
};

//a Project
//tp CipFileDesc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CipFileDesc {
    camera: String,
    image: String,
    pms: String,
}

//tp CipDesc
#[derive(Debug, Serialize, Deserialize)]
pub struct CipDesc {
    camera: CameraInstanceDesc,
    image: String,
    pms: PointMappingSet,
}

//tp Cip
#[derive(Debug)]
pub struct Cip {
    camera: Rrc<CameraInstance>,
    pms: Rrc<PointMappingSet>,
    image: String,
}

//ip Cip
impl Cip {
    pub fn from_json(
        project: &Project,
        desc: &CipFileDesc,
        camera_json: &str,
        pms_json: &str,
    ) -> Result<(Self, String), String> {
        let camera = CameraInstance::from_json(project.cdb(), camera_json)?.into();
        let (pms, warnings) = PointMappingSet::from_json(&project.nps().borrow(), pms_json)?;
        let pms = pms.into();
        let image = desc.image.clone();
        Ok((Self { camera, pms, image }, warnings))
    }
    pub fn from_desc(project: &Project, cip_desc: CipDesc) -> Result<(Self, String), String> {
        let image = cip_desc.image;
        let camera = CameraInstance::from_desc(project.cdb(), cip_desc.camera)?.into();
        let pms: Rrc<PointMappingSet> = cip_desc.pms.into();
        let _warnings = pms
            .borrow_mut()
            .rebuild_with_named_point_set(&project.nps_ref());
        Ok((Self { camera, pms, image }, "".into()))
    }
}

//tp ProjectFileDesc
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectFileDesc {
    cdb: String,
    nps: Vec<String>,
    cips: Vec<CipFileDesc>,
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
#[derive(Debug, Default)]
pub struct Project {
    desc: ProjectFileDesc,
    cdb: CameraDatabase,
    nps: Rrc<NamedPointSet>,
    cips: Vec<Cip>,
}

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

//ip Deserialize for Project
impl<'de> Deserialize<'de> for Project {
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        let project_desc = <ProjectDesc>::deserialize(deserializer)?;
        let cdb = project_desc.cdb;
        let nps = project_desc.nps.into();
        let cips = project_desc.cips;
        let desc = ProjectFileDesc::default();
        let mut project = Self {
            desc,
            cdb,
            nps,
            cips: vec![],
        };
        for cip_desc in cips {
            use serde::de::Error;
            let (cip, _warnings) = Cip::from_desc(&project, cip_desc)
                .map_err(|e| DE::Error::custom(format!("bad CIP desc: {e}")))?;
            project.cips.push(cip);
        }
        Ok(project)
    }
}

//ip Project
impl Project {
    pub fn desc(&self) -> &ProjectFileDesc {
        &self.desc
    }
    pub fn cdb(&self) -> &CameraDatabase {
        &self.cdb
    }
    pub fn nps(&self) -> &Rrc<NamedPointSet> {
        &self.nps
    }
    pub fn set_nps(&mut self, nps: Rrc<NamedPointSet>) {
        self.nps = nps;
    }
    pub fn nps_ref(&self) -> Ref<NamedPointSet> {
        self.nps.borrow()
    }
    pub fn nps_mut(&self) -> RefMut<NamedPointSet> {
        self.nps.borrow_mut()
    }
    pub fn from_desc_json(
        desc: ProjectFileDesc,
        cdb_json: &str,
        nps_json: &str,
    ) -> Result<Self, String> {
        let cdb: CameraDatabase = json::from_json("camera database", cdb_json)?;
        let nps: NamedPointSet = json::from_json("named point set", nps_json)?;
        let nps = nps.into();
        let cips = vec![];
        Ok(Self {
            desc,
            cdb,
            nps,
            cips,
        })
    }
    pub fn add_cip(
        &mut self,
        desc: &CipFileDesc,
        camera_json: &str,
        pms_json: &str,
    ) -> Result<String, String> {
        let (cip, warnings) = Cip::from_json(self, desc, camera_json, pms_json)?;
        self.cips.push(cip);
        Ok(warnings)
    }
    pub fn desc_to_json(&self) {}
    // pub fn to_json(&self) -> Result<String, String> {
    // serde_json::to_string(self).map_err(|e| format!("{}", e))
    // }
    pub fn cip(&self, n: usize) -> &Cip {
        &self.cips[n]
    }
    pub fn cip_mut(&mut self, n: usize) -> &mut Cip {
        &mut self.cips[n]
    }
}
