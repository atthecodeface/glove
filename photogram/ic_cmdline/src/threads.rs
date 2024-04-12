//a Modules
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};

//a ThreadPool
//fp add_threads_arg
pub fn add_threads_arg(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("threads")
            .long("threads")
            .short('t')
            .default_value("4")
            .help("Number of threads")
            .long_help("Number of threads for the server to run to handle simultaneous requests")
            .value_parser(value_parser!(usize))
            .action(ArgAction::Set),
    )
}

//fp add_port_arg
pub fn add_port_arg(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("port")
            .long("port")
            .short('p')
            .default_value("8020")
            .help("Port to use")
            .long_help("The TCP port number to use for the server")
            .value_parser(value_parser!(usize))
            .action(ArgAction::Set),
    )
}

//fp get_threads
pub fn get_threads(matches: &ArgMatches) -> Result<usize, String> {
    let num_threads = *matches.get_one::<usize>("threads").unwrap();
    Ok(num_threads)
}

//fp get_port
pub fn get_port(matches: &ArgMatches) -> Result<usize, String> {
    let port = *matches.get_one::<usize>("port").unwrap();
    Ok(port)
}
