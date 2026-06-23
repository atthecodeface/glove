use clap::Command;

use spherical_image::SphericalImageCommand;

use ic_base::Result;

fn main() -> Result<()> {
    let command = Command::new("spherical_image")
        .about("Spherical image processor")
        .version("0.1.0");

    let mut spherical_image_command = SphericalImageCommand::default();
    let build = spherical_image_command.command_builder(command);
    let mut command = build.main(true, true);
    command
        .execute_env(&mut spherical_image_command)
        .map_err(|e| format!("Error {e:?}"))?;

    Ok(())
}
