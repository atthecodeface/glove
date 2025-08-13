//a Imports
use clap::Command;
use star_catalog::StarFilter;
use thunderclap::CommandBuilder;

use ic_base::RollYaw;
use ic_camera::CameraProjection;

use crate::cmd::{cmd_ok, CmdArgs, CmdResult};

//a Help messages
//hi STAR_LONG_HELP
const STAR_LONG_HELP: &str = "\
This set of commands allows for calibrating a lens using a photograph taken of stars.";

//hi STAR_FIND_STARS_LONG_HELP
const STAR_FIND_STARS_LONG_HELP: &str = "\
Using the camera body, lens, focus distance, and current lens
calibration, find an absolute orientation of an image given two
triangles of star PXY locations in the mapping file.

Two triangles on the image are determined from the mapping - the first
three stars with '1' as the 'magnitude' (second) element, and the
first three stars with '2' for that element.

The PXY of these star locations are mapped to world directions based
on the camera body and lens mapping; the orientation of the camera is
ignored.

For each star triangle, the three angles between these world
directions are calculated; this yields a pair of star triangle angles.

The star catalog is searched for triangles of stars that match these
angles, yielding candidate *actual* star triangles.

For each of these candidate triangles there is a camera orientation
that would map those *actual* star triangles onto the sensor (or at
least, an approximate orientation). For the actual triangles that the
image was taken of, these two orientations should be identical (with
some margin of error, obviously). One way to compare orientations is
to apply the reverse of one to the other, and check to see if it is an
identity orientation.

So the orientation for every pair of candidate triangles are combined
to determine how far off they are from the 'identity'; as orientations
are handled as quaternions, this implies multiplying one by the
conjugate of the other, and comparing the absolute value of the real
value of the resultant quaternion to 1.0; the closer it is, the
smaller the difference between the candidate triangles. This real
value is effectively the cos of the angle of rotation required to
describe the combined mapping - and an identity mapping as no angle of
rotation, and so is 1.0. For close matches (angle x close to 0) the cosine is approximately 1-x^2/2.

  ";

//hi STAR_ORIENT_LONG_HELP
const STAR_ORIENT_LONG_HELP: &str = "\
Using the camera body, lens, focus distance, current lens calibration,
and a *star mapping*, find an absolute orientation of the camera using
all of the mappings in the *star mapping* file.

For each catalog id in the *star mapping* we can retrieve the star
direction vector for the star in the catalog, as a camera relative
direction (ignoring the current camera orientation).

For each pair of star positions A and B we can thus generate a
quaternion that maps camera relative direction A to star catalog
direction A *and* that maps camera relative direction B to
(approximately) star catalog direction B. The approximation comes
from the errors in the sensor image positions and the camera
calibration, which lead to an error in the camera-relative angle
between A and B.

We can calculate quaternions for every pair (A,B) - including the
reverse (B,A), but not (A,A) - and gnerate an average (so for N
positions in the *star mapping* there are N*(N-1) quaternions).

This quaternion provides the best guess for the orientation for
the camera.
  ";

//hi STAR_UPDATE_MAPPING_LONG_HELP
const STAR_UPDATE_MAPPING_LONG_HELP: &str = "\
Using the camera body, lens, focus distance, current lens calibration,
current camera orientation, and a *star mapping*, update the mapping
of pixel XY to stars in the catalog within certain bounds.

For each pixel XY in the *star mapping* we can derive a real world
star direction vector (using the camera calibration and the
orientation); the catalog can be searched to find the closest star to
that direction vector, and if it meets the criteria (brightness, error
in yaw, etc) then the pixel XY can be deemed to be mapped to that star
(using its catalog ID).

All of the star mappings will be updated.
";

//hi STAR_SHOW_STARS_LONG_HELP
const STAR_SHOW_STARS_LONG_HELP: &str = "\
This draws on an image provided (as a JPEG or PNG) details of a star
mapping, given the current camera orientation and calibration.

It adds a small purple cross at every sensor pixel XY indicated by the
*star mapping*.

It adds a cyan cross for every *mapped* star in the mapping, showing
how that star direction maps onto the image using the camera
orientation and calibration.

It adds a yellow X for every star in the catalog, showing
how that star direction maps onto the image using the camera
orientation and calibration.

Hence every cyan cross will appear centred with a yellow X, forming a
yellow-cyan asterisk.

A perfect mapping, orientation and calibration will have all the
purple crosses marking the same pixels as a yellow/cyan asterisk.

It also draws on circles for yaw values of 5, 10, 15, 20, etc degrees.

";

//a Utility functions
//a Star find_initial_orientation
//fp star_find_stars_cmd
fn star_find_stars_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("find_stars")
        .about("Find a camera orientation using two star triangles from an image")
        .long_about(STAR_FIND_STARS_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(star_find_stars_fn)));
    CmdArgs::add_arg_yaw_error(&mut build);
    build
}

//fp star_find_stars_fn
fn star_find_stars_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let brightness = cmd_args.brightness();
    cmd_args.ensure_star_catalog()?;

    cmd_args
        .star_catalog_mut()
        .set_filter(StarFilter::brighter_than(brightness));

    let angle_orientations = cmd_args.star_mapping().find_orientation_from_triangles(
        cmd_args.star_catalog(),
        cmd_args.camera(),
        cmd_args.triangle_closeness(),
    )?;
    let mut best_match = (angle_orientations[0].1, angle_orientations[0].0, usize::MAX);
    for (i, (x, q)) in angle_orientations.iter().enumerate() {
        cmd_args.camera_mut().set_orientation(q);
        let _ = cmd_args.update_star_mappings();
        let Ok(orientation) = cmd_args
            .star_mapping()
            .find_orientation_from_all_mapped_stars(
                cmd_args.star_catalog(),
                cmd_args.camera(),
                brightness,
            )
        else {
            continue;
        };
        cmd_args.camera_mut().set_orientation(&orientation);
        let (num_unmapped, total_error) = cmd_args.update_star_mappings();
        if num_unmapped < best_match.2 {
            best_match = (orientation, *x, num_unmapped);
        }
        eprintln!(
            "Candidate {i} {} unmapped {num_unmapped} total_error {total_error} {q}",
            x.to_degrees()
        );
    }
    eprintln!(
        "Using candidate with triangle angle error {} and {} unmapped stars out of {}",
        best_match.1.to_degrees(),
        best_match.2,
        cmd_args.star_mapping().mappings().len()
    );

    cmd_args.camera_mut().set_orientation(&best_match.0);
    let _ = cmd_args.update_star_mappings();

    cmd_args.write_outputs()?;
    cmd_args.output_camera()
}

//a Star orient
//fp star_orient_cmd
fn star_orient_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("orient")
        .about("Orient on all of the mapped stars")
        .long_about(STAR_ORIENT_LONG_HELP);

    CommandBuilder::new(command, Some(Box::new(star_orient_fn)))
}

//fp star_orient_fn
fn star_orient_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    cmd_args.ensure_star_catalog()?;
    let brightness = cmd_args.brightness();
    let orientation = cmd_args
        .star_mapping()
        .find_orientation_from_all_mapped_stars(
            cmd_args.star_catalog(),
            cmd_args.camera(),
            brightness,
        )?;
    cmd_args.camera_mut().set_orientation(&orientation);
    cmd_args.write_outputs()?;
    cmd_args.output_camera()
}

//a Star update_star_mapping
//fp star_update_mapping_cmd
fn star_update_mapping_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("update_star_mapping")
        .about(
            "Generate an updated mapping of stars from the catalog to with ids frmom the catalog",
        )
        .long_about(STAR_UPDATE_MAPPING_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(star_update_mapping_fn)));
    CmdArgs::add_arg_yaw_error(&mut build);
    build
}

//fp star_update_mapping_fn
fn star_update_mapping_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let brightness = cmd_args.brightness();
    cmd_args.ensure_star_catalog()?;
    cmd_args
        .star_catalog_mut()
        .set_filter(StarFilter::brighter_than(brightness));

    //cb Show the star mappings
    let (num_unmapped, total_error) = cmd_args.update_star_mappings();
    eprintln!(
        "{num_unmapped} stars were not mapped out of {}, total error of mapped stars {total_error:.4e}|",
        cmd_args.star_mapping().mappings().len(),
    );

    cmd_args.write_outputs()?;
    cmd_args.output_star_mapping()
}

//a Star show_star_mapping
//fp star_show_mapping_cmd
fn star_show_mapping_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("show_star_mapping")
        .about("Show the mapped stars onto an output image")
        .long_about(STAR_SHOW_STARS_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(star_show_mapping_fn)));

    CmdArgs::add_arg_read_image(&mut build, Some(1));
    CmdArgs::add_arg_write_image(&mut build, false);
    CmdArgs::add_arg_within(&mut build);
    build
}

//fp star_show_mapping_fn
fn star_show_mapping_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    cmd_args.ensure_star_catalog()?;
    let within = cmd_args.within();
    let brightness = cmd_args.brightness();

    cmd_args
        .star_catalog_mut()
        .set_filter(StarFilter::brighter_than(brightness));

    //cb Show the star mappings
    cmd_args
        .star_mapping()
        .show_star_mappings(cmd_args.star_catalog(), cmd_args.camera());

    let mut mapped_pts = vec![];

    // Mark the points with blue-grey crosses
    cmd_args.star_mapping().img_pts_add_cat_index(
        cmd_args.star_catalog(),
        cmd_args.camera(),
        &mut mapped_pts,
        1,
    )?;

    // Mark the mapping points with small purple crosses
    cmd_args
        .star_mapping()
        .img_pts_add_mapping_pxy(&mut mapped_pts, 0)?;

    // Mark the catalog stars with yellow Xs
    cmd_args.star_mapping().img_pts_add_catalog_stars(
        cmd_args.star_catalog(),
        cmd_args.camera(),
        &mut mapped_pts,
        within,
        2,
    )?;

    // Draw a circle of radius yaw_max
    let camera = cmd_args.camera();
    for yaw in [5, 10, 15, 20, 25, 30, 35, 40, 45, 50] {
        let yaw = (yaw as f64).to_radians();
        for i in 0..3600 {
            let angle = ((i as f64) / 10.0).to_radians();
            let s = angle.sin();
            let c = angle.cos();
            let world_ry = RollYaw::from_roll_yaw(s, c, yaw);
            let sensor_ry = camera.camera_ry_to_sensor_ry(&world_ry);
            let sensor_txty = sensor_ry.into();
            let pxy = camera.sensor_txty_to_px_abs_xy(&sensor_txty);
            mapped_pts.push((pxy, 3).into());
        }
    }

    cmd_args.draw_image(&mapped_pts)?;

    cmd_ok()
}

//a Star calibrate_desc
//fp star_calibrate_desc_cmd
fn star_calibrate_desc_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("calibrate_desc")
        .about("Generate a calibration description")
        .long_about(STAR_LONG_HELP);

    CommandBuilder::new(command, Some(Box::new(star_calibrate_desc_fn)))
}

//fp star_calibrate_desc_fn
fn star_calibrate_desc_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    cmd_args.ensure_star_catalog()?;
    let pc = cmd_args
        .star_mapping()
        .create_calibration_mapping(cmd_args.star_catalog());
    cmd_args.set_calibration_mapping(pc);
    cmd_args.write_outputs()?;
    cmd_args.output_calibration_mapping()
}

//a Star subcommand with its commands
//fp star_cmd
pub fn star_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("star")
        .about("Calibrate a lens using stars")
        .long_about(STAR_LONG_HELP);

    let mut build = CommandBuilder::<CmdArgs>::new(command, None);
    CmdArgs::add_arg_camera_database(&mut build, false);
    CmdArgs::add_arg_camera(&mut build, false);

    CmdArgs::add_arg_triangle_closeness(&mut build);
    CmdArgs::add_arg_closeness(&mut build);
    CmdArgs::add_arg_star_mapping(&mut build);
    CmdArgs::add_arg_star_catalog(&mut build);
    CmdArgs::add_arg_brightness(&mut build);
    CmdArgs::add_arg_write_calibration_mapping(&mut build);
    CmdArgs::add_arg_write_camera(&mut build);
    CmdArgs::add_arg_write_star_mapping(&mut build);
    CmdArgs::add_arg_yaw_min_max(&mut build, Some("1.0"), Some("20.0"));

    build.add_subcommand(star_show_mapping_cmd());
    build.add_subcommand(star_find_stars_cmd());
    build.add_subcommand(star_orient_cmd());
    build.add_subcommand(star_calibrate_desc_cmd());
    build.add_subcommand(star_update_mapping_cmd());

    build
}
