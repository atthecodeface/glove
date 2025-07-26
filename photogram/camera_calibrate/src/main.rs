//a Documentation
//! Test of camera calibration
//!
//! This is trying to calibrate to a grid
//!
//! The grid was captured on a Canon 5D mark IV, with a 50mm lens focuses on 'infinity'
//!
//! The camera is face-on to the grid (which is graph paper); the
//! approximate interscetions of 550 grid lines was capture as sensor pixel
//! coordinates XY and mm XY pairings. The grid is assumed to be at Z=0.
//!
//! Some of the pairings (given by pt_indices) are used to create a
//! ModelLineSet, which is a set of ModelLines (between grid points,
//! hence all in the plane Z=0) and the angles subtended by the
//! camera, given by the sensor pixel coordinates through the
//! camera/lens model from the database (i.e. this includes the lens
//! mapping)
//!
//! Note that this does not assume the orientation of position of the
//! camera; it purely uses the absolute pixel XY to relative pixel XY
//! to TanXTanY to RollYaw through the lens mapping to a new RollYaw
//! to a TanXTanY in model space to a unit direction vector.
//!
//! From this ModelLineSet a position in space is determined, using
//! the 'million possible points on a ModelLinetSubtended surface)
//! approach.
//!
//! This camera position is then optimized further by delta
//! adjustments in the ModelLineSet.
//!
//! From this 'known good position' the best orientation can be
//! determined, by creating quaternion orientations for every pair of
//! pairings in the pt_indices by:
//!
//!   1. Find the unit direction from the camera to both of the model points (A, B)
//!
//!   2. Find the the unit direction for the camera on it sensor (from the pairing)
//!
//!   3. Generate a quaternion qm that rotates model point direction A to the vector (0,0,1)
//!
//!   4. Generate a quaternion qc that rotates camera point direction A to the vector (0,0,1)
//!
//!   5. Apply qm to model point direction B to yield dj_m
//!
//!   6. Apply qc' to camera point direction B to yield dj_c
//!
//!   7. Note that dj_m and dj_c should have the same *yaw* but a different *roll*
//!
//!   8. Determine the roll required to map dj_m to dj_c
//!
//!   9. Generate quaternion qz which is the rotation around Z for the roll
//!
//!   10. Generate quaternion q = qm.qz.qc
//!
//!   11. Note that q transforms model point direction A to camera point direction A
//!
//!   12. Note that q transforms model point direction B to camera point direction B (if the yaws were identical)
//!
//!   13. Note hence that q is the orientation of a camera that matches the view of model points A and B
//!
//! The value 'q' is inaccurate if the *yaw* values are different -
//! i.e. if the angle subtended by the line between the two points on
//! the camera does not match the angle subtended by the line between
//! the two points in model space as seen by the camera at its given location.
//!
//! The value of 'q' for *every* pair of pairings (A to B, and also B
//! to A) is generated, and an average of these quaternions is used as
//! the orientation of the camera
//!
//! Given the position and orientation of the camera the unit
//! direction vector to every model point from the camera can be
//! determined, and converted to a *roll* and *yaw*. The corresponding
//! camera sensor direction (potentially without going through the lens mapping)
//! can be determined, and presented also as a *roll* and *yaw*.
//!
//! A graph of camera yaw versus model yaw can be produced; if no lens
//! mapping had been used the this should be approximately a single
//! curve that is the polynomial for the lens (mapping yaw in camera
//! to yaw in model space).
//!
//! However, if the *centre* of the camera (upon which the absolute
//! camera sensor XY to camera unit direction vectors depend) has an
//! incorrect value (is the lens centred on the mid-point of the
//! sensor?) then the curve for the yaw-yaw for the camera points in
//! the upper right quadrant of the sensor will have approximately the
//! same shape, but will have a differentoffset, to that from the
//! lower right quadrant.
//!
//! So here we plot *four* graphs, one for each quadrant.
//!
//! For *all* of the points together a pair of polynomials (one
//! camera-to-model, the other the inverse) are generated
//!
//! The process to calibrate the camera is thus to:
//!
//!  1. Reset its lens mapping polynomial
//!
//!  2. Reset the centre of the lens (to the middle of the sensor)
//!
//!  3. Run the program and inspect the graphs
//!
//!  4. Adjust the centre of the sensor if the four graphs are
//!     noticeable offset from each other; repeat from step 3
//!
//!  5. Once the graphs are all deemed reasonable, copy the
//!     polynomials calculated in to the lens mapping.
//!
//!  6. Rerun, and the graphs should be near identity, and the
//!     calibration is complete.
//!
//! # Using the tool with a grid
//!
//! If you start with a photo of a grid, then a 'mappings' for each of
//! the grid positions should be gnerated - this is (x,y,0, px, py).
//!
//! Then a CameraPolynomialCalibrateDesc can be generated, which is a
//! basic camera description plus the mappings. This provides the
//! camera body, lens, and focus distance for the image; plus position
//! and orientation, which will be ignored here.
//!
//! With this description use:
//!
//!   camera_calibrate (--db ...) locate -c <desc.json> > <located_desc.json>
//!
//! This will use some points (argh!) to locate the camera as best as possible
//!
//! With a *located* camera, the orientation can be determined.
//!
//!   camera_calibrate (--db ...) orient -c <located_desc.json> > <oriented_desc.json>
//!
//! This will use some points (argh!) to orient the camera as best as possible, given its location
//!
//! Now the lens can be calibrated
//!
//! # Calibrating a lens
//!
//! A lens is calibrated using a set of points in 3D and the positions those map to on a camera, given the camera has been optimally located and oriented.
//!
//! This will also generate an SVG file with the plot of camera Yaw
//! versus world Yaw, plus the polynomials describing this
//! (sensor-to-world, and world-to-sensor):
//!
//!   camera_calibrate (--db ...) lens_calibrate -c <desc.json> > <plot.svg>
//!
//! The lens calibration can be copied to the camera_db.json file if
//! required. Bear in mind that this polynomial is relative to the
//! *current* polynomial provided for the lens; the normal process is
//! to run the location, orientation, and calibration with a *linear*
//! lens mapping, then to copy that lens mapping to the database, and
//! rerun to ensure the SVG shows basically straight lines parallel to
//! the X axis.
//!
//! # Generating a test image
//!
//! Once the calibration is complete, a verification image can be generated
//!
//!   camera_calibrate (--db ...) grid_image -c <desc.json> -r <src image> -w <output image>
//!
//! This draws black crosses on the image for grid intersections
//! (x,y,0) for a range of X and Y; green crosses for each (x,y,z) in
//! the description JSON mappings, and *red* crosses for the pxy of
//! each of those points in the description JSON mappings.
//!
//! If the calibration is very good then the green and red crosses will overlap
//!

//a Imports
use clap::Command;
use geo_nd::{quat, Quaternion, Vector};

use ic_base::{Point3D, Quat, Result, RollYaw, TanXTanY};
use ic_camera::polynomial;
use ic_camera::polynomial::CalcPoly;
use ic_camera::{CalibrationMapping, CameraDatabase, CameraInstance, CameraProjection};
use ic_cmdline::builder::{CommandArgs, CommandBuilder, CommandSet};
use ic_image::{Color, Image, ImageRgb8};
use ic_mapping::{ModelLineSet, NamedPointSet, PointMappingSet};

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
graph and a polynomial of best fit (with the extra assertion that the
point (0,0) is on the Yaw/Yaw graph).

Actually two polynomials are generated - one forward (wts) and one
backward (stw); these should be used in a camera_db JSON file.

The plot that is generated is an SVG file showing Yaw/Yaw-1 - bear in
mind that any lens mapping specified for the camera in the database is
used, so that a perfectly calibrated camera/lens with a perfect
mapping file will have straight lines on the X axis. Furthermore,
there are *four* graphs overlaid, using different colors - one for
each quadrant of the camera sensor; also the polynomial of best fit is
plotted too.";

//hi ROLL_PLOT_LONG_HELP
const ROLL_PLOT_LONG_HELP: &str = "\

Generate a plot for all the mappings of model roll versus world roll
";

//hi GRID_IMAGE_LONG_HELP
const GRID_IMAGE_LONG_HELP: &str = "\
This tool uses the provided camera description and mappings, and
overlays an image with *red* crosses showing the specified coordinates of
each mapping and the derived (i.e. post-camera/lens mapping) positions
of those mappings with *green* crosses.

It also draws black crosses for a range of (x,y,0) values.";

//a Types
//a CmdArgs
//tp CmdArgs
#[derive(Default)]
pub struct CmdArgs {
    verbose: bool,
    cdb: Option<CameraDatabase>,
    camera: CameraInstance,
    mapping: Option<CalibrationMapping>,
    read_img: Vec<String>,
    write_img: Option<String>,
    use_pts: usize,
}

//ip CmdArgs
impl CmdArgs {
    fn get_cdb(&self) -> &CameraDatabase {
        self.cdb.as_ref().unwrap()
    }
    fn set_verbose(&mut self, verbose: bool) -> Result<()> {
        self.verbose = verbose;
        Ok(())
    }
    fn set_cdb(&mut self, cdb: CameraDatabase) -> Result<()> {
        self.cdb = Some(cdb);
        Ok(())
    }
    fn set_camera(&mut self, camera: CameraInstance) -> Result<()> {
        self.camera = camera;
        Ok(())
    }
    fn set_mapping(&mut self, mapping: CalibrationMapping) -> Result<()> {
        self.mapping = Some(mapping);
        Ok(())
    }
    fn set_read_img(&mut self, v: Vec<String>) -> Result<()> {
        self.read_img = v;
        Ok(())
    }
    fn set_write_img(&mut self, s: &str) -> Result<()> {
        self.write_img = Some(s.to_owned());
        Ok(())
    }
    fn set_use_pts(&mut self, v: usize) -> Result<()> {
        self.use_pts = v;
        Ok(())
    }
    fn show_step<S>(&self, s: S)
    where
        S: std::fmt::Display,
    {
        if self.verbose {
            eprintln!("\n{s}");
        }
    }
    fn if_verbose<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        if self.verbose {
            f()
        }
    }
}

//ip CommandArgs for CmdArgs
impl CommandArgs for CmdArgs {
    type Error = ic_base::Error;
    type Value = ();
}

//a Calibrate
//fi calibrate_cmd
fn calibrate_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("calibrate")
        .about("Read image and draw crosses on grid coordinates")
        .long_about(
            "This uses the camera calibration JSON file in conjunction with a camera body/lens and focus distance to generate the correct focal length and tan-tan mapping for the lens as world-to-screen (and vice-versa) polynomials. The camera calibration JSON file includes 'mappings' that is a list of (grid xmm, grid ymm, x pixel, y pixel) tuples each being the mapping of a grid x,y to a frame pixel x,y on an image. If read and write imnages are provided then the immage is read and red crosses superimposed on the image at the post-calibrated points using the provided grid x,y points as sources (so they should align with the actual grid points on the image)")
        .version("0.1.0");

    let mut build = CommandBuilder::new(command, Some(Box::new(calibrate_fn)));

    ic_cmdline::camera::add_arg_calibration_mapping(&mut build, CmdArgs::set_mapping, true);
    ic_cmdline::image::add_arg_read_img(&mut build, CmdArgs::set_read_img, false, Some(1));
    ic_cmdline::image::add_arg_write_img(&mut build, CmdArgs::set_write_img, false);
    build
}

//fi calibrate_fn
fn calibrate_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let calibrate = cmd_args.mapping.as_ref().unwrap();

    let v = calibrate.get_pairings(&cmd_args.camera);
    let mut world_yaws = vec![];
    let mut camera_yaws = vec![];
    for (n, (grid, camera_rel_xyz, pxy_ry)) in v.iter().enumerate() {
        let camera_rel_txty: TanXTanY = camera_rel_xyz.into();
        let camera_rel_ry: RollYaw = camera_rel_txty.into();
        world_yaws.push(camera_rel_ry.yaw());
        camera_yaws.push(pxy_ry.yaw());
        cmd_args.if_verbose(|| {
            eprintln!(
                "{n} {grid} : {camera_rel_xyz} : {camera_rel_ry} : {pxy_ry} : camera_rel_ty {} : pxy_ty {}",
                camera_rel_ry.tan_yaw(),
                pxy_ry.tan_yaw()
            );
        });
    }

    let poly_degree = 5;
    let wts = polynomial::min_squares_dyn(poly_degree, &world_yaws, &camera_yaws);
    let stw = polynomial::min_squares_dyn(poly_degree, &camera_yaws, &world_yaws);
    let (max_sq_err, max_n, sq_err) =
        polynomial::square_error_in_y(&wts, &world_yaws, &camera_yaws);
    let avg_sq_err = sq_err / (world_yaws.len() as f64);

    if false {
        for i in 0..world_yaws.len() {
            let wy = world_yaws[i];
            let cy = camera_yaws[i];
            eprintln!(
                "{i} {wy} : {} : {cy} : {} : {wy}",
                wts.calc(wy),
                stw.calc(cy)
            );
        }
    }
    eprintln!(" wts: {wts:?}");
    eprintln!(" stw: {stw:?}");
    eprintln!(" avg sq_err: {avg_sq_err:.4e} max_sq_err {max_sq_err:.4e} max_n {max_n}");

    eprintln!("cal camera {}", cmd_args.camera);
    let mut camera = cmd_args.camera.clone();
    let mut camera_lens = camera.lens().clone();
    camera_lens.set_polys(stw, wts);
    camera.set_lens(camera_lens);

    //    let m: Point3D = camera.camera_xyz_to_world_xyz([0., 0., -calibrate.distance()].into());
    //    let w: Point3D = camera.world_xyz_to_camera_xyz([0., 0., 0.].into());
    //    eprintln!("Camera {camera} focused on {m} world origin in camera {w}");

    let pxys = calibrate.get_pxys();
    let mut pts = vec![];
    let n = 30;
    let n_f = n as f64;
    let c_f = n_f / 2.0;
    for y in 0..=n {
        let y_f = (y as f64 - c_f) * 10.;
        for x in 0..=n {
            let x_f = (x as f64 - c_f) * 10.;
            let pt: Point3D = [x_f, y_f, 0.].into();
            let rgba = [255, 255, 255, 255].into();
            pts.push((pt, rgba));
        }
    }

    if !cmd_args.read_img.is_empty() && cmd_args.write_img.is_some() {
        let mut img = ImageRgb8::read_image(&cmd_args.read_img[0])?;
        let c = &[255, 0, 0, 0].into();
        for p in pxys.into_iter() {
            img.draw_cross(p, 5.0, c);
        }
        for (p, c) in &pts {
            let mapped = camera.map_model(*p);
            if false {
                let xyz = camera.world_xyz_to_camera_xyz(*p);
                let txy = camera.world_xyz_to_camera_txty(*p);
                eprintln!("{mapped} {xyz} {txy} {p} {c:?}");
            }
            img.draw_cross(mapped, 5.0, c);
        }
        img.write(cmd_args.write_img.as_ref().unwrap())?;
    }
    Ok(())
}

//a Setup
//fi find_closest_n
fn find_closest_n(calibrate: &CalibrationMapping, pts: &[(f64, f64, f64)]) -> Vec<usize> {
    //cb Add calibrations to NamedPointSet and PointMappingSet
    let v = calibrate.get_xyz_pairings();
    let mut closest_n = vec![];
    for pt in pts {
        let mut closest = (0, 1.0E20);
        let pt = [pt.0, pt.1, pt.2].into();
        for (n, (model_xyz, _)) in v.iter().enumerate() {
            let d = model_xyz.distance(&pt);
            if d < closest.1 {
                closest = (n, d);
            }
        }
        closest_n.push(closest.0);
    }
    closest_n
}

//fi setup
fn setup(calibrate: &CalibrationMapping) -> (NamedPointSet, PointMappingSet) {
    let v = calibrate.get_xyz_pairings();
    let mut nps = NamedPointSet::default();
    let mut pms = PointMappingSet::default();

    //cb Add calibrations to NamedPointSet and PointMappingSet
    for (n, (model_xyz, pxy_abs)) in v.into_iter().enumerate() {
        let name = n.to_string();
        let color = [255, 255, 255, 255].into();
        nps.add_pt(name.clone(), color, Some(model_xyz), 0.);
        pms.add_mapping(&nps, &name, &pxy_abs, 0.);
    }
    (nps, pms)
}

//a Locate
fn locate_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("locate")
        .about("Determine an optimal location from a calibration description")
        .long_about(LOCATE_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(locate_fn)));
    ic_cmdline::camera::add_arg_calibration_mapping(&mut build, CmdArgs::set_mapping, true);
    ic_cmdline::add_arg_usize(
        &mut build,
        "num_pts",
        Some('n'),
        "Number of points to use (from start of mapping); if not specified, use all",
        None,
        CmdArgs::set_use_pts,
        false,
    );
    build
}

//fi locate_fn
fn locate_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let calibrate = cmd_args.mapping.as_ref().unwrap();
    cmd_args.camera.set_position([0., 0., 0.].into());
    cmd_args.camera.set_orientation(Quat::default());

    //cb Set up HashMaps and collections
    let n = {
        if cmd_args.use_pts == 0 {
            calibrate.len()
        } else {
            cmd_args.use_pts
        }
    };
    let n = n.min(calibrate.len());
    let closest_n: Vec<usize> = (0..n).into_iter().collect();
    let (_nps, pms) = setup(calibrate);

    //cb For required pairings, display data
    cmd_args.show_step("Using the following mappings ([n] [world] : [pxy] : [world_dir]");
    for pm_n in &closest_n {
        let pm = &pms.mappings()[*pm_n];
        let n = pm.name();
        let grid_xyz = pm.model();
        // Px Abs -> Px Rel -> TxTy -> lens mapping
        let pxy_abs = pm.screen();
        let txty = cmd_args.camera.px_abs_xy_to_camera_txty(pxy_abs);
        let grid_dir = txty.to_unit_vector();
        cmd_args.if_verbose(|| {
            eprintln!(">> {n} {grid_xyz} : {pxy_abs} : {grid_dir}",);
        });
    }

    //cb Create ModelLineSet
    let mut mls = ModelLineSet::new(&cmd_args.camera);

    for n0 in &closest_n {
        let pm0 = &pms.mappings()[*n0];
        let dir0 = cmd_args
            .camera
            .px_abs_xy_to_camera_txty(pm0.screen())
            .to_unit_vector();
        for n1 in &closest_n {
            if n0 == n1 {
                continue;
            }
            let pm1 = &pms.mappings()[*n1];
            let dir1 = cmd_args
                .camera
                .px_abs_xy_to_camera_txty(pm1.screen())
                .to_unit_vector();
            let cos_theta = dir0.dot(&dir1);
            let angle = cos_theta.acos();
            let _ = mls.add_line_of_models(pm0.model(), pm1.model(), angle);
        }
    }

    //cb Find best position given ModelLineSet
    // Find best location 'p' for camera
    let (best_cam_pos, e) = mls.find_best_min_err_location(&|p| p[2] > 0., 1000, 1000);
    cmd_args.if_verbose(|| {
        eprintln!("{best_cam_pos} {e}",);
    });

    cmd_args.camera.set_position(best_cam_pos);

    println!("{}", cmd_args.camera.to_json()?);
    Ok(())
}

//a Orient
fn orient_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("orient")
        .about("Determine an optimal orientation from a calibration description")
        .long_about(ORIENT_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(orient_fn)));
    ic_cmdline::camera::add_arg_calibration_mapping(&mut build, CmdArgs::set_mapping, true);

    ic_cmdline::add_arg_usize(
        &mut build,
        "num_pts",
        Some('n'),
        "Number of points to use (from start of mapping); if not specified, use all",
        None,
        CmdArgs::set_use_pts,
        false,
    );

    build
}

//fi orient_fn
fn orient_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let calibrate = cmd_args.mapping.as_ref().unwrap();

    //cb Set up 'cam' as the camera; use its position (unless otherwise told?)
    let mut camera = cmd_args.camera.clone();
    camera.set_orientation(Quat::default());

    //cb Set up HashMaps and collections
    let n = {
        if cmd_args.use_pts == 0 {
            calibrate.len()
        } else {
            cmd_args.use_pts
        }
    };
    let n = n.min(calibrate.len());
    let closest_n: Vec<usize> = (0..n).into_iter().collect();
    let (_nps, pms) = setup(calibrate);

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
            cmd_args.if_verbose(|| {
                eprintln!("{q}");
            });

            qs.push((1., q.into()));
        }
    }

    cmd_args.show_step("Calculate average orientation");
    let qr: Quat = quat::weighted_average_many(qs.iter().copied()).into();
    camera.set_orientation(qr);
    cmd_args.if_verbose(|| {
        eprintln!("{camera}");
    });

    println!("{}", camera.to_json()?);
    Ok(())
}

//a Lens calibrate
fn lens_calibrate_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("lens_calibrate")
        .about("From calibrate_from_grid")
        .long_about(LENS_CALIBRATE_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(lens_calibrate_fn)));

    ic_cmdline::camera::add_arg_calibration_mapping(&mut build, CmdArgs::set_mapping, true);

    build
}

//fi lens_calibrate_fn
fn lens_calibrate_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let calibrate = cmd_args.mapping.as_ref().unwrap();
    let camera = &cmd_args.camera;

    let yaw_range_min = 0.05;
    let yaw_range_max = 0.35;
    let num_pts = 4;

    //cb Set up HashMaps and collections
    let (_nps, pms) = setup(calibrate);

    //cb Calculate Roll/Yaw for each point given camera
    let mut pts = [vec![], vec![], vec![], vec![]];
    let mut world_yaws = vec![];
    let mut camera_yaws = vec![];
    for pm in pms.mappings() {
        let model_txty = camera.world_xyz_to_camera_txty(pm.model());
        let cam_txty = camera.px_abs_xy_to_camera_txty(pm.screen());

        let model_ry: RollYaw = model_txty.into();
        let cam_ry: RollYaw = cam_txty.into();

        if cam_ry.yaw() > yaw_range_max {
            continue;
        }

        if cam_ry.yaw() > yaw_range_min {
            world_yaws.push(model_ry.yaw());
            camera_yaws.push(cam_ry.yaw());
        }
        if (model_ry.yaw() / cam_ry.yaw()) > 1.2 {
            continue;
        }
        let mut quad = 0;
        if cam_ry.cos_roll() < 0.0 {
            // X < 0
            quad += 1;
        }
        if cam_ry.sin_roll() < 0.0 {
            // Y < 0
            quad += 2;
        }
        if cam_ry.yaw() > yaw_range_min {
            pts[quad].push((cam_ry.yaw(), model_ry.yaw() / cam_ry.yaw() - 1.0));
        }
    }

    //cb Calculate Polynomials for camera-to-world and vice-versa
    // encourage it to go through the origin
    let poly_degree = 5;
    for _ in 0..10 {
        world_yaws.push(0.);
        camera_yaws.push(0.);
    }
    let mut wts = polynomial::min_squares_dyn(poly_degree, &world_yaws, &camera_yaws);
    let mut stw = polynomial::min_squares_dyn(poly_degree, &camera_yaws, &world_yaws);
    wts[0] = 0.0;
    stw[0] = 0.0;
    let (max_sq_err, max_n, sq_err) =
        polynomial::square_error_in_y(&wts, &world_yaws, &camera_yaws);
    let avg_sq_err = sq_err / (world_yaws.len() as f64);

    eprintln!(" \"wts_poly\": {wts:?},");
    eprintln!(" \"stw_poly\": {stw:?},");
    eprintln!(" avg sq_err: {avg_sq_err:.4e} max_sq_err {max_sq_err:.4e} max_n {max_n}");

    //cb Plot 4 graphs for quadrants and one for the polynomial
    use poloto::build::PlotIterator;
    let plots = poloto::build::origin();
    let plot = poloto::build::plot("Quad x>0 y>0");
    let plot = plot.scatter(pts[0].iter());
    let plots = plots.chain(plot);
    let plot = poloto::build::plot("Quad x<0 y>0");
    let plot = plot.scatter(pts[1].iter());
    let plots = plots.chain(plot);
    let plot = poloto::build::plot("Quad x>0 y<0");
    let plot = plot.scatter(pts[2].iter());
    let plots = plots.chain(plot);
    let plot = poloto::build::plot("Quad x<0 y<0");
    let plot = plot.scatter(pts[3].iter());
    let plots = plots.chain(plot);

    let mut wts_poly_pts = vec![];
    for i in 2..=100 {
        let world = (i as f64) * 0.40 / 100.0;
        let sensor = stw.calc(world);
        wts_poly_pts.push((world, sensor / world - 1.0));
    }
    let plot = poloto::build::plot("Wts Poly");
    let plot = plot.scatter(wts_poly_pts.iter());
    let plots = plots.chain(plot);

    let plot_initial = poloto::frame_build()
        .data(plots)
        .build_and_label(("Yaw v Yaw", "x", "y"))
        .append_to(poloto::header().light_theme())
        .render_string()
        .map_err(|e| format!("{e:?}"))?;
    println!("{plot_initial}");

    Ok(())
}

//a Roll plot
fn roll_plot_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("roll_plot")
        .about("Plot roll of model versus roll of camera")
        .long_about(ROLL_PLOT_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(roll_plot_fn)));

    ic_cmdline::camera::add_arg_calibration_mapping(&mut build, CmdArgs::set_mapping, true);

    build
}

//fi roll_plot_fn
fn roll_plot_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let calibrate = cmd_args.mapping.as_ref().unwrap();
    let camera = &cmd_args.camera;

    let yaw_range_min = 0.05;
    let yaw_range_max = 0.35;
    let num_pts = 4;

    //cb Set up HashMaps and collections
    let (_nps, pms) = setup(calibrate);

    //cb Calculate Roll/Yaw for each point given camera
    let mut pts = vec![];
    for pm in pms.mappings() {
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
    use poloto::build::PlotIterator;
    let plots = poloto::build::origin();
    let plot = poloto::build::plot("Roll ");
    let plot = plot.scatter(pts.iter());
    let plots = plots.chain(plot);

    let plot_initial = poloto::frame_build()
        .data(plots)
        .build_and_label(("Roll diff v Yaw", "Yaw (deg)", "Roll C-W  (deg)"))
        .append_to(poloto::header().light_theme())
        .render_string()
        .map_err(|e| format!("{e:?}"))?;
    println!("{plot_initial}");

    Ok(())
}

//a Grid image
fn grid_image_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("grid_image")
        .about("From calibrate_from_grid")
        .long_about(GRID_IMAGE_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(grid_image_fn)));

    ic_cmdline::camera::add_arg_calibration_mapping(&mut build, CmdArgs::set_mapping, true);
    ic_cmdline::image::add_arg_read_img(&mut build, CmdArgs::set_read_img, true, Some(1));
    ic_cmdline::image::add_arg_write_img(&mut build, CmdArgs::set_write_img, true);
    build
}

//fi grid_image_fn
fn grid_image_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let calibrate = cmd_args.mapping.as_ref().unwrap();
    let camera = &cmd_args.camera;

    //cb Set up HashMaps and collections
    let (_nps, pms) = setup(calibrate);

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
    for pm in pms.mappings() {
        pts.push((pm.model(), rgba));
    }

    //cb Read source image and draw on it, write output image
    let pxys = calibrate.get_pxys();
    let mut img = ImageRgb8::read_image(&cmd_args.read_img[0])?;
    let c = &[255, 0, 0, 0].into();
    for p in pxys {
        img.draw_cross(p, 5.0, c);
    }
    for (p, c) in &pts {
        let mapped = camera.map_model(*p);
        if mapped[0] < -10000.0 || mapped[0] > 10000.0 {
            continue;
        }
        if mapped[1] < -10000.0 || mapped[1] > 10000.0 {
            continue;
        }
        img.draw_cross(mapped, 5.0, c);
    }
    img.write(cmd_args.write_img.as_ref().unwrap())?;

    Ok(())
}

//a Main
//fi main
fn main() -> Result<()> {
    let command = Command::new("camera_calibrate")
        .about("Camera calibration tool")
        .long_about(CAMERA_CALIBRATE_LONG_HELP)
        .version("0.1.0");

    let mut build = CommandBuilder::new(command, Some(Box::new(calibrate_fn)));
    ic_cmdline::add_arg_verbose(&mut build, CmdArgs::set_verbose);
    ic_cmdline::camera::add_arg_camera_database(&mut build, CmdArgs::set_cdb, true);
    ic_cmdline::camera::add_arg_camera(&mut build, CmdArgs::get_cdb, CmdArgs::set_camera, true);

    build.add_subcommand(calibrate_cmd());
    build.add_subcommand(locate_cmd());
    build.add_subcommand(orient_cmd());
    build.add_subcommand(lens_calibrate_cmd());
    build.add_subcommand(roll_plot_cmd());
    build.add_subcommand(grid_image_cmd());
    //    build.add_subcommand(grid_calibrate_cmd());

    let mut cmd_args = CmdArgs::default();
    let mut command: CommandSet<_> = build.into();
    command.execute_env(&mut cmd_args)?;
    Ok(())
}
