//a Imports
use std::collections::HashMap;

use clap::{Arg, ArgAction, Command};
use geo_nd::quat;
use image_calibrate::{
    cmdline_args, image, json, CameraDatabase, CameraMapping, NamedPointSet, Point3D, Ray,
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
    let pms = cmdline_args::get_pms(&matches, &nps)?;
    let camera = cmdline_args::get_camera(&matches, &cdb)?;
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
                image::draw_cross(&mut img, mapped, 5.0, &red);
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
    let pms = cmdline_args::get_pms(&matches, &nps)?;
    let camera = cmdline_args::get_camera(&matches, &cdb)?;
    let camera_mapping = CameraMapping::of_camera(camera);
    let mappings = pms.mappings();

    let te = camera_mapping.total_error(mappings);
    let we = camera_mapping.worst_error(mappings);
    camera_mapping.show_mappings(mappings);
    camera_mapping.show_point_set(&nps);
    println!("WE {:.2} TE {:.2}", we, te);

    Ok(())
}

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
        )
        .arg(
            Arg::new("orientations")
                .long("orientations")
                .required(false)
                .help("Number of orientation adjustments to try")
                .value_parser(clap::value_parser!(usize))
                .action(ArgAction::Set),
        );
    let cmd = cmdline_args::add_pms_arg(cmd, true);
    let cmd = cmdline_args::add_camera_arg(cmd, true);
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
    let camera_mapping = CameraMapping::of_camera(camera);
    let mappings = pms.mappings();

    let mut steps = 11;
    if let Some(s) = matches.get_one::<usize>("steps") {
        steps = *s;
    }
    let mut orientations = 11;
    if let Some(o) = matches.get_one::<usize>("orientations") {
        orientations = *o;
    }
    if steps < 1 || steps > 101 {
        return Err(format!(
            "Steps value should be between 1 and 100: was given {}",
            steps
        ));
    }
    if orientations < 1 || orientations > 10001 {
        return Err(format!(
            "Orientations value should be between 1 and 100: was given {}",
            orientations
        ));
    }

    let mut cam = camera_mapping.get_best_location(mappings, steps);

    for _ in 0..orientations {
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
    let pms = cmdline_args::get_pms(&matches, &nps)?;
    let camera = cmdline_args::get_camera(&matches, &cdb)?;
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
    let pms = cmdline_args::get_pms(&matches, &nps)?;
    let camera = cmdline_args::get_camera(&matches, &cdb)?;
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
#[derive(Default, Debug, Clone, Copy)]
struct BestDirn {
    pub we: f64,
    pub te: f64,
    pub dpx: Point3D,
}
impl BestDirn {
    fn new() -> Self {
        Self {
            we: 1.0E8,
            te: 1.0E8,
            dpx: [0., 0., 0.].into(),
        }
    }
    fn best(&mut self, we: f64, te: f64, dpx: &Point3D) {
        if te < self.te {
            self.we = we;
            self.te = te;
            self.dpx = *dpx;
        }
    }
}
fn adjust_model_fn(
    cdb: CameraDatabase,
    nps: NamedPointSet,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let np = cmdline_args::get_np(&matches, &nps)?;
    let camera_pms = cmdline_args::get_camera_pms(&matches, &cdb, &nps)?;
    let steps = 30;
    let orig = np.model();
    let mut best_dirn = BestDirn::new();
    let mut best_for_pts = vec![BestDirn::new(); camera_pms.len()];
    for i in 0..100 {
        let x0 = (i % 10) as f64 / 10.0;
        let z0 = ((i / 10) % 10) as f64 / 10.0;
        let ca = 0.3_f64.cos();
        let sa = 0.3_f64.sin();
        let x1 = 0.1 + x0 * ca - z0 * sa;
        let z1 = 0.1 + z0 * ca + x0 * sa;
        let x2 = if x1 > 1.0 { x1 - 1.0 } else { x1 };
        let z2 = if z1 > 1.0 { z1 - 1.0 } else { z1 };
        let theta = 2.0 * std::f64::consts::PI * x2;
        let phi = (2.0 * z2 - 1.0).acos();
        let dpx: Point3D = [theta.cos() * phi.sin(), theta.sin() * phi.sin(), phi.cos()].into();
        np.set_model(orig + (dpx * 0.1));
        let mut cp_data = vec![];
        let mut total_we = 0.;
        let mut total_te = 0.;
        for (i, (camera, pms)) in camera_pms.iter().enumerate() {
            let camera_mapping = CameraMapping::of_camera(camera.clone());
            let mappings = pms.mappings();
            let mut cam = camera_mapping.get_best_location(mappings, steps);
            for _ in 0..10 {
                let quats = cam.get_quats_for_mappings_given_one(mappings, 1);
                let q_list: Vec<(f64, [f64; 4])> =
                    quats.into_iter().map(|q| (1.0, q.into())).collect();

                let qr = quat::weighted_average_many(&q_list).into();
                cam = cam.with_direction(qr);
                let location = cam.get_location_given_direction(mappings);
                cam = cam.placed_at(location);
            }
            if let Some(_pm) = pms.mapping_of_np(&np) {
                let te = cam.total_error(mappings);
                let we = cam.worst_error(mappings);
                total_we += we;
                total_te += te;
                best_for_pts[i].best(we, te, &dpx);
                cp_data.push((cam.clone(), we, te));
            }
        }
        best_dirn.best(total_we, total_te, &dpx);
    }
    for (i, b) in best_for_pts.iter().enumerate() {
        eprintln!(
            "{} : {} : {} : {} : {} ",
            i,
            b.we,
            b.te,
            b.dpx,
            orig + b.dpx * 0.1
        );
    }
    eprintln!(
        "{} : {} : {} : {} ",
        best_dirn.we,
        best_dirn.te,
        best_dirn.dpx,
        orig + best_dirn.dpx * 0.1
    );
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
    let camera_pms = cmdline_args::get_camera_pms(&matches, &cdb, &nps)?;
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
            eprintln!("{}: {}", name, p);
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
        locate_v2_cmd(),
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
    let cdb = cmdline_args::get_camera_database(&matches).map_err(|e| print_err(e))?;
    let nps = cmdline_args::get_nps(&matches).map_err(|e| print_err(e))?;

    let (subcommand, submatches) = matches.subcommand().unwrap();
    for (name, sub_cmd_fn) in subcmds {
        if subcommand == name {
            return sub_cmd_fn(cdb, nps, submatches).map_err(|e| print_err(e));
        }
    }
    unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`");
}
