//a Imports
use clap::{Arg, ArgAction, Command};
use image_calibrate::{cmdline_args, image, CameraMapping};

//fi main
fn main() -> Result<(), String> {
    let cmd = Command::new("locate_camera")
        .about("Find location and orientation for a camera to map points to model")
        .version("0.1.0")
        .arg(
            Arg::new("steps")
                .long("steps")
                .required(false)
                .help("Number of steps per camera placement to try")
                .value_parser(clap::value_parser!(usize))
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("size")
                .required(true)
                .help("Size of ")
                .value_parser(clap::value_parser!(f64))
                .action(ArgAction::Append),
        );
    let cmd = cmdline_args::add_camera_database_arg(cmd, true);
    let cmd = cmdline_args::add_nps_arg(cmd, true);
    let cmd = cmdline_args::add_pms_arg(cmd, true);
    let cmd = cmdline_args::add_camera_arg(cmd, true);
    let cmd = cmdline_args::add_errors_arg(cmd);
    let cmd = cmdline_args::add_image_read_arg(cmd, false);
    let cmd = cmdline_args::add_image_write_arg(cmd, false);
    let matches = cmd.get_matches();

    let cdb = cmdline_args::get_camera_database(&matches)?;
    let nps = cmdline_args::get_nps(&matches)?;
    let pms = cmdline_args::get_pms(&matches, &nps)?;
    let camera = cmdline_args::get_camera(&matches, &cdb)?;
    let mut camera_mapping = CameraMapping::of_camera(camera);
    let mappings = pms.mappings();
    let error_method = cmdline_args::get_error_fn(&matches);

    let mut steps = 11;
    if let Some(s) = matches.get_one::<usize>("steps") {
        steps = *s;
    }
    if steps < 1 || steps > 101 {
        return Err(format!(
            "Steps value should be between 1 and 100: was given {}",
            steps
        ));
    }
    let mut sizes: Vec<f64> = matches
        .get_many::<f64>("size")
        .unwrap()
        .map(|v| *v)
        .collect();
    sizes.sort_by(|a, b| b.partial_cmp(a).unwrap());

    for s in sizes {
        eprintln!(
            "Adjusting position of camera using a size of {} and {} steps",
            s, steps
        );
        camera_mapping = camera_mapping.find_coarse_position(
            mappings,
            &error_method,
            // &|c, m, _n| c.total_error(m),
            // &|c, m, _n| c.worst_error(m),
            &[s, s, s],
            steps,
        );
    }

    let num = mappings.len();
    for _ in 0..100 {
        for i in 0..num {
            camera_mapping = camera_mapping
                .adjust_direction_rotating_around_one_point(
                    &error_method,
                    0.1_f64.to_radians(),
                    mappings,
                    i,
                    0,
                )
                .0;
        }
    }

    let te = camera_mapping.total_error(mappings);
    let we = camera_mapping.worst_error(mappings);
    camera_mapping.show_mappings(mappings);
    camera_mapping.show_point_set(&nps);

    if let Some(read_filename) = matches.get_one::<String>("read") {
        let mut img = image::read_image(read_filename)?;
        let white = &[255, 255, 255, 255];
        let red = &[255, 180, 255, 255];
        if let Some(write_filename) = matches.get_one::<String>("write") {
            for m in mappings {
                image::draw_cross(&mut img, m.screen(), m.error(), white);
            }
            for (_name, p) in nps.iter() {
                let mapped = camera_mapping.map_model(p.model());
                image::draw_cross(&mut img, &mapped, 5.0, &red);
            }
            image::write_image(&mut img, write_filename)?;
        }
    }

    eprintln!("Final WE {:.2} {:.2} Camera {}", we, te, camera_mapping);
    println!("{}", serde_json::to_string_pretty(&camera_mapping).unwrap());
    Ok(())
}
