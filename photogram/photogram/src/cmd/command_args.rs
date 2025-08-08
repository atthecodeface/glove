//a Imports

use thunderclap::CommandArgs;

use ic_base::Error;

use crate::{CmdArgs, CmdResult};

//ip CommandArgs for CmdArgs
const KEY_GET_SET_FNS: &[(
    &str,
    &dyn Fn(&CmdArgs) -> Option<String>,
    &dyn Fn(&mut CmdArgs, &str) -> Result<bool, Error>,
)] = &[
    (
        "camera",
        &|cmd_args| cmd_args.camera.to_json(false).ok(),
        &|mut cmd_args, s| cmd_args.set_camera_json(s).map(|_| true),
    ),
    (
        "cip.image",
        &|cmd_args| Some(cmd_args.cip.borrow().image().to_owned()),
        &|mut _cmd_args, s| Err(format!("Failed to set key 'cip.image' to '{s}'").into()),
    ),
    (
        "cip.camera",
        &|cmd_args| cmd_args.cip.borrow().camera().borrow().to_json(false).ok(),
        &|mut _cmd_args, s| Err(format!("Failed to set key 'cip.camera' to '{s}'").into()),
    ),
    (
        "cip.point_mappingset",
        &|cmd_args| Some(cmd_args.cip.borrow().pms_file().to_owned()),
        &|mut _cmd_args, s| Err(format!("Failed to set key 'cip.point_mappingset' to '{s}'").into()),
    ),
    (
        "point_mappingset",
        &|cmd_args| cmd_args.pms.borrow().to_json(false).ok(),
        &|mut _cmd_args, s| Err(format!("Failed to set key 'point_mappingset' to '{s}'").into()),
    ),
    (
        "calibration_mapping",
        &|cmd_args| cmd_args.calibration_mapping.to_json(false).ok(),
        &|mut _cmd_args, s| Err(format!("Failed to set key 'calibration_mapping' to '{s}'").into()),
    ),
    (
        "star_mapping",
        &|cmd_args| cmd_args.star_mapping.to_json(false).ok(),
        &|mut _cmd_args, s| Err(format!("Failed to set key 'star_mapping' to '{s}'").into()),
    ),
    (
        "brightness",
        &|cmd_args| Some(cmd_args.brightness.to_string()),
        &|mut _cmd_args, s| Err(format!("Failed to set key 'brightness' to '{s}'").into()),
    ),
    (
        "closeness",
        &|cmd_args| Some(cmd_args.closeness.to_string()),
        &|mut _cmd_args, s| Err(format!("Failed to set key 'closeness' to '{s}'").into()),
    ),
    (
        "poly_degree",
        &|cmd_args| Some(cmd_args.poly_degree.to_string()),
        &|mut _cmd_args, s| Err(format!("Failed to set key 'poly_degree' to '{s}'").into()),
    ),
    (
        "triangle_closeness",
        &|cmd_args| Some(cmd_args.triangle_closeness.to_string()),
        &|mut _cmd_args, s| Err(format!("Failed to set key 'triangle_closeness' to '{s}'").into()),
    ),
    (
        "within",
        &|cmd_args| Some(cmd_args.within.to_string()),
        &|mut _cmd_args, s| Err(format!("Failed to set key 'within' to '{s}'").into()),
    ),
    (
        "yaw_error",
        &|cmd_args| Some(cmd_args.yaw_error.to_string()),
        &|mut _cmd_args, s| Err(format!("Failed to set key 'yaw_error' to '{s}'").into()),
    ),
    (
        "yaw_min",
        &|cmd_args| Some(cmd_args.yaw_min.to_string()),
        &|mut _cmd_args, s| Err(format!("Failed to set key 'yaw_min' to '{s}'").into()),
    ),
    (
        "yaw_max",
        &|cmd_args| Some(cmd_args.yaw_max.to_string()),
        &|mut cmd_args, s| {
            s.parse::<f64>()
                .map_err(|e| e.to_string().into())
                .and_then(|v| cmd_args.set_yaw_max(v))
                .map(|_| true)
        },
    ),
];

impl CommandArgs for CmdArgs {
    type Error = Error;
    type Value = String;

    fn cmd_ok() -> CmdResult {
        Ok("".into())
    }

    fn reset_args(&mut self) {
        self.nps = self.project.nps().clone();
        self.cdb = self.project.cdb().clone();

        self.read_img = vec![];
        self.np = vec![];
        self.kernels = vec![];

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
        self.kernel_size = 8;
        if let Some(catalog) = &mut self.star_catalog {
            catalog.clear_filter();
        }
        self.bg_color = None;
        self.model_color = None;
        self.pms_color = None;
    }

    /// Get the keys (elements) of the arguments - used in batch and interactive only
    fn keys(&self) -> Box<dyn Iterator<Item = &str>> {
        Box::new(KEY_GET_SET_FNS.iter().map(|(k, _g, _s)| *k))
    }

    /// Retrieve the value of a key, in some form, from the arguments - used in batch and interactive only
    fn value_str(&self, key: &str) -> Option<String> {
        for (k, g, _s) in KEY_GET_SET_FNS.iter() {
            if key == *k {
                return g(self);
            }
        }
        None
    }

    /// Set the value
    fn value_set(&mut self, key: &str, value: &str) -> Result<bool, Error> {
        for (k, _g, set) in KEY_GET_SET_FNS.iter() {
            if key == *k {
                return set(self, value);
            }
        }
        Ok(false)
    }
}
