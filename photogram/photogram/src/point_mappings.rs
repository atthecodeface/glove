//a Imports

use clap::Command;
use thunderclap::CommandBuilder;

use ic_base::Rrc;
use ic_camera::CameraProjection;
use ic_image::{Image, Patch};
use ic_mapping::{CameraAdjustMapping, CameraShowMapping, ModelLineSet};
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

//a Interrogate (show_mappings etc)
//fi list_cmd
fn list_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("list").about("Show point mappings");

    let mut build = CommandBuilder::new(command, Some(Box::new(list_fn)));

    CmdArgs::add_arg_named_point(&mut build, (0,));

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
    CmdArgs::add_arg_positional_string(&mut build, "name", "Named point name", Some(1), None);
    CmdArgs::add_arg_positional_string(
        &mut build,
        "pixelxy",
        "Location of point on the sensor",
        Some(1),
        None,
    );
    CmdArgs::add_arg_positional_f64(
        &mut build,
        "error",
        "Error in the location",
        Some(0),
        Some("0.0"),
    );

    build
}

//fi add_fn
fn add_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let name = cmd_args.get_string_arg(0).unwrap();
    let pxy = cmd_args.arg_as_point2d(1)?;
    let error = cmd_args.get_f64_arg(0).unwrap_or(0.0);
    if !cmd_args
        .pms()
        .borrow_mut()
        .add_mapping(&*cmd_args.nps().borrow(), name, &pxy, error)
    {
        Err(format!("Failed to add mapping for '{name}' to the point mapping set; it is probably not in the named point set").into())
    } else {
        Ok("".into())
    }
}

//a point_mappings command
//fp point_mappings_cmd
pub fn point_mappings_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("point_mappings").about("Operate on a point mapping set");

    let mut build = CommandBuilder::new(command, None);
    CmdArgs::add_arg_cip(&mut build, false);

    build.add_subcommand(list_cmd());
    build.add_subcommand(add_cmd());

    build
}
