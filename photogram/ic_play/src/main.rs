//a Documentation
//! Test of camera calibration from stars (IMG_4924.JPG)
//!
//! The stars were captured on a Canon Rebel T2i, with a 50mm lens focused on 'infinity'
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
//!  noticeable offset from each other; repeat from step 3
//!
//!  5. Once the graphs are all deemed reasonable, copy the
//!  polynomials calculated in to the lens mapping.
//!
//!  6. Rerun, and the graphs should be near identity, and the
//!  calibration is complete.
//!  

//a Imports
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};
use geo_nd::{quat, Quaternion, Vector};
use serde::{Deserialize, Serialize};
use star_catalog::{Catalog, CatalogIndex, Subcube};

use ic_base::json;
use ic_base::{Point2D, Point3D, Quat, Result, TanXTanY};
use ic_camera::{serialize_body_name, serialize_lens_name};
use ic_camera::{CameraBody, CameraLens, CameraPolynomial, CameraPolynomialDesc};
use ic_camera::{CameraDatabase, CameraProjection};
use ic_image::{Color, Image, ImageRgb8};
use ic_stars::{ImgPt, StarCalibrate, StarCalibrateDesc};

use ic_cmdline::builder::{CommandArgs, CommandBuilder, CommandSet};

//a CmdArgs
//tp  CmdArgs
#[derive(Default)]
pub struct CmdArgs {
    cdb: Option<CameraDatabase>,
    cal: Option<StarCalibrate>,
    search_brightness: f32,
    match_brightness: f32,
    catalog: Option<Box<Catalog>>,
    read_img: Vec<String>,
    write_img: Option<String>,
}

//ip CommandArgs for CmdArgs
impl CommandArgs for CmdArgs {
    type Error = ic_base::Error;
    type Value = ();
}

//ip CmdArgs
impl CmdArgs {
    pub fn draw_image(&self, pts: &[ImgPt]) -> Result<()> {
        if self.read_img.is_empty() || self.write_img.is_none() {
            return Ok(());
        }
        let mut img = ImageRgb8::read_image(&self.read_img[0])?;
        for p in pts {
            p.draw(&mut img);
        }
        img.write(self.write_img.as_ref().unwrap())?;
        Ok(())
    }
    pub fn borrow_mut(&mut self) -> (&mut StarCalibrate, &mut Box<Catalog>) {
        match (&mut self.cal, &mut self.catalog) {
            (Some(cal), Some(catalog)) => (cal, catalog),
            _ => {
                panic!("Cannot borrow; bad argument setup in the program");
            }
        }
    }
}

//a arg commands
//fp arg_camera_db
fn arg_camera_db(args: &mut CmdArgs, matches: &ArgMatches) -> Result<()> {
    let camera_db_filename = matches.get_one::<String>("camera_db").unwrap();
    let camera_db_json = json::read_file(camera_db_filename)?;
    let mut camera_db: CameraDatabase = json::from_json("camera database", &camera_db_json)?;
    camera_db.derive();
    args.cdb = Some(camera_db);
    Ok(())
}

//fp arg_read_image
fn arg_read_image(args: &mut CmdArgs, matches: &ArgMatches) -> Result<()> {
    for read_filename in matches.get_many::<String>("read_image").unwrap() {
        args.read_img.push(read_filename.clone());
    }
    Ok(())
}

//fp arg_write_image
fn arg_write_image(args: &mut CmdArgs, matches: &ArgMatches) -> Result<()> {
    args.write_img = matches.get_one::<String>("write_image").cloned();
    Ok(())
}

//fp arg_star_calibrate
fn arg_star_calibrate(args: &mut CmdArgs, matches: &ArgMatches) -> Result<()> {
    let calibrate_filename = matches.get_one::<String>("star_calibrate").unwrap();
    let calibrate_json = json::read_file(calibrate_filename)?;
    args.cal = Some(StarCalibrate::from_json(
        args.cdb.as_ref().unwrap(),
        &calibrate_json,
    )?);
    Ok(())
}

//fp arg_star_catalog
fn arg_star_catalog(args: &mut CmdArgs, matches: &ArgMatches) -> Result<()> {
    let catalog_filename = matches.get_one::<String>("star_catalog").unwrap();
    if catalog_filename == "hipp_bright" {
        args.catalog = Some(
            postcard::from_bytes(star_catalog::hipparcos::HIPP_BRIGHT_PST)
                .map_err(|e| format!("{e:?}"))?,
        );
    } else {
        let s = std::fs::read_to_string(catalog_filename)?;
        args.catalog = Some(serde_json::from_str(&s)?);
    }
    args.catalog.as_mut().unwrap().sort();
    args.catalog.as_mut().unwrap().derive_data();
    Ok(())
}

//fp arg_search_brightness
fn arg_search_brightness(args: &mut CmdArgs, matches: &ArgMatches) -> Result<()> {
    args.search_brightness = *matches.get_one::<f32>("search_brightness").unwrap();
    Ok(())
}

//fp arg_match_brightness
fn arg_match_brightness(args: &mut CmdArgs, matches: &ArgMatches) -> Result<()> {
    args.match_brightness = *matches.get_one::<f32>("match_brightness").unwrap();
    Ok(())
}

//a Useful functions
//fi orientation_mapping_triangle
/// Get q which maps model to camera
///
/// dc === quat::apply3(q, dm)
fn orientation_mapping_triangle(
    di_m: &[f64; 3],
    dj_m: &[f64; 3],
    dk_m: &[f64; 3],
    di_c: Point3D,
    dj_c: Point3D,
    dk_c: Point3D,
) -> Quat {
    let mut qs = vec![];
    qs.push((1.0, orientation_mapping(di_m, dj_m, di_c, dj_c).into()));
    qs.push((1.0, orientation_mapping(di_m, dk_m, di_c, dk_c).into()));
    qs.push((1.0, orientation_mapping(dj_m, dk_m, dj_c, dk_c).into()));
    qs.push((1.0, orientation_mapping(dj_m, di_m, dj_c, di_c).into()));
    qs.push((1.0, orientation_mapping(dk_m, di_m, dk_c, di_c).into()));
    qs.push((1.0, orientation_mapping(dk_m, dj_m, dk_c, dj_c).into()));
    quat::weighted_average_many(qs.iter().copied()).into()
}

//fp orientation_mapping
fn orientation_mapping(di_m: &[f64; 3], dj_m: &[f64; 3], di_c: Point3D, dj_c: Point3D) -> Quat {
    let z_axis: Point3D = [0., 0., 1.].into();
    let qi_c: Quat = quat::rotation_of_vec_to_vec(&di_c.into(), &z_axis.into()).into();
    let qi_m: Quat = quat::rotation_of_vec_to_vec(di_m, &z_axis.into()).into();

    let dj_c_rotated: Point3D = quat::apply3(qi_c.as_ref(), dj_c.as_ref()).into();
    let dj_m_rotated: Point3D = quat::apply3(qi_m.as_ref(), dj_m).into();

    let theta_dj_m = dj_m_rotated[0].atan2(dj_m_rotated[1]);
    let theta_dj_c = dj_c_rotated[0].atan2(dj_c_rotated[1]);
    let theta = theta_dj_m - theta_dj_c;
    let theta_div_2 = theta / 2.0;
    let cos_2theta = theta_div_2.cos();
    let sin_2theta = theta_div_2.sin();
    let q_z = Quat::of_rijk(cos_2theta, 0.0, 0.0, sin_2theta);

    qi_c.conjugate() * q_z * qi_m
}

//fp closest_star
fn closest_star(catalog: &Catalog, v: Point3D) -> Option<(f64, CatalogIndex)> {
    let s = Subcube::of_vector(&v.into_array().into());
    let mut closest = None;
    for s in s.iter_range(2) {
        for index in catalog[s].iter() {
            let cv: &[f64; 3] = catalog[*index].vector.as_ref();
            let c = v.dot(&(*cv).into());
            if let Some((cc, _)) = closest {
                if c > cc {
                    closest = Some((c, *index));
                }
            } else {
                closest = Some((c, *index));
            }
        }
    }
    closest
}

//a Main
pub fn main() -> Result<()> {
    let command = Command::new("ic_play")
        .about("Camera calibration tool")
        .version("0.1.0");

    let mut build = CommandBuilder::new(command, None);
    build.add_arg(
        Arg::new("camera_db")
            .long("db")
            .alias("database")
            .required(true)
            .help("Camera database JSON")
            .action(ArgAction::Set),
        Box::new(arg_camera_db),
    );
    build.add_arg(
        Arg::new("star_calibrate")
            // .long("star")
            // .alias("database")
            .required(true)
            .help("Star calibration JSON")
            .action(ArgAction::Set),
        Box::new(arg_star_calibrate),
    );
    build.add_arg(
        Arg::new("star_catalog")
            .long("catalog")
            .required(true)
            .help("Star catalog to use")
            .action(ArgAction::Set),
        Box::new(arg_star_catalog),
    );
    build.add_arg(
        Arg::new("search_brightness")
            .long("search_brightness")
            .value_parser(value_parser!(f32))
            .default_value("5.0")
            .help("Maximum brightness of stars to use for searching with triangles")
            .action(ArgAction::Set),
        Box::new(arg_search_brightness),
    );
    build.add_arg(
        Arg::new("match_brightness")
            .long("match_brightness")
            .value_parser(value_parser!(f32))
            .default_value("5.0")
            .help("Maximum brightness of stars to use for matching all the points")
            .action(ArgAction::Set),
        Box::new(arg_match_brightness),
    );
    build.add_arg(
        Arg::new("read_image")
            .long("read")
            .short('r')
            .required(false)
            .help("Image to read")
            .action(ArgAction::Append),
        Box::new(arg_read_image),
    );
    build.add_arg(
        Arg::new("write_image")
            .long("write")
            .short('w')
            .required(false)
            .help("Image to write")
            .action(ArgAction::Set),
        Box::new(arg_write_image),
    );

    let ms_command =
        Command::new("map_stars").about("Map all stars in the catalog onto an output image");
    let mut ms_build = CommandBuilder::new(ms_command, Some(Box::new(map_stars_cmd)));
    build.add_subcommand(ms_build);

    let fs_command = Command::new("find_stars").about("Find stars from an image");
    let fs_build = CommandBuilder::new(fs_command, Some(Box::new(find_stars_from_image_cmd)));
    build.add_subcommand(fs_build);

    let mut cmd_args = CmdArgs::default();
    let mut command: CommandSet<CmdArgs> = build.into();
    command.execute_env(&mut cmd_args)?;
    Ok(())
}

//fp map_stars_cmd
fn map_stars_cmd(cmd_args: &mut CmdArgs) -> Result<()> {
    let brightness = cmd_args.search_brightness;
    let (calibrate, catalog) = cmd_args.borrow_mut();
    let cat_index = calibrate.map_stars(catalog, brightness)?;

    //cb Show the star mappings
    let _ = calibrate.show_star_mappings(catalog);
    let mut mapped_pts = vec![];
    calibrate.add_catalog_stars(catalog, &mut mapped_pts)?;
    calibrate.add_cat_index(catalog, &cat_index, &mut mapped_pts)?;
    calibrate.add_mapping_pts(&mut mapped_pts)?;
    cmd_args.draw_image(&mapped_pts);
    Ok(())
}

//fp find_stars_from_image_cmd
fn find_stars_from_image_cmd(cmd_args: &mut CmdArgs) -> Result<()> {
    let brightness = cmd_args.search_brightness;
    let (calibrate, catalog) = cmd_args.borrow_mut();
    calibrate.find_stars_from_image(catalog, brightness)?;

    //cb Show the star mappings
    let mut mapped_pts = vec![];
    calibrate.add_catalog_stars(catalog, &mut mapped_pts)?;
    for p in calibrate.show_star_mappings(catalog) {
        mapped_pts.push((p, 1).into());
    }
    calibrate.add_mapping_pts(&mut mapped_pts)?;
    cmd_args.draw_image(&mapped_pts)?;
    Ok(())
}
