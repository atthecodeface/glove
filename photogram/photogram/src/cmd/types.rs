//a Imports

use star_catalog::Catalog;

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
    pub(crate) cip_number: usize,

    // camera is a *specific* camera, not part of a CIP or project
    pub(crate) camera: CameraInstance,

    pub(crate) calibration_mapping: CalibrationMapping,

    pub(crate) star_catalog: Option<Box<Catalog>>,
    pub(crate) star_mapping: StarMapping,

    pub(crate) px: usize,
    pub(crate) py: usize,
    pub(crate) yaw_min: f64,
    pub(crate) yaw_max: f64,
    pub(crate) yaw_error: f64,
    pub(crate) poly_degree: usize,
    pub(crate) triangle_closeness: f64,
    pub(crate) closeness: f64,
    pub(crate) within: f64,
    pub(crate) brightness: f32,

    pub(crate) pretty_json: bool,

    // Items clear during reset
    pub(crate) read_img: Vec<String>,
    pub(crate) np: Vec<String>, // could be name, 3D, pixel XY (from camera mapping of 3D); might need at least 3
    pub(crate) kernels: Vec<String>,
    pub(crate) write_project: Option<String>,
    pub(crate) write_named_points: Option<String>,
    pub(crate) write_point_mapping: Option<String>,
    pub(crate) write_camera: Option<String>,
    pub(crate) write_calibration_mapping: Option<String>,
    pub(crate) write_star_mapping: Option<String>,
    pub(crate) write_polys: Option<String>,
    pub(crate) write_img: Option<String>,
    pub(crate) write_svg: Option<String>,

    // Positional string / f64 / usize arguments
    pub(crate) arg_strings: Vec<String>,
    pub(crate) arg_f64s: Vec<f64>,
    pub(crate) arg_usizes: Vec<usize>,

    pub(crate) bg_color: Option<Color>,
    pub(crate) pms_color: Option<Color>,
    pub(crate) model_color: Option<Color>,

    pub(crate) named_rays: Vec<(String, Ray)>,

    pub(crate) kernel_size: usize,
    pub(crate) scale: f64,
    pub(crate) angle: f64,
    pub(crate) flags: usize,
    pub(crate) use_deltas: bool,
    pub(crate) use_pts: usize,
}
