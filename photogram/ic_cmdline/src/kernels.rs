//a Modules
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};

//a Kernels
//fp add_kernel_arg
pub fn add_kernel_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
        Arg::new("kernel")
            .long("kernel")
            .help("Add a kernel to run")
            .long_help("A kernel to run on the data")
            .required(required)
            .action(ArgAction::Append),
    )
}

//fp get_kernels
pub fn get_kernels(matches: &ArgMatches) -> Result<Vec<String>, String> {
    let mut kernels = vec![];
    if let Some(args) = matches.get_many::<String>("kernel") {
        for kernel in args {
            kernels.push(kernel.into());
        }
    }
    Ok(kernels)
}

//fp add_size_arg
pub fn add_size_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
        Arg::new("kernel_size")
            .long("kernel_size")
            .default_value("8")
            .help("Size (uint) of the kernel (radius, window, etc)")
            .long_help("The size of the kernel (radius, window, etc)")
            .value_parser(value_parser!(usize))
            .required(required)
            .action(ArgAction::Set),
    )
}

//fp get_size
pub fn get_size(matches: &ArgMatches) -> Result<usize, String> {
    let size = *matches.get_one::<usize>("kernel_size").unwrap();
    Ok(size)
}

//fp add_scale_arg
pub fn add_scale_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
        Arg::new("kernel_scale")
            .long("kernel_scale")
            .default_value("1")
            .help("Scale factor (f32) of the kernel")
            .long_help("The scale factor (f32) of the kernel")
            .value_parser(value_parser!(f32))
            .required(required)
            .action(ArgAction::Set),
    )
}

//fp get_scale
pub fn get_scale(matches: &ArgMatches) -> Result<f32, String> {
    let scale = *matches.get_one::<f32>("kernel_scale").unwrap();
    Ok(scale)
}

//fp add_xy_arg
pub fn add_xy_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
        Arg::new("kernel_x")
            .long("kernel_x")
            .default_value("0")
            .help("X coordinate for the kernel")
            .value_parser(value_parser!(usize))
            .required(required)
            .action(ArgAction::Set),
    )
    .arg(
        Arg::new("kernel_y")
            .long("kernel_y")
            .default_value("0")
            .help("Y coordinate for the kernel")
            .value_parser(value_parser!(usize))
            .required(required)
            .action(ArgAction::Set),
    )
}

//fp get_xy
pub fn get_xy(matches: &ArgMatches) -> Result<(usize, usize), String> {
    let x = *matches.get_one::<usize>("kernel_x").unwrap();
    let y = *matches.get_one::<usize>("kernel_y").unwrap();
    Ok((x, y))
}

//fp add_flags_arg
pub fn add_flags_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
        Arg::new("kernel_flags")
            .long("kernel_flags")
            .default_value("0")
            .help("Flags (uint) of the kernel")
            .value_parser(value_parser!(usize))
            .required(required)
            .action(ArgAction::Set),
    )
}

//fp get_flags
pub fn get_flags(matches: &ArgMatches) -> Result<usize, String> {
    let flags = *matches.get_one::<usize>("kernel_flags").unwrap();
    Ok(flags)
}
