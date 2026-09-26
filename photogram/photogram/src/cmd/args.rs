use ic_photogram::SphericalImageShape;
use thunderclap::{ArgCount, ArgDescriptor};

use crate::Result;

use ic_photogram::{JsonParsable, Point2D, Point3D};

use super::CmdArgs;

impl CmdArgs {
    pub(crate) const ARG_VERBOSE: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_flag(
        "verbose",
        Some('v'),
        "Enable verbose output",
        &|s: &mut CmdArgs, v: bool| {
            s.verbose = v;
            Ok(())
        },
    );

    pub(crate) const ARG_PRETTY_JSON: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_flag(
        "pretty_json",
        None,
        "Use pretty-printing for Json output",
        &|s: &mut CmdArgs, v: bool| {
            s.pretty_json = v;
            Ok(())
        },
    );

    pub(crate) const ARG_WIDTH: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_u32(
        "width",
        None,
        "Set the width for the operation",
        ArgCount::Required,
        None,
        &|s: &mut CmdArgs, v: u32| {
            s.width = v;
            Ok(())
        },
    );

    pub(crate) const ARG_HEIGHT: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_u32(
        "height",
        None,
        "Set the height for the operation",
        ArgCount::Required,
        None,
        &|s: &mut CmdArgs, v: u32| {
            s.height = v;
            Ok(())
        },
    );

    pub(crate) const ARG_ADD_KERNEL: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "kernel",
        None,
        "Add a kernel to run; at least one must be specified",
        ArgCount::Min(1),
        None,
        &Self::add_kernel,
    );

    pub(crate) const ARG_NPS: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "nps",
        None,
        "Add a named point set to the list",
        ArgCount::Any,
        None,
        &Self::add_nps,
    );

    pub(crate) const ARG_CAMERA_DB: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "camera_db",
        None,
        "Camera database JSON filename",
        ArgCount::Optional,
        None,
        &Self::set_camera_db,
    );

    pub(crate) const ARG_PROJECT_FILE: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "project_file",
        None,
        "Complete project JSON filename",
        ArgCount::Optional,
        None,
        &Self::set_project_file,
    );

    pub(crate) const ARG_PMS: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "pms",
        None,
        "Add a point mapping set",
        ArgCount::Optional,
        None,
        &Self::add_pms,
    );

    pub(crate) const ARG_CIP: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "cip",
        None,
        "CIP name (camera and PMS) within the project",
        ArgCount::Optional,
        None,
        &Self::set_cip,
    );

    pub(crate) const ARG_CAMERA: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "camera",
        Some('c'),
        "Camera lens, placement and orientation JSON",
        ArgCount::Optional,
        None,
        &Self::set_camera_file,
    );

    pub(crate) const ARG_USE_BODY: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "use_body",
        None,
        "Specify which body to use in the camera",
        ArgCount::Optional,
        None,
        &Self::set_camera_body,
    );

    pub(crate) const ARG_USE_LENS: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "use_lens",
        None,
        "Specify which lens to use in the camera",
        ArgCount::Optional,
        None,
        &Self::set_camera_lens,
    );

    pub(crate) const ARG_USE_FOCUS: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_f64(
        "use_focus",
        None,
        "Specify the focus distance in mm used for the image, in the camera",
        ArgCount::Optional,
        None,
        &Self::set_camera_focus_distance,
    );

    pub(crate) const ARG_USE_POLYS: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "use_polys",
        None,
        "Specify an override for the lens polynomials in the camera",
        ArgCount::Optional,
        None,
        &Self::set_camera_polys,
    );

    pub(crate) const ARG_USE_OPTICAL_AXIS_OFFSET: ArgDescriptor<CmdArgs> =
        ArgDescriptor::arg_string(
            "use_optical_axis_offset",
            None,
            "Specify an override for the optical axis offset in the camera",
            ArgCount::Optional,
            None,
            &Self::set_camera_optical_axis_offset,
        );

    pub(crate) const ARG_USE_ORIENTATION: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "use_orientation",
        None,
        "Specify the orientation for the current camera, explicitly",
        ArgCount::Optional,
        None,
        &Self::set_camera_orientation,
    );

    pub(crate) const ARG_ADD_NAMED_RAY_FILE: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "rays",
        None,
        "Add named ray Json files (list of name, ray)",
        ArgCount::Any,
        None,
        &Self::add_named_ray_file,
    );

    /// Any number of positional arguments to add names of named points to use
    pub(crate) const ARG_ADD_NAMED_POINT: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "np",
        None,
        "The name of a named point to use or look for; can be a regular expression",
        ArgCount::PositionalAny,
        None,
        &Self::add_np,
    );

    pub(crate) const ARG_POSITIONAL_NAME: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "name",
        None,
        "The name of a named point, CIP (as required by the command)",
        ArgCount::PositionalRequired(1),
        None,
        &Self::add_string_arg,
    );

    pub(crate) const ARG_ADD_POINT3D: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "point",
        None,
        "A 3D point (as required by the command)",
        ArgCount::Any,
        None,
        &Self::add_point3d,
    );

    pub(crate) const ARG_ADD_POINT2D: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "point",
        None,
        "A 2D point (as required by the command)",
        ArgCount::Any,
        None,
        &Self::add_point2d,
    );

    pub(crate) const ARG_PX: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_usize(
        "px",
        None,
        "Pixel X value to use",
        ArgCount::Required,
        None,
        &Self::set_px,
    );

    pub(crate) const ARG_PY: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_usize(
        "py",
        None,
        "Pixel Y value to use",
        ArgCount::Required,
        None,
        &Self::set_py,
    );

    pub(crate) const ARG_KERNEL_SIZE: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_usize(
        "kernel_size",
        None,
        "Size parameter for a kernel",
        ArgCount::Required,
        Some("8"),
        &Self::set_kernel_size,
    );

    pub(crate) const ARG_FLAGS: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_usize(
        "flags",
        None,
        "Flags parameter for (e.g.) a kernel",
        ArgCount::Optional,
        Some("0"),
        &Self::set_flags,
    );

    pub(crate) const ARG_STEPS: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_usize(
        "steps",
        None,
        "Number of steps to use",
        ArgCount::Optional,
        Some("40"),
        &Self::set_steps,
    );

    pub(crate) const ARG_RANGE: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_f64(
        "range",
        None,
        "Range parameter for (e.g.) a kernel",
        ArgCount::Optional,
        Some("10.0"),
        &Self::set_range,
    );

    pub(crate) const ARG_SCALE: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_f64(
        "scale",
        None,
        "Scale parameter for (e.g.) a kernel",
        ArgCount::Optional,
        Some("1"),
        &Self::set_scale,
    );

    pub(crate) const ARG_ANGLE: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_f64(
        "angle",
        None,
        "Angle parameter for (e.g.) a kernel",
        ArgCount::Optional,
        Some("0"),
        &Self::set_angle,
    );

    pub(crate) const ARG_BG_COLOR: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "bg_color",
        None,
        "Background color",
        ArgCount::Optional,
        None,
        &Self::set_bg_color,
    );

    pub(crate) const ARG_PMS_COLOR: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "pms_color",
        None,
        "Color for PMS points",
        ArgCount::Optional,
        None,
        &Self::set_pms_color,
    );

    pub(crate) const ARG_MODEL_COLOR: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "model_color",
        None,
        "Color for named point color, or for drawing mapped model crosses",
        ArgCount::Optional,
        None,
        &Self::set_model_color,
    );

    /*
       pub fn add_arg_calibration_mapping(build: &mut CommandBuilder<Self>, required: bool) {
           build.add_arg_string(
               "calibration_mapping",
               Some('m'),
               "Camera calibration mapping JSON",
               required,
               None,
               CmdArgs::set_calibration_mapping_file,
           );
       }
    */

    pub(crate) const ARG_FROM_CAMERA: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_flag(
        "from_camera",
        None,
        "Operate from the camera to the model, rather than the other way round",
        &Self::set_from_camera,
    );

    pub(crate) const ARG_NUM_PTS: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_usize(
        "num_pts",
        Some('n'),
        "Number of points to use (from start of mapping); if not specified, use all",
        ArgCount::Optional,
        None,
        &Self::set_use_pts,
    );

    pub(crate) const ARG_MAX_PAIRS: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_usize(
        "max_pairs",
        None,
        "Set the maximum pairs for a command",
        ArgCount::Optional,
        Some("100"),
        &Self::set_max_pairs,
    );

    pub(crate) const ARG_MAX_POINTS: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_usize(
        "max_pairs",
        None,
        "Set the maximum points for a command",
        ArgCount::Optional,
        Some("100"),
        &Self::set_max_points,
    );

    pub(crate) const ARG_MAX_ERROR: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_f64(
        "max_error",
        None,
        "Set the maximum error for a command - 0.0 means use a default",
        ArgCount::Optional,
        Some("10.0"),
        &Self::set_max_error,
    );

    pub(crate) const ARG_YAW_MIN: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_f64(
        "yaw_min",
        None,
        "Minimum yaw to use for plotting or updating the star mapping, in degrees",
        ArgCount::Optional,
        Some("1.0"),
        &Self::set_yaw_min,
    );

    pub(crate) const ARG_YAW_MAX: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_f64(
        "yaw_max",
        None,
        "Maximum yaw to use for plotting or updating the star mapping, in degrees",
        ArgCount::Optional,
        Some("20.0"),
        &Self::set_yaw_min,
    );

    pub(crate) const ARG_YAW_ERROR: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_f64(
        "yaw_error",
        None,
        "Maximum relative error in yaw to permit a closest match for",
        ArgCount::Optional,
        Some("0.03"),
        &Self::set_yaw_error,
    );

    pub(crate) const ARG_WITHIN: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_f64(
        "within",
        None,
        "Only use catalog stars Within this angle (degrees) for mapping",
        ArgCount::Optional,
        Some("15"),
        &Self::set_within,
    );

    pub(crate) const ARG_CLOSENESS: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_f64(
        "closeness",
        None,
        "Closeness (degrees) to find triangles of stars or degress for calc cal mapping, find stars, map_stars etc",
        ArgCount::Optional,
        Some("0.2"),
        &Self::set_closeness,
    );

    pub(crate) const ARG_TRIANGLE_CLOSENESS: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_f64(
        "triangle_closeness",
        None,
        "Closeness (degrees) to find triangles of stars",
        ArgCount::Optional,
        Some("0.2"),
        &Self::set_triangle_closeness,
    );

    pub(crate) const ARG_BLEND: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_f64(
        "blend",
        None,
        "Set a blend factor",
        ArgCount::Optional,
        None,
        &Self::set_blend,
    );

    pub(crate) const ARG_STAR_CATALOG: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "star_catalog",
        None,
        "Star catalog to use",
        ArgCount::Optional,
        None,
        &Self::set_star_catalog,
    );

    pub(crate) const ARG_BRIGHTNESS: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_f32(
        "brightness",
        None,
        "Maximum brightness of stars to use in the catalog",
        ArgCount::Optional,
        Some("5.0"),
        &Self::set_brightness,
    );

    // Unused at present?
    pub(crate) const ARG_READ_IMAGE_AT_LEAST_ONE: ArgDescriptor<CmdArgs> =
        ArgDescriptor::arg_string(
            "read",
            Some('r'),
            "Image to read",
            ArgCount::Min(1),
            None,
            &Self::add_read_img,
        );

    pub(crate) const ARG_READ_IMAGE_REQUIRED: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "read",
        Some('r'),
        "Image to read",
        ArgCount::Required,
        None,
        &Self::add_read_img,
    );

    pub(crate) const ARG_IMAGE_OPTIONAL: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "image",
        None,
        "Image filename",
        ArgCount::Optional,
        None,
        &Self::add_read_img,
    );

    pub(crate) const ARG_WRITE_IMAGE_OPTIONAL: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "write",
        Some('w'),
        "Image to write",
        ArgCount::Optional,
        None,
        &Self::set_write_img,
    );

    pub(crate) const ARG_WRITE_IMAGE_REQUIRED: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "write",
        Some('w'),
        "Image to write",
        ArgCount::Required,
        None,
        &Self::set_write_img,
    );

    pub(crate) const ARG_WRITE_PROJECT: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "write_project",
        None,
        "File to write the final project JSON to",
        ArgCount::Optional,
        None,
        &Self::set_write_project,
    );

    pub(crate) const ARG_WRITE_CAMERA: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "write_camera",
        None,
        "File to write the final camera JSON to",
        ArgCount::Optional,
        None,
        &Self::set_write_camera,
    );

    pub(crate) const ARG_WRITE_PATCHES: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "write_patches",
        None,
        "File to write the patches image to",
        ArgCount::Optional,
        None,
        &Self::set_write_patches,
    );

    pub(crate) const ARG_WRITE_POLYS: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "write_polys",
        None,
        "File to write the final lens polynomials JSON to",
        ArgCount::Optional,
        None,
        &Self::set_write_polys,
    );

    pub(crate) const ARG_PATCH_SIZE: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_u32(
        "patch_size",
        Some('p'),
        "Set the patch size for the operation",
        ArgCount::Required,
        None,
        &|s: &mut CmdArgs, v: u32| {
            s.patch_size = v;
            Ok(())
        },
    );

    pub(crate) const ARG_FOVH: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_f64(
        "fovh",
        None,
        "Set the horizontal FOV to use, in degrees",
        ArgCount::Required,
        None,
        &|s: &mut CmdArgs, v: f64| {
            s.fov_h = v.abs();
            Ok(())
        },
    );
    pub(crate) const ARG_FOVV: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_f64(
        "fovv",
        None,
        "Set the vertical FOV to use, in degrees",
        ArgCount::Required,
        None,
        &|s: &mut CmdArgs, v: f64| {
            s.fov_v = v.abs();
            Ok(())
        },
    );

    pub(crate) const ARG_H_OFS: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_f64(
        "hofs",
        None,
        "Set the horizontal offset angle to use, in degrees (defaults to 0)",
        ArgCount::Optional,
        Some("0.0"),
        &|s: &mut CmdArgs, v: f64| {
            s.h_ofs = v;
            Ok(())
        },
    );
    pub(crate) const ARG_V_OFS: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_f64(
        "vofs",
        None,
        "Set the vertical offset angle to use, in degrees (defaults to 0)",
        ArgCount::Optional,
        Some("0.0"),
        &|s: &mut CmdArgs, v: f64| {
            s.v_ofs = v;
            Ok(())
        },
    );
    pub(crate) const ARG_X_GRID: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_f64(
        "grid_x",
        None,
        "Set the X grid spacing (0=none)",
        ArgCount::Optional,
        None,
        &|s: &mut CmdArgs, v: f64| {
            s.x_grid = v.abs();
            Ok(())
        },
    );
    pub(crate) const ARG_Y_GRID: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_f64(
        "grid_y",
        None,
        "Set the Y grid spacing (0=none)",
        ArgCount::Optional,
        None,
        &|s: &mut CmdArgs, v: f64| {
            s.y_grid = v.abs();
            Ok(())
        },
    );

    pub(crate) const ARG_CYLINDRICAL_PROJECTION: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "cylinder",
        None,
        "Set the cylindrical projection to use",
        ArgCount::Optional,
        None,
        &Self::set_cylindrical_projection,
    );

    pub(crate) const ARG_SPHERICAL_IMAGE_SHAPE: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "shape",
        None,
        "Set the toplevel shape",
        ArgCount::Optional,
        Some("tetrahedron"),
        &|s: &mut CmdArgs, shape_name: &str| {
            s.shape = shape_name.parse::<SphericalImageShape>()?;
            Ok(())
        },
    );

    pub(crate) const ARG_SPHERICAL_IMAGE_FILE_INDEX: ArgDescriptor<CmdArgs> =
        ArgDescriptor::arg_usize(
            "file_index",
            None,
            "Select which image file in the spherical image to use",
            ArgCount::PositionalRequired(1),
            None,
            &Self::add_usize_arg,
        );

    /// Add an argument for a single required *Positional* XY coordinate
    pub(crate) const ARG_ADD_XY_ONE: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "xy",
        None,
        "Provide an (X,Y) coordinate",
        ArgCount::PositionalRequired(1),
        None,
        &CmdArgs::add_xy,
    );
    pub(crate) const ARG_ADD_XY_LIST: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "xy",
        None,
        "Provide an (X,Y) coordinate",
        ArgCount::Min(1),
        None,
        &CmdArgs::add_xy,
    );
    fn add_xy(&mut self, v: &str) -> Result<()> {
        self.xy.push(Point2D::load_json(v, &())?);
        Ok(())
    }
    pub(crate) const ARG_ADD_XYZ: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "xyz",
        None,
        "Provide a list of (X,Y,Z) coordinates",
        ArgCount::Required,
        None,
        &CmdArgs::add_xyz,
    );
    fn add_xyz(&mut self, v: &str) -> Result<()> {
        self.xyz
            .extend_from_slice(&Vec::<Point3D>::load_json(v, &())?);
        Ok(())
    }

    pub(crate) const ARG_CLEAR_FILE_PATH: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_flag(
        "clear_file_path",
        None,
        "Clear the file path",
        &CmdArgs::clear_file_path,
    );

    pub(crate) const ARG_ADD_FILE_PATH: ArgDescriptor<CmdArgs> = ArgDescriptor::arg_string(
        "file_path",
        Some('P'),
        "Add a file path to the path set",
        ArgCount::Any,
        None,
        &CmdArgs::add_file_path,
    );
}
