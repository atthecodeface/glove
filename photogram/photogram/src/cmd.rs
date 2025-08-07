//a Imports
use std::cell::Ref;
use std::io::Write;
use std::rc::Rc;

use clap::{Arg, ArgAction, ArgMatches, Command};
use star_catalog::{Catalog, StarFilter};
use thunderclap::{ArgCount, CommandArgs, CommandBuilder};

use ic_base::{json, Ray, Rrc};
use ic_base::{Error, Result};
use ic_camera::CameraInstance;
use ic_camera::{CalibrationMapping, CameraDatabase, CameraProjection, LensPolys};
use ic_image::{Color, Image, ImagePt, ImageRgb8};
use ic_mapping::{CameraPtMapping, PointMapping};
use ic_mapping::{NamedPointSet, PointMappingSet};
use ic_project::{Cip, Project};
use ic_stars::StarMapping;

mod types;
pub use types::{cmd_ok, CmdArgs, CmdResult};

mod accessors;
mod args;
mod operation;
mod output;
mod setters;
