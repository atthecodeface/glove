//a Imports
use std::collections::HashMap;

use clap::{Arg, ArgAction, Command};
use geo_nd::{quat, vector, Vector};
use image::Image;
use image_calibrate::{
    cmdline_args, image, json, BestMapping, CameraAdjustMapping, CameraDatabase, CameraMapping,
    CameraPtMapping, CameraShowMapping, CameraView, Color, NamedPointSet, Point3D, PointMappingSet,
    Ray, Region,
};

//a Types
//ti SubCmdFn
type SubCmdFn = fn(CameraDatabase, NamedPointSet, &clap::ArgMatches) -> Result<(), String>;

//a Images
//hi IMAGE_LONG_HELP
const IMAGE_LONG_HELP: &str = "\
Given a Named Point Set, from a camera (type, position/direction), and
a Point Mapping Set draw crosses on an image corresponding to the PMS
frame positions and the Named Point's model position mapped onto the
camera, and write out to a new image.

";

//fi image_cmd
fn image_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("image")
        .about("Read image and draw crosses on named and mapped points")
        .long_about(IMAGE_LONG_HELP);
    let cmd = cmdline_args::add_pms_arg(cmd, true);
    let cmd = cmdline_args::add_camera_arg(cmd, true);
    let cmd = cmdline_args::add_image_read_arg(cmd, false);
    let cmd = cmdline_args::add_image_write_arg(cmd, true);
    let cmd = cmdline_args::add_color_arg(
        cmd,
        "pms_color",
        "Color for original PMS frame crosses",
        false,
    );
    let cmd =
        cmdline_args::add_color_arg(cmd, "model_color", "Color for mapped model crosses", false);
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
    let mut img = cmdline_args::get_image_read_or_create(matches, &camera)?;
    let pms_color = cmdline_args::get_opt_color(matches, "pms_color")?;
    let model_color = cmdline_args::get_opt_color(matches, "model_color")?;
    let use_nps_colors = pms_color.is_none() && model_color.is_none();

    let mappings = pms.mappings();

    let write_filename = cmdline_args::get_opt_image_write_filename(matches)?.unwrap();
    if pms_color.is_some() || use_nps_colors {
        for m in mappings {
            let c = pms_color.as_ref().unwrap_or(m.model.color());
            img.draw_cross(m.screen(), m.error(), c);
        }
    }
    if model_color.is_some() || use_nps_colors {
        for (_name, p) in nps.iter() {
            let c = model_color.as_ref().unwrap_or(p.color());
            let mapped = camera.map_model(p.model());
            img.draw_cross(mapped, 5.0, c);
        }
    }
    img.write(&write_filename)?;
    Ok(())
}

//a Interrogate (show_mappings etc)
//fi show_mappings_cmd
fn show_mappings_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("show_mappings")
        .about("Show the total and worst error for a point mapping set");
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

    let mappings = pms.mappings();

    let te = camera.total_error(mappings);
    let we = camera.worst_error(mappings);
    camera.show_mappings(mappings);
    camera.show_point_set(&nps);
    println!("WE {:.2} TE {:.2}", we, te);

    Ok(())
}

//a Get point mappings
//hi GET_POINT_MAPPINGS_LONG_HELP
const GET_POINT_MAPPINGS_LONG_HELP: &str = "\
This treats the image file read in as a set of non-background color
regions each of which should be of a color representing a Named Point

A region is a contiguous set of non-background pixels. The
centre-of-gravity of each region is determined.

The Named Point associated with the color of the region is found, and
a Point Mapping Set is generated mapping the Named Points onto the
centre of the appropriate region.";

//fi get_point_mappings_cmd
fn get_point_mappings_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("get_point_mappings")
        .about("Read image and find regions")
        .long_about(GET_POINT_MAPPINGS_LONG_HELP);
    let cmd = cmdline_args::add_image_read_arg(cmd, true);
    let cmd = cmdline_args::add_bg_color_arg(cmd, false);
    (cmd, get_point_mappings_fn)
}

//fi get_point_mappings_fn
fn get_point_mappings_fn(
    _cdb: CameraDatabase,
    nps: NamedPointSet,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let img = cmdline_args::get_image_read(matches)?;
    let bg_color = cmdline_args::get_opt_bg_color(matches)?;
    let bg_color = bg_color.unwrap_or(Color::black());
    let regions = Region::regions_of_image(&img, &|c| !c.color_eq(&bg_color));
    let mut pms = PointMappingSet::default();
    for r in regions {
        let c = r.color();
        let np = nps.of_color(&c);
        if np.is_empty() {
            eprintln!("No named point with color {c} @ {:?}", r.cog());
        } else if np.len() > 1 {
            eprintln!(
                "More than one named point with color {c} @ {:?}: {}, {}, ...",
                r.cog(),
                np[0].name(),
                np[1].name(),
            );
        } else {
            let screen = r.cog();
            let screen = [screen.0, screen.1].into();
            let error = r.spread();
            pms.add_mapping(&nps, np[0].name(), &screen, error);
        }
    }
    println!("{}", serde_json::to_string_pretty(&pms).unwrap());
    eprintln!("Exported {} mappings", pms.mappings().len());
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
    let mappings = pms.mappings();

    let mut steps = 11;
    if let Some(s) = matches.get_one::<usize>("steps") {
        steps = *s;
    }
    if !(1..=100).contains(&steps) {
        return Err(format!(
            "Steps value should be between 1 and 100: was given {}",
            steps
        ));
    }

    let best_mapping = camera.get_best_location(mappings, steps);

    eprintln!("Best location {}", best_mapping);
    let camera = best_mapping.into_data();
    camera.show_mappings(mappings);

    println!("{}", serde_json::to_string_pretty(&camera).unwrap());
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
    let mappings = pms.mappings();

    // use worst error
    let mut cam = camera.clone();
    let mut best_mapping = BestMapping::new(true, cam.clone());
    loop {
        let mut q_list: Vec<(f64, [f64; 4])> = vec![];
        for i in 0..mappings.len() {
            for q in cam.get_quats_for_mappings_given_one(mappings, i) {
                q_list.push((1.0, q.into()));
            }
        }
        let qr = quat::weighted_average_many(q_list.into_iter()).into();
        cam = cam.clone_with_direction(qr);
        let we = cam.worst_error(mappings);
        let te = cam.total_error(mappings);
        if !best_mapping.update_best(we, te, &cam) {
            break;
        }
        let location = cam.get_location_given_direction(mappings);
        cam = cam.placed_at(location);
    }

    eprintln!("Best mapping {}", best_mapping);
    let camera = best_mapping.into_data();
    camera.show_mappings(mappings);

    println!("{}", serde_json::to_string_pretty(&camera).unwrap());
    Ok(())
}

//hi REORIENT2_LONG_HELP
const REORIENT2_LONG_HELP: &str = "\

Iteratively reorient the camera by determining the axis and amount *each* PMS
mapped point wants to rotate by, and rotating by the weighted
average.

The rotation desired for a PMS point is the axis/angle required to
rotate the ray vector from the camera through the point on the frame
to the ray of the *actual* model position of the point from the
camera.

The weighted average is biased by adding in some 'zero rotation's; the
camera is attempted to be rotated by this weighted average
(quaternion), and if the total error in the camera mapping is reduced
then the new rotation is kept.

The iteration stops when the new rotation produces a greater total
error in the mapping than the current orientation of the camera.

";

//fi reorient2_cmd
fn reorient2_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("reorient2")
        .about("Improve orientation for a camera to map points to model")
        .long_about(REORIENT2_LONG_HELP);
    let cmd = cmdline_args::add_pms_arg(cmd, true);
    let cmd = cmdline_args::add_camera_arg(cmd, true);
    (cmd, reorient2_fn)
}

//fi reorient2_fn
fn reorient2_fn(
    cdb: CameraDatabase,
    nps: NamedPointSet,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let pms = cmdline_args::get_pms(matches, &nps)?;
    let mut camera = cmdline_args::get_camera(matches, &cdb)?;

    camera.reorient_using_rays_from_model(pms.mappings());

    println!("{}", serde_json::to_string_pretty(&camera).unwrap());
    Ok(())
}

//a CombineFrom
//hi COMBINE_RAYS_FROM_MODEL_LONG_HELP
const COMBINE_RAYS_FROM_MODEL_LONG_HELP: &str = "\
This combines a list of rays from a JSON file and generates 
a model-space best location of ray intersection.

The rays in the file are from different model points and the direction
thereof should have been generated by casting rays through a camera
frame and applying in reverse the camera orientation.

Real-world rays will not intersect precisely; there will be a point
that has the minimum square distance from all the rays, though. This
is the point generated.

";

//fi combine_rays_from_model_cmd
fn combine_rays_from_model_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("combine_rays_from_model")
        .about("Combine rays from a model")
        .long_about(COMBINE_RAYS_FROM_MODEL_LONG_HELP);
    let cmd = cmdline_args::add_camera_arg(cmd, true);
    let cmd = cmd.arg(
        Arg::new("rays")
            .required(true)
            .long("rays")
            .help("Model ray JSON file")
            .action(ArgAction::Append),
    );
    (cmd, combine_rays_from_model_fn)
}

//fi combine_rays_from_model_fn
fn combine_rays_from_model_fn(
    cdb: CameraDatabase,
    _nps: NamedPointSet,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let camera = cmdline_args::get_camera(matches, &cdb)?;
    let ray_filename = matches.get_one::<String>("rays").unwrap();

    let r_json = json::read_file(ray_filename)?;
    let named_rays: Vec<(String, Ray)> = json::from_json("ray list", &r_json)?;
    let mut names = Vec::new();
    let mut ray_list = Vec::new();
    for (name, ray) in named_rays {
        names.push(name);
        ray_list.push(ray);
    }
    if ray_list.len() <= 1 {
        return Err(format!(
            "Not enough rays ({}) to combine to generate a position for the camera",
            ray_list.len()
        ));
    }

    let position = Ray::closest_point(&ray_list, &|r| 1.0 / r.tan_error()).unwrap();
    eprintln!("The rays from the model converge at the camera focal point at {position}",);
    let mut tot_d_sq = 0.0;
    for (_name, ray) in names.iter().zip(ray_list.iter()) {
        let (_k, d_sq) = ray.distances(&position);
        // eprintln!("{}: k {} dsq {} d {}", _name, _k, d_sq, d_sq.sqrt());
        tot_d_sq += d_sq;
    }
    eprintln!("Total dsq {tot_d_sq}");
    let camera = camera.placed_at(position);
    println!("{}", serde_json::to_string_pretty(&camera).unwrap());
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

    let mut k: Vec<String> = named_point_rays.keys().cloned().collect();
    k.sort();
    for name in &k {
        let ray_list = named_point_rays.get(name).unwrap();
        if ray_list.len() > 1 {
            let p = Ray::closest_point(ray_list, &|r| 1.0 / r.tan_error()).unwrap();
            eprintln!("Point '{}' - even weight - {}", name, p);
        }
    }

    Ok(())
}

//a Create Rays
//hi CREATE_RAYS_FROM_MODEL_LONG_HELP
const CREATE_RAYS_FROM_MODEL_LONG_HELP: &str = "\
This combines Named Point model positions, camera *orientation* and
PMS files, to determine rays from those model positions.

This takes the Point Mapping Set and a camera description and uses
only the orientation from that description.

For each Named Point that is mapped it casts a ray from the camera
through the frame to generate the direction of rays *relative* to the
camera orientation, then it applies the inverse camera rotation to get
the real world direction of the ray.

Given the Named Point's model position and world direction, it has a
Model-space ray for the named point.
";

//fi create_rays_from_model_cmd
fn create_rays_from_model_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("create_rays_from_model")
        .about("Create rays for a given located camera and its mappings")
        .long_about(CREATE_RAYS_FROM_MODEL_LONG_HELP);
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
    let mappings = pms.mappings();

    let named_rays = camera.get_rays(mappings, false);
    for (n, r) in &named_rays {
        let end = r.start + r.direction * 400.0;
        eprintln!("{n} {end}");
    }

    println!("{}", serde_json::to_string_pretty(&named_rays).unwrap());
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
    let mappings = pms.mappings();

    println!(
        "{}",
        serde_json::to_string_pretty(&camera.get_rays(mappings, true)).unwrap()
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
        np.set_model(Some(orig + (dpx * 0.1)));
        let mut cp_data = vec![];
        let mut total_we = 0.;
        let mut total_te = 0.;
        for (i, (camera, pms)) in camera_pms.iter().enumerate() {
            let camera_clone = camera.clone();
            let mappings = pms.mappings();
            let mut cam = camera_clone.get_best_location(mappings, steps).into_data();
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
//hi GET_MODEL_POINTS_LONG_HELP
const GET_MODEL_POINTS_LONG_HELP: &str = "\
This combines camera location and PMS files, using them to determine
model positions for Named Points.

This takes a list of Camera/PMS files, and reads them in.  Then, for
each Named Point in the NPS file read in it calculates one ray from
each camera location in the direction given by the PMS for the ray,
creating N rays for N cameras.

The intersection of these N rays is then determined, yielding a
model-space point for the Named Point.

A new NamedPointSet is generated from the original NPS with these new
model-space points from the ray intersections.";

//fi get_model_points_cmd
fn get_model_points_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("get_model_points")
        .about("Get model points from camera and pms")
        .long_about(GET_MODEL_POINTS_LONG_HELP);
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
    let mut result_nps = NamedPointSet::default();
    for (name, np) in nps.iter() {
        let mut ray_list = Vec::new();
        for (camera, pms) in camera_pms.iter() {
            if let Some(pm) = pms.mapping_of_np(np) {
                let ray = camera.get_pm_as_ray(pm, true);
                ray_list.push(ray);
            }
        }
        if ray_list.len() > 1 {
            let p = Ray::closest_point(&ray_list, &|_r| 1.0).unwrap();
            result_nps.add_pt(name, *np.color(), Some(p));
            // eprintln!(r#"["{}", [{},{},{}]],"#, name, p[0], p[1], p[2]);
            for r in ray_list {
                eprintln!("{name} {:?}", r.distances(&p));
            }
        }
    }
    println!("{}", serde_json::to_string_pretty(&result_nps).unwrap());
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
        get_point_mappings_cmd(),
        locate_cmd(),
        reorient_cmd(),
        reorient2_cmd(),
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
