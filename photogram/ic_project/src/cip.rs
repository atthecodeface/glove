//a Imports
use std::cell::{Ref, RefMut};

use serde::{Deserialize, Serialize};

use ic_base::Rrc;
use ic_camera::{CameraInstance, CameraInstanceDesc};
use ic_mapping::{CameraAdjustMapping, PointMappingSet};

use crate::Project;

//a Cip
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
#[derive(Debug, Default, Serialize)]
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

    //cp read_json
    pub fn read_json(
        &mut self,
        project: &Project,
        camera_json: &str,
        pms_json: &str,
    ) -> Result<String, String> {
        let camera = CameraInstance::from_json(&project.cdb().borrow(), camera_json)?;
        let (pms, warnings) = PointMappingSet::from_json(&project.nps().borrow(), pms_json)?;
        self.camera = camera.into();
        self.pms = pms.into();
        Ok(warnings)
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

    //mp locate
    pub fn locate(&self, max_np_error: f64) {
        self.camera
            .borrow_mut()
            .locate_using_model_lines(&self.pms_ref(), max_np_error);
    }

    //zz all done
}
