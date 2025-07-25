//a Documentation
//! Test of camera calibration from stars (IMG_4924.JPG)
//!
//! The stars were captured on a Canon Rebel T2i, with a 50mm lens focused on 'infinity'
//!

//a Imports
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};
use star_catalog::Catalog;

use ic_base::json;
use ic_base::Result;
use ic_camera::{CameraDatabase, CameraInstance, CameraProjection};
use ic_image::{Image, ImagePt, ImageRgb8};
use ic_stars::StarMapping;

use ic_cmdline::builder::{CommandArgs, CommandBuilder, CommandSet};

//a CmdArgs
//tp  CmdArgs
#[derive(Default)]
pub struct CmdArgs {
    cdb: Option<CameraDatabase>,
    camera: CameraInstance,
    mapping: StarMapping,
    closeness: f32,
    search_brightness: f32,
    match_brightness: f32,
    catalog: Option<Box<Catalog>>,
    read_img: Vec<String>,
    write_img: Option<String>,
}

//ip CommandArgs for CmdArgs
impl CommandArgs for CmdArgs {
    type Error = ic_base::Error;
    type Value = ();
}

//ip CmdArgs
impl CmdArgs {
    pub fn draw_image(&self, pts: &[ImagePt]) -> Result<()> {
        if self.read_img.is_empty() || self.write_img.is_none() {
            return Ok(());
        }
        let mut img = ImageRgb8::read_image(&self.read_img[0])?;
        for p in pts {
            p.draw(&mut img);
        }
        img.write(self.write_img.as_ref().unwrap())?;
        Ok(())
    }
}

//a arg commands
//fp arg_star_mapping
fn arg_star_mapping(args: &mut CmdArgs, matches: &ArgMatches) -> Result<()> {
    let filename = matches.get_one::<String>("star_mapping").unwrap();
    let json = json::read_file(filename)?;
    args.mapping = StarMapping::from_json(&json)?;
    Ok(())
}

//fp arg_star_catalog
fn arg_star_catalog(args: &mut CmdArgs, matches: &ArgMatches) -> Result<()> {
    let catalog_filename = matches.get_one::<String>("star_catalog").unwrap();
    if catalog_filename == "hipp_bright" {
        args.catalog = Some(
            postcard::from_bytes(star_catalog::hipparcos::HIPP_BRIGHT_PST)
                .map_err(|e| format!("{e:?}"))?,
        );
    } else {
        let s = std::fs::read_to_string(catalog_filename)?;
        args.catalog = Some(serde_json::from_str(&s)?);
    }
    args.catalog.as_mut().unwrap().sort();
    args.catalog.as_mut().unwrap().derive_data();
    Ok(())
}

//a Main
pub fn main() -> Result<()> {
    let command = Command::new("ic_play")
        .about("Camera calibration tool")
        .version("0.1.0");

    let mut build = CommandBuilder::<CmdArgs>::new(command, None);

    build.add_arg(
        ic_cmdline::camera::camera_database_arg(true),
        Box::new(|args, matches| {
            ic_cmdline::camera::set_opt_camera_database(matches, &mut args.cdb)
        }),
    );

    build.add_arg(
        ic_cmdline::camera::camera_arg(true),
        Box::new(|args, matches| {
            ic_cmdline::camera::set_camera(matches, args.cdb.as_ref().unwrap(), &mut args.camera)
        }),
    );

    build.add_arg(
        Arg::new("star_mapping")
            .required(true)
            .help("Star calibration mapping JSON")
            .action(ArgAction::Set),
        Box::new(arg_star_mapping),
    );

    build.add_arg(
        Arg::new("star_catalog")
            .long("catalog")
            .required(true)
            .help("Star catalog to use")
            .action(ArgAction::Set),
        Box::new(arg_star_catalog),
    );
    build.add_arg(
        Arg::new("search_brightness")
            .long("search_brightness")
            .value_parser(value_parser!(f32))
            .default_value("5.0")
            .help("Maximum brightness of stars to use for searching with triangles")
            .action(ArgAction::Set),
        Box::new(|args, matches| {
            args.search_brightness = *matches.get_one::<f32>("search_brightness").unwrap();
            Ok(())
        }),
    );
    build.add_arg(
        Arg::new("match_brightness")
            .long("match_brightness")
            .value_parser(value_parser!(f32))
            .default_value("5.0")
            .help("Maximum brightness of stars to use for matching all the points")
            .action(ArgAction::Set),
        Box::new(|args, matches| {
            args.match_brightness = *matches.get_one::<f32>("match_brightness").unwrap();
            Ok(())
        }),
    );
    build.add_arg(
        ic_cmdline::image::image_read_arg(false, Some(1)),
        Box::new(|args, matches| {
            ic_cmdline::image::get_image_read_filenames(matches).map(|v| args.read_img = v)
        }),
    );
    build.add_arg(
        ic_cmdline::image::image_write_arg(false),
        Box::new(|args, matches| {
            ic_cmdline::image::get_opt_image_write_filename(matches).map(|v| args.write_img = v)
        }),
    );

    let ms_command =
        Command::new("map_stars").about("Map all stars in the catalog onto an output image");
    let ms_build = CommandBuilder::new(ms_command, Some(Box::new(map_stars_cmd)));
    build.add_subcommand(ms_build);

    let fs_command = Command::new("find_stars").about("Find stars from an image");
    let mut fs_build = CommandBuilder::new(fs_command, Some(Box::new(find_stars_from_image_cmd)));
    fs_build.add_arg(
        Arg::new("closeness")
            .long("closeness")
            .value_parser(value_parser!(f32))
            .default_value("0.0")
            .help("Closeness to find triangles of stars")
            .action(ArgAction::Set),
        Box::new(|args, matches| {
            args.closeness = *matches.get_one::<f32>("closeness").unwrap();
            Ok(())
        }),
    );
    build.add_subcommand(fs_build);

    let cd_command = Command::new("calibrate_desc").about("Generate a calibration description");
    let cd_build = CommandBuilder::new(cd_command, Some(Box::new(calibrate_desc_cmd)));
    build.add_subcommand(cd_build);

    let ms_command =
        Command::new("star_mapping").about("Try to map pxy to stars in catalog using orientation");
    let ms_build = CommandBuilder::new(ms_command, Some(Box::new(star_mapping_cmd)));
    build.add_subcommand(ms_build);

    let mut cmd_args = CmdArgs::default();
    let mut command: CommandSet<CmdArgs> = build.into();
    command.execute_env(&mut cmd_args)?;
    Ok(())
}

//fp calibrate_desc_cmd
fn calibrate_desc_cmd(cmd_args: &mut CmdArgs) -> Result<()> {
    //cb Show the star mappings
    let fov = 60.0;
    let close_enough = fov / 500.0; // degrees;
    let pc = cmd_args.mapping.create_calibration_mapping(
        cmd_args.catalog.as_ref().unwrap(),
        &cmd_args.camera,
        close_enough,
    );
    println!("{}", pc.to_json()?);
    Ok(())
}

//fp star_mapping_cmd
fn star_mapping_cmd(cmd_args: &mut CmdArgs) -> Result<()> {
    //cb Show the star mappings
    let fov = 60.0;
    let close_enough = fov / 500.0; // degrees;
    let (num_unmapped, total_error) = cmd_args.mapping.update_star_mappings(
        cmd_args.catalog.as_ref().unwrap(),
        &cmd_args.camera,
        close_enough,
    );
    eprintln!(
        "{num_unmapped} stars were not mapped, total error of mapped stars {total_error:.4e}"
    );
    println!("{}", cmd_args.mapping.clone().to_json()?);
    Ok(())
}

//fp map_stars_cmd
fn map_stars_cmd(cmd_args: &mut CmdArgs) -> Result<()> {
    let brightness = cmd_args.search_brightness;
    let orientation = cmd_args.mapping.find_orientation_from_all_mapped_stars(
        cmd_args.catalog.as_ref().unwrap(),
        &cmd_args.camera,
        brightness,
    )?;
    cmd_args.camera.set_orientation(orientation);

    //cb Show the star mappings
    let fov = 60.0;
    let close_enough = fov / 500.0; // degrees;
    let _ = cmd_args.mapping.show_star_mappings(
        cmd_args.catalog.as_ref().unwrap(),
        &cmd_args.camera,
        close_enough,
    );
    let mut mapped_pts = vec![];
    cmd_args.mapping.img_pts_add_catalog_stars(
        cmd_args.catalog.as_ref().unwrap(),
        &cmd_args.camera,
        &mut mapped_pts,
        2,
    )?;
    cmd_args.mapping.img_pts_add_cat_index(
        cmd_args.catalog.as_ref().unwrap(),
        &cmd_args.camera,
        &mut mapped_pts,
        1,
        cmd_args.search_brightness,
    )?;
    cmd_args
        .mapping
        .img_pts_add_mapping_pxy(&mut mapped_pts, 0)?;
    cmd_args.draw_image(&mapped_pts)?;
    println!("{}", cmd_args.camera.to_json()?);
    Ok(())
}

//fp find_stars_from_image_cmd
fn find_stars_from_image_cmd(cmd_args: &mut CmdArgs) -> Result<()> {
    let brightness = cmd_args.search_brightness;
    let closeness = {
        if cmd_args.closeness == 0. {
            0.003
        } else {
            cmd_args.closeness
        }
    };

    cmd_args
        .catalog
        .as_mut()
        .unwrap()
        .retain(move |s, _n| s.brighter_than(brightness));
    cmd_args.catalog.as_mut().unwrap().sort();
    cmd_args.catalog.as_mut().unwrap().derive_data();

    let orientation = cmd_args.mapping.find_orientation_from_triangles(
        cmd_args.catalog.as_ref().unwrap(),
        &cmd_args.camera,
        brightness,
        closeness,
    )?;
    cmd_args.camera.set_orientation(orientation);

    //cb Show the star mappings
    let fov = 60.0;
    let close_enough = fov / 500.0; // degrees;
    let _ = cmd_args.mapping.show_star_mappings(
        cmd_args.catalog.as_ref().unwrap(),
        &cmd_args.camera,
        close_enough,
    );
    let mut mapped_pts = vec![];
    cmd_args.mapping.img_pts_add_catalog_stars(
        cmd_args.catalog.as_ref().unwrap(),
        &cmd_args.camera,
        &mut mapped_pts,
        2,
    )?;
    cmd_args.mapping.img_pts_add_cat_index(
        cmd_args.catalog.as_ref().unwrap(),
        &cmd_args.camera,
        &mut mapped_pts,
        1,
        brightness,
    )?;
    cmd_args
        .mapping
        .img_pts_add_mapping_pxy(&mut mapped_pts, 0)?;
    cmd_args.draw_image(&mapped_pts)?;
    println!("{}", cmd_args.camera.to_json()?);
    Ok(())
}
