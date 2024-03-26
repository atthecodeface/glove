//a Imports
use std::cell::{Ref, RefMut};

use serde::{Deserialize, Serialize};

use crate::utils::Rrc;
use crate::{
    json, CameraDatabase, CameraInstance, CameraInstanceDesc, NamedPointSet, PointMappingSet,
};

//a Cip
//tp CipFileDesc
/// Used purely for loading for a project meta-Json file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CipFileDesc {
    camera: String,
    image: String,
    pms: String,
}

//tp CipDesc
#[derive(Debug, Serialize, Deserialize)]
pub struct CipDesc {
    camera_file: String,
    pms_file: String,
    camera: CameraInstanceDesc,
    image: String,
    pms: PointMappingSet,
}

//tp Cip
#[derive(Debug, Default)]
pub struct Cip {
    camera_file: String,
    pms_file: String,
    camera: Rrc<CameraInstance>,
    pms: Rrc<PointMappingSet>,
    image: String,
}

//ip Cip
impl Cip {
    //ap camera_file
    pub fn camera_file(&self) -> &str {
        &self.camera_file
    }

    //ap image
    pub fn image(&self) -> &str {
        &self.image
    }

    //ap pms_file
    pub fn pms_file(&self) -> &str {
        &self.pms_file
    }

    //mp set_camera_file
    pub fn set_camera_file<I: Into<String>>(&mut self, cam_file: I) {
        self.camera_file = cam_file.into();
    }

    //mp set_image
    pub fn set_image<I: Into<String>>(&mut self, image: I) {
        self.image = image.into();
    }

    //mp set_pms_file
    pub fn set_pms_file<I: Into<String>>(&mut self, pms_file: I) {
        self.pms_file = pms_file.into();
    }

    //cp from_json
    pub fn from_json(
        project: &Project,
        desc: &CipFileDesc,
        camera_json: &str,
        pms_json: &str,
    ) -> Result<(Self, String), String> {
        let camera = CameraInstance::from_json(&project.cdb().borrow(), camera_json)?.into();
        let (pms, warnings) = PointMappingSet::from_json(&project.nps().borrow(), pms_json)?;
        let pms = pms.into();
        let camera_file = desc.camera.clone();
        let pms_file = desc.pms.clone();
        let image = desc.image.clone();
        Ok((
            Self {
                camera_file,
                pms_file,
                camera,
                pms,
                image,
            },
            warnings,
        ))
    }

    //cp from_desc
    pub fn from_desc(project: &Project, cip_desc: CipDesc) -> Result<(Self, String), String> {
        let image = cip_desc.image;
        let camera = CameraInstance::from_desc(&project.cdb().borrow(), cip_desc.camera)?.into();
        let pms: Rrc<PointMappingSet> = cip_desc.pms.into();
        let _warnings = pms
            .borrow_mut()
            .rebuild_with_named_point_set(&project.nps_ref());
        let camera_file = cip_desc.camera_file.clone();
        let pms_file = cip_desc.pms_file.clone();
        Ok((
            Self {
                camera_file,
                pms_file,
                camera,
                pms,
                image,
            },
            "".into(),
        ))
    }

    //ap camera
    pub fn camera(&self) -> &Rrc<CameraInstance> {
        &self.camera
    }

    //mp set_camera
    pub fn set_camera(&mut self, camera: Rrc<CameraInstance>) {
        self.camera = camera;
    }

    //mp camera_ref
    /// Get a borrowed reference to the CameraInstance
    pub fn camera_ref(&self) -> Ref<CameraInstance> {
        self.camera.borrow()
    }

    //mp camera_mut
    /// Get a mutable borrowed reference to the CameraInstance
    pub fn camera_mut(&self) -> RefMut<CameraInstance> {
        self.camera.borrow_mut()
    }

    //ap pms
    pub fn pms(&self) -> &Rrc<PointMappingSet> {
        &self.pms
    }

    //mp pms_ref
    /// Get a borrowed reference to the PointMappingSet
    pub fn pms_ref(&self) -> Ref<PointMappingSet> {
        self.pms.borrow()
    }

    //mp pms_mut
    /// Get a mutable borrowed reference to the PointMappingSet
    pub fn pms_mut(&self) -> RefMut<PointMappingSet> {
        self.pms.borrow_mut()
    }
}

//a Project
//tp ProjectFileDesc
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectFileDesc {
    cdb: String,
    nps: Vec<String>,
    cips: Vec<CipFileDesc>,
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
    cdb: Rrc<CameraDatabase>,
    nps: Rrc<NamedPointSet>,
    cips: Vec<Cip>,
}

//ip Deserialize for Project
impl<'de> Deserialize<'de> for Project {
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        let project_desc = <ProjectDesc>::deserialize(deserializer)?;
        let cdb = project_desc.cdb.into();
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
    //ap desc
    pub fn desc(&self) -> &ProjectFileDesc {
        &self.desc
    }

    //ap cdb
    pub fn cdb(&self) -> &Rrc<CameraDatabase> {
        &self.cdb
    }

    //mp set_cdb
    pub fn set_cdb(&mut self, cdb: Rrc<CameraDatabase>) {
        self.cdb = cdb;
    }
    pub fn cdb_ref(&self) -> Ref<CameraDatabase> {
        self.cdb.borrow()
    }

    //ap nps
    pub fn nps(&self) -> &Rrc<NamedPointSet> {
        &self.nps
    }

    //mp set_nps
    pub fn set_nps(&mut self, nps: Rrc<NamedPointSet>) {
        self.nps = nps;
    }

    //mp nps_ref
    /// Get a borrowed reference to the NamedPointSet
    pub fn nps_ref(&self) -> Ref<NamedPointSet> {
        self.nps.borrow()
    }

    //mp nps_mut
    /// Get a mutable borrowed reference to the NamedPointSet
    pub fn nps_mut(&self) -> RefMut<NamedPointSet> {
        self.nps.borrow_mut()
    }

    //ap cip
    pub fn cip(&self, n: usize) -> &Cip {
        &self.cips[n]
    }

    //ap cip_mut
    pub fn cip_mut(&mut self, n: usize) -> &mut Cip {
        &mut self.cips[n]
    }
    //cp from_desc_json
    /// Create from a ProjectFileDesc
    pub fn from_desc_json(
        desc: ProjectFileDesc,
        cdb_json: &str,
        nps_json: &str,
    ) -> Result<Self, String> {
        let cdb: CameraDatabase = json::from_json("camera database", cdb_json)?;
        let nps: NamedPointSet = json::from_json("named point set", nps_json)?;
        let cdb = cdb.into();
        let nps = nps.into();
        let cips = vec![];
        Ok(Self {
            desc,
            cdb,
            nps,
            cips,
        })
    }

    //mp add_cip
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

    // pub fn to_json(&self) -> Result<String, String> {
    // serde_json::to_string(self).map_err(|e| format!("{}", e))
    // }
}