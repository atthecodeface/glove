//a Imports

use clap::Command;
use thunderclap::CommandBuilder;

use ic_base::{Plane, Ray, Rrc};
use ic_camera::CameraProjection;
use ic_image::{Image, Patch};
use ic_mapping::PointMapping;
use ic_project::Cip;

use crate::cmd::{CmdArgs, CmdResult};

//a Help
//hi IMAGE_LONG_HELP
const IMAGE_LONG_HELP: &str = "\
Given a Named Point Set, from a camera (type, position/direction), and
a Point Mapping Set draw crosses on an image corresponding to the PMS
frame positions and the Named Point's model position mapped onto the
camera, and write out to a new image.

";

//hi IMAGE_PATCH_LONG_HELP
const IMAGE_PATCH_LONG_HELP: &str = "\
Extract a triangular patch from an image as if viewed straight on

";

//hi LOCATE_LONG_HELP
const LOCATE_LONG_HELP: &str = "\
Use the CIP's point mappings, for given mapped names points, to locate
the camera, given its focal length, etc.

Pairs of point mappings are chosen that provide good subtended viewing
angles, and that are (from the camera's perspecfive) more orthogonal
to each other, are generated.

For each pair of mappings a surface is generated in model space, where
if the camera were placed on that surface it would 'see' the pair of
mappings with the angle between them the same as is on the image.

The first surfaces is iterated over, to find the position where the
sum of the distances from the surface of the *other* pairs is
minimized. This provides the initial best location for the camera.

This position is then adjusted by small amounts, to reduce the total
error seen by *all* of the surfaces.

";

//hi ORIENT_LONG_HELP
const ORIENT_LONG_HELP: &str = "\
Use the CIP's point mappings, for given mapped names points, to orient
the camera, given its positions etc.

Pairs of point mappings are chosen that provide good subtended viewing
angles, and that are (from the camera's perspecfive) more orthogonal
to each other, are generated.

For each pair of mappings a camera orientation is determined, which
would (if the camera is perfectly positioned) map the two points to
their positions on the camera sensor. This yields an array of
orientations (as quaternions).

The array of orientations is averaged, to produce the camera
orientation.

";

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

//hi SHOW_RAYS_LONG_HELP
const SHOW_RAYS_LONG_HELP: &str = "\
Using the camera, for a subset of the point mapping set for the CIP
(specified by the named points), show rays either to or from the
camera.

The rays are generated in the appropriate direction, and their start
and end points (given the focus distance of the camera) are provided.
";

//a Locate
//fi locate_cmd
fn locate_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("locate")
        .about("Find location and orientation for a camera to map points to model")
        .long_about(LOCATE_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(locate_fn)));

    CmdArgs::add_arg_named_point(&mut build, (None, true));
    CmdArgs::add_arg_max_pairs(&mut build, Some("100"));
    CmdArgs::add_arg_max_error(&mut build, Some("10.0"));

    build
}

//fi locate_fn
fn locate_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let pms_n = cmd_args.get_pms_indices_of_nps()?;
    let n = pms_n.len();
    if n < 3 {
        return Err(format!("Required at least 3 mapped points, but found {n}",).into());
    }

    let max_pairs = cmd_args.max_pairs();
    let max_np_error = cmd_args.max_error();

    let filter = |n, pm: &PointMapping| (pms_n.contains(&n) && pm.model_error() < max_np_error);
    cmd_args.cip().borrow_mut().locate(filter, max_pairs)?;

    let camera = cmd_args.cip().borrow().camera().borrow().clone();
    *cmd_args.camera_mut() = camera;
    cmd_args.if_verbose(|| {
        eprintln!("{}", cmd_args.camera());
    });
    cmd_args.write_outputs()?;
    cmd_args.output_camera()
}

//a orient
//fi orient_cmd
fn orient_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("orient")
        .about("Set the orientation for a camera using weighted average of pairs of point mappings")
        .long_about(ORIENT_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(orient_fn)));

    CmdArgs::add_arg_named_point(&mut build, (None, true));
    CmdArgs::add_arg_max_error(&mut build, Some("10.0"));

    build
}

//fi orient_fn
fn orient_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let pms_n = cmd_args.get_pms_indices_of_nps()?;

    let max_np_error = cmd_args.max_error();

    let filter = |n, pm: &PointMapping| (pms_n.contains(&n) && pm.model_error() < max_np_error);
    let _total_error = cmd_args
        .cip()
        .borrow_mut()
        .orient_camera_using_model_directions(|n, _pm| pms_n.contains(&n))?;

    let camera = cmd_args.cip().borrow().camera().borrow().clone();
    *cmd_args.camera_mut() = camera;
    cmd_args.if_verbose(|| {
        eprintln!("{}", cmd_args.camera());
    });
    cmd_args.write_outputs()?;
    cmd_args.output_camera()
}

//a Image and image_patch commands
//fi image_cmd
fn image_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("image")
        .about("Read image and draw crosses on named and mapped points")
        .long_about(IMAGE_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(image_fn)));

    CmdArgs::add_arg_named_point(&mut build, (0,));
    CmdArgs::add_arg_pms(&mut build);

    CmdArgs::add_arg_read_image(&mut build, Some(1_usize)); // 0 or 1 in a list
    CmdArgs::add_arg_write_image(&mut build, true);

    CmdArgs::add_arg_pms_color(&mut build);
    CmdArgs::add_arg_model_color(&mut build);
    build
}

//fi image_fn
fn image_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let nps_n = cmd_args.get_nps()?;
    let pms_n = cmd_args.get_pms_indices_of_nps()?;

    let pms_color = cmd_args.pms_color();
    let model_color = cmd_args.model_color();
    let use_nps_colors = cmd_args.pms_color().is_none() && cmd_args.model_color().is_none();

    let write_filename = cmd_args.write_img().unwrap();

    let mut img = cmd_args.get_image_read_or_create()?;

    let c = [255, 255, 255, 255].into();
    let cn = [255, 190, 190, 255].into();
    let camera = cmd_args.camera();
    for i in 0_usize..200 {
        let xyz = (i as f64) * 1.0;
        let sz = {
            if i.is_multiple_of(10) {
                3.0
            } else if i.is_multiple_of(5) {
                2.0
            } else {
                1.0
            }
        };
        let mapped = camera.map_model(&[xyz, 0., 0.].into());
        img.draw_cross(&mapped, sz, &c);
        let mapped = camera.map_model(&[0., xyz, 0.].into());
        img.draw_cross(&mapped, sz, &c);
        let mapped = camera.map_model(&[0., 0., xyz].into());
        img.draw_cross(&mapped, sz, &c);

        let mapped = camera.map_model(&[-xyz, 0., 0.].into());
        img.draw_cross(&mapped, sz, &cn);
        let mapped = camera.map_model(&[0., -xyz, 0.].into());
        img.draw_cross(&mapped, sz, &cn);
        let mapped = camera.map_model(&[0., 0., -xyz].into());
        img.draw_cross(&mapped, sz, &cn);
    }

    if pms_color.is_some() || use_nps_colors {
        cmd_args.pms_map(|pms| {
            let mappings = pms.mappings();
            for i in pms_n.iter() {
                let m = &mappings[*i];
                let c = pms_color.unwrap_or(m.named_point().color());
                img.draw_cross(m.screen(), m.error(), c);
            }
            Ok(())
        })?;
    }
    if model_color.is_some() || use_nps_colors {
        let camera = cmd_args.camera();
        for p in nps_n {
            let c = model_color.unwrap_or(p.color());
            let xyz = p.model().0;
            let n = (xyz[2] * 2.0).abs().floor() as usize;
            let dz = xyz[2].signum() / 2.0;
            for z in 0..n {
                let sz = {
                    if z.is_multiple_of(10) {
                        3.0
                    } else if z.is_multiple_of(5) {
                        2.0
                    } else {
                        1.0
                    }
                };
                let xyz = [xyz[0], xyz[1], (z as f64) * dz].into();
                let mapped = camera.map_model(&xyz);
                img.draw_cross(&mapped, sz, c);
            }
            if xyz[0].abs() < xyz[1].abs() {
                let n = (xyz[0] * 2.0).abs().floor() as usize;
                let dx = xyz[0].signum() / 2.0;
                for x in 0..n {
                    let sz = {
                        if x.is_multiple_of(10) {
                            3.0
                        } else if x.is_multiple_of(5) {
                            2.0
                        } else {
                            1.0
                        }
                    };
                    let xyz = [(x as f64) * dx, xyz[1], 0.].into();
                    let mapped = camera.map_model(&xyz);
                    img.draw_cross(&mapped, sz, c);
                }
            } else {
                let n = (xyz[1] * 2.0).abs().floor() as usize;
                let dy = xyz[1].signum() / 2.0;
                for y in 0..n {
                    let sz = {
                        if y.is_multiple_of(10) {
                            3.0
                        } else if y.is_multiple_of(5) {
                            2.0
                        } else {
                            1.0
                        }
                    };
                    let xyz = [xyz[0], (y as f64) * dy, 0.].into();
                    let mapped = camera.map_model(&xyz);
                    img.draw_cross(&mapped, sz, c);
                }
            }
            let mapped = camera.map_model(&xyz);
            img.draw_cross(&mapped, 5.0, c);
        }
    }
    img.write(write_filename)?;
    Ok("".into())
}

//fi image_patch_cmd
fn image_patch_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("image_patch")
        .about("Extract a patch from an image")
        .long_about(IMAGE_PATCH_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(image_patch_fn)));

    // let cmd = cmdline_args::add_image_dir_arg(cmd, false);
    CmdArgs::add_arg_read_image(&mut build, 1_usize);
    CmdArgs::add_arg_write_image(&mut build, true);
    CmdArgs::add_arg_named_point(&mut build, (None, true));
    build
}

//fi image_patch_fn
fn image_patch_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let nps = cmd_args.get_nps()?;

    if nps.len() < 3 {
        return Err(format!("Need at least 3 points for a patch, got {}", nps.len()).into());
    }

    let cip = cmd_args.cip().borrow();
    let camera = cip.camera_ref();

    let src_img = cmd_args.get_image_read_or_create()?;
    let write_filename = cmd_args.write_img().unwrap();

    let patch = ic_mapping::Patch::create(nps.iter().cloned()).unwrap();
    let patch_img = patch.create_img(&*camera, &src_img, 25.0).unwrap();
    patch_img.write(write_filename)?;

    /*
        let model_pts: Vec<_> = nps.iter().map(|np| np.model().0).collect();

        let _ = Plane::best_fit(model_pts.iter());

        if let Some(patch) = Patch::create(&src_img, 10.0, model_pts.iter(), &|m| camera.map_model(m))?
        {
            patch.img().write(write_filename)?;
    }
        */
    Ok("".into())
}

//a Create/show Rays
//fi show_rays_cmd
fn show_rays_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("show_rays")
        .about("Show rays for the CIP using its camera and mappings")
        .long_about(SHOW_RAYS_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(show_rays_fn)));

    CmdArgs::add_arg_named_point(&mut build, (None, true));

    // from_camera

    build
}

//fi show_rays_fn
fn show_rays_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let pms_n = cmd_args.get_pms_indices_of_nps()?;
    let pms = cmd_args.pms();
    let camera = cmd_args.camera();

    let from_camera = false;
    for (_, (pm, ray)) in pms
        .borrow()
        .iter_mapped_rays(camera, from_camera)
        .enumerate()
        .filter(|(n, _pms_ray)| pms_n.contains(n))
    {
        let end = ray.start() + ray.direction() * camera.focus_distance();
        eprintln!("{} {end}", pm.name());
    }
    Ok("".into())
}

//fi create_rays_cmd
fn create_rays_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("create_rays_from_model")
        .about("Create rays for a given located camera and its mappings")
        .long_about(CREATE_RAYS_FROM_MODEL_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(create_rays_fn)));
    CmdArgs::add_arg_named_point(&mut build, (None, true));

    // from_camera

    build
}

//fi create_rays_fn
fn create_rays_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let pms_n = cmd_args.get_pms_indices_of_nps()?;
    let pms = cmd_args.pms();
    let camera = cmd_args.camera();

    let from_camera = false;

    let named_rays: Vec<(String, Ray)> = pms
        .borrow()
        .iter_mapped_rays(camera, from_camera)
        .enumerate()
        .filter(|(n, _pm_ray)| pms_n.contains(n))
        .map(|(_, (pm, ray))| (pm.name().to_owned(), ray))
        .collect();
    Ok(serde_json::to_string_pretty(&named_rays)?)
}

//a Interrogate (show_mappings etc)
//fi show_mappings_cmd
fn show_mappings_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("show_mappings")
        .about("Show the total and worst error for a point mapping set");

    let mut build = CommandBuilder::new(command, Some(Box::new(show_mappings_fn)));
    CmdArgs::add_arg_pms(&mut build);
    CmdArgs::add_arg_camera(&mut build, false);
    build
}

//fi show_mappings_fn
fn show_mappings_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let pms = cmd_args.pms().borrow();
    let nps = cmd_args.nps();
    let camera = cmd_args.camera();

    for pm in pms.mappings() {
        pm.show_mapped_error(camera);
    }
    nps.borrow().show_mappings(camera);

    let te = pms.total_error(camera);
    let we = pms.find_worst_error(camera).1;
    println!("WE {we:.2} TE {te:.2}");

    Ok("".into())
}

//fi list_cmd
fn list_cmd() -> CommandBuilder<CmdArgs> {
    let command =
        Command::new("list").about("Show the total and worst error for a point mapping set");

    let mut build = CommandBuilder::new(command, Some(Box::new(list_fn)));

    CmdArgs::add_arg_named_point(&mut build, (None, true));

    build
}

//fi list_fn
fn list_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let pms_n = cmd_args.get_pms_indices_of_nps()?;
    let pms = cmd_args.pms().borrow();
    let mappings = pms.mappings();

    for i in pms_n {
        let m = &mappings[i];
        println!(
            "{} : {} -> [{:.1}, {:.1}] @ {:.1}",
            m.name(),
            m.model(),
            m.screen()[0],
            m.screen()[1],
            m.error()
        );
    }
    Ok("".into())
}

//fi add_cmd
fn add_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("add").about("Add a new CIP");

    let mut build = CommandBuilder::new(command, Some(Box::new(add_fn)));
    CmdArgs::add_arg_positional_string(
        &mut build,
        "camera",
        "Camera filename for the CIP",
        Some(1),
        None,
    );
    CmdArgs::add_arg_positional_string(
        &mut build,
        "image",
        "Image filename for the CIP",
        Some(1),
        None,
    );
    CmdArgs::add_arg_positional_string(
        &mut build,
        "pms",
        "Point mapping set filename for the CIP",
        Some(1),
        None,
    );

    build
}

//fi add_fn
fn add_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let mut cip = Cip::default();
    let camera_filename = cmd_args.get_string_arg(0).unwrap();
    let image_filename = cmd_args.get_string_arg(1).unwrap();
    let pms_filename = cmd_args.get_string_arg(2).unwrap();
    cip.set_camera_file(camera_filename);
    cip.set_image(image_filename);
    cip.set_pms_file(pms_filename);
    cip.set_camera(cmd_args.camera().clone().into());

    let cip: Rrc<Cip> = cip.into();
    let n = cmd_args.project().ncips();
    cmd_args.project_mut().add_cip(cip.clone());
    let _ = cmd_args.set_cip(n);
    Ok("".into())
}

//a CIP command
//fp cip_cmd
pub fn cip_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("cip")
        .about("Operate on a camera/image/point mapping set")
        .version("0.1.0");

    let mut build = CommandBuilder::new(command, None);
    CmdArgs::add_arg_cip(&mut build, false);

    build.add_subcommand(image_cmd());
    build.add_subcommand(image_patch_cmd());
    build.add_subcommand(show_mappings_cmd());
    build.add_subcommand(list_cmd());
    build.add_subcommand(add_cmd());
    build.add_subcommand(locate_cmd());
    build.add_subcommand(orient_cmd());
    build.add_subcommand(create_rays_cmd());
    build.add_subcommand(show_rays_cmd());

    build
}
