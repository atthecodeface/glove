//a Imports
use std::collections::HashMap;

use clap::Command;
use thunderclap::CommandBuilder;

use ic_base::Result;
use ic_camera::polynomial;
use ic_camera::polynomial::CalcPoly;
use ic_image::{Color, Image, ImageGray16, Region};
use ic_kernel::{KernelArgs, Kernels};

use crate::cmd::{CmdArgs, CmdResult};

//a Help
//hi FIND_REGIONS_LONG_HELP
const FIND_REGIONS_LONG_HELP: &str = "\
This treats the image file read in as a set of non-background color
regions.

A region is a contiguous set of non-background pixels. The
centre-of-gravity of each region is determined, and the result is a
JSON file containing a list of pairs of floats of those cogs.";

//hi FIND_GRID_POINTS_LONG_HELP
const FIND_GRID_POINTS_LONG_HELP: &str = "\
This treats the image file read in as a set of non-background color
regions each of which should be centred on a (cm,cm) grid point from a
square-on photo of a piece of graph paper.

A region is a contiguous set of non-background pixels. The
centre-of-gravity of each region is determined. The region closest to
the centre of the photograph is deemed to be (0cm, 0cm). The grid
should be horizontal and vertical, and the X and Y axes are
determined, with the approximate 1cm spacing determined in X and
Y. Horizontal and vertical curves are generated internally using
points with similar Y and X values, to produce approximations to the
grid lines on the image, using least-square error polynomial
approximations. From this approximate grid points are redetermined,
and a JSON file of a list of tuples of (grid xmm, grid ymm, frame x,
frame y) is produced. If a 'write' image filename is provided then an
image is generated that is black background with red crosses at each
grid point.";

//hi AS_LUMA_LONG_HELP
const AS_LUMA_LONG_HELP: &str = "\
Generate a 16-bit luma image
";

//hi LUMA_WINDOW_LONG_HELP
const LUMA_WINDOW_LONG_HELP: &str = "\
Analyze an image in luma space using a window
";

//hi LUMA_KERNEL_LONG_HELP
const LUMA_KERNEL_LONG_HELP: &str = "\
Analyze kernels to an image in luma space

Convert the image to a 16-bit luma

Apply a number of kernels (with a single set of size, scale etc arguments)

Output the image as a 16-bit luma image (so the kernel output should be in the range 0.0 to 1.0)
";

//hi LUMA_KERNEL_PAIR_LONG_HELP
const LUMA_KERNEL_PAIR_LONG_HELP: &str = "\
Apply kernels to a pair of images in luma space

Convert the images to 16-bit luma

Apply a number of kernels (with a single set of size, scale etc arguments)

Output the image as a 16-bit luma image (so the kernel output should be in the range 0.0 to 1.0)
";

//a Useful functions
//fi find_center_point
/// Find the point closest to the center
fn find_center_point((cx, cy): (f64, f64), pts: &[(f64, f64)]) -> usize {
    let mut min_dsq = f64::MAX;
    let mut min = 0;
    for (n, (px, py)) in pts.iter().enumerate() {
        let dx = *px - cx;
        let dy = *py - cy;
        let dsq = dx * dx + dy * dy;
        if dsq < min_dsq {
            min = n;
            min_dsq = dsq;
        }
    }
    min
}

//fi find_axis_pts
/// From a given origin (cx,cy), follow all those points with similar
/// cy (for an x axis) or cx, and generate the index of the points and
/// the delta along the axis for those points on the axis
fn find_axis_pts(
    x_axis: bool,
    closeness: f64,
    (cx, cy): (f64, f64),
    pts: &[(f64, f64)],
) -> Option<Vec<(usize, f64)>> {
    let mut pts_on_axis = vec![];
    let c_sq = closeness * closeness;
    for (n, (px, py)) in pts.iter().enumerate() {
        let dx = *px - cx;
        let dy = *py - cy;
        let (d_off, d_along) = if x_axis { (dy, dx) } else { (dx, dy) };
        if d_off * d_off < c_sq {
            pts_on_axis.push((n, d_along));
        }
    }
    if pts_on_axis.is_empty() {
        None
    } else {
        Some(pts_on_axis)
    }
}

//fi spacing_of_coords
fn spacing_of_coords(pts: &[(usize, f64)]) -> Result<f64> {
    let mut coords: Vec<f64> = pts.iter().map(|(_, n)| *n).collect();
    coords.sort_by(|f0, f1| f0.partial_cmp(f1).unwrap());
    let mut min_spacing = coords[1] - coords[0];
    for i in 1..coords.len() {
        min_spacing = min_spacing.min(coords[i] - coords[i - 1]);
    }
    let mut total_spaces = 0.0;
    for i in 1..coords.len() {
        let spacing = coords[i] - coords[i - 1];
        let d = spacing / min_spacing;
        let d_n = (d + 0.5).floor();
        total_spaces += d_n;
    }
    let avg_spacing = (coords.last().unwrap() - coords[0]) / total_spaces;
    Ok(avg_spacing)
}

//a Luma
//fi as_luma_cmd
fn as_luma_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("as_luma")
        .about("Generate a 16-bit luma image")
        .long_about(AS_LUMA_LONG_HELP);

    CommandBuilder::new(command, Some(Box::new(as_luma_fn)))
}

//fi as_luma_fn
fn as_luma_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let img = cmd_args.get_image_read_or_create()?;

    eprintln!("Read initial image, size is {:?}", img.size());
    let (w, h, img_data) = img.as_vec_gray_f32(None);

    let img = ImageGray16::of_vec_f32(w, h, img_data, 1.0);

    eprintln!("Created luma image");

    if let Some(write_filename) = cmd_args.write_img() {
        img.write(write_filename)?;
        eprintln!("Image written");
    } else {
        eprintln!("Image not written as no output image provided");
    }
    Ok("".into())
}

//fi luma_window_cmd
fn luma_window_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("luma_window")
        .about("Analyze an image in luma space using a window")
        .long_about(LUMA_WINDOW_LONG_HELP);

    CommandBuilder::new(command, Some(Box::new(luma_window_fn)))
}

//fi luma_window_fn
fn luma_window_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let img = cmd_args.get_image_read_or_create()?;

    eprintln!(
        "Read initial image, size is {:?} (max pixels in kernel is 4M)",
        img.size()
    );
    let (w, h) = img.size();
    let npix = w as usize * h as usize;
    let max = 4 * 1024 * 1024;
    let scale =
        (npix > max).then_some(((max as f32 / npix as f32).sqrt() * w as f32).floor() as usize);
    let (w, h, mut img_data) = img.as_vec_gray_f32(scale);
    eprintln!(
        "Using size {w}, {h} ({:.2} Mpx)",
        (w * h) as f32 / 1024.0 / 1024.0
    );

    let kernels = Kernels::new();
    let ws = 8;
    let args: KernelArgs = (w, h).into();
    let args = args.with_size(ws as usize);
    let ws_f = ws as f32;
    let args_mean = args.with_scale(1.0 / ws_f);

    kernels.run_shader(
        "window_var",
        &args_mean,
        w * h,
        None,
        img_data.as_mut_slice(),
    )?;

    eprintln!("Completed kernel");
    let img = ImageGray16::of_vec_f32(w, h, img_data, 1.0);
    eprintln!("Created luma image");

    if let Some(write_filename) = cmd_args.write_img() {
        img.write(write_filename)?;
        eprintln!("Image written");
    } else {
        eprintln!("Image not written as no output image provided");
    }
    Ok("".into())
}

//fi luma_kernel_cmd
fn luma_kernel_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("luma_kernel")
        .about("Apply kernels to an image in luma space")
        .long_about(LUMA_KERNEL_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(luma_kernel_fn)));
    CmdArgs::add_arg_kernel(&mut build, (1,));
    CmdArgs::add_arg_scale(&mut build);
    CmdArgs::add_arg_kernel_size(&mut build, false);
    CmdArgs::add_arg_px(&mut build, true);
    CmdArgs::add_arg_py(&mut build, true);

    build
}

//fi luma_kernel_fn
fn luma_kernel_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let img = cmd_args.get_read_image(0)?;

    let ws = cmd_args.kernel_size();
    let scale = cmd_args.scale();
    let xy = cmd_args.pxy();
    let kernels_to_apply = cmd_args.kernels();

    eprintln!(
        "Read initial image, size is {:?} (max pixels in kernel is 4M)",
        img.size()
    );
    let (w, h) = img.size();
    let npix = w as usize * h as usize;
    let max = 4 * 1024 * 1024;
    let img_scale =
        (npix > max).then_some(((max as f32 / npix as f32).sqrt() * w as f32).floor() as usize);
    let (w, h, mut img_data) = img.as_vec_gray_f32(img_scale);
    eprintln!(
        "Using size {w}, {h} ({:.2} Mpx)",
        (w * h) as f32 / 1024.0 / 1024.0
    );

    let kernels = Kernels::new();
    let args: KernelArgs = (w, h).into();
    let args = args.with_size(ws as usize);
    let args = args.with_scale(scale as f32);
    let args = args.with_xy(xy);

    for k in kernels_to_apply {
        kernels.run_shader(&k, &args, w * h, None, img_data.as_mut_slice())?;
    }

    eprintln!("Completed kernel");
    let img = ImageGray16::of_vec_f32(w, h, img_data, 1.0);
    eprintln!("Created luma image");

    if let Some(write_filename) = cmd_args.write_img() {
        img.write(write_filename)?;
        eprintln!("Image written");
    } else {
        eprintln!("Image not written as no output image provided");
    }
    Ok("".into())
}

//fi luma_kernel_pair_cmd
fn luma_kernel_pair_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("luma_kernel_pair")
        .about("Apply kernels to a pair of images in luma space")
        .long_about(LUMA_KERNEL_PAIR_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(luma_kernel_pair_fn)));
    CmdArgs::add_arg_kernel(&mut build, (1,));
    CmdArgs::add_arg_scale(&mut build);
    CmdArgs::add_arg_kernel_size(&mut build, false);
    CmdArgs::add_arg_px(&mut build, true);
    CmdArgs::add_arg_py(&mut build, true);
    CmdArgs::add_arg_angle(&mut build);
    CmdArgs::add_arg_flags(&mut build);

    build
}

//fi luma_kernel_pair_fn
fn luma_kernel_pair_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let img2 = cmd_args.get_read_image(1)?;
    let img1 = cmd_args.get_read_image(0)?;

    let ws = cmd_args.kernel_size();
    let scale = cmd_args.scale();
    let angle = cmd_args.angle();
    let xy = cmd_args.pxy();
    let flags = cmd_args.flags();
    let kernels_to_apply = cmd_args.kernels();

    eprintln!(
        "Read initial image, size is {:?} (max pixels in kernel is 4M) : xy {xy:?}",
        img1.size()
    );

    let (src_w, src_h) = img1.size();
    let src_npix = src_w as usize * src_h as usize;
    let src_max = (4 * 1024 * 1024).min(src_npix);
    let src_img_scale =
        Some(((src_max as f32 / src_npix as f32).sqrt() * src_w as f32).floor() as usize);
    let (src_w, src_h, mut src_img) = img1.as_vec_gray_f32(src_img_scale);
    eprintln!(
        "Using size {src_w}, {src_h} ({:.2} Mpx)",
        (src_w * src_h) as f32 / 1024.0 / 1024.0
    );
    {
        let img = ImageGray16::of_vec_f32(src_w, src_h, src_img.clone(), 1.0);
        img.write("src_kernel.png")?;
    }

    let (dst_w, dst_h) = img1.size();
    let dst_npix = dst_w as usize * dst_h as usize;
    let dst_max = (4 * 1024 * 1024).min(dst_npix);
    let dst_img_scale =
        Some(((dst_max as f32 / dst_npix as f32).sqrt() * dst_w as f32).floor() as usize);
    let (dst_w, dst_h, mut img_data) = img2.as_vec_gray_f32(dst_img_scale);
    eprintln!(
        "Other size {dst_w}, {dst_h} ({:.2} Mpx)",
        (dst_w * dst_h) as f32 / 1024.0 / 1024.0
    );
    {
        let img = ImageGray16::of_vec_f32(dst_w, dst_h, img_data.clone(), 1.0);
        img.write("dst_kernel.png")?;
    }

    let kernels = Kernels::new();

    if flags & 1 != 0 {
        eprintln!("Applying window_var_scaled to first");
        let args: KernelArgs = (src_w, src_h).into();
        let args = args.with_size(4);
        kernels.run_shader(
            "window_var_scaled",
            &args,
            src_w * src_h,
            None,
            src_img.as_mut_slice(),
        )?;
        eprintln!("Applying window_var_scaled to second");
        let args: KernelArgs = (dst_w, dst_h).into();
        let args = args.with_size(4);
        kernels.run_shader(
            "window_var_scaled",
            &args,
            dst_w * dst_h,
            None,
            img_data.as_mut_slice(),
        )?;
    }

    {
        let img = ImageGray16::of_vec_f32(dst_w, dst_h, img_data.clone(), 1.0);
        img.write("dst2_kernel.png")?;
    }
    let args: KernelArgs = (dst_w, dst_h).into();
    let args = args.with_size(ws as usize);
    let args = args.with_scale(scale as f32);
    let args = args.with_angle(angle.to_radians() as f32);
    let args = args.with_xy(xy);
    let args = args.with_src((src_w, src_h));

    for k in kernels_to_apply {
        {
            let img = ImageGray16::of_vec_f32(dst_w, dst_h, img_data.clone(), 1.0);
            img.write("dst3_kernel.png")?;
        }
        eprintln!("Applying {k} with {args:?}");
        {
            let img = ImageGray16::of_vec_f32(src_w, src_h, src_img.clone(), 1.0);
            img.write("before_src_kernel.png")?;
        }
        {
            let img = ImageGray16::of_vec_f32(dst_w, dst_h, img_data.clone(), 1.0);
            img.write("before_dst_kernel.png")?;
        }
        kernels.run_shader(
            &k,
            &args,
            dst_w * dst_h,
            Some(src_img.as_slice()),
            img_data.as_mut_slice(),
        )?;
    }

    if flags & 2 != 0 {
        let pts =
            kernels.find_best_n_above_value((dst_w, dst_h), img_data.as_mut_slice(), 500, 0.7, 64);
        eprintln!("Points {pts:?}");
    }

    eprintln!("Completed kernel");
    let img = ImageGray16::of_vec_f32(dst_w, dst_h, img_data, 1.0);
    eprintln!("Created luma image");

    if let Some(write_filename) = cmd_args.write_img() {
        img.write(write_filename)?;
        eprintln!("Image written");
    } else {
        eprintln!("Image not written as no output image provided");
    }
    Ok("".into())
}

//a Image_process command
//fp image_process
pub fn image_process_cmd() -> CommandBuilder<CmdArgs> {
    let command =
        Command::new("image_process").about("Perform image processing, such as applying kernels");

    let mut build = CommandBuilder::new(command, None);

    CmdArgs::add_arg_read_image(&mut build, (1,));
    CmdArgs::add_arg_write_image(&mut build, false);
    CmdArgs::add_arg_bg_color(&mut build);

    build.add_subcommand(as_luma_cmd());
    build.add_subcommand(luma_window_cmd());
    build.add_subcommand(luma_kernel_cmd());
    build.add_subcommand(luma_kernel_pair_cmd());

    build
}
