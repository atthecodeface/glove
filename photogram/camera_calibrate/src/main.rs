//a Imports
use std::collections::HashMap;

use anyhow::Result;
use clap::Command;

use ic_base::{Point3D, RollYaw, TanXTanY};
use ic_camera::polynomial;
use ic_camera::polynomial::CalcPoly;
use ic_camera::{CameraDatabase, CameraInstance, CameraProjection};
use ic_cmdline as cmdline_args;
use ic_image::{Image, ImageRgb8};

//a Types
//ti SubCmdFn
type SubCmdFn = fn(CameraDatabase, &clap::ArgMatches) -> Result<()>;

//a Calibrate
//fi calibrate_cmd
fn calibrate_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("calibrate")
        .about("Read image and draw crosses on grid coordinates")
        .long_about(
            "This uses the camera calibration JSON file in conjunction with a camera body/lens and focus distance to generate the correct focal length and tan-tan mapping for the lens as world-to-screen (and vice-versa) polynomials. The camera calibration JSON file includes 'mappings' that is a list of (grid xmm, grid ymm, x pixel, y pixel) tuples each being the mapping of a grid x,y to a frame pixel x,y on an image. If read and write imnages are provided then the immage is read and red crosses superimposed on the image at the post-calibrated points using the provided grid x,y points as sources (so they should align with the actual grid points on the image)"
        );
    let cmd = cmdline_args::camera::add_camera_calibrate_arg(cmd, true);
    let cmd = cmdline_args::image::add_image_read_arg(cmd, true);
    let cmd = cmdline_args::image::add_image_write_arg(cmd, true);
    (cmd, calibrate_fn)
}

//fi calibrate_fn
fn calibrate_fn(cdb: CameraDatabase, matches: &clap::ArgMatches) -> Result<()> {
    let calibrate = cmdline_args::camera::get_camera_calibrate(matches, &cdb)?;
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
    if let Some(read_filename) = matches.get_one::<String>("read") {
        let mut img = ImageRgb8::read_image(read_filename)?;
        if let Some(write_filename) = matches.get_one::<String>("write") {
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
            img.write(write_filename)?;
        }
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
    let cmd = Command::new("camera_calibrate")
        .about("Camera calibration tool")
        .version("0.1.0")
        .subcommand_required(true);
    let cmd = cmdline_args::camera::add_camera_database_arg(cmd, true);

    let mut subcmds: HashMap<String, SubCmdFn> = HashMap::new();
    let mut cmd = cmd;
    for (c, f) in [
        calibrate_cmd(),
        // image_grid_cmd()
    ] {
        subcmds.insert(c.get_name().into(), f);
        cmd = cmd.subcommand(c);
    }
    let cmd = cmd;

    let matches = cmd.get_matches();
    let cdb = cmdline_args::camera::get_camera_database(&matches)?;

    let (subcommand, submatches) = matches.subcommand().unwrap();
    for (name, sub_cmd_fn) in subcmds {
        if subcommand == name {
            return Ok(sub_cmd_fn(cdb, submatches)?);
        }
    }
    unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`");
}
