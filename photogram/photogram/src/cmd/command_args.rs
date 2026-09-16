use anyhow::{Error, anyhow};
use thunderclap::json;
use thunderclap::{CmdProperty, CommandArgs};

use json::Value;

use ic_photogram::NamedRayList;

use crate::{CmdArgs, CmdResult};

macro_rules! property {
    {$name:expr, $get_fn:ident, $set_fn:ident} => {
        CmdProperty {
            name: $name,
            get_fn: &|cmd_args| json::to_value(cmd_args . $get_fn ()).ok(),
            set_value_fn: &|cmd_args, s| {
                cmd_args . $set_fn (json::from_value(s.clone())?)?;
                Ok(true)
            },
        }
    }
}
impl CommandArgs for CmdArgs {
    type Error = Error;
    type Value = Value;
    const PROPERTIES: &[CmdProperty<'static, Self, Self::Value, Self::Error>] = &[
        CmdProperty {
            name: "cip",
            get_fn: &|cmd_args| {
                cmd_args
                    .cip
                    .as_ref()
                    .map(|c| json::to_value(&*c.borrow()).ok())
                    .flatten()
            },
            set_value_fn: &|mut _cmd_args, s| Err(anyhow!("Failed to set key 'cip' to '{s}'")),
        },
        CmdProperty {
            name: "cip.image_filename",
            get_fn: &|cmd_args| {
                cmd_args
                    .cip
                    .as_ref()
                    .map(|c| json::to_value(c.borrow().image_filename()).ok())
                    .flatten()
            },
            set_value_fn: &|mut _cmd_args, s| {
                Err(anyhow!("Failed to set key 'cip.image_filename' to '{s}'"))
            },
        },
        CmdProperty {
            name: "cip.camera",
            get_fn: &|cmd_args| {
                cmd_args
                    .cip
                    .as_ref()
                    .map(|c| json::to_value(&*c.borrow().camera().borrow()).ok())
                    .flatten()
            },
            set_value_fn: &|mut _cmd_args, s| {
                Err(anyhow!("Failed to set key 'cip.camera' to '{s}'"))
            },
        },
        /*
               CmdProperty {
                   name: "camera",
                   get_fn: &|cmd_args| json::to_value(cmd_args.camera).ok(),
                   set_value_fn: &|cmd_args, s| cmd_args.set_camera_json(sa).map(|_| true),
               },
        */
        property!("brightness", brightness, set_brightness),
        /*
        *
         CmdProperty {
                   name: "brightness",
                   get_fn: &|cmd_args| json::to_value(cmd_args.brightness()).ok(),
                   set_value_fn: &|cmd_args, s| {
                       cmd_args.set_brightness(json::from_value(s.clone())?)?;
                       Ok(true)
                   },
               },
               CmdProperty {
                   name: "closeness",
                   get_fn: &|cmd_args| json::to_value(cmd_args.closeness()).ok(),
                   set_value_fn: &|cmd_args, s| {
                       cmd_args.set_closeness(json::from_value(s.clone())?)?;
                       Ok(true)
                   },
               },
               CmdProperty {
                   name: "triangle_closeness",
                   get_fn: &|cmd_args| json::to_value(cmd_args.triangle_closeness()).ok(),
                   set_value_fn: &|cmd_args, s| {
                       cmd_args.set_triangle_closeness(json::from_value(s.clone())?)?;
                       Ok(true)
                   },
               },
               CmdProperty {
                   name: "within",
                   get_fn: &|cmd_args| json::to_value(cmd_args.within()).ok(),
                   set_value_fn: &|cmd_args, s| {
                       cmd_args.set_within(json::from_value(s.clone())?)?;
                       Ok(true)
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
               CmdProperty {
                   name: "orientation",
                   get_fn: &|cmd_args| serde_json::to_string(&cmd_args.camera.orientation()).ok(),
                   set_value_fn: &|cmd_args, s| {
                       QuaternionDesc::load_json(s, &()).map(|q| {
                           cmd_args.camera.set_orientation(&q);
                           true
                       })
                   },
               },
               CmdProperty {
                   name: "grid_x",
                   get_fn: &|cmd_args| serde_json::to_string(&cmd_args.x_grid).ok(),
                   set_value_fn: &|cmd_args, s| {
                       cmd_args.x_grid = JsonSrc::<f64>::load_json(s, &())?;
                       Ok(true)
                   },
               },
               CmdProperty {
                   name: "grid_y",
                   get_fn: &|cmd_args| serde_json::to_string(&cmd_args.x_grid).ok(),
                   set_value_fn: &|cmd_args, s| {
                       cmd_args.y_grid = JsonSrc::<f64>::load_json(s, &())?;
                       Ok(true)
                   },
               },
               CmdProperty {
                   name: "cylindrical",
                   get_fn: &|cmd_args| {
                       let projection_name = cmd_args.cylindrical_projection.name();
                       serde_json::to_string(projection_name).ok()
                   },
                   set_value_fn: &|cmd_args, s| {
                       cmd_args.cylindrical_projection.set_projection(s)?;
                       Ok(true)
                   },
               },
               */
    ];

    fn cmd_ok() -> CmdResult {
        Ok(json::Value::Null)
    }

    fn value_from_str(s: &str) -> Result<Self::Value, Self::Error> {
        Ok(s.into())
    }

    fn reset_args(&mut self) {
        self.nps = self.project.nps().clone();
        self.cdb = self.project.cdb().clone();

        self.read_img.clear();
        self.np.clear();
        self.kernels.clear();
        self.arg_strings.clear();
        self.arg_f64s.clear();
        self.arg_usizes.clear();
        self.xy.clear();
        self.xyz.clear();

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
