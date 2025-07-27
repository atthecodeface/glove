//a Documentation
//! Test of camera calibration from stars (IMG_4924.JPG)
//!
//! The stars were captured on a Canon Rebel T2i, with a 50mm lens focused on 'infinity'
//!

//a Imports
use clap::{Arg, ArgAction, ArgMatches, Command};
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
    closeness: f64,
    within: f64,
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
    fn get_cdb(&self) -> &CameraDatabase {
        self.cdb.as_ref().unwrap()
    }
    fn set_cdb(&mut self, cdb: CameraDatabase) -> Result<()> {
        self.cdb = Some(cdb);
        Ok(())
    }
    fn set_camera(&mut self, camera: CameraInstance) -> Result<()> {
        self.camera = camera;
        Ok(())
    }
    fn set_mapping(&mut self, mapping: StarMapping) -> Result<()> {
        self.mapping = mapping;
        Ok(())
    }
    fn set_read_img(&mut self, v: Vec<String>) -> Result<()> {
        self.read_img = v;
        Ok(())
    }
    fn set_write_img(&mut self, s: &str) -> Result<()> {
        self.write_img = Some(s.to_owned());
        Ok(())
    }
    fn set_closeness(&mut self, closeness: f64) -> Result<()> {
        self.closeness = closeness;
        Ok(())
    }
    fn set_within(&mut self, within: f64) -> Result<()> {
        self.within = within;
        Ok(())
    }
    fn set_match_brightness(&mut self, brightness: f32) -> Result<()> {
        self.match_brightness = brightness;
        Ok(())
    }
    fn set_search_brightness(&mut self, brightness: f32) -> Result<()> {
        self.search_brightness = brightness;
        Ok(())
    }

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
        args.catalog = Some(json::from_json("star catalog", &s)?);
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

    ic_cmdline::camera::add_arg_camera_database(&mut build, CmdArgs::set_cdb, true);
    ic_cmdline::camera::add_arg_camera(&mut build, CmdArgs::get_cdb, CmdArgs::set_camera, true);
    ic_cmdline::add_arg_f64(&mut build,
                                    "closeness", None,
                                    "Closeness (degrees) to find triangles of stars or degress for calc cal mapping, find stars, map_stars etc",
                                    Some("0.2"),
                                    CmdArgs::set_closeness,
                                    false);
    build.add_arg(
        Arg::new("star_mapping")
            .required(true)
            .help("File mapping sensor coordinates to catalog identifiers")
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

    ic_cmdline::add_arg_f32(
        &mut build,
        "search_brightness",
        None,
        "Maximum brightness of stars to use for searching with triangles",
        Some("5.0"),
        CmdArgs::set_search_brightness,
        false,
    );

    ic_cmdline::add_arg_f32(
        &mut build,
        "match_brightness",
        None,
        "Maximum brightness of stars to use for matching all the points",
        Some("5.0"),
        CmdArgs::set_match_brightness,
        false,
    );

    let sm_command =
        Command::new("show_star_mapping").about("Show the mapped stars onto an output image");
    let mut sm_build = CommandBuilder::new(sm_command, Some(Box::new(show_star_mapping_cmd)));
    ic_cmdline::image::add_arg_read_img(&mut sm_build, CmdArgs::set_read_img, false, Some(1));
    ic_cmdline::image::add_arg_write_img(&mut sm_build, CmdArgs::set_write_img, false);
    ic_cmdline::add_arg_f64(
        &mut sm_build,
        "within",
        None,
        "Only use catalog stars Within this angle (degrees) for mapping",
        Some("15"),
        CmdArgs::set_within,
        false,
    );
    build.add_subcommand(sm_build);

    let ms_command = Command::new("orient").about("Orient on all of the mapped stars");
    let ms_build = CommandBuilder::new(ms_command, Some(Box::new(orient_cmd)));
    build.add_subcommand(ms_build);

    let fs_command = Command::new("find_stars").about("Find stars from an image");
    let mut fs_build = CommandBuilder::new(fs_command, Some(Box::new(find_stars_from_image_cmd)));
    build.add_subcommand(fs_build);

    let cd_command = Command::new("calibrate_desc").about("Generate a calibration description");
    let cd_build = CommandBuilder::new(cd_command, Some(Box::new(calibrate_desc_cmd)));
    build.add_subcommand(cd_build);

    let ms_command = Command::new("update_star_mapping").about(
        "Generate an updated mapping of stars from the catalog to with ids frmom the catalog",
    );
    let mut ms_build = CommandBuilder::new(ms_command, Some(Box::new(update_star_mapping_cmd)));
    ic_cmdline::add_arg_f64(
        &mut ms_build,
        "within",
        None,
        "Only use catalog stars Within this angle (degrees) for mapping",
        Some("15"),
        CmdArgs::set_within,
        false,
    );
    build.add_subcommand(ms_build);

    let mut cmd_args = CmdArgs::default();
    let mut command: CommandSet<CmdArgs> = build.into();
    command.execute_env(&mut cmd_args)?;
    Ok(())
}

//fp calibrate_desc_cmd
fn calibrate_desc_cmd(cmd_args: &mut CmdArgs) -> Result<()> {
    //cb Show the star mappings
    let close_enough = cmd_args.closeness;
    let pc = cmd_args.mapping.create_calibration_mapping(
        cmd_args.catalog.as_ref().unwrap(),
        &cmd_args.camera,
        close_enough,
    );
    println!("{}", pc.to_json()?);
    Ok(())
}

//fp update_star_mapping_cmd
fn update_star_mapping_cmd(cmd_args: &mut CmdArgs) -> Result<()> {
    //cb Show the star mappings
    let close_enough = cmd_args.closeness;
    let within = cmd_args.within;
    let (num_unmapped, total_error) = cmd_args.mapping.update_star_mappings(
        cmd_args.catalog.as_ref().unwrap(),
        &cmd_args.camera,
        close_enough,
        within,
    );
    eprintln!(
        "{num_unmapped} stars were not mapped, total error of mapped stars {total_error:.4e}"
    );
    println!("{}", cmd_args.mapping.clone().to_json()?);
    Ok(())
}

//fp show_star_mapping_cmd
fn show_star_mapping_cmd(cmd_args: &mut CmdArgs) -> Result<()> {
    let brightness = cmd_args.search_brightness;
    let within = cmd_args.within;
    cmd_args
        .catalog
        .as_mut()
        .unwrap()
        .retain(move |s, _n| s.brighter_than(brightness));
    cmd_args.catalog.as_mut().unwrap().sort();
    cmd_args.catalog.as_mut().unwrap().derive_data();

    //cb Show the star mappings
    let close_enough = cmd_args.closeness;
    let _ = cmd_args.mapping.show_star_mappings(
        cmd_args.catalog.as_ref().unwrap(),
        &cmd_args.camera,
        close_enough,
    );

    let mut mapped_pts = vec![];

    // Mark the points with blue-grey crosses
    cmd_args.mapping.img_pts_add_cat_index(
        cmd_args.catalog.as_ref().unwrap(),
        &cmd_args.camera,
        &mut mapped_pts,
        1,
        cmd_args.search_brightness,
    )?;

    // Mark the mapping points with small purple crosses
    cmd_args
        .mapping
        .img_pts_add_mapping_pxy(&mut mapped_pts, 0)?;

    // Mark the catalog stars with yellow Xs
    cmd_args.mapping.img_pts_add_catalog_stars(
        cmd_args.catalog.as_ref().unwrap(),
        &cmd_args.camera,
        &mut mapped_pts,
        within,
        2,
    )?;

    cmd_args.draw_image(&mapped_pts)?;

    Ok(())
}

//fp orient_cmd
fn orient_cmd(cmd_args: &mut CmdArgs) -> Result<()> {
    let brightness = cmd_args.search_brightness;
    let orientation = cmd_args.mapping.find_orientation_from_all_mapped_stars(
        cmd_args.catalog.as_ref().unwrap(),
        &cmd_args.camera,
        brightness,
    )?;
    cmd_args.camera.set_orientation(orientation);
    println!("{}", cmd_args.camera.to_json()?);
    Ok(())
}

//fp find_stars_from_image_cmd
fn find_stars_from_image_cmd(cmd_args: &mut CmdArgs) -> Result<()> {
    let brightness = cmd_args.search_brightness;
    let closeness = cmd_args.closeness;

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
        closeness.to_radians() as f32,
    )?;
    cmd_args.camera.set_orientation(orientation);

    //cb Show the star mappings
    let close_enough = cmd_args.closeness;
    let _ = cmd_args.mapping.show_star_mappings(
        cmd_args.catalog.as_ref().unwrap(),
        &cmd_args.camera,
        close_enough,
    );
    println!("{}", cmd_args.camera.to_json()?);
    Ok(())
}
