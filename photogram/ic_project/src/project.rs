//a To do
//
// Add named patches
//
// Use tags in pms

//a Imports
use std::cell::{Ref, RefMut};
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use ic_base::{JsonParsable, PathSet, Point3D, Ray, Result, Rrc, TagSet};
use ic_camera::CameraDatabase;
use ic_mapping::{NamedPointSet, PointMapping};

use crate::{Cip, CipDesc, CipFileDesc, ImageSquareSets, ImageSquareSetsDesc};

//a ProjectFileDesc
//tp ProjectFileDesc
/// A project description is a deserializable that can be stored in a
/// JSON file
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProjectFileDesc {
    cdb: String,
    nps: String,
    /// A list of (camera filename, image filename, point mapping set filename)
    cips: Vec<CipFileDesc>,
    /// File containing the patch sets
    #[serde(default)]
    patches: String,
    /// Files containing image squares
    #[serde(default)]
    image_squares: ImageSquareSetsDesc,
}

//ip ProjectFileDesc
impl ProjectFileDesc {
    //mp to_json
    pub fn to_json(&self, pretty: bool) -> Result<String> {
        if pretty {
            Ok(serde_json::to_string_pretty(self)?)
        } else {
            Ok(serde_json::to_string(self)?)
        }
    }

    //mp load_project
    pub fn load_project(&self, path_set: &PathSet) -> Result<Project> {
        let mut project = Project::default();

        let (cdb_filename, cdb) = CameraDatabase::load_json_file(path_set, &self.cdb, &())?;

        project.set_cdb(cdb);
        project.set_cdb_filename(cdb_filename);

        let (nps_filename, nps) = NamedPointSet::load_json_file(path_set, &self.nps, &())?;

        project.set_nps(Rrc::new(nps));
        project.set_nps_filename(nps_filename);
        for cip in &self.cips {
            let cip = Rrc::new(cip.load_cip(path_set, &project)?);
            project.add_cip(cip);
        }
        // project.set_patches(Rrc::new(path_set.load_from_json_file("patches", &self.patches)?,
        //));
        Ok(project)
    }
}

//ip JsonParsable for ProjectFileDesc
impl JsonParsable for ProjectFileDesc {
    fn reason() -> &'static str {
        "project file descriptor"
    }
    type PostParseArg = ();
    type PostParseResult = Self;
    fn post_parse(self, _: &()) -> Result<Self> {
        Ok(self)
    }
}

//a ProjectDesc
//tp ProjectDesc
/// A project description is a deserializable that can be stored in a
/// JSON file
#[derive(Debug, Default, Serialize, Deserialize)]
struct ProjectDesc {
    cdb: CameraDatabase,
    nps: Rrc<NamedPointSet>,
    /// A list of CameraInstanceDesc and PointMappingSet, and an image name
    cips: Vec<CipDesc>,
    #[serde(default)]
    cdb_filename: String,
    #[serde(default)]
    nps_filename: String,
    /// Files containing image squares
    #[serde(default)]
    image_squares: ImageSquareSetsDesc,
}

//a Project
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
    #[serde(default)]
    cdb_filename: String,
    #[serde(default)]
    nps_filename: String,
    image_squares: Rrc<ImageSquareSets>,
    #[serde(skip)]
    np_tag_set: Rc<TagSet>,
    #[serde(skip)]
    image_tag_set: Rc<TagSet>,
}

//ip JsonParsable for Project
impl JsonParsable for Project {
    fn reason() -> &'static str {
        "project"
    }
    type PostParseArg = ();
    type PostParseResult = Self;
    fn post_parse(self, _: &()) -> Result<Self> {
        // All the hard work is in deserialize
        Ok(self)
    }
}

//ip Deserialize for Project
impl<'de> Deserialize<'de> for Project {
    fn deserialize<DE>(deserializer: DE) -> std::result::Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        let mut project_desc = <ProjectDesc>::deserialize(deserializer)?;
        project_desc.cdb.derive();

        let mut project = Self::default();
        let cdb = project_desc.cdb.into();
        project.set_cdb_filename(project_desc.cdb_filename);
        project.set_cdb(cdb);
        let nps = project_desc.nps;
        project.set_nps(nps);
        project.set_nps_filename(project_desc.nps_filename);

        let image_squares = ImageSquareSets::from_desc(project_desc.image_squares);
        project.image_squares = image_squares.into();

        for cip_desc in project_desc.cips {
            use serde::de::Error;
            let (cip, warnings) = Cip::from_desc(&project, cip_desc)
                .map_err(|e| DE::Error::custom(format!("bad CIP desc: {e}")))?;
            if !warnings.is_empty() {
                eprintln!("Warning loading project: {warnings}");
            }
            project.add_cip(cip.into());
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

    //ap cdb_filename
    pub fn cdb_filename(&self) -> &str {
        &self.cdb_filename
    }

    //ap nps
    pub fn nps(&self) -> &Rrc<NamedPointSet> {
        &self.nps
    }

    //ap nps_filename
    pub fn nps_filename(&self) -> &str {
        &self.nps_filename
    }

    //mp set_cdb_filename
    pub fn set_cdb_filename<S: Into<String>>(&mut self, cdb_filename: S) {
        self.cdb_filename = cdb_filename.into();
    }

    //mp set_nps_filename
    pub fn set_nps_filename<S: Into<String>>(&mut self, nps_filename: S) {
        self.nps_filename = nps_filename.into();
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

    //ap isqs_ref
    /// Get a borrowed reference to the ImageSquareSets
    pub fn isqs_ref(&self) -> Ref<ImageSquareSets> {
        self.image_squares.borrow()
    }

    //ap isqs_mut
    /// Get a mutable borrowed reference to the ImageSquareSets
    pub fn isqs_mut(&self) -> RefMut<ImageSquareSets> {
        self.image_squares.borrow_mut()
    }

    //ap ncips
    pub fn ncips(&self) -> usize {
        self.cips.len()
    }

    //ap cip
    pub fn cip(&self, n: usize) -> &Rrc<Cip> {
        &self.cips[n]
    }

    //mp set_cdb
    #[track_caller]
    pub fn set_cdb(&self, cdb: CameraDatabase) {
        assert_eq!(
            self.ncips(),
            0,
            "Project must have no CIPS to set the camera database"
        );
        *self.cdb.borrow_mut() = cdb;
    }

    //mp set_nps
    /// Set the NamedPointSet for the [Project]
    #[track_caller]
    pub fn set_nps(&mut self, nps: Rrc<NamedPointSet>) {
        assert_eq!(
            self.ncips(),
            0,
            "Project must have no CIPS to *set* the NPS"
        );
        nps.borrow_mut().set_tag_set(self.np_tag_set.clone());
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
    pub fn locate_all<F>(&self, filter: F, max_pairs: usize) -> Result<f64>
    where
        F: Clone + Fn(usize, &PointMapping) -> bool,
    {
        let mut total_error = 0.0;
        for cip in &self.cips {
            total_error += cip.borrow().locate(filter.clone(), max_pairs)?;
        }
        Ok(total_error)
    }

    //mp derive_nps_location
    pub fn derive_nps_location(&self, name: &str) -> Option<(Point3D, f64)> {
        let mut rays = vec![];
        for cip in &self.cips {
            let cip = cip.borrow();
            for m in cip.pms_ref().mappings() {
                if m.name() == name {
                    rays.push(m.get_mapped_ray(&*cip.camera_ref(), true));
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
