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
