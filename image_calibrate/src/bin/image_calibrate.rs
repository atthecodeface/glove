//a Imports
use std::collections::HashMap;

use clap::{Arg, ArgAction, Command};
use geo_nd::quat;
use image_calibrate::{
    cmdline_args, image, json, CameraDatabase, CameraMapping, NamedPointSet, Ray,
};

//a Types
//ti SubCmdFn
type SubCmdFn = fn(CameraDatabase, NamedPointSet, &clap::ArgMatches) -> Result<(), String>;

//a Locate V2
//fi locate_v2_cmd
fn locate_v2_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("locate_v2")
        .about("Find location and orientation for a camera to map points to model")
        .arg(
            Arg::new("steps")
                .long("steps")
                .required(false)
                .help("Number of steps per camera placement to try")
                .value_parser(clap::value_parser!(usize))
                .action(ArgAction::Set),
        );
    let cmd = cmdline_args::add_pms_arg(cmd, true);
    let cmd = cmdline_args::add_camera_arg(cmd, true);
    let cmd = cmdline_args::add_errors_arg(cmd);
    let cmd = cmdline_args::add_image_read_arg(cmd, false);
    let cmd = cmdline_args::add_image_write_arg(cmd, false);
    (cmd, locate_v2_fn)
}

//fi locate_v2_fn
fn locate_v2_fn(
    cdb: CameraDatabase,
    nps: NamedPointSet,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let pms = cmdline_args::get_pms(&matches, &nps)?;
    let camera = cmdline_args::get_camera(&matches, &cdb)?;
    let mut camera_mapping = CameraMapping::of_camera(camera);
    let mappings = pms.mappings();
    let error_method = cmdline_args::get_error_fn(&matches);

    let mut steps = 11;
    if let Some(s) = matches.get_one::<usize>("steps") {
        steps = *s;
    }
    if steps < 1 || steps > 101 {
        return Err(format!(
            "Steps value should be between 1 and 100: was given {}",
            steps
        ));
    }

    let mut cam = camera_mapping.get_best_location(mappings, steps);

    for _ in 0..10 {
        let quats = cam.get_quats_for_mappings_given_one(mappings, 1);
        let q_list: Vec<(f64, [f64; 4])> = quats.into_iter().map(|q| (1.0, q.into())).collect();

        let qr = quat::weighted_average_many(&q_list).into();
        cam = cam.with_direction(qr);
        let location = cam.get_location_given_direction(mappings);
        cam = cam.placed_at(location);
    }

    let camera_mapping = cam;

    let te = camera_mapping.total_error(mappings);
    let we = camera_mapping.worst_error(mappings);
    camera_mapping.show_mappings(mappings);
    // camera_mapping.show_point_set(&nps);

    if let Some(read_filename) = matches.get_one::<String>("read") {
        let mut img = image::read_image(read_filename)?;
        let white = &[255, 255, 255, 255];
        let red = &[255, 180, 255, 255];
        if let Some(write_filename) = matches.get_one::<String>("write") {
            for m in mappings {
                image::draw_cross(&mut img, m.screen(), m.error(), white);
            }
            for (_name, p) in nps.iter() {
                let mapped = camera_mapping.map_model(p.model());
                image::draw_cross(&mut img, &mapped, 5.0, &red);
            }
            image::write_image(&mut img, write_filename)?;
        }
    }

    eprintln!("Final WE {:.2} {:.2} Camera {}", we, te, camera_mapping);
    println!("{}", serde_json::to_string_pretty(&camera_mapping).unwrap());
    Ok(())
}

//a CombineFrom
//fi combine_rays_from_model_cmd
fn combine_rays_from_model_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("combine_rays_from_model")
        .about("Combine rays from a model")
        .arg(
            Arg::new("rays")
                .required(true)
                .help("Ray JSON files to be combined")
                .action(ArgAction::Append),
        );
    (cmd, combine_rays_from_model_fn)
}

//fi combine_rays_from_model_fn
fn combine_rays_from_model_fn(
    cdb: CameraDatabase,
    nps: NamedPointSet,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let mut ray_filenames: Vec<String> = matches
        .get_many::<String>("rays")
        .unwrap()
        .map(|v| v.into())
        .collect();

    for r in ray_filenames {
        let r_json = json::read_file(r)?;
        let mut named_rays: Vec<(String, Ray)> = json::from_json("ray list", &r_json)?;
        let mut names = Vec::new();
        let mut ray_list = Vec::new();
        for (name, ray) in named_rays {
            names.push(name);
            ray_list.push(ray);
        }
        if ray_list.len() > 1 {
            let p = Ray::closest_point(&ray_list, &|r| 1.0 / r.tan_error()).unwrap();
            println!("Camera at {}", p);
            for (name, ray) in names.iter().zip(ray_list.iter()) {
                let (k, d_sq) = ray.distances(&p);
                eprintln!("{}: k {} dsq {} d {}", name, k, d_sq, d_sq.sqrt());
            }
        }
    }
    Ok(())
}

//fi combine_rays_from_camera_cmd
fn combine_rays_from_camera_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("combine_rays_from_camera")
        .about("Combine rays from a camera")
        .arg(
            Arg::new("rays")
                .required(true)
                .help("Ray JSON files to be combined")
                .action(ArgAction::Append),
        );
    (cmd, combine_rays_from_camera_fn)
}

//fi combine_rays_from_camera_fn
fn combine_rays_from_camera_fn(
    cdb: CameraDatabase,
    nps: NamedPointSet,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let mut ray_filenames: Vec<String> = matches
        .get_many::<String>("rays")
        .unwrap()
        .map(|v| v.into())
        .collect();

    let mut named_point_rays = HashMap::new();
    for r in ray_filenames {
        let r_json = json::read_file(r)?;
        let mut rays: Vec<(String, Ray)> = json::from_json("ray list", &r_json)?;
        for (name, ray) in rays {
            if nps.get_pt(&name).is_none() {
                eprintln!(
                    "Warning: failed to find point name '{}' in named point set",
                    &name
                );
            } else {
                if !named_point_rays.contains_key(&name) {
                    named_point_rays.insert(name.clone(), Vec::new());
                }
                named_point_rays.get_mut(&name).unwrap().push(ray);
            }
        }
    }

    for (name, ray_list) in named_point_rays {
        if ray_list.len() > 1 {
            let p = Ray::closest_point(&ray_list, &|r| 1.0 / r.tan_error()).unwrap();
            eprintln!("Point '{}' - even weight - {}", name, p);
        }
    }

    Ok(())
}

//a Create Rays
//fi create_rays_from_model_cmd
fn create_rays_from_model_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("create_rays_from_model")
        .about("Create rays for a given located camera and its mappings");
    let cmd = cmdline_args::add_pms_arg(cmd, true);
    let cmd = cmdline_args::add_camera_arg(cmd, true);
    (cmd, create_rays_from_model_fn)
}

//fi create_rays_from_model_fn
fn create_rays_from_model_fn(
    cdb: CameraDatabase,
    nps: NamedPointSet,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let pms = cmdline_args::get_pms(&matches, &nps)?;
    let camera = cmdline_args::get_camera(&matches, &cdb)?;
    let mut camera_mapping = CameraMapping::of_camera(camera);
    let mappings = pms.mappings();

    println!(
        "{}",
        serde_json::to_string_pretty(&camera_mapping.get_rays(mappings, false)).unwrap()
    );
    Ok(())
}

//fi create_rays_from_camera_cmd
fn create_rays_from_camera_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("create_rays_from_camera")
        .about("Create rays for a given located camera and its mappings");
    let cmd = cmdline_args::add_pms_arg(cmd, true);
    let cmd = cmdline_args::add_camera_arg(cmd, true);
    (cmd, create_rays_from_camera_fn)
}

//fi create_rays_from_camera_fn
fn create_rays_from_camera_fn(
    cdb: CameraDatabase,
    nps: NamedPointSet,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let pms = cmdline_args::get_pms(&matches, &nps)?;
    let camera = cmdline_args::get_camera(&matches, &cdb)?;
    let mut camera_mapping = CameraMapping::of_camera(camera);
    let mappings = pms.mappings();

    println!(
        "{}",
        serde_json::to_string_pretty(&camera_mapping.get_rays(mappings, true)).unwrap()
    );
    Ok(())
}

//a Main
//fi main
fn main() -> Result<(), String> {
    let cmd = Command::new("image_calibrate")
        .about("Image calibration tool")
        .version("0.1.0")
        .subcommand_required(true);
    let cmd = cmdline_args::add_camera_database_arg(cmd, true);
    let cmd = cmdline_args::add_nps_arg(cmd, true);

    let mut subcmds: HashMap<String, SubCmdFn> = HashMap::new();
    let mut cmd = cmd;
    for (c, f) in [
        locate_v2_cmd(),
        combine_rays_from_model_cmd(),
        combine_rays_from_camera_cmd(),
        create_rays_from_model_cmd(),
        create_rays_from_camera_cmd(),
    ] {
        subcmds.insert(c.get_name().into(), f);
        cmd = cmd.subcommand(c);
    }
    let cmd = cmd;

    let matches = cmd.get_matches();
    let cdb = cmdline_args::get_camera_database(&matches)?;
    let nps = cmdline_args::get_nps(&matches)?;

    let (subcommand, submatches) = matches.subcommand().unwrap();
    for (name, sub_cmd_fn) in subcmds {
        if subcommand == name {
            return sub_cmd_fn(cdb, nps, submatches);
        }
    }
    unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`");
}
