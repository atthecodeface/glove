//a Imports

use thunderclap::{CmdProperty, CommandArgs};

use ic_base::{Error, NamedRayList};

use crate::{CmdArgs, CmdResult};

impl CommandArgs for CmdArgs {
    type Error = Error;
    type Value = String;
    const PROPERTIES: &[CmdProperty<'static, Self, Self::Value, Self::Error>] = &[
        CmdProperty {
            name: "camera",
            get_fn: &|cmd_args| cmd_args.camera.to_json(false).ok(),
            set_value_fn: &|cmd_args, s| cmd_args.set_camera_json(s).map(|_| true),
        },
        CmdProperty {
            name: "cip",
            get_fn: &|cmd_args| {
                Some(
                    cmd_args
                        .cip
                        .as_ref()
                        .map(|c| c.borrow().image().to_string())
                        .unwrap_or_default(),
                )
            },
            set_value_fn: &|mut _cmd_args, s| {
                Err(format!("Failed to set key 'cip' to '{s}'").into())
            },
        },
        CmdProperty {
            name: "cip.image_filename",
            get_fn: &|cmd_args| {
                cmd_args
                    .cip
                    .as_ref()
                    .map(|c| c.borrow().image_filename().to_string())
            },
            set_value_fn: &|mut _cmd_args, s| {
                Err(format!("Failed to set key 'cip.image_filename' to '{s}'").into())
            },
        },
        CmdProperty {
            name: "cip.camera",
            get_fn: &|cmd_args| {
                cmd_args
                    .cip
                    .as_ref()
                    .and_then(|c| c.borrow().camera().borrow().to_json(false).ok())
            },
            set_value_fn: &|mut _cmd_args, s| {
                Err(format!("Failed to set key 'cip.camera' to '{s}'").into())
            },
        },
        CmdProperty {
            name: "cip.point_mapping_set",
            get_fn: &|cmd_args| {
                cmd_args
                    .cip
                    .as_ref()
                    .map(|c| c.borrow().pms_filename().to_owned())
            },
            set_value_fn: &|mut _cmd_args, s| {
                Err(format!("Failed to set key 'cip.point_mapping_set' to '{s}'").into())
            },
        },
        CmdProperty {
            name: "point_mapping_set",
            get_fn: &|cmd_args| cmd_args.pms.borrow().to_json(false).ok(),
            set_value_fn: &|mut _cmd_args, s| {
                Err(format!("Failed to set key 'point_mapping_set' to '{s}'").into())
            },
        },
        CmdProperty {
            name: "calibration_mapping",
            get_fn: &|cmd_args| cmd_args.calibration_mapping.to_json(false).ok(),
            set_value_fn: &|mut _cmd_args, s| {
                Err(format!("Failed to set key 'calibration_mapping' to '{s}'").into())
            },
        },
        CmdProperty {
            name: "star_mapping",
            get_fn: &|cmd_args| cmd_args.star_mapping.to_json(false).ok(),
            set_value_fn: &|mut _cmd_args, s| {
                Err(format!("Failed to set key 'star_mapping' to '{s}'").into())
            },
        },
        CmdProperty {
            name: "brightness",
            get_fn: &|cmd_args| Some(cmd_args.brightness.to_string()),
            set_value_fn: &|cmd_args, s| {
                s.parse::<f32>()
                    .map_err(|e| e.to_string().into())
                    .and_then(|v| cmd_args.set_brightness(v))
                    .map(|_| true)
            },
        },
        CmdProperty {
            name: "closeness",
            get_fn: &|cmd_args| Some(cmd_args.closeness.to_string()),
            set_value_fn: &|cmd_args, s| {
                s.parse::<f64>()
                    .map_err(|e| e.to_string().into())
                    .and_then(|v| cmd_args.set_closeness(v))
                    .map(|_| true)
            },
        },
        CmdProperty {
            name: "poly_degree",
            get_fn: &|cmd_args| Some(cmd_args.poly_degree.to_string()),
            set_value_fn: &|cmd_args, s| {
                s.parse::<usize>()
                    .map_err(|e| e.to_string().into())
                    .and_then(|v| cmd_args.set_poly_degree(v))
                    .map(|_| true)
            },
        },
        CmdProperty {
            name: "triangle_closeness",
            get_fn: &|cmd_args| Some(cmd_args.triangle_closeness.to_string()),
            set_value_fn: &|cmd_args, s| {
                s.parse::<f64>()
                    .map_err(|e| e.to_string().into())
                    .and_then(|v| cmd_args.set_triangle_closeness(v))
                    .map(|_| true)
            },
        },
        CmdProperty {
            name: "within",
            get_fn: &|cmd_args| Some(cmd_args.within.to_string()),
            set_value_fn: &|cmd_args, s| {
                s.parse::<f64>()
                    .map_err(|e| e.to_string().into())
                    .and_then(|v| cmd_args.set_within(v))
                    .map(|_| true)
            },
        },
        CmdProperty {
            name: "yaw_error",
            get_fn: &|cmd_args| Some(cmd_args.yaw_error.to_string()),
            set_value_fn: &|cmd_args, s| {
                s.parse::<f64>()
                    .map_err(|e| e.to_string().into())
                    .and_then(|v| cmd_args.set_yaw_error(v))
                    .map(|_| true)
            },
        },
        CmdProperty {
            name: "yaw_min",
            get_fn: &|cmd_args| Some(cmd_args.yaw_min.to_string()),
            set_value_fn: &|cmd_args, s| {
                s.parse::<f64>()
                    .map_err(|e| e.to_string().into())
                    .and_then(|v| cmd_args.set_yaw_min(v))
                    .map(|_| true)
            },
        },
        CmdProperty {
            name: "yaw_max",
            get_fn: &|cmd_args| Some(cmd_args.yaw_max.to_string()),
            set_value_fn: &|cmd_args, s| {
                s.parse::<f64>()
                    .map_err(|e| e.to_string().into())
                    .and_then(|v| cmd_args.set_yaw_max(v))
                    .map(|_| true)
            },
        },
    ];

    fn cmd_ok() -> CmdResult {
        Ok("".into())
    }

    fn value_from_str(s: &str) -> Result<Self::Value, Self::Error> {
        Ok(s.into())
    }

    fn reset_args(&mut self) {
        self.nps = self.project.nps().clone();
        self.cdb = self.project.cdb().clone();

        self.read_img = vec![];
        self.np = vec![];
        self.kernels = vec![];
        self.arg_strings = vec![];
        self.arg_f64s = vec![];
        self.arg_usizes = vec![];
        self.named_rays = NamedRayList::default();

        self.write_project = None;
        self.write_named_points = None;
        self.write_point_mapping = None;
        self.write_camera = None;
        self.write_img = None;
        self.write_calibration_mapping = None;
        self.write_star_mapping = None;
        self.write_polys = None;
        self.write_svg = None;

        self.max_pairs = 0;
        self.max_points = 0;
        self.max_error = 0.0;
        self.use_pts = 0;
        self.use_deltas = false;
        self.from_camera = false;
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
}
