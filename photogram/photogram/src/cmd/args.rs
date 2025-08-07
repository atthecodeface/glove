//a Imports

use clap::{Arg, ArgAction, ArgMatches};
use star_catalog::Catalog;
use thunderclap::{ArgCount, CommandBuilder};

use ic_base::json;
use ic_base::Result;
use ic_stars::StarMapping;

use super::CmdArgs;

//a CmdArgs accessors
//ip CmdArgs arg build methods
impl CmdArgs {
    //mp add_arg_verbose
    pub fn add_arg_verbose(build: &mut CommandBuilder<Self>) {
        build.add_flag(
            "verbose",
            Some('v'),
            "Enable verbose output",
            CmdArgs::set_verbose,
        );
    }

    //fp add_arg_read_image
    pub fn add_arg_read_image<I: Into<ArgCount>>(build: &mut CommandBuilder<Self>, arg_count: I) {
        build.add_arg_string(
            "read",
            Some('r'),
            "Image to read",
            arg_count.into(),
            None,
            CmdArgs::add_read_img,
        );
    }

    //fp add_arg_write_image
    pub fn add_arg_write_image(build: &mut CommandBuilder<Self>, required: bool) {
        build.add_arg_string(
            "write",
            Some('w'),
            "Image to write",
            required.into(),
            None,
            CmdArgs::set_write_img,
        );
    }

    //fp add_arg_kernel
    pub fn add_arg_kernel<I: Into<ArgCount>>(build: &mut CommandBuilder<Self>, arg_count: I) {
        build.add_arg_string(
            "kernel",
            None,
            "Add a kernel to run",
            arg_count.into(),
            None,
            CmdArgs::add_kernel,
        );
    }

    //fp add_arg_nps
    pub fn add_arg_nps(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "nps",
            None,
            "Add a named point set to the list",
            (0,).into(),
            None,
            CmdArgs::add_nps,
        );
    }

    //fp add_arg_camera_database
    pub fn add_arg_camera_database(build: &mut CommandBuilder<Self>, required: bool) {
        build.add_arg_string(
            "camera_db",
            None,
            "Camera database JSON filename",
            required.into(),
            None,
            CmdArgs::set_camera_db,
        );
    }

    //fp add_arg_project
    pub fn add_arg_project(build: &mut CommandBuilder<Self>, required: bool) {
        build.add_arg_string(
            "project",
            None,
            "Project JSON filename",
            required.into(),
            None,
            CmdArgs::set_project,
        );
    }

    //fp add_arg_pms
    pub fn add_arg_pms(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "pms",
            None,
            "Add a point mapping set",
            false.into(), // Perhaps should allow some in...
            None,
            CmdArgs::add_pms,
        );
    }

    //fp add_arg_cip
    pub fn add_arg_cip(build: &mut CommandBuilder<Self>, required: bool) {
        let arg_count = required.into();
        build.add_arg_usize(
            "cip",
            None,
            "CIP number (camera and PMS) within the project",
            arg_count,
            None,
            CmdArgs::set_cip,
        );
    }

    //fp add_arg_camera
    pub fn add_arg_camera(build: &mut CommandBuilder<Self>, required: bool) {
        build.add_arg_string(
            "camera",
            Some('c'),
            "Camera lens, placement and orientation JSON",
            required.into(),
            None,
            CmdArgs::set_camera_file,
        );

        build.add_arg_string(
            "use_body",
            None,
            "Specify which body to use in the camera",
            false.into(),
            None,
            CmdArgs::set_camera_body,
        );

        build.add_arg_string(
            "use_lens",
            None,
            "Specify which lens to use in the camera",
            false.into(),
            None,
            CmdArgs::set_camera_lens,
        );

        build.add_arg_f64(
            "use_focus",
            None,
            "Specify the focus distance in mm used for the image, in the camera",
            false.into(),
            None,
            CmdArgs::set_camera_focus_distance,
        );

        build.add_arg_string(
            "use_polys",
            None,
            "Specify an override for the lens polynomials in the camera",
            false.into(),
            None,
            CmdArgs::set_camera_polys,
        );
    }

    //fp add_arg_ray_file
    pub fn add_arg_ray_file<I: Into<ArgCount>>(build: &mut CommandBuilder<Self>, arg_count: I) {
        build.add_arg_string(
            "rays",
            None,
            "Model ray Json files (list of name, ray)",
            arg_count.into(),
            None,
            CmdArgs::set_ray_file,
        );
    }

    //fp add_arg_named_point
    pub fn add_arg_named_point<I: Into<ArgCount>>(build: &mut CommandBuilder<Self>, arg_count: I) {
        build.add_arg_string(
            "np",
            None,
            "Specifies named points for the patch",
            arg_count.into(),
            None,
            CmdArgs::set_np,
        );
    }

    //fp add_arg_px
    pub fn add_arg_px(build: &mut CommandBuilder<Self>, required: bool) {
        build.add_arg_usize(
            "px",
            None,
            "Pixel X value to use",
            required.into(),
            None,
            CmdArgs::set_px,
        );
    }

    //fp add_arg_py
    pub fn add_arg_py(build: &mut CommandBuilder<Self>, required: bool) {
        build.add_arg_usize(
            "py",
            None,
            "Pixel Y value to use",
            required.into(),
            None,
            CmdArgs::set_py,
        );
    }

    //fp add_arg_kernel_size
    pub fn add_arg_kernel_size(build: &mut CommandBuilder<Self>, required: bool) {
        build.add_arg_usize(
            "kernel_size",
            None,
            "Size parameter for a kernel",
            required.into(),
            Some("8"),
            CmdArgs::set_kernel_size,
        );
    }

    //fp add_arg_flags
    pub fn add_arg_flags(build: &mut CommandBuilder<Self>) {
        build.add_arg_usize(
            "flags",
            None,
            "Flags parameter for (e.g.) a kernel",
            false.into(),
            Some("0"),
            CmdArgs::set_flags,
        );
    }

    //fp add_arg_scale
    pub fn add_arg_scale(build: &mut CommandBuilder<Self>) {
        build.add_arg_f64(
            "scale",
            None,
            "Scale parameter for (e.g.) a kernel",
            false.into(),
            Some("1"),
            CmdArgs::set_scale,
        );
    }

    //fp add_arg_angle
    pub fn add_arg_angle(build: &mut CommandBuilder<Self>) {
        build.add_arg_f64(
            "angle",
            None,
            "Angle parameter for (e.g.) a kernel",
            false.into(),
            Some("0"),
            CmdArgs::set_angle,
        );
    }

    //fp add_arg_bg_color
    pub fn add_arg_bg_color(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "bg_color",
            None,
            "Background color",
            ArgCount::Optional,
            None,
            CmdArgs::set_bg_color,
        );
    }

    //mp add_arg_pms_color
    pub fn add_arg_pms_color(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "pms_color",
            None,
            "Color for PMS points",
            ArgCount::Optional,
            None,
            CmdArgs::set_pms_color,
        );
    }

    //mp add_arg_model_color
    pub fn add_arg_model_color(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "model_color",
            None,
            "Color for mapped model crosses",
            ArgCount::Optional,
            None,
            CmdArgs::set_model_color,
        );
    }

    //mp add_arg_calibration_mapping
    pub fn add_arg_calibration_mapping(build: &mut CommandBuilder<Self>, required: bool) {
        build.add_arg_string(
            "calibration_mapping",
            Some('m'),
            "Camera calibration mapping JSON",
            required.into(),
            None,
            CmdArgs::set_calibration_mapping_file,
        );
    }

    //fp add_arg_write_camera
    pub fn add_arg_write_camera(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "write_camera",
            None,
            "File to write the final camera JSON to",
            ArgCount::Optional,
            None,
            CmdArgs::set_write_camera,
        );
    }

    //fp add_arg_write_calibration_mapping
    pub fn add_arg_write_calibration_mapping(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "write_calibration_mapping",
            None,
            "File to write a derived mapping JSON to",
            ArgCount::Optional,
            None,
            CmdArgs::set_write_calibration_mapping,
        );
    }

    //fp add_arg_write_star_mapping
    pub fn add_arg_write_star_mapping(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "write_star_mapping",
            None,
            "File to write a derived star mapping JSON to",
            ArgCount::Optional,
            None,
            CmdArgs::set_write_star_mapping,
        );
    }

    //fp add_arg_write_polys
    pub fn add_arg_write_polys(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "write_polys",
            None,
            "File to write a derived polynomials JSON to",
            ArgCount::Optional,
            None,
            CmdArgs::set_write_polys,
        );
    }

    //fp add_arg_write_svg
    pub fn add_arg_write_svg(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "write_svg",
            None,
            "File to write an output SVG to",
            ArgCount::Optional,
            None,
            CmdArgs::set_write_svg,
        );
    }
    //fp add_arg_poly_degree
    pub fn add_arg_poly_degree(build: &mut CommandBuilder<Self>) {
        build.add_arg_usize(
            "poly_degree",
            None,
            "Degree of polynomial to use for the lens calibration (5 for 50mm)",
            ArgCount::Optional,
            Some("5"),
            CmdArgs::set_poly_degree,
        );
    }

    //fp add_arg_use_deltas
    pub fn add_arg_use_deltas(build: &mut CommandBuilder<Self>) {
        build.add_flag(
            "use_deltas",
            None,
            "Use deltas for plotting rather than absolute values",
            CmdArgs::set_use_deltas,
        );
    }

    //fp add_arg_num_pts
    pub fn add_arg_num_pts(build: &mut CommandBuilder<Self>) {
        build.add_arg_usize(
            "num_pts",
            Some('n'),
            "Number of points to use (from start of mapping); if not specified, use all",
            ArgCount::Optional,
            None,
            CmdArgs::set_use_pts,
        );
    }

    //fp add_arg_yaw_min_max
    pub fn add_arg_yaw_min_max(
        build: &mut CommandBuilder<Self>,
        min: Option<&'static str>,
        max: Option<&'static str>,
    ) {
        build.add_arg_f64(
            "yaw_min",
            None,
            "Minimim yaw to use for plotting or updating the star mapping, in degrees",
            ArgCount::Optional,
            min,
            CmdArgs::set_yaw_min,
        );
        build.add_arg_f64(
            "yaw_max",
            None,
            "Maximim yaw to use for plotting or updating the star mapping, in degrees",
            ArgCount::Optional,
            max,
            CmdArgs::set_yaw_max,
        );
    }

    //fp add_arg_yaw_error
    pub fn add_arg_yaw_error(build: &mut CommandBuilder<Self>) {
        build.add_arg_f64(
            "yaw_error",
            None,
            "Maximum relative error in yaw to permit a closest match for",
            ArgCount::Optional,
            Some("0.03"),
            CmdArgs::set_yaw_error,
        );
    }

    //fp add_arg_within
    pub fn add_arg_within(build: &mut CommandBuilder<Self>) {
        build.add_arg_f64(
            "within",
            None,
            "Only use catalog stars Within this angle (degrees) for mapping",
            ArgCount::Optional,
            Some("15"),
            CmdArgs::set_within,
        );
    }

    //fp add_arg_closeness
    pub fn add_arg_closeness(build: &mut CommandBuilder<Self>) {
        build.add_arg_f64(
            "closeness",
            None,
            "Closeness (degrees) to find triangles of stars or degress for calc cal mapping, find stars, map_stars etc",
            ArgCount::Optional,
            Some("0.2"),
            CmdArgs::set_closeness,
        );
    }

    //fp add_arg_triangle_closeness
    pub fn add_arg_triangle_closeness(build: &mut CommandBuilder<Self>) {
        build.add_arg_f64(
            "triangle_closeness",
            None,
            "Closeness (degrees) to find triangles of stars",
            ArgCount::Optional,
            Some("0.2"),
            CmdArgs::set_triangle_closeness,
        );
    }

    //fp add_arg_star_mapping
    pub fn add_arg_star_mapping(build: &mut CommandBuilder<Self>) {
        build.add_arg(
            Arg::new("star_mapping")
                .required(false)
                .help("File mapping sensor coordinates to catalog identifiers")
                .action(ArgAction::Set),
            Box::new(CmdArgs::arg_star_mapping),
        );
    }

    //fp add_arg_star_catalog
    pub fn add_arg_star_catalog(build: &mut CommandBuilder<Self>) {
        build.add_arg(
            Arg::new("star_catalog")
                .long("catalog")
                .required(false)
                .help("Star catalog to use")
                .action(ArgAction::Set),
            Box::new(CmdArgs::arg_star_catalog),
        );
    }

    //fp add_arg_brightness
    pub fn add_arg_brightness(build: &mut CommandBuilder<Self>) {
        build.add_arg_f32(
            "brightness",
            None,
            "Maximum brightness of stars to use in the catalog",
            ArgCount::Optional,
            Some("5.0"),
            CmdArgs::set_brightness,
        );
    }

    //fp arg_star_mapping
    pub fn arg_star_mapping(args: &mut CmdArgs, matches: &ArgMatches) -> Result<()> {
        let filename = matches.get_one::<String>("star_mapping").unwrap();
        let json = json::read_file(filename)?;
        args.star_mapping = StarMapping::from_json(&json)?;
        Ok(())
    }

    //fp arg_star_catalog
    pub fn arg_star_catalog(args: &mut CmdArgs, matches: &ArgMatches) -> Result<()> {
        let catalog_filename = matches.get_one::<String>("star_catalog").unwrap();
        let mut catalog = Catalog::load_catalog(catalog_filename, 99.)?;
        catalog.derive_data();
        args.star_catalog = Some(Box::new(catalog));
        Ok(())
    }
}
