//a Imports
use std::io::Write;

use clap::Command;
use geo_nd::{quat, Quaternion, Vector};
use thunderclap::CommandBuilder;

use ic_base::{Point3D, Quat, RollYaw};
use ic_camera::polynomial;
use ic_camera::{CameraProjection, LensPolys};
use ic_image::{Color, Image};
use ic_mapping::ModelLineSet;

use crate::cmd::{cmd_ok, CmdArgs, CmdResult};

//a Help messages
//hi CAMERA_CALIBRATE_LONG_HELP
const CAMERA_CALIBRATE_LONG_HELP: &str = "\
This provides various tools to help calibrate a lens mapping.

Some of the tools are based on a description of a photograph of a
regular grid (such as graph paper); alternatively, a photograph of
stars may be used as a starting point (replacing *some* of the tools
provided here).

The description of a photograph of a regular grid can be obtained
somewhat automatically using the 'image_analyze' tool.

Using a grid, the approach is first to *locate* the camera.

With a located camera and a grid, or with a camera ar the origin and a
star description, the orientation of the camera can be determined.

With an orientation, the lens calibration can be determined, and
verified with images.";

//hi LOCATE_LONG_HELP
const LOCATE_LONG_HELP: &str = "\
Determine the 'best' location for a camera/lens from a mapping
description file, ignoring the given camera position and orientation.

The mapping is a list of (x,y,z, px,py); this indicates that the point
in the 'world' at (x,y,z) was seen on the camera sensor at absolute
camera sensor position (px,py).

The algorithm uses is to determine for every pair of selected mappings
the angle subtended as seen by the camera/lens - based on the (px,py)
values - including using any lens mapping the camera has. Each such
pair corresponds to a line in space (between the two (x,y,z) for the
pair); so there is then a set of (line, angle subtended) for each pair
of mappings. Any one such mapping describes a surface in the world
space from where the line would subtend such an angle. Two such
mappings thus describe a line in world space (at best), and three a
point in world space. Hence three or more pairs can be used to
determine a position in space. The 'best' position in space is deemed
to be that point where the sum of the absolute errors in the angles
for each line subtended is minimized.";

//hi ORIENT_LONG_HELP
const ORIENT_LONG_HELP: &str = "\
Determine the 'best' orientation for a camera/lens from a mapping
description file, ignoring the orientation specified in the
description.

The mapping is a list of (x,y,z, px,py); this indicates that the point
in the 'world' at (x,y,z) was seen on the camera sensor at absolute
camera sensor position (px,py).

For every mapping in the file, given the camera position, a direction
vector (dx,dy,dz) can be generated for that mapping - and this
presumably corresponds to (px,py) for that mapping, which in turn
describes some camera-relative direction (dpx,dpy,dpz).

Hence (given the camera position) we have two lists of directions,
which *ought* to map through an orientation mapping (an arbitrary
rotation matrix in 3D, or a unit quaternion). For any one mapping
there is a quaternion (q0) that maps (dx,dy,dz) to the Z axis, and
another quaternion that maps the Z axis to (dpx,dpy,dpz) (q1); if we
take a second mapping and apply q0 to its (dx,dy,dz), we can apply
*some* rotation around the Z axis (qz), and the apply q1', and we
should end up at its (dpx, dpy, dpz); this combination q0.qz.q1c is a
good best effort for this pair of mappings.

The tool generates all N(N-1) such mapping quaternions for every pair
of mappings, and then determines the average quaternion; this is the 'best' orientation.";

//hi LENS_CALIBRATE_LONG_HELP
const LENS_CALIBRATE_LONG_HELP: &str = "\
Using a mappings description determine the polynomial of best fit to
map the image Yaw to the world Yaw

The mapping is a list of (x,y,z, px,py); this indicates that the point
in the 'world' at (x,y,z) was seen on the camera sensor at absolute
camera sensor position (px,py).

Given a camera position and orientation every mapping has a direction
in both 'world' space (relative to the camera axis) and in 'sensor'
space (relative to the center of the sensor); such directions can be
encoded as a roll and yaw - that is, the angle that the direction is
'away' from the axis of view; and the angle that the direction is
'around' the clock. For example, a direction vector could be described
as 30 degrees off straight-ahead, in the direction of '2' on a clock
(which would be 60 degrees clockwise from the vertical). The first of
these is the Yaw, the second the Roll.

A *spherical* camera lens mapping is a function of Yaw in world space
to Yaw in sensor space - Roll is not important. This tool therefore
generates the two Yaw values (world and sensor) for all of the mapping
points, given the camera position and orientation, and it generates a
polynomial of best fit.

In fact two polynomials are generated - one forward (wts) and one
backward (stw); these should be used in a camera_db JSON file.

";

//hi YAW_PLOT_LONG_HELP
const YAW_PLOT_LONG_HELP: &str = "\
Generate an SVG file with a plot of world 'yaw' versus sensor 'yaw'

The plot that is generated is an SVG file with the X axis being the
yaw value in 'sensor' space, and the Y axis either the world yaw minus
the sensor yaw, or alternatively relative delta (world/sensor-1).

Additional lines are overlaid for different standard lens calibrations
(linear, equiangular, equisolid, stereographic, and orthographic).

Finally a scatter plot is included with the *filtered* values from the *mapping*.
";

//hi ROLL_PLOT_LONG_HELP
const ROLL_PLOT_LONG_HELP: &str = "\
Generate a plot for all the mappings of model roll versus world roll

This is provided for completeness, and graphs roll delta versus yaw delta
";

//hi GRID_IMAGE_LONG_HELP
const GRID_IMAGE_LONG_HELP: &str = "\
This tool uses the provided camera description and mappings, and
overlays an image with *red* crosses showing the specified coordinates of
each mapping and the derived (i.e. post-camera/lens mapping) positions
of those mappings with *green* crosses.

It also draws black crosses for a range of (x,y,0) values.";

//a Locate
//fi locate_cmd
fn locate_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("locate")
        .about("Determine an optimal location from a calibration description")
        .long_about(LOCATE_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(locate_fn)));
    CmdArgs::add_arg_calibration_mapping(&mut build, true);
    CmdArgs::add_arg_num_pts(&mut build);
    CmdArgs::add_arg_write_camera(&mut build);

    build
}

//fi locate_fn
fn locate_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let pms = cmd_args.calibration_mapping_to_pms();

    //cb Reset the camera position and orientation, defensively
    cmd_args.camera_mut().set_position([0., 0., 0.].into());
    cmd_args.camera_mut().set_orientation(Quat::default());

    //cb Set up HashMaps and collections
    let n = cmd_args.use_pts(pms.len());
    let closest_n: Vec<usize> = (0..n).collect();

    //cb For required pairings, display data
    cmd_args.show_step("Using the following mappings ([n] [world] : [pxy] : [world_dir]");
    for pm_n in &closest_n {
        let pm = &pms.mappings()[*pm_n];
        let n = pm.name();
        let grid_xyz = pm.model();
        // Px Abs -> Px Rel -> TxTy -> lens mapping
        let pxy_abs = pm.screen();
        let txty = cmd_args.camera().px_abs_xy_to_camera_txty(pxy_abs);
        let grid_dir = txty.to_unit_vector();
        cmd_args.if_verbose(|| {
            eprintln!(">> {n} {grid_xyz} : {pxy_abs} : {grid_dir}",);
        });
    }

    //cb Create ModelLineSet
    let mut mls = ModelLineSet::new(cmd_args.camera().clone());

    for n0 in &closest_n {
        let pm0 = &pms.mappings()[*n0];
        let dir0 = cmd_args
            .camera()
            .px_abs_xy_to_camera_txty(pm0.screen())
            .to_unit_vector();
        for n1 in &closest_n {
            if n0 == n1 {
                continue;
            }
            let pm1 = &pms.mappings()[*n1];
            let dir1 = cmd_args
                .camera()
                .px_abs_xy_to_camera_txty(pm1.screen())
                .to_unit_vector();
            let cos_theta = dir0.dot(&dir1);
            let angle = cos_theta.acos();
            let _ = mls.add_line_of_models(pm0.model(), pm1.model(), angle);
        }
    }

    if mls.num_lines() == 0 {
        return Err(format!(
            "no lines generated for locating the camera; was using {} points",
            closest_n.len()
        )
        .into());
    }

    //cb Find best position given ModelLineSet
    // Find best location 'p' for camera
    let (best_cam_pos, e) = mls.find_best_min_err_location(&|p| p[2] > 0., 1000, 1000);
    cmd_args.if_verbose(|| {
        eprintln!("{best_cam_pos} {e}",);
    });

    cmd_args.camera_mut().set_position(best_cam_pos);
    cmd_args.write_outputs()?;
    cmd_args.output_camera()
}

//a Orient
//fi orient_cmd
fn orient_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("orient")
        .about("Determine an optimal orientation from a calibration description")
        .long_about(ORIENT_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(orient_fn)));
    CmdArgs::add_arg_calibration_mapping(&mut build, true);
    CmdArgs::add_arg_num_pts(&mut build);
    CmdArgs::add_arg_write_camera(&mut build);

    build
}

//fi orient_fn
fn orient_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let pms = cmd_args.calibration_mapping_to_pms();

    //cb Set up 'cam' as the camera; use its position (unless otherwise told?)
    let mut camera = cmd_args.camera().clone();
    camera.set_orientation(Quat::default());

    //cb Set up HashMaps and collections
    let n = cmd_args.use_pts(pms.len());
    let closest_n: Vec<usize> = (0..n).collect();

    //cb For required pairings, display data
    cmd_args.show_step("All the following mappings ([n] [world] : [pxy] : [world_dir]");
    cmd_args.if_verbose(|| {
        for pm in pms.mappings() {
            let n = pm.name();
            let grid_xyz = pm.model();
            // Px Abs -> Px Rel -> TxTy -> lens mapping
            let pxy_abs = pm.screen();
            let txty = camera.px_abs_xy_to_camera_txty(pxy_abs);
            let grid_dir = txty.to_unit_vector();
            eprintln!("{n} {grid_xyz} : {pxy_abs} : {grid_dir}",);
        }
    });

    //cb Find best orientation given position
    // We can get N model direction vectors given the camera position,
    // and for each we have a camera direction vector
    cmd_args.show_step("Derive orientations from *specified* mappings");
    let best_cam_pos = camera.position();
    let mut qs = vec![];

    for n0 in &closest_n {
        let pm0 = &pms.mappings()[*n0];
        let di_c = -camera
            .px_abs_xy_to_camera_txty(pm0.screen())
            .to_unit_vector();
        let di_m = (best_cam_pos - pm0.model()).normalize();
        let z_axis: Point3D = [0., 0., 1.].into();
        let qi_c: Quat = quat::rotation_of_vec_to_vec(&di_c.into(), &z_axis.into()).into();
        let qi_m: Quat = quat::rotation_of_vec_to_vec(&di_m.into(), &z_axis.into()).into();
        for n1 in &closest_n {
            if n0 == n1 {
                continue;
            }
            let pm1 = &pms.mappings()[*n1];
            let dj_c = -camera
                .px_abs_xy_to_camera_txty(pm1.screen())
                .to_unit_vector();
            let dj_m = (best_cam_pos - pm1.model()).normalize();

            let dj_c_rotated: Point3D = quat::apply3(qi_c.as_ref(), dj_c.as_ref()).into();
            let dj_m_rotated: Point3D = quat::apply3(qi_m.as_ref(), dj_m.as_ref()).into();

            let theta_dj_m = dj_m_rotated[0].atan2(dj_m_rotated[1]);
            let theta_dj_c = dj_c_rotated[0].atan2(dj_c_rotated[1]);
            let theta = theta_dj_m - theta_dj_c;
            let theta_div_2 = theta / 2.0;
            let cos_2theta = theta_div_2.cos();
            let sin_2theta = theta_div_2.sin();
            let q_z = Quat::of_rijk(cos_2theta, 0.0, 0.0, sin_2theta);

            // At this point, qi_m * di_m = (0,0,1)
            //
            // At this point, q_z.conj * qi_m * di_m = (0,0,1)
            //                q_z.conj * qi_m * dj_m = dj_c_rotated
            //
            let q = qi_c.conjugate() * q_z * qi_m;

            // dc_i === quat::apply3(q.as_ref(), di_m.as_ref()).into();
            // dc_j === quat::apply3(q.as_ref(), dj_m.as_ref()).into();
            //            eprintln!(
            //                "di_c==q*di_m? {di_c} ==? {:?}",
            //                quat::apply3(q.as_ref(), di_m.as_ref())
            //            );
            //            eprintln!(
            //                "dj_c==q*dj_m? {dj_c} ==? {:?}",
            //                quat::apply3(q.as_ref(), dj_m.as_ref())
            //            );
            qs.push((1., q.into()));
        }
    }
    drop(pms);

    cmd_args.show_step("Calculate average orientation");
    let qr: Quat = quat::weighted_average_many(qs.iter().copied()).into();
    camera.set_orientation(qr);
    cmd_args.if_verbose(|| {
        eprintln!("{camera}");
    });
    cmd_args.set_camera(camera);

    cmd_args.write_outputs()?;
    cmd_args.output_camera()
}

//a Lens calibrate
//fi lens_calibrate_cmd
fn lens_calibrate_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("lens_calibrate")
        .about("From calibrate_from_grid")
        .long_about(LENS_CALIBRATE_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(lens_calibrate_fn)));

    CmdArgs::add_arg_calibration_mapping(&mut build, true);
    CmdArgs::add_arg_yaw_min_max(&mut build, Some("1.0"), Some("20.0"));
    CmdArgs::add_arg_num_pts(&mut build);
    CmdArgs::add_arg_poly_degree(&mut build);
    CmdArgs::add_arg_write_polys(&mut build);

    build
}

//fi lens_calibrate_fn
fn lens_calibrate_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let pms = cmd_args.calibration_mapping_to_pms();
    let mut camera_linear = cmd_args.camera().clone();
    let mut lens_linear = camera_linear.lens().clone();
    lens_linear.set_polys(LensPolys::default());
    camera_linear.set_lens(lens_linear);

    let yaw_range_min = cmd_args.yaw_min().to_radians();
    let yaw_range_max = cmd_args.yaw_max().to_radians();
    let num_pts = cmd_args.use_pts(pms.len());

    //cb Calculate Roll/Yaw for each point given camera, and store in a mapping
    let mut sensor_yaws = vec![];
    let mut world_yaws = vec![];
    for pm in pms.mappings().iter().take(num_pts) {
        let world_txty = camera_linear.world_xyz_to_camera_txty(pm.model());
        let sensor_txty = camera_linear.px_abs_xy_to_camera_txty(pm.screen());

        let world_ry: RollYaw = world_txty.into();
        let sensor_ry: RollYaw = sensor_txty.into();
        world_yaws.push(world_ry.yaw());
        sensor_yaws.push(sensor_ry.yaw());
    }
    if sensor_yaws.len() < 30 {
        eprintln!("Lens calibration being attempted with only {} points; it is unwise to use fewer than 30",
                  sensor_yaws.len());
    }

    let lens_poly = LensPolys::calibration(
        cmd_args.poly_degree(),
        &sensor_yaws,
        &world_yaws,
        yaw_range_min,
        yaw_range_max,
    )
    .map_err(|e| {
        (
            e,
            format!(
                "using num_pts {}={}, yaw range {}->{}",
                num_pts,
                pms.len(),
                yaw_range_min.to_degrees(),
                yaw_range_max.to_degrees()
            ),
        )
    })?;

    let mut camera_lens = cmd_args.camera().lens().clone();
    camera_lens.set_polys(lens_poly);
    cmd_args.camera_mut().set_lens(camera_lens);

    cmd_args.write_outputs()?;

    cmd_args.output_polynomials()
}

//a Yaw plot
//fi yaw_plot_cmd
fn yaw_plot_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("yaw_plot")
        .about("Plot yaw")
        .long_about(YAW_PLOT_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(yaw_plot_fn)));

    CmdArgs::add_arg_calibration_mapping(&mut build, true);
    CmdArgs::add_arg_yaw_min_max(&mut build, Some("1.0"), Some("20.0"));
    CmdArgs::add_arg_num_pts(&mut build);
    CmdArgs::add_arg_use_deltas(&mut build);
    CmdArgs::add_arg_write_svg(&mut build);

    build
}

//fi yaw_plot_fn
fn yaw_plot_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let pms = cmd_args.calibration_mapping_to_pms();
    let camera = cmd_args.camera();
    let mut camera_linear = camera.clone();
    let mut lens_linear = camera.lens().clone();
    lens_linear.set_polys(LensPolys::default());
    camera_linear.set_lens(lens_linear);

    let yaw_range_min = cmd_args.yaw_min().to_radians();
    let yaw_range_max = cmd_args.yaw_max().to_radians();
    let num_pts = cmd_args.use_pts(pms.len());

    let f_world = |w: f64, s: f64| (s.to_degrees(), (s - w).to_degrees());
    let f_rel_error = |w: f64, s: f64| (s.to_degrees(), s / w - 1.0);
    let plot_f = {
        if cmd_args.use_deltas() {
            f_rel_error
        } else {
            f_world
        }
    };

    //cb Calculate Error in yaw/Yaw for each point given camera
    let mut pts = [vec![], vec![], vec![], vec![]];
    let mut ws_yaws = vec![];
    for pm in pms.mappings().iter().take(num_pts) {
        let world_txty = camera_linear.world_xyz_to_camera_txty(pm.model());
        let sensor_txty = camera_linear.px_abs_xy_to_camera_txty(pm.screen());

        let world_ry: RollYaw = world_txty.into();
        let sensor_ry: RollYaw = sensor_txty.into();

        if sensor_ry.yaw() > yaw_range_max {
            continue;
        }
        if sensor_ry.yaw() < yaw_range_min {
            continue;
        }
        ws_yaws.push((world_ry.yaw(), sensor_ry.yaw()));
    }
    ws_yaws.sort_by(|a, b| (a.1).partial_cmp(&b.1).unwrap());

    let mean_median_ws_yaws = polynomial::filter_ws_yaws(&ws_yaws);

    for (w, s) in mean_median_ws_yaws.iter() {
        pts[0].push(plot_f(*w, *s));
    }

    //cb Plot 4 graphs for quadrants and one for the polynomial
    use poloto::prelude::*;
    use tagu::prelude::*;
    let theme = poloto::render::Theme::light();
    let theme = theme.append(tagu::build::raw(".poloto_scatter{stroke-width:1.5px;}"));
    let theme = theme.append(tagu::build::raw(
        ".poloto_text.poloto_legend{font-size:10px;}",
    ));
    let theme = theme.append(tagu::build::raw(
        ".poloto_line{stroke-dasharray:2;stroke-width:2;}",
    ));

    let plots = poloto::build::origin();

    let mut wts_poly_pts = vec![];
    for i in 0..=100 {
        let model_yaw = (i as f64) / 100.0 * (yaw_range_max - yaw_range_min) + yaw_range_min;
        let model_ry = RollYaw::of_yaw(model_yaw);
        let sensor_ry = camera.camera_ry_to_sensor_ry(model_ry);
        if sensor_ry.yaw() > yaw_range_min && sensor_ry.yaw() < yaw_range_max {
            wts_poly_pts.push(plot_f(model_yaw, sensor_ry.yaw()));
        }
    }

    let mut stw_poly_pts = vec![];
    for i in 0..=400 {
        let sensor_yaw = (i as f64) / 400.0 * (yaw_range_max - yaw_range_min) + yaw_range_min;
        let sensor_ry = RollYaw::of_yaw(sensor_yaw);
        let model_ry = camera.sensor_ry_to_camera_ry(sensor_ry);
        if model_ry.yaw() > yaw_range_min && model_ry.yaw() < yaw_range_max {
            stw_poly_pts.push(plot_f(model_ry.yaw(), sensor_yaw));
        }
    }

    let mut linear_pts = vec![];
    let mut stereographic_pts = vec![];
    let mut equiangular_pts = vec![];
    let mut equisolid_pts = vec![];
    let mut orthographic_pts = vec![];
    for i in 0..=100 {
        let world_yaw = (i as f64) / 100.0 * (yaw_range_max - yaw_range_min) + yaw_range_min;
        linear_pts.push(plot_f(world_yaw, world_yaw));
        stereographic_pts.push(plot_f(world_yaw, ((world_yaw / 2.0).tan() * 2.0).atan()));
        equiangular_pts.push(plot_f(world_yaw, world_yaw.atan()));
        equisolid_pts.push(plot_f(world_yaw, (world_yaw / 2.0).sin().atan()));
        orthographic_pts.push(plot_f(world_yaw, world_yaw.sin().atan()));
    }

    let plot = poloto::build::plot("Linear");
    let plot = plot.line(linear_pts.iter());
    let plots = plots.chain(plot);

    let plot = poloto::build::plot("Stereographic");
    let plot = plot.line(stereographic_pts.iter());
    let plots = plots.chain(plot);

    let plot = poloto::build::plot("Equiangular");
    let plot = plot.line(equiangular_pts.iter());
    let plots = plots.chain(plot);

    //    let plot = poloto::build::plot("Equisolid");
    //    let plot = plot.line(equisolid_pts.iter());
    //    let plots = plots.chain(plot);

    let plot = poloto::build::plot("Orthographic");
    let plot = plot.line(orthographic_pts.iter());
    let plots = plots.chain(plot);

    let plot = poloto::build::plot("Camera wts mapping");
    let plot = plot.line(wts_poly_pts.iter());
    let plots = plots.chain(plot);

    let plot = poloto::build::plot("Camera stw mapping");
    let plot = plot.line(stw_poly_pts.iter());
    let plots = plots.chain(plot);

    let plot = poloto::build::plot("Quad x>0 y>0");
    let plot = plot.scatter(pts[0].iter());
    let plots = plots.chain(plot);

    // let plot = poloto::build::plot("Quad x<0 y>0");
    // let plot = plot.scatter(pts[1].iter());
    // let plots = plots.chain(plot);
    // let plot = poloto::build::plot("Quad x>0 y<0");
    // let plot = plot.scatter(pts[2].iter());
    // let plots = plots.chain(plot);
    // let plot = poloto::build::plot("Quad x<0 y<0");
    // let plot = plot.scatter(pts[3].iter());
    // let plots = plots.chain(plot);

    let plot_initial = poloto::frame_build()
        .data(plots)
        .build_and_label(("Yaw plot", "Sensor yaw / °", "s/w-1 or s-w"))
        .append_to(poloto::header().append(theme))
        .render_string()
        .map_err(|e| format!("{e:?}"))?;

    let s = plot_initial.to_string();
    if let Some(filename) = cmd_args.write_svg() {
        let mut f = std::fs::File::create(filename)?;
        f.write_all(s.as_bytes())?;
    } else {
        println!("{s}");
    }
    cmd_ok()
}

//a Roll plot
//fi roll_plot_cmd
fn roll_plot_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("roll_plot")
        .about("Plot roll of model versus roll of camera")
        .long_about(ROLL_PLOT_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(roll_plot_fn)));

    CmdArgs::add_arg_calibration_mapping(&mut build, true);
    CmdArgs::add_arg_write_svg(&mut build);
    CmdArgs::add_arg_yaw_min_max(&mut build, Some("1.0"), Some("20.0"));
    CmdArgs::add_arg_num_pts(&mut build);

    build
}

//fi roll_plot_fn
fn roll_plot_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let pms = cmd_args.calibration_mapping_to_pms();
    let camera = cmd_args.camera();

    let num_pts = cmd_args.use_pts(pms.len());

    //cb Calculate Roll/Yaw for each point given camera
    let mut pts = vec![];
    for pm in pms.mappings().iter().take(num_pts) {
        let model_txty = camera.world_xyz_to_camera_txty(pm.model());
        let cam_txty = camera.px_abs_xy_to_camera_txty(pm.screen());

        let model_ry: RollYaw = model_txty.into();
        let cam_ry: RollYaw = cam_txty.into();

        pts.push((
            (cam_ry.yaw() - model_ry.yaw()).to_degrees(),
            (cam_ry.roll() - model_ry.roll()).to_degrees(),
        ));
    }

    //cb Plot 4 graphs for quadrants and one for the polynomial
    use poloto::prelude::*;
    use tagu::prelude::*;
    let theme = poloto::render::Theme::light();
    let theme = theme.append(tagu::build::raw(".poloto_scatter{stroke-width:1.5px;}"));
    let theme = theme.append(tagu::build::raw(
        ".poloto_text.poloto_legend{font-size:10px;}",
    ));
    let theme = theme.append(tagu::build::raw(
        ".poloto_line{stroke-dasharray:2;stroke-width:2;}",
    ));

    let plots = poloto::build::origin();
    let plot = poloto::build::plot("Roll ");
    let plot = plot.scatter(pts.iter());
    let plots = plots.chain(plot);

    let plot_initial = poloto::frame_build()
        .data(plots)
        .build_and_label(("Roll diff v Yaw diff", "Yaw C-W / °", "Roll C-W / °"))
        .append_to(poloto::header().append(theme))
        .render_string()
        .map_err(|e| format!("{e:?}"))?;

    let s = plot_initial.to_string();
    if let Some(filename) = cmd_args.write_svg() {
        let mut f = std::fs::File::create(filename)?;
        f.write_all(s.as_bytes())?;
    } else {
        println!("{s}");
    }

    cmd_ok()
}

//a Grid image
fn grid_image_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("grid_image")
        .about("From calibrate_from_grid")
        .long_about(GRID_IMAGE_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(grid_image_fn)));

    CmdArgs::add_arg_calibration_mapping(&mut build, true);
    CmdArgs::add_arg_read_image(&mut build, Some(1));
    CmdArgs::add_arg_write_image(&mut build, true);
    CmdArgs::add_arg_num_pts(&mut build);
    build
}

//fi grid_image_fn
fn grid_image_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let pms = cmd_args.calibration_mapping_to_pms();
    let camera = cmd_args.camera();
    let num_pts = cmd_args.use_pts(pms.len());

    //cb Create points for crosses for output image
    let mut pts = vec![];
    let n = 30;
    let n_f = n as f64;
    let c_f = n_f / 2.0;
    let rgba: Color = { [0, 0, 0, 255] }.into();
    for y in 0..=n {
        let y_f = (y as f64 - c_f) * 10.;
        for x in 0..=n {
            let x_f = (x as f64 - c_f) * 10.;
            let pt: Point3D = [x_f, y_f, 0.].into();
            pts.push((pt, rgba));
        }
    }
    let rgba: Color = { [100, 255, 100, 255] }.into();
    for pm in pms.mappings().iter().take(num_pts) {
        pts.push((pm.model(), rgba));
    }

    //cb Read source image and draw on it, write output image
    let pxys = cmd_args.calibration_mapping().get_pxys();
    let mut img = cmd_args.get_image_read_or_create()?;
    let c = &[255, 0, 0, 0].into();
    for p in pxys {
        img.draw_cross(&p, 5.0, c);
    }
    for (p, c) in &pts {
        let mapped = camera.map_model(*p);
        if mapped[0] < -10000.0 || mapped[0] > 10000.0 {
            continue;
        }
        if mapped[1] < -10000.0 || mapped[1] > 10000.0 {
            continue;
        }
        img.draw_cross(&mapped, 5.0, c);
    }
    img.write(cmd_args.write_img().unwrap())?;

    cmd_ok()
}

//a calibration command
//fp calibration
pub fn calibration_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("calibration")
        .about("Use a calibration mapping to calibrate a lens")
        .long_about(CAMERA_CALIBRATE_LONG_HELP)
        .version("0.1.0");

    let mut build = CommandBuilder::new(command, None);

    build.add_subcommand(locate_cmd());
    build.add_subcommand(orient_cmd());
    build.add_subcommand(lens_calibrate_cmd());
    build.add_subcommand(yaw_plot_cmd());
    build.add_subcommand(roll_plot_cmd());
    build.add_subcommand(grid_image_cmd());

    build
}
