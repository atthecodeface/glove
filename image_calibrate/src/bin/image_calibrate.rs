//a Imports
use std::cell::Ref;
use std::collections::HashMap;

use clap::{Arg, ArgAction, Command};
use geo_nd::{quat, vector, SqMatrix, Vector, Vector3};
use image::Image;
use image_calibrate::{
    cmdline_args, image, json, BestMapping, CameraAdjustMapping, CameraDatabase, CameraPtMapping,
    CameraShowMapping, Color, Mat3x3, ModelLineSet, NamedPointSet, Point3D, PointMappingSet,
    Project, Ray, Region,
};

//a Types
//ti SubCmdFn
type SubCmdFn = fn(BaseArgs, &clap::ArgMatches) -> Result<(), String>;

//tp BaseArgs
struct BaseArgs {
    project: Project,
    verbose: bool,
}
impl BaseArgs {
    fn project(&self) -> &Project {
        &self.project
    }
    fn nps(&self) -> Ref<NamedPointSet> {
        self.project.nps_ref()
    }
    fn cdb(&self) -> Ref<CameraDatabase> {
        self.project.cdb_ref()
    }
}

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
fn image_fn(base_args: BaseArgs, matches: &clap::ArgMatches) -> Result<(), String> {
    let pms = cmdline_args::get_pms(matches, &base_args.nps())?;
    let camera = cmdline_args::get_camera(matches, base_args.project())?;
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
        for (_name, p) in base_args.nps().iter() {
            let c = model_color.as_ref().unwrap_or(p.color());
            let mapped = camera.map_model(p.model().0);
            img.draw_cross(mapped, 5.0, c);
        }
    }
    img.write(write_filename)
}

//hi IMAGE_PATCH_LONG_HELP
const IMAGE_PATCH_LONG_HELP: &str = "\
Extract a triangular patch from an image as if viewed straight on

";

//fi image_patch_cmd
fn image_patch_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("image_patch")
        .about("Extract a patch from an image")
        .long_about(IMAGE_PATCH_LONG_HELP);
    let cmd = cmdline_args::add_cip_arg(cmd, false);
    // let cmd = cmdline_args::add_image_dir_arg(cmd, false);
    let cmd = cmdline_args::add_image_read_arg(cmd, false);
    let cmd = cmdline_args::add_image_write_arg(cmd, true);
    let cmd = cmd.arg(
        Arg::new("np")
            .required(true)
            .help("Specifies named points for the patch")
            .action(ArgAction::Append),
    );
    (cmd, image_patch_fn)
}

//fi image_patch_fn
fn image_patch_fn(base_args: BaseArgs, matches: &clap::ArgMatches) -> Result<(), String> {
    let cip = cmdline_args::get_cip(matches, base_args.project())?;
    let cip = cip.borrow();
    let camera = cip.camera_ref();
    let src_img = cmdline_args::get_image_read(matches)?;
    let write_filename = cmdline_args::get_opt_image_write_filename(matches)?.unwrap();

    let mut model_pts = vec![];
    for name in matches.get_many::<String>("np").unwrap() {
        if let Some(n) = base_args.project.nps().borrow().get_pt(name) {
            let model = n.model().0;
            model_pts.push((name, model, camera.map_model(model)))
        } else {
            return Err(format!("Could not find NP {name} in the project"));
        }
    }
    if model_pts.len() < 3 {
        return Err(format!(
            "Need at least 3 points for a patch, got {}",
            model_pts.len()
        ));
    }

    for m in &model_pts {
        println!("{} {} {}", m.0, m.1, m.2);
    }

    let origin = model_pts[0].1;
    let d_10 = (model_pts[1].1 - model_pts[0].1).normalize();
    let d_20 = (model_pts[2].1 - model_pts[0].1).normalize();
    let normal = d_10.cross_product(&d_20).normalize();

    let x_axis = d_10;
    let y_axis = normal.cross_product(&d_10).normalize();

    let flat_to_model: Mat3x3 = [
        x_axis[0], y_axis[0], normal[0], x_axis[1], y_axis[1], normal[1], x_axis[2], y_axis[2],
        normal[2],
    ]
    .into();
    let model_to_flat = flat_to_model.inverse();

    let corners: Vec<_> = model_pts
        .iter()
        .map(|(_, p, _)| model_to_flat.transform(&(*p - origin)))
        .collect();
    println!("{x_axis}, {y_axis}, {model_to_flat:?}");

    let (lx, rx, by, ty) = corners.iter().fold(
        (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64),
        |(lx, rx, by, ty), p| (lx.min(p[0]), rx.max(p[0]), by.min(p[1]), ty.max(p[1])),
    );
    let px_per_mm = 10.0;
    let ilx = (lx * px_per_mm).floor() as isize;
    let iby = (by * px_per_mm).floor() as isize;
    let irx = (rx * px_per_mm).ceil() as isize;
    let ity = (ty * px_per_mm).ceil() as isize;
    println!("{ilx}, {irx}, {iby}, {ity}");

    let width = (irx - ilx) as usize;
    let height = (ity - iby) as usize;
    let mut patch_img = image::read_or_create_image(width, height, None)?;

    let src_w = 3360.0 * 2.0;
    let src_h = 2240.0 * 2.0;
    for x in 0..width {
        let mfx = x_axis * ((x as f64) / px_per_mm);
        for y in 0..height {
            let mfy = y_axis * ((y as f64) / px_per_mm);
            let model_pt = origin + mfx + mfy;
            let pxy = camera.map_model(model_pt);
            if pxy[0] < 0.0 || pxy[1] < 0.0 || pxy[0] >= src_w || pxy[1] >= src_h {
                continue;
            }
            let c = src_img.get(pxy[0] as u32, pxy[1] as u32);
            patch_img.put(x as u32, y as u32, &c);
        }
    }
    patch_img.write(write_filename)
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
fn show_mappings_fn(base_args: BaseArgs, matches: &clap::ArgMatches) -> Result<(), String> {
    let pms = cmdline_args::get_pms(matches, &base_args.nps())?;
    let camera = cmdline_args::get_camera(matches, base_args.project())?;

    let mappings = pms.mappings();

    let te = camera.total_error(mappings);
    let we = camera.worst_error(mappings);
    camera.show_mappings(mappings);
    camera.show_point_set(&base_args.nps());
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
fn get_point_mappings_fn(base_args: BaseArgs, matches: &clap::ArgMatches) -> Result<(), String> {
    let img = cmdline_args::get_image_read(matches)?;
    let bg_color = cmdline_args::get_opt_bg_color(matches)?;
    let bg_color = bg_color.unwrap_or(Color::black());
    let regions = Region::regions_of_image(&img, &|c| !c.color_eq(&bg_color));
    let mut pms = PointMappingSet::default();
    for r in regions {
        let c = r.color();
        let np = base_args.nps().of_color(&c);
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
            pms.add_mapping(&base_args.nps(), np[0].name(), &screen, error);
        }
    }
    println!("{}", serde_json::to_string_pretty(&pms).unwrap());
    if base_args.verbose {
        eprintln!("Exported {} mappings", pms.mappings().len());
    }
    Ok(())
}

//a Locate
//fi locate_cmd
fn locate_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("locate")
        .about("Find location and orientation for a camera to map points to model");
    let cmd = cmdline_args::add_cip_arg(cmd, false);
    let cmd = cmdline_args::add_pms_arg(cmd, false);
    let cmd = cmdline_args::add_camera_arg(cmd, false);
    (cmd, locate_fn)
}

//fi locate_fn
fn locate_fn(base_args: BaseArgs, matches: &clap::ArgMatches) -> Result<(), String> {
    let cip = cmdline_args::get_cip(matches, base_args.project())?;
    let cip = cip.borrow();
    let camera = cip.camera_ref();
    let pms = cip.pms_ref();
    let mappings = pms.mappings();

    let mut mls = ModelLineSet::new(&camera);
    for (i, j) in pms.get_good_screen_pairs(&|_f| true) {
        mls.add_line((&mappings[i], &mappings[j]));
    }
    let (location, _err) = mls.find_best_min_err_location(100, 500);

    let camera = camera.clone_placed_at(location);
    println!("{}", serde_json::to_string_pretty(&camera).unwrap());
    Ok(())
}

//a orient / reorient
//hi ORIENT_LONG_HELP
const ORIENT_LONG_HELP: &str = "\
Use consecutive pairs of point mappings to determine a camera
orientation, and average them.

*An* orientation is generated to rotate the first of each pair of
point mappings to the Z axis from its screen direction, and from its
to-model direction; these are applied to the second points in the
pairs, and then a rotation around the Z axis to map on onto the other
(assumming the angle they subtend is the same!) is generated. This
yields three quaternions which are combined to generate an orientation
of the camera.

The orientations from each pair of point mappings should be identical;
an average is generated, and the camera orientation set to this.

";

//fi orient_cmd
fn orient_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("orient")
        .about("Set the orientation for a camera using weighted average of pairs of point mappings")
        .long_about(ORIENT_LONG_HELP);
    let cmd = cmdline_args::add_pms_arg(cmd, true);
    let cmd = cmdline_args::add_camera_arg(cmd, true);
    (cmd, orient_fn)
}

//fi orient_fn
fn orient_fn(base_args: BaseArgs, matches: &clap::ArgMatches) -> Result<(), String> {
    let pms = cmdline_args::get_pms(matches, &base_args.nps())?;
    let mut camera = cmdline_args::get_camera(matches, base_args.project())?;

    camera.orient_using_rays_from_model(pms.mappings());

    println!("{}", serde_json::to_string_pretty(&camera).unwrap());
    Ok(())
}

//hi REORIENT_LONG_HELP
const REORIENT_LONG_HELP: &str = "\
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

//fi reorient_cmd
fn reorient_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("reorient")
        .about("Improve orientation for a camera to map points to model")
        .long_about(REORIENT_LONG_HELP);
    let cmd = cmdline_args::add_pms_arg(cmd, true);
    let cmd = cmdline_args::add_camera_arg(cmd, true);
    (cmd, reorient_fn)
}

//fi reorient_fn
fn reorient_fn(base_args: BaseArgs, matches: &clap::ArgMatches) -> Result<(), String> {
    let pms = cmdline_args::get_pms(matches, &base_args.nps())?;
    let mut camera = cmdline_args::get_camera(matches, base_args.project())?;
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
    base_args: BaseArgs,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let camera = cmdline_args::get_camera(matches, base_args.project())?;
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
        if base_args.verbose {
            eprintln!("{}: k {} dsq {} d {}", _name, _k, d_sq, d_sq.sqrt());
        }
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
    base_args: BaseArgs,
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
            if base_args.nps().get_pt(&name).is_none() {
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
    base_args: BaseArgs,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let pms = cmdline_args::get_pms(matches, &base_args.nps())?;
    let camera = cmdline_args::get_camera(matches, base_args.project())?;
    let mappings = pms.mappings();

    let named_rays = camera.get_rays(mappings, false);

    if base_args.verbose {
        for (n, r) in &named_rays {
            let end = r.start + r.direction * 400.0;
            eprintln!("{n} {end}");
        }
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
    base_args: BaseArgs,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let pms = cmdline_args::get_pms(matches, &base_args.nps())?;
    let camera = cmdline_args::get_camera(matches, base_args.project())?;
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
fn adjust_model_fn(base_args: BaseArgs, matches: &clap::ArgMatches) -> Result<(), String> {
    let np = cmdline_args::get_np(matches, &base_args.nps())?;
    let camera_pms = cmdline_args::get_camera_pms(matches, &base_args.cdb(), &base_args.nps())?;
    let steps = 30;
    let orig = np.model();
    let mut best_dirn: BestMapping<Point3D> = BestMapping::new(true, [0., 0., 0.].into());
    let mut best_for_pts: Vec<BestMapping<Point3D>> =
        vec![BestMapping::new(true, [0., 0., 0.].into()); camera_pms.len()];
    for i in 0..100 {
        let x0 = (i % 10) as f64 / 10.0;
        let z0 = ((i / 10) % 10) as f64 / 10.0;
        let dpx: Point3D = vector::uniform_dist_sphere3([x0, z0], true).into();
        np.set_model(Some((orig.0 + (dpx * 0.1), orig.1)));
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
        eprintln!("{} : {} : {} ", i, b, orig.0 + (*b.data()) * 0.1);
    }
    eprintln!("{} : {} ", best_dirn, orig.0 + (*best_dirn.data()) * 0.1);
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
fn get_model_points_fn(base_args: BaseArgs, matches: &clap::ArgMatches) -> Result<(), String> {
    let camera_pms = cmdline_args::get_camera_pms(matches, &base_args.cdb(), &base_args.nps())?;
    let mut result_nps = NamedPointSet::default();
    for (name, np) in base_args.nps().iter() {
        let mut ray_list = Vec::new();
        for (camera, pms) in camera_pms.iter() {
            if let Some(pm) = pms.mapping_of_np(np) {
                let ray = camera.get_pm_as_ray(pm, true);
                ray_list.push(ray);
            }
        }
        if ray_list.len() > 1 {
            if let Some(pt) = Ray::closest_point(&ray_list, &|_r| 1.0) {
                let e_sq = ray_list
                    .iter()
                    .fold(f64::MAX, |acc, r| acc.min(r.distances(&pt).1));
                result_nps.add_pt(name, *np.color(), Some(pt), e_sq.sqrt());
                if base_args.verbose {
                    for r in ray_list {
                        eprintln!("Ray to {name} {:?}", r.distances(&pt));
                    }
                }
            }
        }
    }
    println!("{}", serde_json::to_string_pretty(&result_nps).unwrap());
    Ok(())
}

//a Project as a whole
//hi PROJECT_LONG_HELP
const PROJECT_LONG_HELP: &str = "\
Project help";

//fi project_cmd
fn project_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("project")
        .about("Get model points from camera and pms")
        .long_about(PROJECT_LONG_HELP);
    // let cmd = cmdline_args::add_project_arg(cmd, true);
    (cmd, project_fn)
}

//fi project_fn
fn project_fn(base_args: BaseArgs, _matches: &clap::ArgMatches) -> Result<(), String> {
    let camera = base_args.project.cip(0).borrow().camera().clone();
    eprintln!("Camera {camera:?}");
    eprintln!("Mapping {}", camera.borrow().map_model([0., 0., 0.].into()));
    println!(
        "{}",
        serde_json::to_string_pretty(base_args.project()).unwrap()
    );
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
    let cmd = cmdline_args::add_project_arg(cmd, false);
    let cmd = cmdline_args::add_camera_database_arg(cmd, false);
    let cmd = cmdline_args::add_nps_arg(cmd, false);
    let cmd = cmdline_args::add_verbose_arg(cmd);

    let mut subcmds: HashMap<String, SubCmdFn> = HashMap::new();
    let mut cmd = cmd;
    for (c, f) in [
        image_cmd(),
        image_patch_cmd(),
        show_mappings_cmd(),
        get_point_mappings_cmd(),
        locate_cmd(),
        orient_cmd(),
        reorient_cmd(),
        combine_rays_from_model_cmd(),
        combine_rays_from_camera_cmd(),
        create_rays_from_model_cmd(),
        create_rays_from_camera_cmd(),
        adjust_model_cmd(),
        get_model_points_cmd(),
        project_cmd(),
    ] {
        subcmds.insert(c.get_name().into(), f);
        cmd = cmd.subcommand(c);
    }
    let cmd = cmd;

    let matches = cmd.get_matches();
    let project = cmdline_args::get_project(&matches).map_err(print_err)?;
    let verbose = cmdline_args::get_verbose(&matches);

    let base_args = BaseArgs { project, verbose };
    let (subcommand, submatches) = matches.subcommand().unwrap();
    for (name, sub_cmd_fn) in subcmds {
        if subcommand == name {
            return sub_cmd_fn(base_args, submatches).map_err(print_err);
        }
    }
    unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`");
}
