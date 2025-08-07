//a Imports

use star_catalog::Catalog;
use thunderclap::CommandArgs;

use ic_base::{PathSet, Ray, Rrc};
use ic_camera::CameraInstance;
use ic_camera::{CalibrationMapping, CameraDatabase};
use ic_image::Color;
use ic_mapping::{NamedPointSet, PointMappingSet};
use ic_project::{Cip, Project};
use ic_stars::StarMapping;

//a CmdResult
pub type CmdResult = std::result::Result<String, ic_base::Error>;
pub fn cmd_ok() -> CmdResult {
    Ok("".into())
}

//a CmdArgs
//tp CmdArgs
#[derive(Default)]
pub struct CmdArgs {
    pub(crate) verbose: bool,

    pub(crate) path_set: PathSet,

    pub(crate) project: Project,

    // Camera database that is part of the project
    pub(crate) cdb: Rrc<CameraDatabase>,

    // nps that is part of the project
    pub(crate) nps: Rrc<NamedPointSet>,

    // pms that is part of the project
    pub(crate) pms: Rrc<PointMappingSet>,

    // CIP that is part of the project
    pub(crate) cip: Rrc<Cip>,

    // camera is a *specific* camera, not part of a CIP or project
    pub(crate) camera: CameraInstance,

    pub(crate) calibration_mapping: CalibrationMapping,

    pub(crate) star_catalog: Option<Box<Catalog>>,
    pub(crate) star_mapping: StarMapping,

    pub(crate) read_img: Vec<String>,

    pub(crate) write_project: Option<String>,
    pub(crate) write_named_points: Option<String>,
    pub(crate) write_point_mapping: Option<String>,
    pub(crate) write_camera: Option<String>,
    pub(crate) write_calibration_mapping: Option<String>,
    pub(crate) write_star_mapping: Option<String>,
    pub(crate) write_polys: Option<String>,
    pub(crate) write_img: Option<String>,
    pub(crate) write_svg: Option<String>,

    pub(crate) bg_color: Option<Color>,
    pub(crate) pms_color: Option<Color>,
    pub(crate) model_color: Option<Color>,

    pub(crate) np: Vec<String>, // could be name, 3D, pixel XY (from camera mapping of 3D); might need at least 3

    pub(crate) named_rays: Vec<(String, Ray)>,

    pub(crate) kernels: Vec<String>,
    pub(crate) kernel_size: usize,
    pub(crate) scale: f64,
    pub(crate) angle: f64,
    pub(crate) px: usize,
    pub(crate) py: usize,
    pub(crate) flags: usize,
    pub(crate) use_deltas: bool,
    pub(crate) use_pts: usize,
    pub(crate) yaw_min: f64,
    pub(crate) yaw_max: f64,
    pub(crate) poly_degree: usize,
    pub(crate) triangle_closeness: f64,
    pub(crate) closeness: f64,
    pub(crate) yaw_error: f64,
    pub(crate) within: f64,
    pub(crate) brightness: f32,
}

//ip CommandArgs for CmdArgs
impl CommandArgs for CmdArgs {
    type Error = ic_base::Error;
    type Value = String;

    fn reset_args(&mut self) {
        self.nps = self.project.nps().clone();
        self.cdb = self.project.cdb().clone();
        self.read_img = vec![];
        self.np = vec![];

        self.write_project = None;
        self.write_named_points = None;
        self.write_point_mapping = None;
        self.write_camera = None;
        self.write_img = None;
        self.write_calibration_mapping = None;
        self.write_star_mapping = None;
        self.write_polys = None;
        self.write_svg = None;

        self.use_pts = 0;
        self.use_deltas = false;
        self.flags = 0;
        self.scale = 1.0;
        self.angle = 0.0;
        if let Some(catalog) = &mut self.star_catalog {
            catalog.clear_filter();
        }
    }
}
