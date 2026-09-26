//a To do
//
// Add named patches
//
// Use tags in pms

//a Imports
use std::cell::{Ref, RefMut};
use std::rc::Rc;

use geo_nd::Quaternion;
use ic_image::{Color8, FromPatchFn, ImageDrawable, ImageRgb8};
use serde::{Deserialize, Serialize};

use ic_base::{JsonParsable, PathSet, Point3D, Quat, Ray, Result, Rrc, TagSet, TanXTanY};
use ic_camera::{
    AdjustableCameraProjection, CameraDatabase, CameraLensProjection, CameraProjection,
    RectilinearLens, SimpleSensorLensCamera,
};
use ic_mapping::{NamedPoint, NamedPointSet, PointMapping};

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

    /// Derive the location of a specific NamedPoint from rays using equal weighting
    pub fn derive_np_location(&self, name: &str) -> Option<(Point3D, f64)> {
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
            if let Some(pt) = Ray::closest_point(rays.iter(), &|_r, _n| 1.0) {
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

    pub fn create_np_cip_image(
        &self,
        np: &NamedPoint,
        cip: &Cip,
        cip_image: &ImageRgb8,
        width: u32,
        height: u32,
    ) -> bool {
        let np_images = &self.np_images;
        let blend = 0.0; // replace completely
        let Some(isq) = np_images
            .borrow_mut()
            .np_find_or_add_cip(np, cip, width, height)
        else {
            eprintln!("Failed to find or add NP/CIP");
            return false;
        };
        let pms = cip.pms().borrow();
        let Some(pm) = pms.mapping_of_np_name(np.ref_tag().as_str()) else {
            return false;
        };

        let mut patch = isq.as_patch(blend);
        let camera = cip.camera().borrow();

        let mut pci = ProjectedCameraImage {
            image: cip_image,
            camera: &*camera,
            w: cip_image.size().0,
            h: cip_image.size().1,
        };

        // This impacts the
        let mm_distance_to_point = 1000.0;
        let tan_hfov = np.facet_tan_hfov(mm_distance_to_point);

        // tan_hfov is mm_width/2 compared to mm_focal_length, if focused at infinity

        let mm_focal_length = 50.0;
        let mm_per_pixel = tan_hfov * mm_focal_length * 2.0 / (width as f64);
        eprintln!("{mm_per_pixel} fov {}", tan_hfov.atan() * 2.0);

        let mut sslc = SimpleSensorLensCamera::new(
            width,
            height,
            mm_per_pixel,
            RectilinearLens::default(),
            mm_focal_length,
        );

        // Set camera to be at a position and orientation for the given *named point*
        np.set_camera_for_facet(&mut sslc, mm_distance_to_point);

        // Adjust camera orientation so that the pm.screen is the centre (at [0,0,-1])
        let pm_world_dir = camera.sensor_px_abs_xy_to_world_dir(pm.screen());
        let sslc_pm_camera_dir = sslc.world_dir_to_camera_dir(pm_world_dir);
        let center_on_pm = Quat::rotation_of_vec_to_vec(&sslc_pm_camera_dir, &[0., 0., -1.]);
        sslc.set_orientation(&(center_on_pm * sslc.orientation()));

        // This uses distance to camera from model, so uses model position; this has to happen after set_camera_for_facet, OR use mm_distance_to_point...
        let mut patch_iterator = PatchIterator {
            camera: &sslc,
            projected_image: &mut pci,
        };
        patch.fill_img(&mut patch_iterator);
        true
    }
}

struct PatchIterator<'a, C1, C2>
where
    C1: CameraLensProjection,
    C2: CameraImageProjection,
{
    camera: &'a C1,
    projected_image: &'a mut C2,
}

impl<'a, C1, C2> FromPatchFn for PatchIterator<'a, C1, C2>
where
    C1: CameraLensProjection,
    C2: CameraImageProjection,
{
    type Pixel = C2::Pixel;
    fn set_mapping(&mut self, _patch_x: u32, _patch_y: u32) {}
    fn map_from_patch(&mut self, patch_x: u32, patch_y: u32) -> Option<Self::Pixel> {
        let world_dir = self
            .camera
            .sensor_px_abs_xy_to_world_dir([patch_x as f64, patch_y as f64].into());
        let camera_dir = self.projected_image.world_dir_to_camera_dir(world_dir);
        self.projected_image.opt_pixel_of_camera_dir(camera_dir)
    }
}

#[derive(Debug, Clone)]
struct ProjectedCameraImage<'a, C, I>
where
    C: CameraLensProjection,
    I: ImageDrawable,
{
    image: &'a I,
    camera: &'a C,
    w: u32,
    h: u32,
}
pub trait CameraImageProjection {
    type Pixel;
    /// Invoked occassionally (at the start of a line, for example, when
    /// filling a square) to indicate the next pixel fetch is unrelated to
    /// the last
    fn set_mapping_to_camera_dir(&mut self, _dirn: Point3D) {}

    /// Return the pixel value of the given camera direction (vector is
    /// *outward* from the camera) if it hits the sensor/image
    fn opt_pixel_of_camera_dir(&mut self, camera_dir: Point3D) -> Option<Self::Pixel>;

    /// Map the world direction to a camera direction
    fn world_dir_to_camera_dir(&self, world_dir: Point3D) -> Point3D;
}

impl<'a, C, I> CameraImageProjection for ProjectedCameraImage<'a, C, I>
where
    C: CameraLensProjection,
    I: ImageDrawable,
{
    type Pixel = I::Pixel;
    fn set_mapping_to_camera_dir(&mut self, _dirn: Point3D) {}
    fn opt_pixel_of_camera_dir(&mut self, dirn: Point3D) -> Option<I::Pixel> {
        let Some(pxy) = self.camera.camera_dir_to_opt_sensor_px_abs_xy(dirn) else {
            return None;
        };
        if pxy[0] < 0.0 || pxy[1] < 0.0 {
            return None;
        }
        if (pxy[0] >= self.w as f64) || (pxy[1] >= self.h as f64) {
            return None;
        }
        Some(self.image.get(pxy[0] as u32, pxy[1] as u32))
    }
    fn world_dir_to_camera_dir(&self, world_dir: Point3D) -> Point3D {
        self.camera.world_dir_to_camera_dir(world_dir)
    }
}
