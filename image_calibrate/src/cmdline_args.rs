//a Modules
use std::rc::Rc;

use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};

use crate::{
    image, json, CameraDatabase, CameraInstance, CameraMapping, CameraPolynomial,
    CameraPolynomialCalibrate, Color, NamedPoint, NamedPointSet, PointMapping, PointMappingSet,
};
use image::DynamicImage;

//a NamedPointSet / NamedPoint
//fp add_nps_arg
pub fn add_nps_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
            Arg::new("nps")
            .long("nps")
            .short('n')
            .required(required)
            .help("Specifies a named point set json")
            .long_help("A filename of a JSON file that provides the named set of points and their 3D locations for a particular model")
            .action(ArgAction::Set),
    )
}

//fp get_nps
pub fn get_nps(matches: &ArgMatches) -> Result<NamedPointSet, String> {
    let nps_filename = matches.get_one::<String>("nps").unwrap();
    let nps_json = json::read_file(nps_filename)?;
    NamedPointSet::from_json(&nps_json)
}

//fp add_np_arg
pub fn add_np_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
        Arg::new("np")
            .long("np")
            .required(required)
            .help("Specifies a named point")
            .long_help("A named point within the named point set required for the command")
            .action(ArgAction::Set),
    )
}

//fp get_np
pub fn get_np(matches: &ArgMatches, nps: &NamedPointSet) -> Result<Rc<NamedPoint>, String> {
    let np = matches.get_one::<String>("np").unwrap();
    nps.get_pt_err(np)
}

//a PointMappingSet
//fp add_pms_arg
pub fn add_pms_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
            Arg::new("pms")
                .long("pms")
                .short('p')
                .required(required)
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
        let nf = pms.read_json(nps, &pms_json, true)?;
        if !nf.is_empty() {
            eprintln!("Warning: {}", nf);
        }
    }
    Ok(pms)
}

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
    let body = db.get_body_err(body_name)?;
    let lens = db.get_lens_err(lens_name)?;
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
pub fn get_camera(matches: &ArgMatches, cdb: &CameraDatabase) -> Result<CameraInstance, String> {
    let camera_filename = matches
        .get_one::<String>("camera")
        .ok_or("A camera position/orientation JSON is required")?;
    let camera_json = json::read_file(camera_filename)?;
    CameraInstance::from_json(cdb, &camera_json)
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

//a Camera and associated point maps - Positional
//fp add_camera_pms_arg
pub fn add_camera_pms_arg(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("camera_pms")
            .required(true)
            .help("Pairs of camera and JSON files to be read")
            .action(ArgAction::Append),
    )
}

//fp get_camera_pms
pub fn get_camera_pms(
    matches: &ArgMatches,
    cdb: &CameraDatabase,
    nps: &NamedPointSet,
) -> Result<Vec<(CameraInstance, PointMappingSet)>, String> {
    let mut result = vec![];
    let mut cam = None;
    for filename in matches.get_many::<String>("camera_pms").unwrap() {
        if cam.is_none() {
            let camera_json = json::read_file(filename)?;
            cam = Some(CameraInstance::from_json(cdb, &camera_json)?);
        } else {
            let mut pms = PointMappingSet::new();
            let pms_json = json::read_file(filename)?;
            let nf = pms.read_json(nps, &pms_json, true)?;
            if !nf.is_empty() {
                eprintln!("Warning: {}", nf);
            }
            result.push((cam.unwrap(), pms));
            cam = None;
        }
    }
    Ok(result)
}

//a Image options
//fp add_image_read_arg
pub fn add_image_read_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
        Arg::new("read")
            .long("read")
            .short('r')
            .required(required)
            .help("Image to read")
            .action(ArgAction::Set),
    )
}

//fp get_image_read
pub fn get_image_read(matches: &ArgMatches) -> Result<DynamicImage, String> {
    let read_filename = matches
        .get_one::<String>("read")
        .ok_or("An image filename to read is required")?;
    let img = image::read_image(read_filename)?;
    Ok(img)
}

//fp get_image_read_or_create
pub fn get_image_read_or_create(
    matches: &ArgMatches,
    camera: &CameraInstance,
) -> Result<DynamicImage, String> {
    let read_filename = matches.get_one::<String>("read");
    let img = image::read_or_create_image(
        camera.body().px_width() as usize,
        camera.body().px_height() as usize,
        read_filename.map(|x| x.as_str()),
    )?;
    Ok(img)
}

//fp add_image_write_arg
pub fn add_image_write_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
        Arg::new("write")
            .long("write")
            .short('w')
            .required(required)
            .help("Image to write")
            .action(ArgAction::Set),
    )
}

//fp get_opt_image_write_filename
pub fn get_opt_image_write_filename(matches: &ArgMatches) -> Result<Option<String>, String> {
    Ok(matches.get_one::<String>("write").cloned())
}

//a Colors
//fp add_color_arg
pub fn add_color_arg(cmd: Command, prefix: &str, help: &str, required: bool) -> Command {
    let (id, long) = {
        if prefix.is_empty() {
            ("c".to_string(), "color".to_string())
        } else {
            (prefix.to_string(), format!("{prefix}"))
        }
    };
    cmd.arg(
        Arg::new(id)
            .long(long)
            .required(required)
            .help(help.to_string())
            .action(ArgAction::Set),
    )
}

//fp get_opt_color
pub fn get_opt_color(matches: &ArgMatches, prefix: &str) -> Result<Option<Color>, String> {
    if let Some(bg) = matches.get_one::<String>(prefix) {
        let c: Color = bg.as_str().try_into()?;
        Ok(Some(c))
    } else {
        Ok(None)
    }
}

//fp add_bg_color_arg
pub fn add_bg_color_arg(cmd: Command, required: bool) -> Command {
    add_color_arg(cmd, "bg", "Image background color", required)
}

//fp get_opt_bg_color
pub fn get_opt_bg_color(matches: &ArgMatches) -> Result<Option<Color>, String> {
    get_opt_color(matches, "bg")
}

//a Errors
//fp add_errors_arg
pub fn add_errors_arg(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("worst_error")
            .long("worst")
            .help("Use worst error for resolving")
            .action(ArgAction::SetTrue),
    )
    .arg(
        Arg::new("total_error")
            .long("total")
            .help("Use total error for resolving")
            .action(ArgAction::SetTrue),
    )
}

//fp get_error_fn
pub fn get_error_fn(
    matches: &ArgMatches,
) -> for<'a, 'b> fn(&'a CameraMapping, &'b [PointMapping], usize) -> f64 {
    if matches.get_flag("worst_error") {
        let error_method: for<'a, 'b> fn(&'a CameraMapping, &'b [PointMapping], usize) -> f64 =
            |c, m, _n| c.worst_error(m);
        error_method
    } else {
        let error_method: for<'a, 'b> fn(&'a CameraMapping, &'b [PointMapping], usize) -> f64 =
            |c, m, _n| c.total_error(m);
        error_method
    }
}
