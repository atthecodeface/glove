//a Imports
use std::collections::HashMap;

use clap::{Arg, ArgAction, Command};

use image_calibrate::{cmdline_args, image, json, CameraMapping, Ray};

//fi main
fn main() -> Result<(), String> {
    let cmd = Command::new("create_rays")
        .about("Create rays for a given located camera and its mappings")
        .version("0.1.0")
        .arg(
            Arg::new("rays")
                .required(true)
                .help("Ray JSON files to be combined")
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("from_model")
                .long("from_model")
                .help("If rays are from the model then try to find a camera position")
                .action(ArgAction::SetTrue),
        );

    let cmd = cmdline_args::add_nps_arg(cmd, true);
    let matches = cmd.get_matches();
    let from_model = matches.get_flag("from_model");

    let nps = cmdline_args::get_nps(&matches)?;
    let mut ray_filenames: Vec<String> = matches
        .get_many::<String>("rays")
        .unwrap()
        .map(|v| v.into())
        .collect();

    if from_model {
        for r in ray_filenames {
            let r_json = json::read_file(r)?;
            let mut named_rays: Vec<(String, Ray)> = json::from_json("ray list", &r_json)?;
            let mut names = Vec::new();
            let mut ray_list = Vec::new();
            for (name, ray) in named_rays {
                names.push(name);
                ray_list.push(ray);
            }
            if ray_list.len() > 1 {
                let p = Ray::closest_point(&ray_list, &|r| 1.0 / r.tan_error()).unwrap();
                println!("Camera at {}", p);
                for (name, ray) in names.iter().zip(ray_list.iter()) {
                    let (k, d_sq) = ray.distances(&p);
                    eprintln!("{}: k {} dsq {} d {}", name, k, d_sq, d_sq.sqrt());
                }
            }
        }
    } else {
        let mut named_point_rays = HashMap::new();
        for r in ray_filenames {
            let r_json = json::read_file(r)?;
            let mut rays: Vec<(String, Ray)> = json::from_json("ray list", &r_json)?;
            for (name, ray) in rays {
                if nps.get_pt(&name).is_none() {
                    eprintln!(
                        "Warning: failed to find point name '{}' in named point set",
                        &name
                    );
                } else {
                    if !named_point_rays.contains_key(&name) {
                        named_point_rays.insert(name.clone(), Vec::new());
                    }
                    named_point_rays.get_mut(&name).unwrap().push(ray);
                }
            }
        }

        for (name, ray_list) in named_point_rays {
            if ray_list.len() > 1 {
                let p = Ray::closest_point(&ray_list, &|r| 1.0 / r.tan_error()).unwrap();
                eprintln!("Point '{}' - even weight - {}", name, p);
            }
        }
    }

    Ok(())
}
