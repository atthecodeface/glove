//a Modules
use std::rc::Rc;

use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};

use crate::{
    image, json, CameraDatabase, CameraInstance, CameraPolynomial, CameraPolynomialCalibrate,
    CameraPtMapping, Cip, Color, NamedPoint, NamedPointSet, PointMapping, PointMappingSet, Project,
    Rrc,
};
use image::DynamicImage;

//a ThreadPool
//fp add_threads_arg
pub fn add_threads_arg(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("threads")
            .long("threads")
            .short('t')
            .default_value("4")
            .help("Number of threads")
            .long_help("Number of threads for the server to run to handle simultaneous requests")
            .value_parser(value_parser!(usize))
            .action(ArgAction::Set),
    )
}

//fp add_port_arg
pub fn add_port_arg(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("port")
            .long("port")
            .short('p')
            .default_value("8020")
            .help("Port to use")
            .long_help("The TCP port number to use for the server")
            .value_parser(value_parser!(usize))
            .action(ArgAction::Set),
    )
}

//fp get_threads
pub fn get_threads(matches: &ArgMatches) -> Result<usize, String> {
    let num_threads = *matches.get_one::<usize>("threads").unwrap();
    Ok(num_threads)
}

//fp get_port
pub fn get_port(matches: &ArgMatches) -> Result<usize, String> {
    let port = *matches.get_one::<usize>("port").unwrap();
    Ok(port)
}

//a File system
//fp add_file_root_arg
pub fn add_file_root_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
        Arg::new("file_root")
            .long("file_root")
            .short('F')
            .required(required)
            .help("Root of files to serve")
            .long_help("Root of the files to server from the HTTP server, for file requests")
            .action(ArgAction::Set),
    )
}

//fp get_file_root
pub fn get_file_root(matches: &ArgMatches) -> Result<String, String> {
    let file_root = matches.get_one::<String>("file_root").unwrap().to_owned();
    Ok(file_root)
}

//fp add_image_root_arg
pub fn add_image_root_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
        Arg::new("image_root")
            .long("image_root")
            .short('I')
            .required(required)
            .help("Root of images to serve")
            .long_help("Root of the image files to server from the HTTP server")
            .action(ArgAction::Set),
    )
}

//fp get_image_root
pub fn get_image_root(matches: &ArgMatches) -> Result<String, String> {
    let image_root = matches.get_one::<String>("image_root").unwrap().to_owned();
    Ok(image_root)
}

//fp add_project_root_arg
pub fn add_project_root_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
        Arg::new("project_root")
            .long("project_root")
            .short('P')
            .required(required)
            .help("Directory containing the projects to serve")
            .long_help("Root on the server where projects are held")
            .action(ArgAction::Set),
    )
}

//fp get_project_root
pub fn get_project_root(matches: &ArgMatches) -> Result<String, String> {
    let project_root = matches
        .get_one::<String>("project_root")
        .unwrap()
        .to_owned();
    Ok(project_root)
}

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
            .action(ArgAction::Append),
    )
}

//fp get_nps
pub fn get_nps(matches: &ArgMatches) -> Result<NamedPointSet, String> {
    let mut nps = NamedPointSet::new();
    for nps_filename in matches.get_many::<String>("nps").unwrap() {
        let nps_json = json::read_file(nps_filename)?;
        nps.merge(&NamedPointSet::from_json(&nps_json)?);
    }
    Ok(nps)
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
    let body = db.get_body_err(body_name)?.clone();
    let lens = db.get_lens_err(lens_name)?;
    let camera = CameraPolynomial::new(body, lens, mm_focus_distance);
    Ok(Rc::new(camera))
}

//a Project
//fp add_project_arg
pub fn add_project_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
        Arg::new("project")
            .long("proj")
            .alias("project")
            .required(required)
            .help("Project JSON")
            .action(ArgAction::Set),
    )
}

//fp get_project
pub fn get_project(matches: &ArgMatches) -> Result<Project, String> {
    if let Some(project_filename) = matches.get_one::<String>("project") {
        let project_json = json::read_file(project_filename)?;
        let project: Project = json::from_json("project", &project_json)?;
        Ok(project)
    } else {
        let mut project = Project::default();
        let cdb = get_camera_database(matches)?;
        let nps = get_nps(matches)?;
        project.set_cdb(cdb.into());
        project.set_nps(nps.into());
        Ok(project)
    }
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

//a Cip
//fp add_cip_arg
pub fn add_cip_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
        Arg::new("cip")
            .long("cip")
            .value_parser(value_parser!(usize))
            .required(required)
            .help("CIP number (camera and PMS) within the project")
            .action(ArgAction::Set),
    )
}

//fp get_cip
pub fn get_cip(matches: &ArgMatches, project: &Project) -> Result<Rrc<Cip>, String> {
    if let Some(cip) = matches.get_one::<usize>("cip") {
        let cip = *cip;
        if cip < project.ncips() {
            Ok(project.cip(cip).clone())
        } else {
            Err(format!(
                "CIP {cip} is too large for the project (it has {} cips)",
                project.ncips()
            ))
        }
    } else {
        let cip = Cip::default();
        let camera = get_camera(matches, project)?;
        let pms = get_pms(matches, &project.nps_ref())?;
        *cip.camera_mut() = camera;
        *cip.pms_mut() = pms;
        let cip = cip.into();
        Ok(cip)
    }
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
pub fn get_camera(matches: &ArgMatches, project: &Project) -> Result<CameraInstance, String> {
    let camera_filename = matches
        .get_one::<String>("camera")
        .ok_or("A camera position/orientation JSON is required")?;
    let camera_json = json::read_file(camera_filename)?;
    CameraInstance::from_json(&project.cdb_ref(), &camera_json)
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
            (prefix.to_string(), prefix.to_string())
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
) -> for<'a, 'b> fn(&'a CameraInstance, &'b [PointMapping], usize) -> f64 {
    if matches.get_flag("worst_error") {
        let error_method: for<'a, 'b> fn(&'a CameraInstance, &'b [PointMapping], usize) -> f64 =
            |c, m, _n| c.worst_error(m);
        error_method
    } else {
        let error_method: for<'a, 'b> fn(&'a CameraInstance, &'b [PointMapping], usize) -> f64 =
            |c, m, _n| c.total_error(m);
        error_method
    }
}

//a General
//fp add_verbose_arg
pub fn add_verbose_arg(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("verbose")
            .long("verbose")
            .short('v')
            .help("Set verbosity")
            .long_help("Set verbose")
            .action(ArgAction::Set),
    )
}

//fp get_verbose
pub fn get_verbose(matches: &ArgMatches) -> bool {
    matches.get_one::<String>("verbose").is_some()
}
