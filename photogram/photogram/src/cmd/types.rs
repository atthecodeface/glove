//a Imports

use star_catalog::Catalog;

use ic_photogram::CameraInstance;
use ic_photogram::Cylinder;
use ic_photogram::{CalibrationMapping, CameraDatabase};
use ic_photogram::{Cip, Project};
use ic_photogram::{Color8, ImageRgb8};
use ic_photogram::{NamedPointSet, PointMappingSet};
use ic_photogram::{NamedRayList, PathSet, Point2D, Point3D, Rrc};
use ic_photogram::{SphericalImage, SphericalImageShape};

//a CmdResult
use thunderclap::json;
pub type CmdResult = std::result::Result<json::Value, anyhow::Error>;

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
    // Lose this
    pub(crate) pms: Rrc<PointMappingSet>,
    // Lose this
    pub(crate) calibration_mapping: CalibrationMapping,

    // CIP that is part of the project
    pub(crate) cip: Option<Rrc<Cip>>,

    // camera is a *specific* camera, not part of a CIP or project
    pub(crate) camera: CameraInstance,

    pub(crate) star_catalog: Option<Box<Catalog>>,

    pub(crate) px: usize,
    pub(crate) py: usize,
    pub(crate) yaw_min: f64,
    pub(crate) yaw_max: f64,
    pub(crate) yaw_error: f64,
    pub(crate) triangle_closeness: f64,
    pub(crate) closeness: f64,
    pub(crate) within: f64,
    pub(crate) brightness: f32,

    // Lose this
    pub(crate) poly_degree: usize,

    pub(crate) pretty_json: bool,

    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) kernel_size: usize,
    pub(crate) scale: f64,
    pub(crate) angle: f64,
    pub(crate) flags: usize,
    pub(crate) from_camera: bool,
    pub(crate) fov_h: f64,
    pub(crate) fov_v: f64,
    pub(crate) h_ofs: f64,
    pub(crate) v_ofs: f64,
    pub(crate) x_grid: f64,
    pub(crate) y_grid: f64,
    pub(crate) patch_size: u32,
    pub(crate) use_deltas: bool,
    pub(crate) use_pts: usize,
    pub(crate) max_error: f64,
    pub(crate) max_points: usize,
    pub(crate) max_pairs: usize,
    pub(crate) steps: usize,
    pub(crate) range: f64,
    pub(crate) cylindrical_projection: Cylinder,

    pub(crate) shape: SphericalImageShape,
    pub(crate) spherical_image: Option<Rrc<SphericalImage<ImageRgb8>>>,
    pub(crate) blend: f64,

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
    pub(crate) render_vertical: bool,

    pub(crate) xy: Vec<Point2D>,
    pub(crate) xyz: Vec<Point3D>,

    // Positional string / f64 / usize arguments
    pub(crate) arg_strings: Vec<String>,
    pub(crate) arg_f64s: Vec<f64>,
    pub(crate) arg_usizes: Vec<usize>,

    pub(crate) bg_color: Option<Color8>,
    pub(crate) pms_color: Option<Color8>,
    pub(crate) model_color: Option<Color8>,

    pub(crate) named_rays: NamedRayList,
}
