//a Imports
use std::rc::Rc;

use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};

use ic_base::{json, Error, Point3D, Quat, Result};
use ic_camera::{CalibrationMapping, CameraDatabase, CameraInstance, LensPolys};
use ic_project::Project;

use crate::builder::{CommandArgs, CommandBuilder};

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
        .help("Camera lens, placement and orientation JSON")
        .action(ArgAction::Set)
}

//fp use_body_arg
pub fn use_body_arg() -> Arg {
    Arg::new("use_body")
        .long("use_body")
        .help("Specify which body to use in the camera")
        .action(ArgAction::Set)
}

//fp use_lens_arg
pub fn use_lens_arg() -> Arg {
    Arg::new("use_lens")
        .long("use_lens")
        .help("Specify which lens to use in the camera")
        .action(ArgAction::Set)
}

//fp use_focus_arg
pub fn use_focus_arg() -> Arg {
    Arg::new("use_focus")
        .long("use_focus")
        .help("Specify the focus distance in mm used for the image, in the camera")
        .value_parser(value_parser!(f64))
        .action(ArgAction::Set)
}

//fp use_polys_arg
pub fn use_polys_arg() -> Arg {
    Arg::new("use_polys")
        .long("use_polys")
        .help("Specify an override for the lens polynomials in the camera")
        .action(ArgAction::Set)
}

//mp add_arg_camera
pub fn add_arg_camera<C, F, G>(build: &mut CommandBuilder<C>, get_db: G, set: F, required: bool)
where
    C: CommandArgs<Error = Error>,
    F: Fn(&mut C, CameraInstance) -> Result<()> + 'static,
    G: Fn(&C) -> &CameraDatabase + 'static,
{
    build.add_arg(
        camera_arg(required),
        Box::new(move |args, matches| {
            let mut camera = get_camera_of_db(matches, get_db(args))?;
            if let Some(body) = matches.get_one::<String>("use_body") {
                camera.set_body(get_db(args).get_body_err(body)?.clone());
            }
            if let Some(lens) = matches.get_one::<String>("use_lens") {
                camera.set_lens(get_db(args).get_lens_err(lens)?.clone());
            }
            if let Some(focus) = matches.get_one::<f64>("use_focus") {
                camera.set_mm_focus_distance(*focus);
            }
            if let Some(polys) = matches.get_one::<String>("use_polys") {
                let json = json::read_file(polys)?;
                let lens_polys: LensPolys = json::from_json("lens polynomials", &json)?;
                let mut lens = camera.lens().clone();
                lens.set_polys(lens_polys);
                camera.set_lens(lens);
            }
            set(args, camera)
        }),
    );
    build.add_arg(use_body_arg(), Box::new(move |_, _| Ok(())));
    build.add_arg(use_lens_arg(), Box::new(move |_, _| Ok(())));
    build.add_arg(use_focus_arg(), Box::new(move |_, _| Ok(())));
    build.add_arg(use_polys_arg(), Box::new(move |_, _| Ok(())));
}

//mp add_arg_camera_database
pub fn add_arg_camera_database<C, F>(build: &mut CommandBuilder<C>, set: F, required: bool)
where
    C: CommandArgs<Error = Error>,
    F: Fn(&mut C, CameraDatabase) -> Result<()> + 'static,
{
    build.add_arg(
        camera_database_arg(required),
        Box::new(move |args, matches| get_camera_database(matches).and_then(|v| set(args, v))),
    );
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

//fp set_opt_camera_database
pub fn set_opt_camera_database(
    matches: &ArgMatches,
    cdb: &mut Option<CameraDatabase>,
) -> Result<()> {
    let filename = matches.get_one::<String>("camera_db").unwrap();
    let json = json::read_file(filename)?;
    *cdb = Some(CameraDatabase::from_json(&json)?);
    Ok(())
}

//fi get_camera_of_db
fn get_camera_of_db(matches: &ArgMatches, cdb: &CameraDatabase) -> Result<CameraInstance> {
    let camera_filename = matches.get_one::<String>("camera").unwrap();
    let camera_json = json::read_file(camera_filename)?;
    Ok(CameraInstance::from_json(cdb, &camera_json)?)
}

//fp get_camera
pub fn get_camera(matches: &ArgMatches, project: &Project) -> Result<CameraInstance> {
    let camera_filename = matches.get_one::<String>("camera").unwrap();
    let camera_json = json::read_file(camera_filename)?;
    CameraInstance::from_json(&project.cdb_ref(), &camera_json)
}

//a CameraCalibrate
//fi get_calibration_mapping
fn get_calibration_mapping(matches: &ArgMatches) -> Result<CalibrationMapping> {
    let filename = matches.get_one::<String>("calibration_mapping").unwrap();
    let json = json::read_file(filename)?;
    Ok(CalibrationMapping::from_json(&json)?)
}

//mp add_arg_calibration_mapping
pub fn add_arg_calibration_mapping<C, F>(build: &mut CommandBuilder<C>, set: F, required: bool)
where
    C: CommandArgs<Error = Error>,
    F: Fn(&mut C, CalibrationMapping) -> Result<()> + 'static,
{
    build.add_arg(
        Arg::new("calibration_mapping")
            .long("mappings")
            .short('m')
            .required(required)
            .help("Camera calibration mapping JSON")
            .action(ArgAction::Set),
        Box::new(move |mut args, matches| {
            get_calibration_mapping(matches).and_then(|v| set(&mut args, v))
        }),
    );
}
