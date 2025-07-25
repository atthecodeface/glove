//a Modules
use std::rc::Rc;

use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};

use ic_base::{json, Point3D, Quat, Result};
use ic_camera::{CalibrationMapping, CameraDatabase, CameraInstance};
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
) -> Result<Rc<CameraInstance>> {
    let mm_focus_distance = *matches.get_one::<f64>("focus").unwrap();
    let body_name = matches.get_one::<String>("body").unwrap();
    let lens_name = matches.get_one::<String>("lens").unwrap();
    let body = db.get_body_err(body_name)?.clone();
    let lens = db.get_lens_err(lens_name)?.clone();
    let position = Point3D::default();
    let orientation = Quat::default();
    let camera = CameraInstance::new(body, lens, mm_focus_distance, position, orientation);
    Ok(Rc::new(camera))
}

//a Arg functions - CameraDatabase, CameraInstance / placement / orientation, Mapping
//fp camera_database_arg
pub fn camera_database_arg(required: bool) -> Arg {
    Arg::new("camera_db")
        .long("db")
        .alias("database")
        .required(required)
        .help("Camera database JSON")
        .action(ArgAction::Set)
}

//fp camera_arg
pub fn camera_arg(required: bool) -> Arg {
    Arg::new("camera")
        .long("camera")
        .short('c')
        .required(required)
        .help("Camera placement and orientation JSON")
        .action(ArgAction::Set)
}

//a Retrieve data functions
//fp get_camera_database
pub fn get_camera_database(matches: &ArgMatches) -> Result<CameraDatabase> {
    let camera_db_filename = matches.get_one::<String>("camera_db").unwrap();
    let camera_db_json = json::read_file(camera_db_filename)?;
    let mut camera_db: CameraDatabase = json::from_json("camera database", &camera_db_json)?;
    camera_db.derive();
    Ok(camera_db)
}

//fp set_camera
pub fn set_camera(
    matches: &ArgMatches,
    cdb: &CameraDatabase,
    camera: &mut CameraInstance,
) -> Result<()> {
    let camera_filename = matches.get_one::<String>("camera").unwrap();
    let camera_json = json::read_file(camera_filename)?;
    *camera = CameraInstance::from_json(cdb, &camera_json)?;
    Ok(())
}

//fp get_camera
pub fn get_camera(matches: &ArgMatches, project: &Project) -> Result<CameraInstance> {
    let camera_filename = matches.get_one::<String>("camera").unwrap();
    let camera_json = json::read_file(camera_filename)?;
    CameraInstance::from_json(&project.cdb_ref(), &camera_json)
}

//a CameraCalibrate
//fp calibration_mapping_arg
pub fn calibration_mapping_arg(required: bool) -> Arg {
    Arg::new("calibration_mapping")
        .long("calibrate")
        .short('c')
        .required(required)
        .help("Camera calibration mapping JSON")
        .action(ArgAction::Set)
}

//fp set_opt_calibration_mapping
pub fn set_opt_calibration_mapping(
    matches: &ArgMatches,
    mapping: &mut Option<CalibrationMapping>,
) -> Result<()> {
    let filename = matches.get_one::<String>("calibration_mapping").unwrap();
    let json = json::read_file(filename)?;
    *mapping = Some(CalibrationMapping::from_json(&json)?);
    Ok(())
}
