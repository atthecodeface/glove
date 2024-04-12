//a Modules
use std::rc::Rc;

use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};

use ic_base::{json, Point3D, Quat};
use ic_camera::{CameraDatabase, CameraPolynomial, CameraPolynomialCalibrate};
use ic_project::Project;

//a CameraProjection
// should move to using the camera database - need a new section - and body and lens names from within the database plus the focus distance
//fp add_camera_projection_args
pub fn add_camera_projection_args(cmd: Command, required: bool) -> Command {
    cmd.arg(
        Arg::new("focus")
            .long("focus")
            .short('f')
            .required(required)
            .help("focussed distance")
            .value_parser(value_parser!(f64))
            .action(ArgAction::Set),
    )
    .arg(
        Arg::new("body")
            .long("body")
            .short('b')
            .required(required)
            .help("Camera body name")
            .action(ArgAction::Set),
    )
    .arg(
        Arg::new("lens")
            .long("lens")
            .short('l')
            .required(required)
            .help("Lens name")
            .action(ArgAction::Set),
    )
}

//fp get_camera_projection
pub fn get_camera_projection(
    matches: &ArgMatches,
    db: &CameraDatabase,
) -> Result<Rc<CameraPolynomial>, String> {
    let mm_focus_distance = *matches
        .get_one::<f64>("focus")
        .ok_or("A mm focus distance is required (float)")?;
    let body_name = matches
        .get_one::<String>("body")
        .ok_or("A camera body name is required")?;
    let lens_name = matches
        .get_one::<String>("lens")
        .ok_or("A lens name is required")?;
    let body = db.get_body_err(body_name)?.clone();
    let lens = db.get_lens_err(lens_name)?.clone();
    let position = Point3D::default();
    let orientation = Quat::default();
    let camera = CameraPolynomial::new(body, lens, mm_focus_distance, position, orientation);
    Ok(Rc::new(camera))
}

//a CameraDatabase
//fp add_camera_database_arg
pub fn add_camera_database_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
        Arg::new("camera_db")
            .long("db")
            .alias("database")
            .required(required)
            .help("Camera database JSON")
            .action(ArgAction::Set),
    )
}

//fp get_camera_database
pub fn get_camera_database(matches: &ArgMatches) -> Result<CameraDatabase, String> {
    let camera_db_filename = matches
        .get_one::<String>("camera_db")
        .ok_or("Either a project or a camera database JSON is required")?;
    let camera_db_json = json::read_file(camera_db_filename)?;
    let mut camera_db: CameraDatabase = json::from_json("camera database", &camera_db_json)?;
    camera_db.derive();
    Ok(camera_db)
}
//a Camera
//fp add_camera_arg
pub fn add_camera_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
        Arg::new("camera")
            .long("camera")
            .short('c')
            .required(required)
            .help("Camera placement and orientation JSON")
            .action(ArgAction::Set),
    )
}

//fp get_camera
pub fn get_camera(matches: &ArgMatches, project: &Project) -> Result<CameraPolynomial, String> {
    let camera_filename = matches
        .get_one::<String>("camera")
        .ok_or("A camera position/orientation JSON is required")?;
    let camera_json = json::read_file(camera_filename)?;
    CameraPolynomial::from_json(&project.cdb_ref(), &camera_json)
}

//a CameraCalibrate
//fp add_camera_calibrate_arg
pub fn add_camera_calibrate_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
        Arg::new("camera")
            .long("camera")
            .short('c')
            .required(required)
            .help("Camera calibration placement and orientation JSON")
            .action(ArgAction::Set),
    )
}

//fp get_camera_calibrate
pub fn get_camera_calibrate(
    matches: &ArgMatches,
    cdb: &CameraDatabase,
) -> Result<CameraPolynomialCalibrate, String> {
    let camera_filename = matches
        .get_one::<String>("camera")
        .ok_or("A camera calibration position/orientation JSON is required")?;
    let camera_json = json::read_file(camera_filename)?;
    CameraPolynomialCalibrate::from_json(cdb, &camera_json)
}
