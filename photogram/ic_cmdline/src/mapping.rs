//a Modules
use std::rc::Rc;

use clap::{Arg, ArgAction, ArgMatches, Command};

use ic_base::{JsonParsable, PathSet, Result};
use ic_camera::{CameraDatabase, CameraInstance, CameraInstanceDesc};
use ic_mapping::{NamedPoint, NamedPointSet, PointMappingSet};

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
pub fn get_nps(matches: &ArgMatches) -> Result<NamedPointSet> {
    let mut nps = NamedPointSet::default();
    for nps_filename in matches.get_many::<String>("nps").unwrap() {
        let (_, new_nps) = NamedPointSet::load_json_file(&PathSet::default(), nps_filename, &())?;
        nps.merge(new_nps);
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
pub fn get_np(matches: &ArgMatches, nps: &NamedPointSet) -> Result<Rc<NamedPoint>> {
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
pub fn get_pms(matches: &ArgMatches, nps: &NamedPointSet) -> Result<PointMappingSet> {
    let pms_filename = matches.get_one::<String>("pms").unwrap();

    let (_, (pms, pms_not_found)) =
        PointMappingSet::load_json_file(&PathSet::default(), pms_filename, nps)?;
    if !pms_not_found.is_empty() {
        eprintln!("Warning: {pms_not_found:?}");
    }
    Ok(pms)
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
) -> Result<Vec<(CameraInstance, PointMappingSet)>> {
    let mut result = vec![];
    let mut cam = None;
    for filename in matches.get_many::<String>("camera_pms").unwrap() {
        if cam.is_none() {
            cam = {
                let (_, camera) =
                    CameraInstanceDesc::load_json_file(&PathSet::default(), filename, cdb)?;
                Some(camera)
            };
        } else {
            let (_, (pms, pms_not_found)) =
                PointMappingSet::load_json_file(&PathSet::default(), filename, nps)?;

            if !pms_not_found.is_empty() {
                eprintln!("Warning: {pms_not_found:?}");
            }
            result.push((cam.unwrap(), pms));
            cam = None;
        }
    }
    Ok(result)
}
