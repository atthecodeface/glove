//a Imports
use std::path::{Path, PathBuf};

use clap::Command;
use star_catalog::Catalog as StarCatalog;
use thunderclap::CommandBuilder;
use thunderclap::{ArgCount, ArgDescriptor, CommandArgs};

use ic_base::{PathSet, Result};

use ic_base::Error;

pub type CmdResult = std::result::Result<String, ic_base::Error>;

#[derive(Default)]
pub struct SphericalImageCommand {
    verbose: bool,
    pretty_json: bool,

    file_path_set: PathSet,

    star_magnitude: f32,
    star_catalog: Option<StarCatalog>,

    // Positional string / f64 / usize arguments
    arg_strings: Vec<String>,
    arg_f64s: Vec<f64>,
    arg_usizes: Vec<usize>,
}

impl std::fmt::Debug for SphericalImageCommand {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "CmdArgs{{")?;
        if self.verbose {
            write!(fmt, "verbose, ")?;
        }
        if self.pretty_json {
            write!(fmt, "pretty_json")?;
        }
        Ok(())
    }
}

impl CommandArgs for SphericalImageCommand {
    type Error = ic_base::Error;
    type Value = String;
    fn cmd_ok() -> std::result::Result<Self::Value, Self::Error> {
        Ok("".into())
    }
    fn value_from_str(s: &str) -> std::result::Result<Self::Value, Self::Error> {
        Ok(s.into())
    }
    fn reset_args(&mut self) {}
    fn keys(&self) -> Box<dyn Iterator<Item = &str>> {
        const KEYS: [String; 0] = [];
        Box::new(KEYS.iter().map(|s| s.as_str()))
    }
    fn value_set(
        &mut self,
        _key: &str,
        _value: &Self::Value,
    ) -> std::result::Result<bool, Self::Error> {
        Ok((false))
    }
}

impl SphericalImageCommand {
    pub fn command_builder(&self, command: Command) -> CommandBuilder<Self> {
        let mut build = CommandBuilder::new(command);
        build.add_args(SA);
        build
    }
    fn set_verbose(&mut self, verbose: bool) -> Result<()> {
        self.verbose = verbose;
        Ok(())
    }

    fn set_pretty_json(&mut self, pretty_json: bool) -> Result<()> {
        self.pretty_json = pretty_json;
        Ok(())
    }

    fn set_star_magnitude(&mut self, v: f32) -> Result<()> {
        self.star_magnitude = v;
        Ok(())
    }

    fn set_star_catalog(&mut self, filename: &str) -> Result<()> {
        let mut catalog = StarCatalog::load_catalog(filename, self.star_magnitude)?;
        catalog.derive_data();
        self.star_catalog = Some(catalog);
        Ok(())
    }

    fn add_file_path(&mut self, s: &str) -> Result<()> {
        self.file_path_set.add_path(s)?;
        Ok(())
    }

    fn add_string_arg(&mut self, s: &str) -> Result<()> {
        self.arg_strings.push(s.to_owned());
        Ok(())
    }

    fn add_f64_arg(&mut self, v: f64) -> Result<()> {
        self.arg_f64s.push(v);
        Ok(())
    }

    fn add_usize_arg(&mut self, v: usize) -> Result<()> {
        self.arg_usizes.push(v);
        Ok(())
    }
}

const ARG_VERBOSE: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_flag(
    "verbose",
    Some('v'),
    "Enable verbose output",
    &SphericalImageCommand::set_verbose,
);

const ARG_PRETTY_JSON: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_flag(
    "pretty_json",
    None,
    "Use pretty-printing for Json output",
    &SphericalImageCommand::set_pretty_json,
);

const ARG_STAR_MAGNITUDE: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_f32(
    "star_magnitude",
    None,
    "Set the star magnitude, using a default value of 8.0",
    ArgCount::Required,
    Some("8.0"),
    &SphericalImageCommand::set_star_magnitude,
);

const ARG_STAR_CATALOG: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_string(
    "star_catalog",
    None,
    "Set the star catalog, used fo generate sky map images of the stars",
    ArgCount::Optional,
    None,
    &SphericalImageCommand::set_star_catalog,
);

const SA: &[ArgDescriptor<SphericalImageCommand>] = &[ARG_VERBOSE, ARG_PRETTY_JSON];
