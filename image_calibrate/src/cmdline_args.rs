//a Modules
use std::rc::Rc;

use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};

use crate::{
    json, CameraDatabase, CameraInstance, CameraPolynomial, CameraProjection, NamedPointSet,
    PointMappingSet,
};

//a NamedPointSet
//fp add_nps_arg
pub fn add_nps_arg(cmd: Command) -> Command {
    cmd.arg(
            Arg::new("nps")
            .long("nps")
            .short('n')
            .required(true)
            .help("Specifies a named point set json")
            .long_help("A filename of a JSON file that provides the named set of points and there 3D locations for a particular model")
            .action(ArgAction::Set),
    )
}

//fp get_nps
pub fn get_nps(matches: &ArgMatches) -> Result<NamedPointSet, String> {
    let nps_filename = matches.get_one::<String>("nps").unwrap();
    let nps_json = json::read_file(nps_filename)?;
    NamedPointSet::from_json(&nps_json)
}

//a PointMappingSet
//fp add_pms_arg
pub fn add_pms_arg(cmd: Command) -> Command {
    cmd.arg(
            Arg::new("pms")
                .long("pms")
                .short('p')
                .required(true)
                .help("point mapping set json")
            .long_help("A filename of a JSON file that provides a point mapping set, mapping named points (in an NPS) to XY coordinates in a camera image")
        .action(ArgAction::Set),
    )
}

//fp get_pms
pub fn get_pms(matches: &ArgMatches, nps: &NamedPointSet) -> Result<PointMappingSet, String> {
    let mut pms = PointMappingSet::new();
    for pms_filename in matches.get_many::<String>("pms").unwrap() {
        let pms_json = json::read_file(pms_filename)?;
        pms.read_json(nps, &pms_json)?;
    }
    Ok(pms)
}

//a CameraProjection
// should move to using the camera database - need a new section - and body and lens names from within the database plus the focus distance
//fp add_camera_projection_args
pub fn add_camera_projection_args(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("focus")
            .long("focus")
            .short('f')
            .required(true)
            .help("focussed distance")
            .value_parser(value_parser!(f64))
            .action(ArgAction::Set),
    )
    .arg(
        Arg::new("body")
            .long("body")
            .short('b')
            .required(true)
            .help("Camera body name")
            .action(ArgAction::Set),
    )
    .arg(
        Arg::new("lens")
            .long("lens")
            .short('l')
            .required(true)
            .help("Lens name")
            .action(ArgAction::Set),
    )
}

//fp get_camera_projection
pub fn get_camera_projection(
    matches: &ArgMatches,
    db: &CameraDatabase,
) -> Result<Rc<dyn CameraProjection>, String> {
    let mm_focus_distance = *matches
        .get_one::<f64>("focus")
        .ok_or("A mm focus distance is required (float)")?;
    let body_name = matches
        .get_one::<String>("body")
        .ok_or("A camera body name is required")?;
    let lens_name = matches
        .get_one::<String>("lens")
        .ok_or("A lens name is required")?;
    let body = db
        .get_body(&body_name)
        .ok_or(format!("Body '{}' was not in the database", body_name))?;
    let lens = db
        .get_lens(&lens_name)
        .ok_or(format!("Lens '{}' was not in the database", lens_name))?;
    let camera = CameraPolynomial::new(body, lens, mm_focus_distance);
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
        .ok_or("A camera database JSON is required")?;
    let camera_db_json = json::read_file(camera_db_filename)?;
    let mut camera_db: CameraDatabase = json::from_json("camera database", &camera_db_json)?;
    camera_db.derive();
    Ok(camera_db)
}

//a Camera
//fp add_camera_arg
pub fn add_camera_arg(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("camera")
            .long("camera")
            .short('c')
            .required(false)
            .help("Camera placement and orientation JSON")
            .action(ArgAction::Set),
    )
}

//fp get_camera
pub fn get_camera(
    matches: &ArgMatches,
    camera_projection: Rc<dyn CameraProjection>,
) -> Result<CameraInstance, String> {
    let camera_filename = matches
        .get_one::<String>("camera")
        .ok_or("A camera position/orientation JSON is required")?;
    let camera_json = json::read_file(camera_filename)?;
    let mut camera: CameraInstance = json::from_json("camera", &camera_json)?;
    camera.set_projection(camera_projection);
    Ok(camera)
}
