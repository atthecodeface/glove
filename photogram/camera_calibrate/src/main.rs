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
use std::collections::HashMap;
use std::rc::Rc;

use clap::Command;
use geo_nd::{quat, Quaternion, Vector};

use ic_base::{Point3D, Quat, Result, RollYaw, TanXTanY};
use ic_camera::polynomial;
use ic_camera::polynomial::CalcPoly;
use ic_camera::{CameraDatabase, CameraInstance, CameraPolynomialCalibrate, CameraProjection};
use ic_cmdline::builder::{CommandArgs, CommandBuilder, CommandSet};
use ic_image::{Color, Image, ImageRgb8};
use ic_mapping::{ModelLineSet, NamedPoint, NamedPointSet, PointMappingSet};

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
        ic_cmdline::image::image_read_arg(false, Some(1)),
        Box::new(|args, matches| {
            ic_cmdline::image::get_image_read_filenames(matches).map(|v| args.read_img = v)
        }),
    );
    build.add_arg(
        ic_cmdline::image::image_write_arg(false),
        Box::new(|args, matches| {
            ic_cmdline::image::get_opt_image_write_filename(matches).map(|v| args.write_img = v)
        }),
    );
    build
}

//fi calibrate_fn
fn calibrate_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let cdb = &cmd_args.cdb.as_ref().unwrap();

    let calibrate = CameraPolynomialCalibrate::from_json(cdb, cmd_args.cal.as_ref().unwrap())?;

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
    //    let m: Point3D = camera.camera_xyz_to_world_xyz([0., 0., -calibrate.distance()].into());
    //    let w: Point3D = camera.world_xyz_to_camera_xyz([0., 0., 0.].into());
    //    eprintln!("Camera {camera} focused on {m} world origin in camera {w}");

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

//a Grid locate
fn grid_locate_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("grid_locate")
        .about("Locate a camera given a grid calibration")
        .long_about(
            "
Given a subset of the calibration points, find the best location for the camera.
",
        );

    let mut build = CommandBuilder::new(command, Some(Box::new(grid_locate_fn)));
    build.add_arg(
        ic_cmdline::camera::camera_calibrate_arg(true),
        Box::new(|args, matches| {
            ic_cmdline::camera::get_camera_calibrate(matches).map(|v| args.cal = Some(v))
        }),
    );
    build
}

//fi grid_locate_fn
fn grid_locate_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let cdb = &cmd_args.cdb.as_ref().unwrap();

    //cb Load Calibration JSON
    let mut calibrate = CameraPolynomialCalibrate::from_json(cdb, cmd_args.cal.as_ref().unwrap())?;

    //cb Set up 'cam' as the camera
    let mut cam = calibrate.camera().clone();
    cam.set_position([0., 0., 0.].into());
    cam.set_orientation(Quat::default());

    //cb Set up HashMaps and collections
    let mut grid_dir_of_xy = HashMap::new();

    let pt_indices = &[(40, -40), (-40, -40), (40, 40), (-40, 40)];

    let mut nps = NamedPointSet::default();
    let mut pms = PointMappingSet::default();
    let mut nps_of_pts: HashMap<(isize, isize), Rc<NamedPoint>> = HashMap::default();

    //cb Add calibrations to NamedPointSet and PointMappingSet
    let v = calibrate.get_xy_pairings();
    for (grid_xy, pxy_abs) in v.iter() {
        let name = format!("{},{}", grid_xy[0] as isize, grid_xy[1] as isize);
        let model_xyz: Point3D = [grid_xy[0], grid_xy[1], 0.].into();
        let color = [255, 255, 255, 255].into();
        nps.add_pt(name.clone(), color, Some(model_xyz), 0.);
        pms.add_mapping(&nps, &name, pxy_abs, 0.);
    }

    //cb Add all pairings to grid_dir_of_xy
    for (n, (grid_xy, pxy_abs)) in v.iter().enumerate() {
        let txty = cam.px_abs_xy_to_camera_txty(*pxy_abs);
        let grid_dir = txty.to_unit_vector();
        grid_dir_of_xy.insert((grid_xy[0] as isize, grid_xy[1] as isize), (n, grid_dir));
    }

    //cb For required pairings, display data
    for p in pt_indices {
        let name = format!("{},{}", p.0, p.1);
        if let Some(np) = nps.get_pt(&name) {
            nps_of_pts.insert(*p, np);
            let (n, _grid_dir) = grid_dir_of_xy.get(p).unwrap();
            let (grid_xy, pxy_abs) = &v[*n];
            // Px Abs -> Px Rel -> TxTy -> lens mapping
            let txty = cam.px_abs_xy_to_camera_txty(*pxy_abs);
            let grid_dir = txty.to_unit_vector();
            eprintln!("{n} {grid_xy} : {pxy_abs} : {grid_dir}",);
        }
    }

    //cb Create ModelLineSet
    let pairings = calibrate.get_xy_pairings();

    let mut mls = ModelLineSet::new(&cam);

    for p0 in pt_indices {
        let (n0, grid_dir) = grid_dir_of_xy.get(p0).unwrap();
        let dir0 = *grid_dir;
        let _pm0 = pms.mapping_of_np(nps_of_pts.get(p0).unwrap()).unwrap();
        for p1 in pt_indices {
            if *p1 == *p0 {
                continue;
            }
            let (n1, grid_dir) = grid_dir_of_xy.get(p1).unwrap();
            let dir1 = *grid_dir;
            let cos_theta = dir0.dot(&dir1);
            let angle = cos_theta.acos();
            let model0_xy = pairings[*n0].0;
            let model1_xy = pairings[*n1].0;
            let model0_xyz = [model0_xy[0], model0_xy[1], 0.].into();
            let model1_xyz = [model1_xy[0], model1_xy[1], 0.].into();
            let _ = mls.add_line_of_models(model0_xyz, model1_xyz, angle);
        }
    }

    //cb Find best position given ModelLineSet
    // Find best location 'p' for camera
    let (best_cam_pos, e) = mls.find_best_min_err_location(&|p| p[2] > 0., 1000, 1000);
    eprintln!("{best_cam_pos} {e}",);

    calibrate.camera_mut().set_position(best_cam_pos);

    println!("{}", calibrate.clone().to_desc_json()?);
    Ok(())
}

//a Grid orient
fn grid_orient_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("grid_orient")
        .about("From calibrate_from_grid")
        .long_about("");

    let mut build = CommandBuilder::new(command, Some(Box::new(grid_orient_fn)));
    build.add_arg(
        ic_cmdline::camera::camera_calibrate_arg(true),
        Box::new(|args, matches| {
            ic_cmdline::camera::get_camera_calibrate(matches).map(|v| args.cal = Some(v))
        }),
    );
    build.add_arg(
        ic_cmdline::image::image_read_arg(false, Some(1)),
        Box::new(|args, matches| {
            ic_cmdline::image::get_image_read_filenames(matches).map(|v| args.read_img = v)
        }),
    );
    build.add_arg(
        ic_cmdline::image::image_write_arg(false),
        Box::new(|args, matches| {
            ic_cmdline::image::get_opt_image_write_filename(matches).map(|v| args.write_img = v)
        }),
    );
    build
}

//fi grid_orient_fn
fn grid_orient_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let cdb = &cmd_args.cdb.as_ref().unwrap();

    //cb Load Calibration JSON
    let mut calibrate = CameraPolynomialCalibrate::from_json(cdb, cmd_args.cal.as_ref().unwrap())?;

    //cb Set up 'cam' as the camera; use its position (unless otherwise told?)
    let mut cam = calibrate.camera().clone();
    cam.set_orientation(Quat::default());

    //cb Set up HashMaps and collections
    let mut grid_dir_of_xy = HashMap::new();

    let pt_indices = &[(40, -40), (-40, -40), (40, 40), (-40, 40)];

    let mut nps = NamedPointSet::default();
    let mut pms = PointMappingSet::default();
    let mut nps_of_pts: HashMap<(isize, isize), Rc<NamedPoint>> = HashMap::default();

    //cb Add calibrations to NamedPointSet and PointMappingSet
    let v = calibrate.get_xy_pairings();
    for (grid_xy, pxy_abs) in v.iter() {
        let name = format!("{},{}", grid_xy[0] as isize, grid_xy[1] as isize);
        let model_xyz: Point3D = [grid_xy[0], grid_xy[1], 0.].into();
        let color = [255, 255, 255, 255].into();
        nps.add_pt(name.clone(), color, Some(model_xyz), 0.);
        pms.add_mapping(&nps, &name, pxy_abs, 0.);
    }

    //cb Add all pairings to grid_dir_of_xy
    for (n, (grid_xy, pxy_abs)) in v.iter().enumerate() {
        let txty = cam.px_abs_xy_to_camera_txty(*pxy_abs);
        let grid_dir = txty.to_unit_vector();
        grid_dir_of_xy.insert((grid_xy[0] as isize, grid_xy[1] as isize), (n, grid_dir));
    }

    //cb For required pairings, display data
    for p in pt_indices {
        let name = format!("{},{}", p.0, p.1);
        if let Some(np) = nps.get_pt(&name) {
            nps_of_pts.insert(*p, np);
            let (n, _grid_dir) = grid_dir_of_xy.get(p).unwrap();
            let (grid_xy, pxy_abs) = &v[*n];
            // Px Abs -> Px Rel -> TxTy -> lens mapping
            let txty = cam.px_abs_xy_to_camera_txty(*pxy_abs);
            let grid_dir = txty.to_unit_vector();
            eprintln!("{n} {grid_xy} : {pxy_abs} : {grid_dir}",);
        }
    }

    //cb Create ModelLineSet
    let pairings = calibrate.get_xy_pairings();

    let mut mls = ModelLineSet::new(&cam);

    for p0 in pt_indices {
        let (n0, grid_dir) = grid_dir_of_xy.get(p0).unwrap();
        let dir0 = *grid_dir;
        let _pm0 = pms.mapping_of_np(nps_of_pts.get(p0).unwrap()).unwrap();
        for p1 in pt_indices {
            if *p1 == *p0 {
                continue;
            }
            let (n1, grid_dir) = grid_dir_of_xy.get(p1).unwrap();
            let dir1 = *grid_dir;
            let cos_theta = dir0.dot(&dir1);
            let angle = cos_theta.acos();
            let model0_xy = pairings[*n0].0;
            let model1_xy = pairings[*n1].0;
            let model0_xyz = [model0_xy[0], model0_xy[1], 0.].into();
            let model1_xyz = [model1_xy[0], model1_xy[1], 0.].into();
            let _ = mls.add_line_of_models(model0_xyz, model1_xyz, angle);
        }
    }

    //cb Find best orientation given position
    // We can get N model direction vectors given the camera position,
    // and for each we have a camera direction vector
    let best_cam_pos = cam.position();
    let mut qs = vec![];
    for p0 in pt_indices {
        let (n, grid_dir) = grid_dir_of_xy.get(p0).unwrap();
        let di_c = -*grid_dir;
        let model_xy = pairings[*n].0;
        let model_xyz: Point3D = [model_xy[0], model_xy[1], 0.].into();
        let di_m = (best_cam_pos - model_xyz).normalize();
        let z_axis: Point3D = [0., 0., 1.].into();
        let qi_c: Quat = quat::rotation_of_vec_to_vec(&di_c.into(), &z_axis.into()).into();
        let qi_m: Quat = quat::rotation_of_vec_to_vec(&di_m.into(), &z_axis.into()).into();
        for p1 in pt_indices {
            if *p1 == *p0 {
                continue;
            }
            let (n, grid_dir) = grid_dir_of_xy.get(p1).unwrap();
            let dj_c = -*grid_dir;
            let model_xy = pairings[*n].0;
            let model_xyz: Point3D = [model_xy[0], model_xy[1], 0.].into();
            let dj_m = (best_cam_pos - model_xyz).normalize();

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
            eprintln!(
                "di_c==q*di_m? {di_c} ==? {:?}",
                quat::apply3(q.as_ref(), di_m.as_ref())
            );
            eprintln!(
                "dj_c==q*dj_m? {dj_c} ==? {:?}",
                quat::apply3(q.as_ref(), dj_m.as_ref())
            );
            eprintln!("{q}");

            qs.push((1., q.into()));
        }
    }

    let qr: Quat = quat::weighted_average_many(qs.iter().copied()).into();
    calibrate.camera_mut().set_orientation(qr);

    println!("{}", calibrate.clone().to_desc_json()?);
    Ok(())
}

//a Grid lens calibrate
fn grid_lens_calibrate_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("grid_lens_calibrate")
        .about("From calibrate_from_grid")
        .long_about("");

    let mut build = CommandBuilder::new(command, Some(Box::new(grid_lens_calibrate_fn)));
    build.add_arg(
        ic_cmdline::camera::camera_calibrate_arg(true),
        Box::new(|args, matches| {
            ic_cmdline::camera::get_camera_calibrate(matches).map(|v| args.cal = Some(v))
        }),
    );
    build.add_arg(
        ic_cmdline::image::image_read_arg(false, Some(1)),
        Box::new(|args, matches| {
            ic_cmdline::image::get_image_read_filenames(matches).map(|v| args.read_img = v)
        }),
    );
    build.add_arg(
        ic_cmdline::image::image_write_arg(false),
        Box::new(|args, matches| {
            ic_cmdline::image::get_opt_image_write_filename(matches).map(|v| args.write_img = v)
        }),
    );
    build
}

//fi grid_lens_calibrate_fn
fn grid_lens_calibrate_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let cdb = &cmd_args.cdb.as_ref().unwrap();

    //cb Load Calibration JSON
    let calibrate = CameraPolynomialCalibrate::from_json(cdb, cmd_args.cal.as_ref().unwrap())?;
    let cam = calibrate.camera();

    //cb Set up HashMaps and collections
    let mut grid_dir_of_xy = HashMap::new();

    let pt_indices = &[(40, -40), (-40, -40), (40, 40), (-40, 40)];

    let mut nps = NamedPointSet::default();
    let mut pms = PointMappingSet::default();
    let mut nps_of_pts: HashMap<(isize, isize), Rc<NamedPoint>> = HashMap::default();

    //cb Add calibrations to NamedPointSet and PointMappingSet
    let v = calibrate.get_xy_pairings();
    for (grid_xy, pxy_abs) in v.iter() {
        let name = format!("{},{}", grid_xy[0] as isize, grid_xy[1] as isize);
        let model_xyz: Point3D = [grid_xy[0], grid_xy[1], 0.].into();
        let color = [255, 255, 255, 255].into();
        nps.add_pt(name.clone(), color, Some(model_xyz), 0.);
        pms.add_mapping(&nps, &name, pxy_abs, 0.);
    }

    //cb Add all pairings to grid_dir_of_xy
    for (n, (grid_xy, pxy_abs)) in v.iter().enumerate() {
        let txty = cam.px_abs_xy_to_camera_txty(*pxy_abs);
        let grid_dir = txty.to_unit_vector();
        grid_dir_of_xy.insert((grid_xy[0] as isize, grid_xy[1] as isize), (n, grid_dir));
    }

    //cb For required pairings, display data
    for p in pt_indices {
        let name = format!("{},{}", p.0, p.1);
        if let Some(np) = nps.get_pt(&name) {
            nps_of_pts.insert(*p, np);
            let (n, _grid_dir) = grid_dir_of_xy.get(p).unwrap();
            let (grid_xy, pxy_abs) = &v[*n];
            // Px Abs -> Px Rel -> TxTy -> lens mapping
            let txty = cam.px_abs_xy_to_camera_txty(*pxy_abs);
            let grid_dir = txty.to_unit_vector();
            eprintln!("{n} {grid_xy} : {pxy_abs} : {grid_dir}",);
        }
    }

    //cb Create ModelLineSet
    let pairings = calibrate.get_xy_pairings();

    //cb Clone to new camera with correct position/orientation
    let camera = cam.clone();

    //cb Calculate Roll/Yaw for each point given camera
    let mut pts = [vec![], vec![], vec![], vec![]];
    let mut world_yaws = vec![];
    let mut camera_yaws = vec![];
    for (grid_xy, pxy_abs) in pairings {
        let model_xyz: Point3D = [grid_xy[0], grid_xy[1], 0.].into();
        let model_txty = camera.world_xyz_to_camera_txty(model_xyz);
        let cam_txty = camera.px_abs_xy_to_camera_txty(pxy_abs);
        // let pxy_rel = [pxy_abs[0] - 3590.0, 2235.0 - pxy_abs[1]].into();
        // let cam_txty2 = camera.px_rel_xy_to_txty(pxy_rel); // Uses projection
        // eprintln!("{cam_txty}, {cam_txty2} {model_txty}");
        let model_ry: RollYaw = model_txty.into();
        let cam_ry: RollYaw = cam_txty.into();
        if cam_ry.yaw() > 0.01 {
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
        if cam_ry.yaw() > 0.01 {
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

    //cb Create points for crosses for output image
    let mut pts = vec![];
    let n = 30;
    let n_f = n as f64;
    let c_f = n_f / 2.0;
    for y in 0..=n {
        let y_f = (y as f64 - c_f) * 10.;
        let y_i = y_f as isize;
        for x in 0..=n {
            let x_f = (x as f64 - c_f) * 10.;
            let x_i = x_f as isize;
            let pt: Point3D = [x_f, y_f, 0.].into();
            let rgba: Color = {
                if pt_indices.contains(&(x_i, y_i)) {
                    [100, 255, 100, 255]
                } else {
                    [0, 0, 0, 255]
                }
            }
            .into();
            pts.push((pt, rgba));
        }
    }

    Ok(())
}

//a Grid image
fn grid_image_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("grid_image")
        .about("From calibrate_from_grid")
        .long_about("");

    let mut build = CommandBuilder::new(command, Some(Box::new(grid_image_fn)));
    build.add_arg(
        ic_cmdline::camera::camera_calibrate_arg(true),
        Box::new(|args, matches| {
            ic_cmdline::camera::get_camera_calibrate(matches).map(|v| args.cal = Some(v))
        }),
    );
    build.add_arg(
        ic_cmdline::image::image_read_arg(false, Some(1)),
        Box::new(|args, matches| {
            ic_cmdline::image::get_image_read_filenames(matches).map(|v| args.read_img = v)
        }),
    );
    build.add_arg(
        ic_cmdline::image::image_write_arg(false),
        Box::new(|args, matches| {
            ic_cmdline::image::get_opt_image_write_filename(matches).map(|v| args.write_img = v)
        }),
    );
    build
}

//fi grid_image_fn
fn grid_image_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let cdb = &cmd_args.cdb.as_ref().unwrap();

    //cb Load Calibration JSON
    let calibrate = CameraPolynomialCalibrate::from_json(cdb, cmd_args.cal.as_ref().unwrap())?;

    let cam = calibrate.camera().clone();

    //cb Set up HashMaps and collections
    let mut grid_dir_of_xy = HashMap::new();

    let pt_indices = &[(40, -40), (-40, -40), (40, 40), (-40, 40)];

    let mut nps = NamedPointSet::default();
    let mut pms = PointMappingSet::default();
    let mut nps_of_pts: HashMap<(isize, isize), Rc<NamedPoint>> = HashMap::default();

    //cb Add calibrations to NamedPointSet and PointMappingSet
    let v = calibrate.get_xy_pairings();
    for (grid_xy, pxy_abs) in v.iter() {
        let name = format!("{},{}", grid_xy[0] as isize, grid_xy[1] as isize);
        let model_xyz: Point3D = [grid_xy[0], grid_xy[1], 0.].into();
        let color = [255, 255, 255, 255].into();
        nps.add_pt(name.clone(), color, Some(model_xyz), 0.);
        pms.add_mapping(&nps, &name, pxy_abs, 0.);
    }

    //cb Add all pairings to grid_dir_of_xy
    for (n, (grid_xy, pxy_abs)) in v.iter().enumerate() {
        let txty = cam.px_abs_xy_to_camera_txty(*pxy_abs);
        let grid_dir = txty.to_unit_vector();
        grid_dir_of_xy.insert((grid_xy[0] as isize, grid_xy[1] as isize), (n, grid_dir));
    }

    //cb For required pairings, display data
    for p in pt_indices {
        let name = format!("{},{}", p.0, p.1);
        if let Some(np) = nps.get_pt(&name) {
            nps_of_pts.insert(*p, np);
            let (n, _grid_dir) = grid_dir_of_xy.get(p).unwrap();
            let (grid_xy, pxy_abs) = &v[*n];
            // Px Abs -> Px Rel -> TxTy -> lens mapping
            let txty = cam.px_abs_xy_to_camera_txty(*pxy_abs);
            let grid_dir = txty.to_unit_vector();
            eprintln!("{n} {grid_xy} : {pxy_abs} : {grid_dir}",);
        }
    }

    //cb Create ModelLineSet
    let pairings = calibrate.get_xy_pairings();

    let mut mls = ModelLineSet::new(&cam);

    for p0 in pt_indices {
        let (n0, grid_dir) = grid_dir_of_xy.get(p0).unwrap();
        let dir0 = *grid_dir;
        let _pm0 = pms.mapping_of_np(nps_of_pts.get(p0).unwrap()).unwrap();
        for p1 in pt_indices {
            if *p1 == *p0 {
                continue;
            }
            let (n1, grid_dir) = grid_dir_of_xy.get(p1).unwrap();
            let dir1 = *grid_dir;
            let cos_theta = dir0.dot(&dir1);
            let angle = cos_theta.acos();
            let model0_xy = pairings[*n0].0;
            let model1_xy = pairings[*n1].0;
            let model0_xyz = [model0_xy[0], model0_xy[1], 0.].into();
            let model1_xyz = [model1_xy[0], model1_xy[1], 0.].into();
            let _ = mls.add_line_of_models(model0_xyz, model1_xyz, angle);
        }
    }

    //cb Calculate Roll/Yaw for each point given camera
    let camera = calibrate.camera();
    // dbg!(&camera);
    let mut pts = [vec![], vec![], vec![], vec![]];
    let mut world_yaws = vec![];
    let mut camera_yaws = vec![];
    for (grid_xy, pxy_abs) in pairings {
        let model_xyz: Point3D = [grid_xy[0], grid_xy[1], 0.].into();
        let model_txty = camera.world_xyz_to_camera_txty(model_xyz);
        let cam_txty = camera.px_abs_xy_to_camera_txty(pxy_abs);
        // let pxy_rel = [pxy_abs[0] - 3590.0, 2235.0 - pxy_abs[1]].into();
        // let cam_txty2 = camera.px_rel_xy_to_txty(pxy_rel); // Uses projection
        // eprintln!("{cam_txty}, {cam_txty2} {model_txty}");
        let model_ry: RollYaw = model_txty.into();
        let cam_ry: RollYaw = cam_txty.into();
        if cam_ry.yaw() > 0.01 {
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
        if cam_ry.yaw() > 0.01 {
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

    //cb Create points for crosses for output image
    let xy_pairs = calibrate.get_xy_pairings();
    let mut pts = vec![];
    let n = 30;
    let n_f = n as f64;
    let c_f = n_f / 2.0;
    for y in 0..=n {
        let y_f = (y as f64 - c_f) * 10.;
        let y_i = y_f as isize;
        for x in 0..=n {
            let x_f = (x as f64 - c_f) * 10.;
            let x_i = x_f as isize;
            let pt: Point3D = [x_f, y_f, 0.].into();
            let rgba: Color = {
                if pt_indices.contains(&(x_i, y_i)) {
                    [100, 255, 100, 255]
                } else {
                    [0, 0, 0, 255]
                }
            }
            .into();
            pts.push((pt, rgba));
        }
    }

    //cb Read source image and draw on it, write output image
    if !cmd_args.read_img.is_empty() && cmd_args.write_img.is_some() {
        let mut img = ImageRgb8::read_image(&cmd_args.read_img[0])?;
        let c = &[255, 0, 0, 0].into();
        for (_g, p) in &xy_pairs {
            img.draw_cross(*p, 5.0, c);
        }
        for (p, c) in &pts {
            let mapped = camera.map_model(*p);
            if c.0[0] == 100 {
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

//a Grid calibrate
fn grid_calibrate_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("grid_calibrate")
        .about("From calibrate_from_grid")
        .long_about("");

    let mut build = CommandBuilder::new(command, Some(Box::new(grid_calibrate_fn)));
    build.add_arg(
        ic_cmdline::camera::camera_calibrate_arg(true),
        Box::new(|args, matches| {
            ic_cmdline::camera::get_camera_calibrate(matches).map(|v| args.cal = Some(v))
        }),
    );
    build.add_arg(
        ic_cmdline::image::image_read_arg(false, Some(1)),
        Box::new(|args, matches| {
            ic_cmdline::image::get_image_read_filenames(matches).map(|v| args.read_img = v)
        }),
    );
    build.add_arg(
        ic_cmdline::image::image_write_arg(false),
        Box::new(|args, matches| {
            ic_cmdline::image::get_opt_image_write_filename(matches).map(|v| args.write_img = v)
        }),
    );
    build
}

//fi grid_calibrate_fn
fn grid_calibrate_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let cdb = &cmd_args.cdb.as_ref().unwrap();

    //cb Load Calibration JSON
    let calibrate = CameraPolynomialCalibrate::from_json(cdb, cmd_args.cal.as_ref().unwrap())?;

    //cb Set up 'cam' as the camera
    let mut cam = calibrate.camera().clone();
    cam.set_position([0., 0., 0.].into());
    cam.set_orientation(Quat::default());

    //cb Set up HashMaps and collections
    let mut grid_dir_of_xy = HashMap::new();

    let pt_indices = &[(40, -40), (-40, -40), (40, 40), (-40, 40)];

    let mut nps = NamedPointSet::default();
    let mut pms = PointMappingSet::default();
    let mut nps_of_pts: HashMap<(isize, isize), Rc<NamedPoint>> = HashMap::default();

    //cb Add calibrations to NamedPointSet and PointMappingSet
    let v = calibrate.get_xy_pairings();
    for (grid_xy, pxy_abs) in v.iter() {
        let name = format!("{},{}", grid_xy[0] as isize, grid_xy[1] as isize);
        let model_xyz: Point3D = [grid_xy[0], grid_xy[1], 0.].into();
        let color = [255, 255, 255, 255].into();
        nps.add_pt(name.clone(), color, Some(model_xyz), 0.);
        pms.add_mapping(&nps, &name, pxy_abs, 0.);
    }

    //cb Add all pairings to grid_dir_of_xy
    for (n, (grid_xy, pxy_abs)) in v.iter().enumerate() {
        let txty = cam.px_abs_xy_to_camera_txty(*pxy_abs);
        let grid_dir = txty.to_unit_vector();
        grid_dir_of_xy.insert((grid_xy[0] as isize, grid_xy[1] as isize), (n, grid_dir));
    }

    //cb For required pairings, display data
    for p in pt_indices {
        let name = format!("{},{}", p.0, p.1);
        if let Some(np) = nps.get_pt(&name) {
            nps_of_pts.insert(*p, np);
            let (n, _grid_dir) = grid_dir_of_xy.get(p).unwrap();
            let (grid_xy, pxy_abs) = &v[*n];
            // Px Abs -> Px Rel -> TxTy -> lens mapping
            let txty = cam.px_abs_xy_to_camera_txty(*pxy_abs);
            let grid_dir = txty.to_unit_vector();
            eprintln!("{n} {grid_xy} : {pxy_abs} : {grid_dir}",);
        }
    }

    //cb Create ModelLineSet
    let pairings = calibrate.get_xy_pairings();

    let mut mls = ModelLineSet::new(&cam);

    for p0 in pt_indices {
        let (n0, grid_dir) = grid_dir_of_xy.get(p0).unwrap();
        let dir0 = *grid_dir;
        let _pm0 = pms.mapping_of_np(nps_of_pts.get(p0).unwrap()).unwrap();
        for p1 in pt_indices {
            if *p1 == *p0 {
                continue;
            }
            let (n1, grid_dir) = grid_dir_of_xy.get(p1).unwrap();
            let dir1 = *grid_dir;
            let cos_theta = dir0.dot(&dir1);
            let angle = cos_theta.acos();
            let model0_xy = pairings[*n0].0;
            let model1_xy = pairings[*n1].0;
            let model0_xyz = [model0_xy[0], model0_xy[1], 0.].into();
            let model1_xyz = [model1_xy[0], model1_xy[1], 0.].into();
            let _ = mls.add_line_of_models(model0_xyz, model1_xyz, angle);
        }
    }

    //cb Find best position given ModelLineSet
    // Find best location 'p' for camera
    let (best_cam_pos, e) = mls.find_best_min_err_location(&|p| p[2] > 0., 1000, 1000);
    eprintln!("{best_cam_pos} {e}",);

    // let best_cam_pos: Point3D = [13.76943098455281, -4.4539157030506376, 410.03914507909536].into();
    // let best_cam_pos: Point3D = [7.54435219975766, -2.2904012588912086, -407.86139540073606].into();

    //cb Find best orientation given position
    // We can get N model direction vectors given the camera position,
    // and for each we have a camera direction vector
    let mut qs = vec![];
    for p0 in pt_indices {
        let (n, grid_dir) = grid_dir_of_xy.get(p0).unwrap();
        let di_c = -*grid_dir;
        let model_xy = pairings[*n].0;
        let model_xyz: Point3D = [model_xy[0], model_xy[1], 0.].into();
        let di_m = (best_cam_pos - model_xyz).normalize();
        let z_axis: Point3D = [0., 0., 1.].into();
        let qi_c: Quat = quat::rotation_of_vec_to_vec(&di_c.into(), &z_axis.into()).into();
        let qi_m: Quat = quat::rotation_of_vec_to_vec(&di_m.into(), &z_axis.into()).into();
        for p1 in pt_indices {
            if *p1 == *p0 {
                continue;
            }
            let (n, grid_dir) = grid_dir_of_xy.get(p1).unwrap();
            let dj_c = -*grid_dir;
            let model_xy = pairings[*n].0;
            let model_xyz: Point3D = [model_xy[0], model_xy[1], 0.].into();
            let dj_m = (best_cam_pos - model_xyz).normalize();

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
            eprintln!(
                "di_c==q*di_m? {di_c} ==? {:?}",
                quat::apply3(q.as_ref(), di_m.as_ref())
            );
            eprintln!(
                "dj_c==q*dj_m? {dj_c} ==? {:?}",
                quat::apply3(q.as_ref(), dj_m.as_ref())
            );
            eprintln!("{q}");

            qs.push((1., q.into()));
        }
    }

    let qr: Quat = quat::weighted_average_many(qs.iter().copied()).into();

    //cb Clone to new camera with correct position/orientation
    let mut camera = cam.clone();
    camera.set_position(best_cam_pos);
    camera.set_orientation(qr);

    //cb Calculate Roll/Yaw for each point given camera
    // dbg!(&camera);
    let mut pts = [vec![], vec![], vec![], vec![]];
    let mut world_yaws = vec![];
    let mut camera_yaws = vec![];
    for (grid_xy, pxy_abs) in pairings {
        let model_xyz: Point3D = [grid_xy[0], grid_xy[1], 0.].into();
        let model_txty = camera.world_xyz_to_camera_txty(model_xyz);
        let cam_txty = camera.px_abs_xy_to_camera_txty(pxy_abs);
        // let pxy_rel = [pxy_abs[0] - 3590.0, 2235.0 - pxy_abs[1]].into();
        // let cam_txty2 = camera.px_rel_xy_to_txty(pxy_rel); // Uses projection
        // eprintln!("{cam_txty}, {cam_txty2} {model_txty}");
        let model_ry: RollYaw = model_txty.into();
        let cam_ry: RollYaw = cam_txty.into();
        if cam_ry.yaw() > 0.01 {
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
        if cam_ry.yaw() > 0.01 {
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

    //cb Create points for crosses for output image
    let xy_pairs = calibrate.get_xy_pairings();
    let mut pts = vec![];
    let n = 30;
    let n_f = n as f64;
    let c_f = n_f / 2.0;
    for y in 0..=n {
        let y_f = (y as f64 - c_f) * 10.;
        let y_i = y_f as isize;
        for x in 0..=n {
            let x_f = (x as f64 - c_f) * 10.;
            let x_i = x_f as isize;
            let pt: Point3D = [x_f, y_f, 0.].into();
            let rgba: Color = {
                if pt_indices.contains(&(x_i, y_i)) {
                    [100, 255, 100, 255]
                } else {
                    [0, 0, 0, 255]
                }
            }
            .into();
            pts.push((pt, rgba));
        }
    }

    //cb Read source image and draw on it, write output image
    if !cmd_args.read_img.is_empty() && cmd_args.write_img.is_some() {
        let mut img = ImageRgb8::read_image(&cmd_args.read_img[0])?;
        let c = &[255, 0, 0, 0].into();
        for (_g, p) in &xy_pairs {
            img.draw_cross(*p, 5.0, c);
        }
        for (p, c) in &pts {
            let mapped = camera.map_model(*p);
            if c.0[0] == 100 {
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
    build.add_subcommand(grid_locate_cmd());
    build.add_subcommand(grid_orient_cmd());
    build.add_subcommand(grid_lens_calibrate_cmd());
    build.add_subcommand(grid_image_cmd());
    build.add_subcommand(grid_calibrate_cmd());

    let mut cmd_args = CmdArgs::default();
    let mut command: CommandSet<_> = build.into();
    command.execute_env(&mut cmd_args)?;
    Ok(())
}
