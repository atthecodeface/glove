//a Imports
use std::collections::HashMap;

use clap::Command;
use image_calibrate::polynomial;
use image_calibrate::polynomial::CalcPoly;
use image_calibrate::{cmdline_args, Color, Image, ImageBuffer, Region};

//a Types
//ti SubCmdFn
type SubCmdFn = fn(base_args: BaseArgs, &clap::ArgMatches) -> Result<(), String>;

//tp BaseArgs
struct BaseArgs {
    image: ImageBuffer,
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
fn spacing_of_coords(pts: &[(usize, f64)]) -> Result<f64, String> {
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
        let Some(intercept_pts) = find_axis_pts(y_intercept, spacing / 3.0, (cx, cy), pts) else {
            return None;
        };
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
fn find_regions_fn(base_args: BaseArgs, _matches: &clap::ArgMatches) -> Result<(), String> {
    let img = base_args.image;
    let bg = base_args.bg_color.unwrap_or(Color::black());
    let regions = Region::regions_of_image(&img, &|c| !c.color_eq(&bg));
    let cogs: Vec<(Color, (f64, f64))> =
        regions.into_iter().map(|x| (x.color(), x.cog())).collect();
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
fn find_grid_points_fn(base_args: BaseArgs, _matches: &clap::ArgMatches) -> Result<(), String> {
    let mut img = base_args.image;
    let bg = base_args.bg_color.unwrap_or(Color::black());
    let regions = Region::regions_of_image(&img, &|c| !c.color_eq(&bg));
    let cogs: Vec<(f64, f64)> = regions.into_iter().map(|x| x.cog()).collect();
    let (xsz, ysz) = img.size();
    let xsz = xsz as f64;
    let ysz = ysz as f64;
    let origin = find_center_point((xsz / 2.0, ysz / 2.0), &cogs);
    eprintln!("{origin} {:?}", cogs[origin]);
    let x_axis = find_axis_pts(true, 50.0, cogs[origin], &cogs).unwrap();
    eprintln!("{:?}", x_axis);
    let x_spacing = spacing_of_coords(&x_axis)?;
    eprintln!("{x_spacing}");
    let y_axis = find_axis_pts(false, 50.0, cogs[origin], &cogs).unwrap();
    eprintln!("{:?}", y_axis);
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

//a Main
//fi print_err
fn print_err(s: String) -> String {
    eprintln!("{}", s);
    s
}

//fi main
fn main() -> Result<(), String> {
    let cmd = Command::new("image_analyze")
        .about("Image analysis tool")
        .version("0.1.0")
        .subcommand_required(true);
    let cmd = cmdline_args::add_image_read_arg(cmd, true);
    let cmd = cmdline_args::add_image_write_arg(cmd, false);
    let cmd = cmdline_args::add_bg_color_arg(cmd, false);

    let mut subcmds: HashMap<String, SubCmdFn> = HashMap::new();
    let mut cmd = cmd;
    for (c, f) in [find_regions_cmd(), find_grid_points_cmd()] {
        subcmds.insert(c.get_name().into(), f);
        cmd = cmd.subcommand(c);
    }
    let cmd = cmd;

    let matches = cmd.get_matches();
    let image = cmdline_args::get_image_read(&matches)?;
    let write_filename = cmdline_args::get_opt_image_write_filename(&matches)?;
    let bg_color = cmdline_args::get_opt_bg_color(&matches)?;
    let base_args = BaseArgs {
        image,
        write_filename,
        bg_color,
    };

    let (subcommand, submatches) = matches.subcommand().unwrap();
    for (name, sub_cmd_fn) in subcmds {
        if subcommand == name {
            return sub_cmd_fn(base_args, submatches).map_err(print_err);
        }
    }
    unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`");
}
