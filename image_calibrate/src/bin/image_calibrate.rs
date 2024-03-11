//a Imports
use std::collections::HashMap;

use clap::{Arg, ArgAction, Command};
use geo_nd::{quat, vector};
use image_calibrate::{
    cmdline_args, image, json, BestMapping, CameraDatabase, CameraMapping, NamedPointSet, Point3D,
    Ray,
};

//a Types
//ti SubCmdFn
type SubCmdFn = fn(CameraDatabase, NamedPointSet, &clap::ArgMatches) -> Result<(), String>;

//a Images
//fi image_cmd
fn image_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("image").about("Read image and draw crosses on named and mapped points");
    let cmd = cmdline_args::add_pms_arg(cmd, true);
    let cmd = cmdline_args::add_camera_arg(cmd, true);
    let cmd = cmdline_args::add_image_read_arg(cmd, true);
    let cmd = cmdline_args::add_image_write_arg(cmd, true);
    (cmd, image_fn)
}

//fi image_fn
fn image_fn(
    cdb: CameraDatabase,
    nps: NamedPointSet,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let pms = cmdline_args::get_pms(matches, &nps)?;
    let camera = cmdline_args::get_camera(matches, &cdb)?;
    let camera_mapping = CameraMapping::of_camera(camera);
    let mappings = pms.mappings();

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
                image::draw_cross(&mut img, mapped, 5.0, red);
            }
            image::write_image(&mut img, write_filename)?;
        }
    }
    Ok(())
}

//a Interrogate (show_mappings etc)
//fi show_mappings_cmd
fn show_mappings_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("show_mappings")
        .about("Find location and orientation for a camera to map points to model");
    let cmd = cmdline_args::add_pms_arg(cmd, true);
    let cmd = cmdline_args::add_camera_arg(cmd, true);
    (cmd, show_mappings_fn)
}

//fi show_mappings_fn
fn show_mappings_fn(
    cdb: CameraDatabase,
    nps: NamedPointSet,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let pms = cmdline_args::get_pms(matches, &nps)?;
    let camera = cmdline_args::get_camera(matches, &cdb)?;
    let camera_mapping = CameraMapping::of_camera(camera);
    let mappings = pms.mappings();

    let te = camera_mapping.total_error(mappings);
    let we = camera_mapping.worst_error(mappings);
    camera_mapping.show_mappings(mappings);
    camera_mapping.show_point_set(&nps);
    println!("WE {:.2} TE {:.2}", we, te);

    Ok(())
}

//a Locate
//fi locate_cmd
fn locate_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("locate")
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
    (cmd, locate_fn)
}

//fi locate_fn
fn locate_fn(
    cdb: CameraDatabase,
    nps: NamedPointSet,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let pms = cmdline_args::get_pms(matches, &nps)?;
    let camera = cmdline_args::get_camera(matches, &cdb)?;
    let camera_mapping = CameraMapping::of_camera(camera);
    let mappings = pms.mappings();

    let mut steps = 11;
    if let Some(s) = matches.get_one::<usize>("steps") {
        steps = *s;
    }
    if !(1..100).contains(&steps) {
        return Err(format!(
            "Steps value should be between 1 and 100: was given {}",
            steps
        ));
    }

    let best_mapping = camera_mapping.get_best_location(mappings, steps);

    eprintln!("Best location {}", best_mapping);
    let camera_mapping = best_mapping.into_data();
    camera_mapping.show_mappings(mappings);
    // camera_mapping.show_point_set(&nps);

    println!("{}", serde_json::to_string_pretty(&camera_mapping).unwrap());
    Ok(())
}

//a Reorient
//fi reorient_cmd
fn reorient_cmd() -> (Command, SubCmdFn) {
    let cmd =
        Command::new("reorient").about("Improve orientation for a camera to map points to model");
    let cmd = cmdline_args::add_pms_arg(cmd, true);
    let cmd = cmdline_args::add_camera_arg(cmd, true);
    (cmd, reorient_fn)
}

//fi reorient_fn
fn reorient_fn(
    cdb: CameraDatabase,
    nps: NamedPointSet,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let pms = cmdline_args::get_pms(matches, &nps)?;
    let camera = cmdline_args::get_camera(matches, &cdb)?;
    let camera_mapping = CameraMapping::of_camera(camera);
    let mappings = pms.mappings();

    // use worst error
    let mut cam = camera_mapping.clone();
    let mut best_mapping = BestMapping::new(true, camera_mapping);
    loop {
        let mut q_list: Vec<(f64, [f64; 4])> = vec![];
        for i in 0..mappings.len() {
            for q in cam.get_quats_for_mappings_given_one(mappings, i) {
                q_list.push((1.0, q.into()));
            }
        }
        let qr = quat::weighted_average_many(q_list.into_iter()).into();
        cam = cam.with_direction(qr);
        let we = cam.worst_error(mappings);
        let te = cam.total_error(mappings);
        if !best_mapping.update_best(we, te, &cam) {
            break;
        }
        let location = cam.get_location_given_direction(mappings);
        cam = cam.placed_at(location);
    }

    eprintln!("Best mapping {}", best_mapping);
    let camera_mapping = best_mapping.into_data();
    camera_mapping.show_mappings(mappings);
    // camera_mapping.show_point_set(&nps);

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
    _cdb: CameraDatabase,
    _nps: NamedPointSet,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let ray_filenames: Vec<String> = matches
        .get_many::<String>("rays")
        .unwrap()
        .map(|v| v.into())
        .collect();

    for r in ray_filenames {
        let r_json = json::read_file(r)?;
        let named_rays: Vec<(String, Ray)> = json::from_json("ray list", &r_json)?;
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
    _cdb: CameraDatabase,
    nps: NamedPointSet,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let ray_filenames: Vec<String> = matches
        .get_many::<String>("rays")
        .unwrap()
        .map(|v| v.into())
        .collect();

    let mut named_point_rays = HashMap::new();
    for r in ray_filenames {
        let r_json = json::read_file(r)?;
        let rays: Vec<(String, Ray)> = json::from_json("ray list", &r_json)?;
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
    let pms = cmdline_args::get_pms(matches, &nps)?;
    let camera = cmdline_args::get_camera(matches, &cdb)?;
    let camera_mapping = CameraMapping::of_camera(camera);
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
    let pms = cmdline_args::get_pms(matches, &nps)?;
    let camera = cmdline_args::get_camera(matches, &cdb)?;
    let camera_mapping = CameraMapping::of_camera(camera);
    let mappings = pms.mappings();

    println!(
        "{}",
        serde_json::to_string_pretty(&camera_mapping.get_rays(mappings, true)).unwrap()
    );
    Ok(())
}

//a Adjust model
//fi adjust_model_cmd
fn adjust_model_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("adjust_model").about("Adjust *a* model point to get minimum error");
    let cmd = cmdline_args::add_np_arg(cmd, true);
    let cmd = cmdline_args::add_camera_pms_arg(cmd); // positional
    (cmd, adjust_model_fn)
}

//fi adjust_model_fn
fn adjust_model_fn(
    cdb: CameraDatabase,
    nps: NamedPointSet,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let np = cmdline_args::get_np(matches, &nps)?;
    let camera_pms = cmdline_args::get_camera_pms(matches, &cdb, &nps)?;
    let steps = 30;
    let orig = np.model();
    let mut best_dirn: BestMapping<Point3D> = BestMapping::new(true, [0., 0., 0.].into());
    let mut best_for_pts: Vec<BestMapping<Point3D>> =
        vec![BestMapping::new(true, [0., 0., 0.].into()); camera_pms.len()];
    for i in 0..100 {
        let x0 = (i % 10) as f64 / 10.0;
        let z0 = ((i / 10) % 10) as f64 / 10.0;
        let dpx: Point3D = vector::uniform_dist_sphere3([x0, z0], true).into();
        np.set_model(orig + (dpx * 0.1));
        let mut cp_data = vec![];
        let mut total_we = 0.;
        let mut total_te = 0.;
        for (i, (camera, pms)) in camera_pms.iter().enumerate() {
            let camera_mapping = CameraMapping::of_camera(camera.clone());
            let mappings = pms.mappings();
            let mut cam = camera_mapping
                .get_best_location(mappings, steps)
                .into_data();
            for _ in 0..10 {
                let quats = cam.get_quats_for_mappings_given_one(mappings, 1);
                let q_list: Vec<(f64, [f64; 4])> =
                    quats.into_iter().map(|q| (1.0, q.into())).collect();

                let qr = quat::weighted_average_many(q_list.into_iter()).into();
                cam = cam.with_direction(qr);
                let location = cam.get_location_given_direction(mappings);
                cam = cam.placed_at(location);
            }
            if let Some(_pm) = pms.mapping_of_np(&np) {
                let te = cam.total_error(mappings);
                let we = cam.worst_error(mappings);
                total_we += we;
                total_te += te;
                best_for_pts[i].update_best(we, te, &dpx);
                cp_data.push((cam.clone(), we, te));
            }
        }
        best_dirn.update_best(total_we, total_te, &dpx);
    }
    for (i, b) in best_for_pts.iter().enumerate() {
        eprintln!("{} : {} : {} ", i, b, orig + (*b.data()) * 0.1);
    }
    eprintln!("{} : {} ", best_dirn, orig + (*best_dirn.data()) * 0.1);
    Ok(())
}

//a Get model points
//fi get_model_points_cmd
fn get_model_points_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("get_model_points").about("Get model points from camers and pms");
    let cmd = cmdline_args::add_camera_pms_arg(cmd); // positional
    (cmd, get_model_points_fn)
}

//fi get_model_points_fn
fn get_model_points_fn(
    cdb: CameraDatabase,
    nps: NamedPointSet,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let camera_pms = cmdline_args::get_camera_pms(matches, &cdb, &nps)?;
    for (name, np) in nps.iter() {
        let mut ray_list = Vec::new();
        for (camera, pms) in camera_pms.iter() {
            if let Some(pm) = pms.mapping_of_np(np) {
                let camera_mapping = CameraMapping::of_camera(camera.clone());
                let ray = camera_mapping.get_pm_as_ray(pm, true);
                ray_list.push(ray);
            }
        }
        if ray_list.len() > 1 {
            let p = Ray::closest_point(&ray_list, &|_r| 1.0).unwrap();
            eprintln!(r#"["{}", [{},{},{}]],"#, name, p[0], p[1], p[2]);
        }
    }
    Ok(())
}

//a Main
//fi print_err
fn print_err(s: String) -> String {
    eprintln!("{}", s);
    s
}

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
        image_cmd(),
        show_mappings_cmd(),
        locate_cmd(),
        reorient_cmd(),
        combine_rays_from_model_cmd(),
        combine_rays_from_camera_cmd(),
        create_rays_from_model_cmd(),
        create_rays_from_camera_cmd(),
        adjust_model_cmd(),
        get_model_points_cmd(),
    ] {
        subcmds.insert(c.get_name().into(), f);
        cmd = cmd.subcommand(c);
    }
    let cmd = cmd;

    let matches = cmd.get_matches();
    let cdb = cmdline_args::get_camera_database(&matches).map_err(print_err)?;
    let nps = cmdline_args::get_nps(&matches).map_err(print_err)?;

    let (subcommand, submatches) = matches.subcommand().unwrap();
    for (name, sub_cmd_fn) in subcmds {
        if subcommand == name {
            return sub_cmd_fn(cdb, nps, submatches).map_err(print_err);
        }
    }
    unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`");
}
