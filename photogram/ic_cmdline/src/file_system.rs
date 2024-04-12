//a Modules
use clap::{Arg, ArgAction, ArgMatches, Command};

//a File system
//fp add_file_root_arg
pub fn add_file_root_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
        Arg::new("file_root")
            .long("file_root")
            .short('F')
            .required(required)
            .help("Root of files to serve")
            .long_help("Root of the files to server from the HTTP server, for file requests")
            .action(ArgAction::Set),
    )
}

//fp get_file_root
pub fn get_file_root(matches: &ArgMatches) -> Result<String, String> {
    let file_root = matches.get_one::<String>("file_root").unwrap().to_owned();
    Ok(file_root)
}

//fp add_image_root_arg
pub fn add_image_root_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
        Arg::new("image_root")
            .long("image_root")
            .short('I')
            .required(required)
            .help("Root of images to serve")
            .long_help("Root of the image files to server from the HTTP server")
            .action(ArgAction::Set),
    )
}

//fp get_image_root
pub fn get_image_root(matches: &ArgMatches) -> Result<String, String> {
    let image_root = matches.get_one::<String>("image_root").unwrap().to_owned();
    Ok(image_root)
}

//fp add_project_root_arg
pub fn add_project_root_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
        Arg::new("project_root")
            .long("project_root")
            .short('P')
            .required(required)
            .help("Directory containing the projects to serve")
            .long_help("Root on the server where projects are held")
            .action(ArgAction::Set),
    )
}

//fp get_project_root
pub fn get_project_root(matches: &ArgMatches) -> Result<String, String> {
    let project_root = matches
        .get_one::<String>("project_root")
        .unwrap()
        .to_owned();
    Ok(project_root)
}
