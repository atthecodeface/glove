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

use crate::{Cip, CipDesc, CipFileDesc, ImageSquareSets, ImageSquareSetsDesc, NamedPointImages};

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
    #[serde(skip)]
    np_images: Rrc<NamedPointImages>,
}

impl<'de> Deserialize<'de> for Project {
    fn deserialize<DE>(deserializer: DE) -> std::result::Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        let mut project_desc = <ProjectDesc>::deserialize(deserializer)?;
        project_desc.cdb.derive();

        let mut project = Self::default();
        let cdb = project_desc.cdb;
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

impl Project {
    pub fn cdb(&self) -> &Rrc<CameraDatabase> {
        &self.cdb
    }

    pub fn cdb_filename(&self) -> &str {
        &self.cdb_filename
    }

    pub fn nps(&self) -> &Rrc<NamedPointSet> {
        &self.nps
    }

    pub fn nps_filename(&self) -> &str {
        &self.nps_filename
    }

    pub fn set_cdb_filename<S: Into<String>>(&mut self, cdb_filename: S) {
        self.cdb_filename = cdb_filename.into();
    }

    pub fn set_nps_filename<S: Into<String>>(&mut self, nps_filename: S) {
        self.nps_filename = nps_filename.into();
    }

    pub fn cdb_ref(&self) -> Ref<'_, CameraDatabase> {
        self.cdb.borrow()
    }

    /// Get a borrowed reference to the NamedPointSet
    pub fn nps_ref(&self) -> Ref<'_, NamedPointSet> {
        self.nps.borrow()
    }

    pub fn np_images_mut(&self) -> RefMut<'_, NamedPointImages> {
        self.np_images.borrow_mut()
    }

    pub fn np_images_ref(&self) -> Ref<'_, NamedPointImages> {
        self.np_images.borrow()
    }

    /// Get a mutable borrowed reference to the NamedPointSet
    pub fn nps_mut(&self) -> RefMut<'_, NamedPointSet> {
        self.nps.borrow_mut()
    }

    /// Get a borrowed reference to the ImageSquareSets
    pub fn isqs_ref(&self) -> Ref<'_, ImageSquareSets> {
        self.image_squares.borrow()
    }

    /// Get a mutable borrowed reference to the ImageSquareSets
    pub fn isqs_mut(&self) -> RefMut<'_, ImageSquareSets> {
        self.image_squares.borrow_mut()
    }

    pub fn ncips(&self) -> usize {
        self.cips.len()
    }

    pub fn cip_name(&self, n: usize) -> Option<String> {
        self.cips
            .get(n)
            .map(|c| c.borrow().name_as_tag().to_string())
    }

    pub fn find_cip<A: AsRef<str>>(&self, name: A) -> Option<&Rrc<Cip>> {
        self.cips
            .iter()
            .find(|&c| c.borrow().name_as_tag().as_str() == name.as_ref())
    }

    pub fn add_cip(&mut self, cip: Rrc<Cip>) {
        cip.borrow_mut().resolve_name(&self.image_tag_set);
        self.cips.push(cip);
    }

    #[track_caller]
    pub fn set_cdb(&self, cdb: CameraDatabase) {
        assert_eq!(
            self.ncips(),
            0,
            "Project must have no CIPS to set the camera database"
        );
        *self.cdb.borrow_mut() = cdb;
    }

    /// Set the NamedPointSet for the [Project]
    #[track_caller]
    pub fn set_nps(&mut self, nps: Rrc<NamedPointSet>) {
        assert_eq!(
            self.ncips(),
            0,
            "Project must have no CIPS to *set* the NPS"
        );
        eprintln!("{self:?}, {nps:?}");
        nps.borrow_mut().set_tag_set(self.np_tag_set.clone());
        self.nps = nps;
    }

    pub fn to_json(&self, pretty: bool) -> Result<String> {
        if pretty {
            Ok(serde_json::to_string_pretty(self)?)
        } else {
            Ok(serde_json::to_string(self)?)
        }
    }

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

    pub fn derive_nps_location(&self, name: &str) -> Option<(Point3D, f64)> {
        let mut rays = vec![];
        for cip in &self.cips {
            let cip = cip.borrow();
            for m in cip.pms_ref().mappings() {
                if m.named_point().has_name(name) {
                    rays.push(m.get_mapped_ray(&*cip.camera_ref(), true));
                }
            }
        }
        if rays.len() > 1 {
            if let Some(pt) = Ray::closest_point(rays.iter(), &|_r| 1.0) {
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
}
