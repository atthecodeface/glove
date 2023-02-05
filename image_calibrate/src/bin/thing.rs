//a Modules
use clap::{Arg, ArgAction, Command};
use image::io::Reader as ImageReader;
use image::GenericImage;

use image_calibrate::{cmdline_args, CameraMapping, Point2D};

//fi get_image
fn get_image(filename: &str) -> Result<image::DynamicImage, String> {
    let img = ImageReader::open(filename)
        .map_err(|e| format!("Failed to open file {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode jpeg {}", e))?;
    Ok(img)
}

//fi write_image
fn write_image(img: &mut image::DynamicImage, filename: &str) -> Result<(), String> {
    image::save_buffer(
        filename,
        img.as_bytes(),
        img.width(),
        img.height(),
        image::ColorType::Rgb8,
    )
    .map_err(|e| format!("Failed to encode jpeg {}", e))?;
    Ok(())
}

//fi draw_cross
fn draw_cross(img: &mut image::DynamicImage, p: &Point2D, color: image::Rgba<u8>) {
    let cx = p[0] as u32;
    let cy = p[1] as u32;
    for i in 0..30 {
        img.put_pixel(cx - 15 + i, cy, color);
        img.put_pixel(cx, cy - 15 + i, color);
    }
}

//fi main
fn main() -> Result<(), String> {
    let cmd = Command::new("stuff")
        .about("about stuff")
        .version("0.1.0")
        .arg(
            Arg::new("read")
                .long("read")
                .short('r')
                .required(false)
                .help("Input image")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("write")
                .long("write")
                .short('w')
                .required(false)
                .help("Outnput image")
                .action(ArgAction::Set),
        );
    let cmd = cmdline_args::add_camera_database_arg(cmd, true);
    let cmd = cmdline_args::add_nps_arg(cmd, true);
    let cmd = cmdline_args::add_pms_arg(cmd, true);
    let cmd = cmdline_args::add_camera_projection_args(cmd, false);
    let cmd = cmdline_args::add_camera_arg(cmd, true);
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
        &|c, m, _n| c.total_error_with_bar(m),
        // &|c, m, _n| c.worst_error_with_bar(m),
        &[30., 30., 30.],
        31,
    );
    camera_mapping = camera_mapping.find_coarse_position(
        mappings,
        &|c, m, _n| c.total_error_with_bar(m),
        // &|c, m, _n| c.worst_error_with_bar(m),
        &[3., 3., 3.],
        31,
    );
    let num = mappings.len();
    for _ in 0..100 {
        for i in 0..num {
            camera_mapping = camera_mapping
                .adjust_direction_rotating_around_one_point(
                    // &|c, m, _n| c.total_error(m),
                    &|c, m, _n| c.total_error_with_bar(m),
                    // &|c, m, _n| c.worst_error(m),
                    // &|c, m, _n| c.worst_error_with_bar(m),
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
        let mut img = get_image(read_filename)?;
        let white = image::Rgba([255, 255, 255, 255]);
        let red = image::Rgba([255, 180, 255, 255]);
        if let Some(write_filename) = matches.get_one::<String>("write") {
            for m in mappings {
                draw_cross(&mut img, m.screen(), white);
            }
            for (_name, p) in nps.iter() {
                let mapped = camera_mapping.map_model(p.model());
                draw_cross(&mut img, &mapped, red);
            }
            write_image(&mut img, write_filename)?;
        }
    }

    println!("Final WE {:.2} {:.2} Camera {}", we, te, camera_mapping);
    println!("{}", serde_json::to_string(&camera_mapping).unwrap());
    Ok(())
}
