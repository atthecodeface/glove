//a Imports

use clap::Command;
use thunderclap::CommandBuilder;

use ic_camera::CameraProjection;
use ic_image::{Image, Patch};
use ic_mapping::{CameraAdjustMapping, CameraPtMapping, CameraShowMapping, ModelLineSet};

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

//a Locate
//fi locate_cmd
fn locate_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("locate")
        .about("Find location and orientation for a camera to map points to model");

    let mut build = CommandBuilder::new(command, Some(Box::new(locate_fn)));

    CmdArgs::add_arg_named_point(&mut build, (0,));
    CmdArgs::add_arg_pms(&mut build);

    CmdArgs::add_arg_write_camera(&mut build);

    build
}

//fi locate_fn
fn locate_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let pms_n = cmd_args.get_pms_indices_of_nps()?;
    let n = pms_n.len();
    if n < 3 {
        return Err(format!("Required at least 3 mapped points, but found {n}",).into());
    }

    let mut mls = ModelLineSet::new(cmd_args.camera().clone());

    cmd_args.pms_map(|pms| {
        let mappings = pms.mappings();
        if n < 6 {
            for i in 0..n {
                for j in (i + 1)..n {
                    let pms_i = pms_n[i];
                    let pms_j = pms_n[j];
                    mls.add_line((&mappings[pms_i], &mappings[pms_j]));
                }
            }
            Ok(())
        } else {
            todo!();
            Ok(())
        }
    })?;
    if mls.num_lines() < 2 {
        return Err(format!(
            "Required at least 2 good screen pairs, but found {}",
            mls.num_lines()
        )
        .into());
    }

    cmd_args.if_verbose(|| {
        eprintln!("Using {} model lines", mls.num_lines());
    });

    // let (location, err) = mls.find_best_min_err_location(&|p| p[0] < 0., 1000, 1000);
    let (location, _err) = mls.find_best_min_err_location(&|_| true, 1000, 1000);

    cmd_args.camera_mut().set_position(location);
    cmd_args.if_verbose(|| {
        eprintln!("{}", cmd_args.camera());
    });
    *cmd_args.cip.borrow().camera_mut() = cmd_args.camera().clone();
    cmd_args.write_outputs()?;
    cmd_args.output_camera()
}

//a orient / reorient
//fi orient_cmd
fn orient_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("orient")
        .about("Set the orientation for a camera using weighted average of pairs of point mappings")
        .long_about(ORIENT_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(orient_fn)));

    CmdArgs::add_arg_named_point(&mut build, (0,));
    CmdArgs::add_arg_pms(&mut build);

    CmdArgs::add_arg_write_camera(&mut build);

    build
}

//fi orient_fn
fn orient_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let pms_n = cmd_args.get_pms_indices_of_nps()?;

    let mut camera = cmd_args.camera().clone();

    cmd_args.pms_map(|pms| {
        camera.orient_using_rays_from_model(pms.mappings());
        Ok(())
    })?;

    cmd_args.camera_mut().set_orientation(camera.orientation());

    cmd_args.if_verbose(|| {
        eprintln!("{}", cmd_args.camera());
    });

    *cmd_args.cip.borrow().camera_mut() = cmd_args.camera().clone();
    cmd_args.write_outputs()?;
    cmd_args.output_camera()
}

//fi reorient_cmd
fn reorient_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("reorient")
        .about("Improve orientation for a camera to map points to model")
        .long_about(REORIENT_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(reorient_fn)));
    CmdArgs::add_arg_pms(&mut build);
    CmdArgs::add_arg_camera(&mut build, true); // required=true
    build
}

//fi reorient_fn
fn reorient_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let pms = cmd_args.pms();
    let mut camera = cmd_args.camera().clone();

    camera.reorient_using_rays_from_model(pms.borrow().mappings());
    *cmd_args.camera_mut() = camera;
    *cmd_args.cip.borrow().camera_mut() = cmd_args.camera().clone();

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
        let mapped = camera.map_model([xyz, 0., 0.].into());
        img.draw_cross(mapped, sz, &c);
        let mapped = camera.map_model([0., xyz, 0.].into());
        img.draw_cross(mapped, sz, &c);
        let mapped = camera.map_model([0., 0., xyz].into());
        img.draw_cross(mapped, sz, &c);

        let mapped = camera.map_model([-xyz, 0., 0.].into());
        img.draw_cross(mapped, sz, &cn);
        let mapped = camera.map_model([0., -xyz, 0.].into());
        img.draw_cross(mapped, sz, &cn);
        let mapped = camera.map_model([0., 0., -xyz].into());
        img.draw_cross(mapped, sz, &cn);
    }

    if pms_color.is_some() || use_nps_colors {
        cmd_args.pms_map(|pms| {
            let mappings = pms.mappings();
            for i in pms_n.iter() {
                let m = &mappings[*i];
                let c = pms_color.unwrap_or(m.model.color());
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
                let xyz = [xyz[0], xyz[1], (z as f64) * dz];
                let mapped = camera.map_model(xyz.into());
                img.draw_cross(mapped, sz, c);
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
                    let xyz = [(x as f64) * dx, xyz[1], 0.];
                    let mapped = camera.map_model(xyz.into());
                    img.draw_cross(mapped, sz, c);
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
                    let xyz = [xyz[0], (y as f64) * dy, 0.];
                    let mapped = camera.map_model(xyz.into());
                    img.draw_cross(mapped, sz, c);
                }
            }
            let mapped = camera.map_model(xyz);
            img.draw_cross(mapped, 5.0, c);
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
    CmdArgs::add_arg_named_point(&mut build, (1,));
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

    let model_pts: Vec<_> = nps.iter().map(|np| np.model().0).collect();

    if let Some(patch) = Patch::create(&src_img, 10.0, &model_pts, &|m| camera.map_model(m))? {
        patch.img().write(write_filename)?;
    }
    Ok("".into())
}

//a Create Rays
//fi create_rays_from_model_cmd
fn create_rays_from_model_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("create_rays_from_model")
        .about("Create rays for a given located camera and its mappings")
        .long_about(CREATE_RAYS_FROM_MODEL_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(create_rays_from_model_fn)));
    CmdArgs::add_arg_pms(&mut build);
    CmdArgs::add_arg_camera(&mut build, false);

    build
}

//fi create_rays_from_model_fn
fn create_rays_from_model_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let pms = cmd_args.pms();
    let camera = cmd_args.camera();

    let named_rays = camera.get_rays(pms.borrow().mappings(), false);

    cmd_args.if_verbose(|| {
        for (n, r) in &named_rays {
            let end = r.start + r.direction * 400.0;
            eprintln!("{n} {end}");
        }
    });

    println!("{}", serde_json::to_string_pretty(&named_rays).unwrap());
    Ok("".into())
}

//fi create_rays_from_camera_cmd
fn create_rays_from_camera_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("create_rays_from_camera")
        .about("Create rays for a given located camera and its mappings");

    let mut build = CommandBuilder::new(command, Some(Box::new(create_rays_from_camera_fn)));
    CmdArgs::add_arg_pms(&mut build);

    build
}

//fi create_rays_from_camera_fn
fn create_rays_from_camera_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let pms = cmd_args.pms();
    let camera = cmd_args.camera();

    println!(
        "{}",
        serde_json::to_string_pretty(&camera.get_rays(pms.borrow().mappings(), true)).unwrap()
    );
    Ok("".into())
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
    let mappings = pms.mappings();

    let te = camera.total_error(mappings);
    let we = camera.worst_error(mappings);
    camera.show_mappings(mappings);
    camera.show_point_set(&nps.borrow());
    println!("WE {we:.2} TE {te:.2}");

    Ok("".into())
}

//fi list_cmd
fn list_cmd() -> CommandBuilder<CmdArgs> {
    let command =
        Command::new("list").about("Show the total and worst error for a point mapping set");

    let mut build = CommandBuilder::new(command, Some(Box::new(list_fn)));

    CmdArgs::add_arg_named_point(&mut build, (0,));
    CmdArgs::add_arg_pms(&mut build);
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
    build.add_subcommand(locate_cmd());
    build.add_subcommand(orient_cmd());
    build.add_subcommand(reorient_cmd());
    build.add_subcommand(create_rays_from_model_cmd());
    build.add_subcommand(create_rays_from_camera_cmd());

    build
}
