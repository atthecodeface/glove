//a Imports
use std::collections::HashMap;

use clap::Command;
use ic_base::Result;
use ic_camera::polynomial;
use ic_camera::polynomial::CalcPoly;
use ic_cmdline as cmdline_args;
use ic_image::{Color, Image, ImageGray16, ImageRgb8, Region};
use ic_kernel::{KernelArgs, Kernels};

//a Types
//ti SubCmdFn
type SubCmdFn = fn(base_args: BaseArgs, &clap::ArgMatches) -> Result<()>;

//tp BaseArgs
struct BaseArgs {
    images: Vec<ImageRgb8>,
    write_filename: Option<String>,
    bg_color: Option<Color>,
}

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

//tp PolyIntercept
struct PolyIntercept {
    /// Degress of polynomial
    #[allow(dead_code)]
    poly_degree: usize,
    /// True if this is horizontal
    #[allow(dead_code)]
    y_intercept: bool,
    /// Y grid value if this is horizontal; else X grid value
    #[allow(dead_code)]
    intercept: f64,
    /// Polynomial to yield px from X grid value if horiz (else Y)
    px_of_g: Vec<f64>,
    /// Polynomial to yield py from X grid value if horiz (else Y)
    py_of_g: Vec<f64>,
    /// Polynomial to yield X grid value from px if horiz
    ///
    /// Polynomial to yield Y grid value from py if vertical
    g_of_p: Vec<f64>,
}

//ip PolyIntercept
impl PolyIntercept {
    //fi from_pts
    fn from_pts(
        // True if horizontal
        y_intercept: bool,
        // Y spacing if y_intercept is true
        spacing: f64,
        // Y value if y_intercept is true
        n: isize,
        origin: usize,
        pts: &[(f64, f64)],
    ) -> Option<Self> {
        let n = n as f64;
        let (cx, cy) = pts[origin];
        let intercept = {
            if y_intercept {
                cy + n * spacing
            } else {
                cx + n * spacing
            }
        };
        let (cx, cy) = {
            if y_intercept {
                (cx, intercept)
            } else {
                (intercept, cy)
            }
        };
        let intercept_pts = find_axis_pts(y_intercept, spacing / 3.0, (cx, cy), pts)?;
        let mut g = vec![];
        // if y_intercept then the x values will be diverse and the y values similar
        let mut x = vec![];
        let mut y = vec![];
        for (pt, dxy) in &intercept_pts {
            x.push(pts[*pt].0);
            y.push(pts[*pt].1);
            g.push(((*dxy / spacing) + 0.5).floor());
        }
        let poly_degree = 5;
        let px_of_g = polynomial::min_squares_dyn(poly_degree, &g, &x);
        let py_of_g = polynomial::min_squares_dyn(poly_degree, &g, &y);
        let g_of_p = {
            if y_intercept {
                polynomial::min_squares_dyn(poly_degree, &x, &g)
            } else {
                polynomial::min_squares_dyn(poly_degree, &y, &g)
            }
        };
        Some(Self {
            poly_degree,
            y_intercept,
            intercept,
            px_of_g,
            py_of_g,
            g_of_p,
        })
    }
}

//tp PolyGrid
struct PolyGrid {
    x_polys: HashMap<isize, PolyIntercept>,
    y_polys: HashMap<isize, PolyIntercept>,
}
impl PolyGrid {
    fn of_pts(
        grid_size: (usize, usize),
        spacings: (f64, f64),
        origin: usize,
        pts: &[(f64, f64)],
    ) -> Self {
        let mut x_polys = HashMap::new();
        let mut y_polys = HashMap::new();
        for i in 0..grid_size.1 {
            let gy = i as isize;
            if let Some(pi) = PolyIntercept::from_pts(true, spacings.0, gy, origin, pts) {
                x_polys.insert(gy, pi);
            }
            if i > 0 {
                if let Some(pi) = PolyIntercept::from_pts(true, spacings.0, -gy, origin, pts) {
                    x_polys.insert(-gy, pi);
                }
            }
        }
        for i in 0..grid_size.0 {
            let gx = i as isize;
            if let Some(pi) = PolyIntercept::from_pts(false, spacings.0, gx, origin, pts) {
                y_polys.insert(gx, pi);
            }
            if i > 0 {
                if let Some(pi) = PolyIntercept::from_pts(false, spacings.0, -gx, origin, pts) {
                    y_polys.insert(-gx, pi);
                }
            }
        }
        Self { x_polys, y_polys }
    }
    fn focus(&self, (w, h): (f64, f64)) -> (f64, f64) {
        let gx_c = self.x_polys[&0].g_of_p.calc(w / 2.0);
        let gy_c = self.y_polys[&0].g_of_p.calc(h / 2.0);
        (gx_c, gy_c)
    }
    fn pxy_of_gxy(&self, gxy: (isize, isize)) -> Option<(f64, f64)> {
        if let Some(xpi) = self.x_polys.get(&gxy.1) {
            if let Some(ypi) = self.y_polys.get(&gxy.0) {
                let gx = gxy.0 as f64;
                let gy = gxy.1 as f64;
                let px_x = xpi.px_of_g.calc(gx);
                let px_y = xpi.py_of_g.calc(gx);
                let py_x = ypi.px_of_g.calc(gy);
                let py_y = ypi.py_of_g.calc(gy);
                let px = (px_x + py_x) / 2.0;
                let py = (px_y + py_y) / 2.0;
                Some((px, py))
            } else {
                None
            }
        } else {
            None
        }
    }
    fn as_grid(&self) -> Vec<(isize, isize, usize, usize)> {
        let mut grid = vec![];
        for gx in self.y_polys.keys() {
            for gy in self.x_polys.keys() {
                if let Some(pxy) = self.pxy_of_gxy((*gx, *gy)) {
                    let gx = (*gx) * 10;
                    let gy = (*gy) * 10;
                    grid.push((gx, gy, pxy.0 as usize, pxy.1 as usize));
                }
            }
        }
        grid
    }
}

//a Calibrate
//hi FIND_REGIONS_LONG_HELP
const FIND_REGIONS_LONG_HELP: &str = "\
This treats the image file read in as a set of non-background color
regions.

A region is a contiguous set of non-background pixels. The
centre-of-gravity of each region is determined, and the result is a
JSON file containing a list of pairs of floats of those cogs.";

//fi find_regions_cmd
fn find_regions_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("find_regions")
        .about("Read image and find regions")
        .long_about(FIND_REGIONS_LONG_HELP);

    (cmd, find_regions_fn)
}

//fi find_regions_fn
fn find_regions_fn(mut base_args: BaseArgs, _matches: &clap::ArgMatches) -> Result<()> {
    let img = &mut base_args.images[0];
    let _bg = base_args.bg_color.unwrap_or(Color::black());
    let min_brightness = 0.6;
    // let regions = Region::regions_of_image(img, &|c| !c.color_eq(&bg));
    let regions =
        Region::regions_of_image(&img, &|c| c.brightness() > min_brightness, &|c0, c1| {
            (c0.brightness() - c1.brightness()).abs() < 0.1
        });
    let cogs: Vec<(Color, (f64, f64))> =
        regions.into_iter().map(|x| (x.color(), x.cog())).collect();

    if let Some(write_filename) = &base_args.write_filename {
        let b = [0_u8, 0, 0, 255].into();
        let (w, h) = img.size();
        for y in 0..h {
            for x in 0..w {
                img.put(x, y, &b);
            }
        }

        for (c, (px, py)) in &cogs {
            img.draw_cross([*px as f64, *py as f64].into(), 5.0, c);
        }
        img.write(write_filename)?;
    }
    let mut cogs: Vec<_> = cogs.into_iter().map(|(_, (x, y))| (x, y)).collect();
    let minx = cogs.iter().fold(0.0_f64, |a, x| a.min(x.0));
    let maxx = cogs.iter().fold(0.0_f64, |a, x| a.max(x.0));
    let miny = cogs.iter().fold(0.0_f64, |a, x| a.min(x.1));
    let maxy = cogs.iter().fold(0.0_f64, |a, x| a.max(x.1));
    let ax = (minx + maxx) / 2.0;
    let ay = (miny + maxy) / 2.0;
    eprintln!("{ax}, {ay}");
    cogs.sort_by_key(|xy| ((xy.0 - ax).powi(2) + (xy.1 - ay).powi(2)) as usize);
    let cogs: Vec<_> = cogs
        .into_iter()
        .map(|(x, y)| (x as isize, y as isize, 5_usize, 0_usize))
        .collect();
    println!("{}", serde_json::to_string_pretty(&cogs).unwrap());

    Ok(())
}

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

//fi find_grid_points_cmd
fn find_grid_points_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("find_grid_points")
        .about("Read image and find grid points")
        .long_about(FIND_GRID_POINTS_LONG_HELP);
    (cmd, find_grid_points_fn)
}

//fi find_grid_points_fn
fn find_grid_points_fn(mut base_args: BaseArgs, _matches: &clap::ArgMatches) -> Result<()> {
    let img = &mut base_args.images[0];
    let bg = base_args.bg_color.unwrap_or(Color::black());
    let regions = Region::regions_of_image(&img, &|c| !c.color_eq(&bg), &|c0, c1| {
        (c0.brightness() - c1.brightness()).abs() < 0.1
    });
    let cogs: Vec<(f64, f64)> = regions.into_iter().map(|x| x.cog()).collect();
    let (xsz, ysz) = img.size();
    let xsz = xsz as f64;
    let ysz = ysz as f64;
    let origin = find_center_point((xsz / 2.0, ysz / 2.0), &cogs);
    eprintln!("{origin} {:?}", cogs[origin]);
    let x_axis = find_axis_pts(true, 50.0, cogs[origin], &cogs).unwrap();
    eprintln!("{x_axis:?}");
    let x_spacing = spacing_of_coords(&x_axis)?;
    eprintln!("{x_spacing}");
    let y_axis = find_axis_pts(false, 50.0, cogs[origin], &cogs).unwrap();
    eprintln!("{y_axis:?}");
    let y_spacing = spacing_of_coords(&y_axis)?;
    eprintln!("{y_spacing}");

    let grid = PolyGrid::of_pts((15, 10), (x_spacing, y_spacing), origin, &cogs);

    eprintln!("Focus {:?}", grid.focus((xsz, ysz)));
    let mappings = grid.as_grid();

    if let Some(write_filename) = &base_args.write_filename {
        let b = [0_u8, 0, 0, 255].into();
        let (w, h) = img.size();
        for y in 0..h {
            for x in 0..w {
                img.put(x, y, &b);
            }
        }

        let c = &[255, 0, 0, 255].into();
        for (_gx, _gy, px, py) in &mappings {
            img.draw_cross([*px as f64, *py as f64].into(), 5.0, c);
        }
        img.write(write_filename)?;
    }

    println!("{}", serde_json::to_string_pretty(&mappings).unwrap());
    Ok(())
}

//a Luma
//hi AS_LUMA_LONG_HELP
const AS_LUMA_LONG_HELP: &str = "\
Generate a 16-bit luma image
";

//fi as_luma_cmd
fn as_luma_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("as_luma")
        .about("Generate a 16-bit luma image")
        .long_about(AS_LUMA_LONG_HELP);
    (cmd, as_luma_fn)
}

//fi as_luma_fn
fn as_luma_fn(base_args: BaseArgs, _matches: &clap::ArgMatches) -> Result<()> {
    let img = &base_args.images[0];
    eprintln!("Read initial image, size is {:?}", img.size());
    let (w, h, img_data) = img.as_vec_gray_f32(None);
    let img = ImageGray16::of_vec_f32(w, h, img_data, 1.0);
    eprintln!("Created luma image");

    if let Some(write_filename) = &base_args.write_filename {
        img.write(write_filename)?;
        eprintln!("Image written");
    } else {
        eprintln!("Image not written as no output image provided");
    }
    Ok(())
}

//hi LUMA_WINDOW_LONG_HELP
const LUMA_WINDOW_LONG_HELP: &str = "\
Analyze an image in luma space using a window
";

//fi luma_window_cmd
fn luma_window_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("luma_window")
        .about("Analyze an image in luma space using a window")
        .long_about(LUMA_WINDOW_LONG_HELP);
    (cmd, luma_window_fn)
}

//fi luma_window_fn
fn luma_window_fn(base_args: BaseArgs, _matches: &clap::ArgMatches) -> Result<()> {
    let img = &base_args.images[0];
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

    if let Some(write_filename) = &base_args.write_filename {
        img.write(write_filename)?;
        eprintln!("Image written");
    } else {
        eprintln!("Image not written as no output image provided");
    }
    Ok(())
}

//hi LUMA_KERNEL_LONG_HELP
const LUMA_KERNEL_LONG_HELP: &str = "\
Analyze kernels to an image in luma space

Convert the image to a 16-bit luma

Apply a number of kernels (with a single set of size, scale etc arguments)

Output the image as a 16-bit luma image (so the kernel output should be in the range 0.0 to 1.0)
";

//fi luma_kernel_cmd
fn luma_kernel_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("luma_kernel")
        .about("Apply kernels to an image in luma space")
        .long_about(LUMA_KERNEL_LONG_HELP);
    let cmd = cmdline_args::kernels::add_kernel_arg(cmd, true);
    let cmd = cmdline_args::kernels::add_scale_arg(cmd, false);
    let cmd = cmdline_args::kernels::add_size_arg(cmd, false);
    let cmd = cmdline_args::kernels::add_xy_arg(cmd, false);
    (cmd, luma_kernel_fn)
}

//fi luma_kernel_fn
fn luma_kernel_fn(base_args: BaseArgs, matches: &clap::ArgMatches) -> Result<()> {
    let img = &base_args.images[0];
    let ws = cmdline_args::kernels::get_size(matches)?;
    let scale = cmdline_args::kernels::get_scale(matches)?;
    let xy = cmdline_args::kernels::get_xy(matches)?;
    let kernels_to_apply = cmdline_args::kernels::get_kernels(matches)?;
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
    let args = args.with_scale(scale);
    let args = args.with_xy(xy);

    for k in kernels_to_apply {
        kernels.run_shader(&k, &args, w * h, None, img_data.as_mut_slice())?;
    }

    eprintln!("Completed kernel");
    let img = ImageGray16::of_vec_f32(w, h, img_data, 1.0);
    eprintln!("Created luma image");

    if let Some(write_filename) = &base_args.write_filename {
        img.write(write_filename)?;
        eprintln!("Image written");
    } else {
        eprintln!("Image not written as no output image provided");
    }
    Ok(())
}

//hi LUMA_KERNEL_PAIR_LONG_HELP
const LUMA_KERNEL_PAIR_LONG_HELP: &str = "\
Apply kernels to a pair of images in luma space

Convert the images to 16-bit luma

Apply a number of kernels (with a single set of size, scale etc arguments)

Output the image as a 16-bit luma image (so the kernel output should be in the range 0.0 to 1.0)
";

//fi luma_kernel_pair_cmd
fn luma_kernel_pair_cmd() -> (Command, SubCmdFn) {
    let cmd = Command::new("luma_kernel_pair")
        .about("Apply kernels to a pair of images in luma space")
        .long_about(LUMA_KERNEL_PAIR_LONG_HELP);
    let cmd = cmdline_args::kernels::add_kernel_arg(cmd, true);
    let cmd = cmdline_args::kernels::add_scale_arg(cmd, false);
    let cmd = cmdline_args::kernels::add_angle_arg(cmd, false);
    let cmd = cmdline_args::kernels::add_size_arg(cmd, false);
    let cmd = cmdline_args::kernels::add_xy_arg(cmd, false);
    let cmd = cmdline_args::kernels::add_flags_arg(cmd, false);
    (cmd, luma_kernel_pair_fn)
}

//fi luma_kernel_pair_fn
fn luma_kernel_pair_fn(base_args: BaseArgs, matches: &clap::ArgMatches) -> Result<()> {
    if base_args.images.len() != 2 {
        return Err("Two 'read' images are required for a kernel on a pair of images".into());
    }
    let ws = cmdline_args::kernels::get_size(matches)?;
    let scale = cmdline_args::kernels::get_scale(matches)?;
    let angle = cmdline_args::kernels::get_angle(matches)?;
    let xy = cmdline_args::kernels::get_xy(matches)?;
    let flags = cmdline_args::kernels::get_flags(matches)?;
    let kernels_to_apply = cmdline_args::kernels::get_kernels(matches)?;

    let img = &base_args.images[0];
    eprintln!(
        "Read initial image, size is {:?} (max pixels in kernel is 4M) : xy {xy:?}",
        img.size()
    );

    let (src_w, src_h) = img.size();
    let src_npix = src_w as usize * src_h as usize;
    let src_max = (4 * 1024 * 1024).min(src_npix);
    let src_img_scale =
        Some(((src_max as f32 / src_npix as f32).sqrt() * src_w as f32).floor() as usize);
    let (src_w, src_h, mut src_img) = img.as_vec_gray_f32(src_img_scale);
    eprintln!(
        "Using size {src_w}, {src_h} ({:.2} Mpx)",
        (src_w * src_h) as f32 / 1024.0 / 1024.0
    );
    {
        let img = ImageGray16::of_vec_f32(src_w, src_h, src_img.clone(), 1.0);
        img.write("src_kernel.png")?;
    }

    let (dst_w, dst_h) = img.size();
    let dst_npix = dst_w as usize * dst_h as usize;
    let dst_max = (4 * 1024 * 1024).min(dst_npix);
    let dst_img_scale =
        Some(((dst_max as f32 / dst_npix as f32).sqrt() * dst_w as f32).floor() as usize);
    let (dst_w, dst_h, mut img_data) = base_args.images[1].as_vec_gray_f32(dst_img_scale);
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
    let args = args.with_scale(scale);
    let args = args.with_angle(angle.to_radians());
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

    if let Some(write_filename) = &base_args.write_filename {
        img.write(write_filename)?;
        eprintln!("Image written");
    } else {
        eprintln!("Image not written as no output image provided");
    }
    Ok(())
}

//a Main
//fi main
fn main() -> Result<()> {
    let cmd = Command::new("image_analyze")
        .about("Image analysis tool")
        .version("0.1.0")
        .subcommand_required(true);
    let cmd = cmdline_args::image::add_image_read_arg(cmd, true);
    let cmd = cmdline_args::image::add_image_write_arg(cmd, false);
    let cmd = cmdline_args::image::add_bg_color_arg(cmd, false);

    let mut subcmds: HashMap<String, SubCmdFn> = HashMap::new();
    let mut cmd = cmd;
    for (c, f) in [
        find_regions_cmd(),
        find_grid_points_cmd(),
        as_luma_cmd(),
        luma_window_cmd(),
        luma_kernel_cmd(),
        luma_kernel_pair_cmd(),
    ] {
        subcmds.insert(c.get_name().into(), f);
        cmd = cmd.subcommand(c);
    }
    let cmd = cmd;

    let matches = cmd.get_matches();
    let images = cmdline_args::image::get_image_read_all(&matches)?;
    let write_filename = cmdline_args::image::get_opt_image_write_filename(&matches)?;
    let bg_color = cmdline_args::image::get_opt_bg_color(&matches)?;
    let base_args = BaseArgs {
        images,
        write_filename,
        bg_color,
    };

    let (subcommand, submatches) = matches.subcommand().unwrap();
    for (name, sub_cmd_fn) in subcmds {
        if subcommand == name {
            return sub_cmd_fn(base_args, submatches);
        }
    }
    unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`");
}
