//a Imports
use clap::{Arg, ArgAction, Command};
use image_calibrate::{cmdline_args, image, CameraMapping};

//fi main
fn main() -> Result<(), String> {
    let cmd = Command::new("create_rays")
        .about("Create rays for a given located camera and its mappings")
        .version("0.1.0")
        .arg(
            Arg::new("from_camera")
                .long("from_camera")
                .help("Create rays from the camera rather than from the model")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("from_model")
                .long("from_model")
                .help("Create rays from the model to the camera")
                .action(ArgAction::SetTrue),
        );

    let cmd = cmdline_args::add_camera_database_arg(cmd, true);
    let cmd = cmdline_args::add_nps_arg(cmd, true);
    let cmd = cmdline_args::add_pms_arg(cmd, true);
    let cmd = cmdline_args::add_camera_arg(cmd, true);
    let matches = cmd.get_matches();
    let from_camera = matches.get_flag("from_camera");
    let from_model = matches.get_flag("from_model");

    let cdb = cmdline_args::get_camera_database(&matches)?;
    let nps = cmdline_args::get_nps(&matches)?;
    let pms = cmdline_args::get_pms(&matches, &nps)?;
    let camera = cmdline_args::get_camera(&matches, &cdb)?;
    let mut camera_mapping = CameraMapping::of_camera(camera);
    let mappings = pms.mappings();

    println!(
        "{}",
        serde_json::to_string_pretty(&camera_mapping.get_rays(mappings, !from_model)).unwrap()
    );
    Ok(())
}
