//a Modules
use clap::{Arg, ArgAction, ArgMatches, Command};

use ic_base::{Error, Result};
use ic_camera::CameraInstance;
use ic_image::{Color, ImageRgb8};

use crate::builder::{CommandArgs, CommandBuilder};

//a Image options
//fp image_read_arg
pub fn image_read_arg(required: bool, num_args: Option<usize>) -> Arg {
    let arg = Arg::new("read")
        .long("read")
        .short('r')
        .required(required)
        .help("Image to read")
        .action(ArgAction::Append);
    if let Some(num_args) = num_args {
        arg.num_args(num_args)
    } else {
        arg
    }
}

//fp image_write_arg
pub fn image_write_arg(required: bool) -> Arg {
    Arg::new("write")
        .long("write")
        .short('w')
        .required(required)
        .help("Image to write")
        .action(ArgAction::Set)
}

//mp add_arg_read_img
pub fn add_arg_read_img<C, F>(
    build: &mut CommandBuilder<C>,
    set: F,
    required: bool,
    num_args: Option<usize>,
) where
    C: CommandArgs<Error = Error>,
    F: Fn(&mut C, Vec<String>) -> Result<()> + 'static,
{
    build.add_arg(
        image_read_arg(required, num_args),
        Box::new(move |args, matches| get_image_read_filenames(matches).and_then(|v| set(args, v))),
    );
}

//mp add_arg_write_img
pub fn add_arg_write_img<C, F>(build: &mut CommandBuilder<C>, set: F, required: bool)
where
    C: CommandArgs<Error = Error>,
    F: Fn(&mut C, &str) -> Result<()> + 'static,
{
    build.add_arg(
        image_write_arg(required),
        Box::new(move |args, matches| set(args, matches.get_one::<String>("write").unwrap())),
    );
}

//fp add_image_read_arg
pub fn add_image_read_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(image_read_arg(required, None))
}

//fp get_image_read
pub fn get_image_read(matches: &ArgMatches) -> Result<ImageRgb8> {
    let read_filename = matches.get_one::<String>("read").unwrap();
    let img = ImageRgb8::read_image(read_filename)?;
    Ok(img)
}

//fp get_image_read_filenames
pub fn get_image_read_filenames(matches: &ArgMatches) -> Result<Vec<String>> {
    Ok(matches
        .get_many::<String>("read")
        .unwrap()
        .cloned()
        .collect())
}

//fp get_image_read_all
pub fn get_image_read_all(matches: &ArgMatches) -> Result<Vec<ImageRgb8>> {
    let mut images = vec![];
    for read_filename in matches.get_many::<String>("read").unwrap() {
        let img = ImageRgb8::read_image(read_filename)?;
        images.push(img);
    }
    Ok(images)
}

//fp get_image_read_or_create
pub fn get_image_read_or_create(
    matches: &ArgMatches,
    camera: &CameraInstance,
) -> Result<ImageRgb8> {
    let read_filename = matches.get_one::<String>("read");
    let img = ImageRgb8::read_or_create_image(
        camera.body().px_width() as usize,
        camera.body().px_height() as usize,
        read_filename.map(|x| x.as_str()),
    )?;
    Ok(img)
}

//fp add_image_write_arg
pub fn add_image_write_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(image_write_arg(required))
}

//fp get_opt_image_write_filename
pub fn get_opt_image_write_filename(matches: &ArgMatches) -> Result<Option<String>> {
    Ok(matches.get_one::<String>("write").cloned())
}

//a Colors
//fp add_color_arg
pub fn add_color_arg(cmd: Command, prefix: &str, help: &str, required: bool) -> Command {
    let (id, long) = {
        if prefix.is_empty() {
            ("c".to_string(), "color".to_string())
        } else {
            (prefix.to_string(), prefix.to_string())
        }
    };
    cmd.arg(
        Arg::new(id)
            .long(long)
            .required(required)
            .help(help.to_string())
            .action(ArgAction::Set),
    )
}

//fp get_opt_color
pub fn get_opt_color(matches: &ArgMatches, prefix: &str) -> Result<Option<Color>> {
    if let Some(bg) = matches.get_one::<String>(prefix) {
        let c: Color = bg.as_str().try_into()?;
        Ok(Some(c))
    } else {
        Ok(None)
    }
}

//fp add_bg_color_arg
pub fn add_bg_color_arg(cmd: Command, required: bool) -> Command {
    add_color_arg(cmd, "bg", "Image background color", required)
}

//fp get_opt_bg_color
pub fn get_opt_bg_color(matches: &ArgMatches) -> Result<Option<Color>> {
    get_opt_color(matches, "bg")
}
