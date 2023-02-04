//a Modules
use std::rc::Rc;

use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};

use crate::{
    json, Camera, CameraBody, CameraPolynomial, CameraProjection, NamedPointSet, PointMappingSet,
    SphericalLensPoly,
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
            .required(false)
            .help("Camera body JSON")
            .action(ArgAction::Set),
    )
    .arg(
        Arg::new("sph")
            .long("spherical")
            .short('s')
            .required(false)
            .help("Spherical camera lens mapping JSON")
            .action(ArgAction::Set),
    )
}

//fp get_camera_projection
pub fn get_camera_projection(matches: &ArgMatches) -> Result<Rc<dyn CameraProjection>, String> {
    let mm_focus_distance = *matches
        .get_one::<f64>("focus")
        .ok_or("A mm focus distance is required (float)")?;
    let body_filename = matches
        .get_one::<String>("body")
        .ok_or("A camera body JSON is required")?;
    let body_json = json::read_file(body_filename)?;
    let body: CameraBody = json::from_json("camera body", &body_json)?;

    if let Some(lens_filename) = matches.get_one::<String>("sph") {
        let lens_json = json::read_file(lens_filename)?;
        let lens: SphericalLensPoly = json::from_json("camera lens", &lens_json)?;
        let camera = CameraPolynomial::new(body, lens, mm_focus_distance);
        Ok(Rc::new(camera))
    } else {
        Err("No camera lens JSON specified".into())
    }
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
) -> Result<Camera, String> {
    let camera_filename = matches
        .get_one::<String>("camera")
        .ok_or("A camera position/orientation JSON is required")?;
    let camera_json = json::read_file(camera_filename)?;
    let mut camera: Camera = json::from_json("camera", &camera_json)?;
    camera.set_projection(camera_projection);
    Ok(camera)
}
