//a Imports
use clap::Command;
use image_calibrate::{cmdline_args, image, CameraMapping};

//fi main
fn main() -> Result<(), String> {
    let cmd = Command::new("stuff").about("about stuff").version("0.1.0");
    let cmd = cmdline_args::add_camera_database_arg(cmd, true);
    let cmd = cmdline_args::add_nps_arg(cmd, true);
    let cmd = cmdline_args::add_pms_arg(cmd, true);
    let cmd = cmdline_args::add_camera_projection_args(cmd, false);
    let cmd = cmdline_args::add_camera_arg(cmd, true);
    let cmd = cmdline_args::add_image_read_arg(cmd, false);
    let cmd = cmdline_args::add_image_write_arg(cmd, false);
    let matches = cmd.get_matches();

    let cdb = cmdline_args::get_camera_database(&matches)?;
    let nps = cmdline_args::get_nps(&matches)?;
    let pms = cmdline_args::get_pms(&matches, &nps)?;
    // let camera_projection = cmdline_args::get_camera_projection(&matches, &cdb)?;
    let camera = cmdline_args::get_camera(&matches, &cdb)?;
    let mut camera_mapping = CameraMapping::of_camera(camera);
    let mappings = pms.mappings();

    //    camera_mapping = camera_mapping.find_coarse_position(mappings, &[3000., 3000., 3000.], 31);
    //    camera_mapping = camera_mapping.find_coarse_position(mappings, &[300., 300., 300.], 31);
    camera_mapping = camera_mapping.find_coarse_position(
        mappings,
        &|c, m, _n| c.total_error(m),
        // &|c, m, _n| c.worst_error(m),
        &[30., 30., 30.],
        31,
    );
    camera_mapping = camera_mapping.find_coarse_position(
        mappings,
        &|c, m, _n| c.total_error(m),
        // &|c, m, _n| c.worst_error(m),
        &[3., 3., 3.],
        31,
    );
    let num = mappings.len();
    for _ in 0..100 {
        for i in 0..num {
            camera_mapping = camera_mapping
                .adjust_direction_rotating_around_one_point(
                    &|c, m, _n| c.total_error(m),
                    // &|c, m, _n| c.worst_error(m),
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
                image::draw_cross(&mut img, mapped, 5.0, red);
            }
            image::write_image(&mut img, write_filename)?;
        }
    }

    println!("Final WE {:.2} {:.2} Camera {}", we, te, camera_mapping);
    println!("{}", serde_json::to_string(&camera_mapping).unwrap());
    Ok(())
}
