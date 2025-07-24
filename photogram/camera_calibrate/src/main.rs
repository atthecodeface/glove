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

//a Imports
use clap::Command;

use ic_base::{Point3D, Result, RollYaw, TanXTanY};
use ic_camera::polynomial;
use ic_camera::polynomial::CalcPoly;
use ic_camera::{CameraDatabase, CameraInstance, CameraPolynomialCalibrate, CameraProjection};
use ic_cmdline::builder::{CommandArgs, CommandBuilder, CommandSet};
use ic_image::{Image, ImageRgb8};

//a Types
//a CmdArgs
//tp  CmdArgs
#[derive(Default)]
pub struct CmdArgs {
    cdb: Option<CameraDatabase>,
    cal: Option<String>,
    read_img: Vec<String>,
    write_img: Option<String>,
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
    build.add_arg(
        ic_cmdline::camera::camera_calibrate_arg(true),
        Box::new(|args, matches| {
            ic_cmdline::camera::get_camera_calibrate(matches).map(|v| args.cal = Some(v))
        }),
    );
    build.add_arg(
        ic_cmdline::image::image_read_arg(true, Some(1)),
        Box::new(|args, matches| {
            ic_cmdline::image::get_image_read_filenames(matches).map(|v| args.read_img = v)
        }),
    );
    build.add_arg(
        ic_cmdline::image::image_write_arg(true),
        Box::new(|args, matches| {
            ic_cmdline::image::get_opt_image_write_filename(matches).map(|v| args.write_img = v)
        }),
    );
    build
}

//fi calibrate_fn
fn calibrate_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let cdb = &cmd_args.cdb.as_ref().unwrap();

    let calibrate = CameraPolynomialCalibrate::from_json(&cdb, cmd_args.cal.as_ref().unwrap())?;

    let v = calibrate.get_pairings();
    let mut world_yaws = vec![];
    let mut camera_yaws = vec![];
    for (n, (grid, camera_rel_xyz, pxy_ry)) in v.iter().enumerate() {
        let camera_rel_txty: TanXTanY = camera_rel_xyz.into();
        let camera_rel_ry: RollYaw = camera_rel_txty.into();
        world_yaws.push(camera_rel_ry.yaw());
        camera_yaws.push(pxy_ry.yaw());
        if false {
            eprintln!(
                "{n} {grid} : {camera_rel_xyz} : {camera_rel_ry} : {pxy_ry} : camera_rel_ty {} : pxy_ty {}",
                camera_rel_ry.tan_yaw(),
                pxy_ry.tan_yaw()
            );
        }
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

    eprintln!("cal camera {}", calibrate.camera());
    let mut camera_lens = calibrate.camera().lens().clone();
    camera_lens.set_polys(stw, wts);
    let camera = CameraInstance::new(
        calibrate.camera().body().clone(),
        camera_lens,
        calibrate.camera().focus_distance(),
        calibrate.camera().position(),
        calibrate.camera().orientation(),
    );
    let m: Point3D = camera.camera_xyz_to_world_xyz([0., 0., -calibrate.distance()].into());
    let w: Point3D = camera.world_xyz_to_camera_xyz([0., 0., 0.].into());
    eprintln!("Camera {camera} focused on {m} world origin in camera {w}");

    let xy_pairs = calibrate.get_xy_pairings();
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
        for (_g, p) in &xy_pairs {
            img.draw_cross(*p, 5.0, c);
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

//a Images
/*a  Deprecated until we allow a project?
//fi image_grid_cmd
fn image_grid_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("image_grid").about("Read image and draw crosses on grid coordinates");
    let cmd = cmdline_args::camera::add_camera_arg(cmd, true);
    let cmd = cmdline_args::image::add_image_read_arg(cmd, true);
    let cmd = cmdline_args::image::add_image_write_arg(cmd, true);
    (cmd, image_grid_fn)
}

//fi image_grid_fn
fn image_grid_fn(cdb: CameraDatabase, matches: &clap::ArgMatches) -> Result<(), String> {
    let mut camera = cmdline_args::camera::get_camera(matches, &cdb)?;
    let camera_pt_z: Point3D = [-0.2, -1.2, -460.].into();
    let q = quat::new();
    camera = camera.placed_at(camera_pt_z);
    camera = camera.with_orientation(q.into());

    eprintln!("{camera}");
    let mut pts = vec![];
    let n = 40;
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
    if let Some(read_filename) = matches.get_one::<String>("read") {
        let mut img = image::read_image(read_filename)?;
        if let Some(write_filename) = matches.get_one::<String>("write") {
            for (p, c) in &pts {
                let mapped = camera.map_model(*p);
                // eprintln!("{mapped} {p} {c:?}");
                img.draw_cross(mapped, 5.0, c);
            }
            img.write(write_filename)?;
        }
    }
    Ok(())
}
 */

//a Main
//fi main
fn main() -> Result<()> {
    let command = Command::new("camera_calibrate")
        .about("Camera calibration tool")
        .version("0.1.0")
        .subcommand_required(true);

    let mut build = CommandBuilder::new(command, Some(Box::new(calibrate_fn)));
    build.add_arg(
        ic_cmdline::camera::camera_database_arg(true),
        Box::new(|args, matches| {
            ic_cmdline::camera::get_camera_database(matches).map(|v| args.cdb = Some(v))
        }),
    );

    build.add_subcommand(calibrate_cmd());

    let mut cmd_args = CmdArgs::default();
    let mut command: CommandSet<_> = build.into();
    command.execute_env(&mut cmd_args)?;
    Ok(())
}
