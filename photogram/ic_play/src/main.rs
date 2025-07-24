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
use ic_camera::CameraDatabase;
use ic_image::{Image, ImagePt, ImageRgb8};
use ic_stars::StarCalibrate;

use ic_cmdline::builder::{CommandArgs, CommandBuilder, CommandSet};

//a CmdArgs
//tp  CmdArgs
#[derive(Default)]
pub struct CmdArgs {
    cdb: Option<CameraDatabase>,
    cal: Option<StarCalibrate>,
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
    pub fn borrow_mut(&mut self) -> (&mut StarCalibrate, &mut Box<Catalog>) {
        match (&mut self.cal, &mut self.catalog) {
            (Some(cal), Some(catalog)) => (cal, catalog),
            _ => {
                panic!("Cannot borrow; bad argument setup in the program");
            }
        }
    }
}

//a arg commands
//fp arg_camera_db
fn arg_camera_db(args: &mut CmdArgs, matches: &ArgMatches) -> Result<()> {
    let camera_db_filename = matches.get_one::<String>("camera_db").unwrap();
    let camera_db_json = json::read_file(camera_db_filename)?;
    let mut camera_db: CameraDatabase = json::from_json("camera database", &camera_db_json)?;
    camera_db.derive();
    args.cdb = Some(camera_db);
    Ok(())
}

//fp arg_star_calibrate
fn arg_star_calibrate(args: &mut CmdArgs, matches: &ArgMatches) -> Result<()> {
    let calibrate_filename = matches.get_one::<String>("star_calibrate").unwrap();
    let calibrate_json = json::read_file(calibrate_filename)?;
    args.cal = Some(StarCalibrate::from_json(
        args.cdb.as_ref().unwrap(),
        &calibrate_json,
    )?);
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

//fp arg_search_brightness
fn arg_search_brightness(args: &mut CmdArgs, matches: &ArgMatches) -> Result<()> {
    args.search_brightness = *matches.get_one::<f32>("search_brightness").unwrap();
    Ok(())
}

//fp arg_match_brightness
fn arg_match_brightness(args: &mut CmdArgs, matches: &ArgMatches) -> Result<()> {
    args.match_brightness = *matches.get_one::<f32>("match_brightness").unwrap();
    Ok(())
}

//a Main
pub fn main() -> Result<()> {
    let command = Command::new("ic_play")
        .about("Camera calibration tool")
        .version("0.1.0");

    let mut build = CommandBuilder::new(command, None);
    build.add_arg(
        ic_cmdline::camera::camera_database_arg(true),
        Box::new(arg_camera_db),
    );
    build.add_arg(
        Arg::new("star_calibrate")
            // .long("star")
            // .alias("database")
            .required(true)
            .help("Star calibration JSON")
            .action(ArgAction::Set),
        Box::new(arg_star_calibrate),
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
        Box::new(arg_search_brightness),
    );
    build.add_arg(
        Arg::new("match_brightness")
            .long("match_brightness")
            .value_parser(value_parser!(f32))
            .default_value("5.0")
            .help("Maximum brightness of stars to use for matching all the points")
            .action(ArgAction::Set),
        Box::new(arg_match_brightness),
    );
    build.add_arg(
        ic_cmdline::image::image_read_arg(true, Some(1)),
        Box::new(|args, matches| {
            ic_cmdline::image::get_image_read_filenames(matches).map(|v| args.read_img = v)
        }),
    );
    build.add_arg(
        ic_cmdline::image::image_write_arg(true),
        Box::new(|args, matches| {
            ic_cmdline::image::get_opt_image_write_filename(matches).map(|v| args.write_img = v)
        }),
    );

    let ms_command =
        Command::new("map_stars").about("Map all stars in the catalog onto an output image");
    let ms_build = CommandBuilder::new(ms_command, Some(Box::new(map_stars_cmd)));
    build.add_subcommand(ms_build);

    let fs_command = Command::new("find_stars").about("Find stars from an image");
    let fs_build = CommandBuilder::new(fs_command, Some(Box::new(find_stars_from_image_cmd)));
    build.add_subcommand(fs_build);

    let cd_command = Command::new("calibrate_desc").about("Generate a calibration description");
    let cd_build = CommandBuilder::new(cd_command, Some(Box::new(calibrate_desc_cmd)));
    build.add_subcommand(cd_build);

    let mut cmd_args = CmdArgs::default();
    let mut command: CommandSet<CmdArgs> = build.into();
    command.execute_env(&mut cmd_args)?;
    Ok(())
}

//fp calibrate_desc_cmd
fn calibrate_desc_cmd(cmd_args: &mut CmdArgs) -> Result<()> {
    let brightness = cmd_args.search_brightness;
    let (calibrate, catalog) = cmd_args.borrow_mut();
    let cat_index = calibrate.map_stars(catalog, brightness)?;

    //cb Show the star mappings
    let pc = calibrate.create_polynomial_calibrate(catalog);
    println!("{}", pc.to_desc_json()?);
    Ok(())
}

//fp map_stars_cmd
fn map_stars_cmd(cmd_args: &mut CmdArgs) -> Result<()> {
    let brightness = cmd_args.search_brightness;
    let (calibrate, catalog) = cmd_args.borrow_mut();
    let cat_index = calibrate.map_stars(catalog, brightness)?;

    //cb Show the star mappings
    let _ = calibrate.show_star_mappings(catalog);
    let mut mapped_pts = vec![];
    calibrate.add_catalog_stars(catalog, &mut mapped_pts)?;
    calibrate.add_cat_index(catalog, &cat_index, &mut mapped_pts)?;
    calibrate.add_mapping_pts(&mut mapped_pts)?;
    cmd_args.draw_image(&mapped_pts)?;
    Ok(())
}

//fp find_stars_from_image_cmd
fn find_stars_from_image_cmd(cmd_args: &mut CmdArgs) -> Result<()> {
    let brightness = cmd_args.search_brightness;
    let (calibrate, catalog) = cmd_args.borrow_mut();
    calibrate.find_stars_from_image(catalog, brightness)?;

    //cb Show the star mappings
    let mut mapped_pts = vec![];
    calibrate.add_catalog_stars(catalog, &mut mapped_pts)?;
    for p in calibrate.show_star_mappings(catalog) {
        mapped_pts.push((p, 1).into());
    }
    calibrate.add_mapping_pts(&mut mapped_pts)?;
    cmd_args.draw_image(&mapped_pts)?;
    Ok(())
}
