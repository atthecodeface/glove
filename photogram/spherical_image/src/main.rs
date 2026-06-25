use ic_base::Result;

use spherical_image::SphericalImageCommand;

fn main() -> Result<()> {
    let build = SphericalImageCommand::command_builder();
    let mut spherical_image_command = SphericalImageCommand::default();
    let mut command = build.main(true, true);
    command
        .execute_env(&mut spherical_image_command)
        .map_err(|e| format!("Error {e:?}"))?;

    Ok(())
}
