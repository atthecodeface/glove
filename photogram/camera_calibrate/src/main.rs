//a Documentation
//! Test of camera calibration
//!
//! This is trying to calibrate to a grid
//!
//! The example grid was captured on a Canon 5D mark IV, with a 50mm lens focuses on 'infinity'
//!
//! The camera is face-on to the grid (which is graph paper); the
//! approximate intersections of 550 grid lines was capture as sensor pixel
//! coordinates XY and mm XY pairings. The grid is assumed to be at Z=0.
//!
//! # Calibration first step - locate the camera
//!
//! The first step is to locate the camera, given the camera body, an
//! uncalibrated lens (i.e. something with an identity sensor to world
//! mapping), and with a given focusing distance (known ideally!) for
//! the image and hence pairings.
//!
//! The first 'N' pairings are used to create a
//! ModelLineSet, which is a set of ModelLines (between grid points,
//! hence all in the plane Z=0) and the angles subtended by the
//! camera, given by the sensor pixel coordinates through the
//! camera/lens model from the database (i.e. this includes the lens
//! mapping)
//!
//! Note that this does not assume the orientation of position of the
//! camera; it purely uses the absolute pixel XY to relative pixel XY
//! to TanXTanY to RollYaw through the lens mapping to a new RollYaw
//! to a TanXTanY in model space to a unit direction vector.
//!
//! From this ModelLineSet a position in space is determined, using
//! the 'million possible points on a ModelLinetSubtended surface)
//! approach.
//!
//! This camera position is then optimized further by delta
//! adjustments in the ModelLineSet.
//!
//! # Orientation of camera given a location
//!
//! From this 'known good position' the best orientation can be
//! determined, by creating quaternion orientations for every pair of
//! pairings (or for the first N pairings) by:
//!
//!   1. Find the unit direction from the camera to both of the model points (A, B)
//!
//!   2. Find the the unit direction for the camera on it sensor (from the pairing)
//!
//!   3. Generate a quaternion qm that rotates model point direction A to the vector (0,0,1)
//!
//!   4. Generate a quaternion qc that rotates camera point direction A to the vector (0,0,1)
//!
//!   5. Apply qm to model point direction B to yield dj_m
//!
//!   6. Apply qc' to camera point direction B to yield dj_c
//!
//!   7. Note that dj_m and dj_c should have the same *yaw* but a different *roll*
//!
//!   8. Determine the roll required to map dj_m to dj_c
//!
//!   9. Generate quaternion qz which is the rotation around Z for the roll
//!
//!   10. Generate quaternion q = qm.qz.qc
//!
//!   11. Note that q transforms model point direction A to camera point direction A
//!
//!   12. Note that q transforms model point direction B to camera point direction B (if the yaws were identical)
//!
//!   13. Note hence that q is the orientation of a camera that matches the view of model points A and B
//!
//! The value 'q' is inaccurate if the *yaw* values are different -
//! i.e. if the angle subtended by the line between the two points on
//! the camera does not match the angle subtended by the line between
//! the two points in model space as seen by the camera at its given location.
//!
//! The value of 'q' for *every* pair of pairings (A to B, and also B
//! to A) is generated, and an average of these quaternions is used as
//! the orientation of the camera.
//!
//! When initially generating a lens calibration it is normal to use
//! the same first 'N' points as the locating of the camera.
//!
//! # Generating a test image
//!
//! An image can be generated that overlays on the source image the
//! pairings as provided, and mapped through the lens.
//!
//! This draws *green* crosses for each (x,y,z) in
//! the description JSON mappings, and *red* crosses for the pxy of
//! each of those points in the description JSON mappings.
//!
//! If the calibration is very good then the green and red crosses
//! will overlap; at this stage, pre-lens calibration, the crosses
//! should correlate better at the center of the image than towards
//! the edge.
//!
//! # Plotting Yaw and Roll
//!
//! Given the location and orientation of the camera the unit
//! direction vector to every model point from the camera can be
//! determined, and converted to a *roll* and *yaw*. The corresponding
//! camera sensor direction can be determined, and presented also as a
//! *roll* and *yaw*.
//!
//! Graphs of camera yaw versus model yaw can be produced; if no lens
//! mapping had been used the this should be approximately a single
//! curve that is the polynomial for the lens (mapping yaw in camera
//! to yaw in model space). The actual graph drawn by 'yaw_plot' is
//! the *relative* error in yaw versus the camera yaw, so that when a
//! lens is calibrated it should be a line along the X axis.
//!
//! For the yaw, we plot *four* graphs, one for each quadrant.
//!
//! The roll plot plots the error in the roll agains the error in the
//! yaw, for each point.
//!
//! # Calibrating the lens
//!
//! For each pairing the sensor roll (given by the pixel XY position
//! of the pairing) and the roll for the world position given the
//! camera location and orientation can be generated; these *ought* to
//! be equal for a spherical lends, and should only really be impacted
//! by the camera location/orientation (and possibly the centring of
//! the lens in the sensor).
//!
//! For each pairing the sensor yaw (given by the pixel XY position of
//! the pairing) and the yaw for the world position given the camera
//! location and orientation can be generated; this should be a
//! (potentially nonlinear) mapping (that is independent of the roll,
//! for example). The calibration is this mapping (and its inverse),
//! and a polynomial can be fitted to the data generated from the
//! pairings.
//!
//! Hence given the first 'N' pairings, a pair of polynomials (one
//! camera-to-model, the other the inverse) are generated
//!
//! The process to calibrate the camera is thus to:
//!
//!  1. Start with a linear lens mapping polynomial
//!
//!  2. Reset the centre of the lens (to the middle of the sensor)
//!
//!  3. Locate the camera using a few pairings
//!
//!  3. Orient the camera using some or all of the pairings
//!
//!  4. Generate a first lens calibration using all of the pairings.
//!
//!  5. Relocate the camera with the newly calibrated lens
//!
//!  6. Reorient the camera with the newly calibrated lens and its new location
//!
//!  7. Generate an improved lens calibration using all of the pairings.
//!
//!  8. Generate a yaw plot to check the error. Potentially the
//!     locate/orient/calibrate can be rerun, to improve the quality of
//!     the calibration.
//!
//! # Using the tool with a grid
//!
//! If you start with a photo of a grid, then a 'mappings' for each of
//! the grid positions should be gnerated - this is (x,y,0, px, py).
//!
//! Then a CameraPolynomialCalibrateDesc can be generated, which is a
//! basic camera description plus the mappings. This provides the
//! camera body, lens, and focus distance for the image; plus position
//! and orientation, which will be ignored here.
//!
//! With this description use:
//!
//!   camera_calibrate (--db ...) locate -c <desc.json> > <located_desc.json>
//!
//! This will use some points (argh!) to locate the camera as best as possible
//!
//! With a *located* camera, the orientation can be determined.
//!
//!   camera_calibrate (--db ...) orient -c <located_desc.json> > <oriented_desc.json>
//!
//! This will use some points (argh!) to orient the camera as best as possible, given its location
//!
//! Now the lens can be calibrated
//!
//! # Calibrating a lens
//!
//! A lens is calibrated using a set of points in 3D and the positions those map to on a camera, given the camera has been optimally located and oriented.
//!
//! This will also generate an SVG file with the plot of camera Yaw
//! versus world Yaw, plus the polynomials describing this
//! (sensor-to-world, and world-to-sensor):
//!
//!   camera_calibrate (--db ...) lens_calibrate -c <desc.json> > <plot.svg>
//!
//! The lens calibration can be copied to the camera_db.json file if
//! required. Bear in mind that this polynomial is relative to the
//! *current* polynomial provided for the lens; the normal process is
//! to run the location, orientation, and calibration with a *linear*
//! lens mapping, then to copy that lens mapping to the database, and
//! rerun to ensure the SVG shows basically straight lines parallel to
//! the X axis.
//!

//a Imports
use std::io::Write;

use clap::{Arg, ArgAction, ArgMatches, Command};
use geo_nd::{quat, Quaternion, Vector};
use star_catalog::{Catalog, StarFilter};
use thunderclap::{CommandArgs, CommandBuilder};

use ic_base::json;
use ic_base::{Point3D, Quat, Result, RollYaw, TanXTanY};
use ic_camera::polynomial;
use ic_camera::polynomial::CalcPoly;
use ic_camera::{CalibrationMapping, CameraDatabase, CameraInstance, CameraProjection, LensPolys};
use ic_image::{Color, Image, ImagePt, ImageRgb8};
use ic_mapping::{ModelLineSet, NamedPointSet, PointMappingSet};
use ic_stars::StarMapping;

//a Help messages
//hi CAMERA_CALIBRATE_LONG_HELP
const CAMERA_CALIBRATE_LONG_HELP: &str = "\
This provides various tools to help calibrate a lens mapping.

Some of the tools are based on a description of a photograph of a
regular grid (such as graph paper); alternatively, a photograph of
stars may be used as a starting point (replacing *some* of the tools
provided here).

The description of a photograph of a regular grid can be obtained
somewhat automatically using the 'image_analyze' tool.

Using a grid, the approach is first to *locate* the camera.

With a located camera and a grid, or with a camera ar the origin and a
star description, the orientation of the camera can be determined.

With an orientation, the lens calibration can be determined, and
verified with images.";

//hi LOCATE_LONG_HELP
const LOCATE_LONG_HELP: &str = "\
Determine the 'best' location for a camera/lens from a mapping
description file, ignoring the given camera position and orientation.

The mapping is a list of (x,y,z, px,py); this indicates that the point
in the 'world' at (x,y,z) was seen on the camera sensor at absolute
camera sensor position (px,py).

The algorithm uses is to determine for every pair of selected mappings
the angle subtended as seen by the camera/lens - based on the (px,py)
values - including using any lens mapping the camera has. Each such
pair corresponds to a line in space (between the two (x,y,z) for the
pair); so there is then a set of (line, angle subtended) for each pair
of mappings. Any one such mapping describes a surface in the world
space from where the line would subtend such an angle. Two such
mappings thus describe a line in world space (at best), and three a
point in world space. Hence three or more pairs can be used to
determine a position in space. The 'best' position in space is deemed
to be that point where the sum of the absolute errors in the angles
for each line subtended is minimized.";

//hi ORIENT_LONG_HELP
const ORIENT_LONG_HELP: &str = "\
Determine the 'best' orientation for a camera/lens from a mapping
description file, ignoring the orientation specified in the
description.

The mapping is a list of (x,y,z, px,py); this indicates that the point
in the 'world' at (x,y,z) was seen on the camera sensor at absolute
camera sensor position (px,py).

For every mapping in the file, given the camera position, a direction
vector (dx,dy,dz) can be generated for that mapping - and this
presumably corresponds to (px,py) for that mapping, which in turn
describes some camera-relative direction (dpx,dpy,dpz).

Hence (given the camera position) we have two lists of directions,
which *ought* to map through an orientation mapping (an arbitrary
rotation matrix in 3D, or a unit quaternion). For any one mapping
there is a quaternion (q0) that maps (dx,dy,dz) to the Z axis, and
another quaternion that maps the Z axis to (dpx,dpy,dpz) (q1); if we
take a second mapping and apply q0 to its (dx,dy,dz), we can apply
*some* rotation around the Z axis (qz), and the apply q1', and we
should end up at its (dpx, dpy, dpz); this combination q0.qz.q1c is a
good best effort for this pair of mappings.

The tool generates all N(N-1) such mapping quaternions for every pair
of mappings, and then determines the average quaternion; this is the 'best' orientation.";

//hi LENS_CALIBRATE_LONG_HELP
const LENS_CALIBRATE_LONG_HELP: &str = "\
Using a mappings description determine the polynomial of best fit to
map the image Yaw to the world Yaw

The mapping is a list of (x,y,z, px,py); this indicates that the point
in the 'world' at (x,y,z) was seen on the camera sensor at absolute
camera sensor position (px,py).

Given a camera position and orientation every mapping has a direction
in both 'world' space (relative to the camera axis) and in 'sensor'
space (relative to the center of the sensor); such directions can be
encoded as a roll and yaw - that is, the angle that the direction is
'away' from the axis of view; and the angle that the direction is
'around' the clock. For example, a direction vector could be described
as 30 degrees off straight-ahead, in the direction of '2' on a clock
(which would be 60 degrees clockwise from the vertical). The first of
these is the Yaw, the second the Roll.

A *spherical* camera lens mapping is a function of Yaw in world space
to Yaw in sensor space - Roll is not important. This tool therefore
generates the two Yaw values (world and sensor) for all of the mapping
points, given the camera position and orientation, and it generates a
graph and a polynomial of best fit (with the extra assertion that the
point (0,0) is on the Yaw/Yaw graph).

Actually two polynomials are generated - one forward (wts) and one
backward (stw); these should be used in a camera_db JSON file.

";

//hi YAW_PLOT_LONG_HELP
const YAW_PLOT_LONG_HELP: &str = "\
The plot that is generated is an SVG file showing Yaw/Yaw-1 - bear in
mind that any lens mapping specified for the camera in the database is
used, so that a perfectly calibrated camera/lens with a perfect
mapping file will have straight lines on the X axis. Furthermore,
there are *four* graphs overlaid, using different colors - one for
each quadrant of the camera sensor; also the polynomial of best fit is
plotted too.
";

//hi ROLL_PLOT_LONG_HELP
const ROLL_PLOT_LONG_HELP: &str = "\
Generate a plot for all the mappings of model roll versus world roll
";

//hi GRID_IMAGE_LONG_HELP
const GRID_IMAGE_LONG_HELP: &str = "\
This tool uses the provided camera description and mappings, and
overlays an image with *red* crosses showing the specified coordinates of
each mapping and the derived (i.e. post-camera/lens mapping) positions
of those mappings with *green* crosses.

It also draws black crosses for a range of (x,y,0) values.";

//hi STAR_LONG_HELP
const STAR_LONG_HELP: &str = "\
This set of commands allows for calibrating a lens using a photograph taken of stars.";

//hi STAR_FIND_STARS_LONG_HELP
const STAR_FIND_STARS_LONG_HELP: &str = "\

Using the camera body, lens, focus distance, and current lens
calibration, find an absolute orientation of an image given two
triangles of star PXY locations in the mapping file.

Two triangles on the image are determined from the mapping - the first
three stars with '1' as the 'magnitude' (second) element, and the
first three stars with '2' for that element.

The PXY of these star locations are mapped to world directions based
on the camera body and lens mapping; the orientation of the camera is
ignored.

For each star triangle, the three angles between these world
directions are calculated; this yields a pair of star triangle angles.

The star catalog is searched for triangles of stars that match these
angles, yielding candidate *actual* star triangles.

For each of these candidate triangles there is a camera orientation
that would map those *actual* star triangles onto the sensor (or at
least, an approximate orientation). For the actual triangles that the
image was taken of, these two orientations should be identical (with
some margin of error, obviously). One way to compare orientations is
to apply the reverse of one to the other, and check to see if it is an
identity orientation.

So the orientation for every pair of candidate triangles are combined
to determine how far off they are from the 'identity'; as orientations
are handled as quaternions, this implies multiplying one by the
conjugate of the other, and comparing the absolute value of the real
value of the resultant quaternion to 1.0; the closer it is, the
smaller the difference between the candidate triangles. This real
value is effectively the cos of the angle of rotation required to
describe the combined mapping - and an identity mapping as no angle of
rotation, and so is 1.0. For close matches (angle x close to 0) the cosine is approximately 1-x^2/2.

  ";

//a Types
//a CmdArgs
//tp CmdArgs
#[derive(Default)]
pub struct CmdArgs {
    verbose: bool,
    cdb: Option<CameraDatabase>,
    camera: CameraInstance,
    write_camera: Option<String>,
    write_mapping: Option<String>,
    write_polys: Option<String>,

    mapping: Option<CalibrationMapping>,
    star_catalog: Option<Box<Catalog>>,
    star_mapping: StarMapping,

    read_img: Vec<String>,
    write_img: Option<String>,

    use_pts: usize,
    yaw_min: f64,
    yaw_max: f64,
    poly_degree: usize,
    closeness: f64,
    yaw_error: f64,
    within: f64,
    brightness: f32,
}

//ip CmdArgs - setters and getters
impl CmdArgs {
    fn get_cdb(&self) -> &CameraDatabase {
        self.cdb.as_ref().unwrap()
    }
    fn set_verbose(&mut self, verbose: bool) -> Result<()> {
        self.verbose = verbose;
        Ok(())
    }
    fn set_cdb(&mut self, cdb: CameraDatabase) -> Result<()> {
        self.cdb = Some(cdb);
        Ok(())
    }
    fn set_camera(&mut self, camera: CameraInstance) -> Result<()> {
        self.camera = camera;
        Ok(())
    }
    fn set_mapping(&mut self, mapping: CalibrationMapping) -> Result<()> {
        self.mapping = Some(mapping);
        Ok(())
    }
    fn set_read_img(&mut self, v: Vec<String>) -> Result<()> {
        self.read_img = v;
        Ok(())
    }
    fn set_write_img(&mut self, s: &str) -> Result<()> {
        self.write_img = Some(s.to_owned());
        Ok(())
    }
    fn set_write_camera(&mut self, s: &str) -> Result<()> {
        self.write_camera = Some(s.to_owned());
        Ok(())
    }
    fn set_write_mapping(&mut self, s: &str) -> Result<()> {
        self.write_mapping = Some(s.to_owned());
        Ok(())
    }
    fn set_write_polys(&mut self, s: &str) -> Result<()> {
        self.write_polys = Some(s.to_owned());
        Ok(())
    }
    fn set_use_pts(&mut self, v: usize) -> Result<()> {
        self.use_pts = thunderclap::bound(v, Some(6), None, |v, _| {
            format!("Number of points ({v}) must be at least six")
        })?;
        Ok(())
    }
    fn set_yaw_min(&mut self, v: f64) -> Result<()> {
        self.yaw_min = thunderclap::bound(v, Some(0.0), Some(90.0), |v, _| {
            format!("Minimum yaw {v} must be in the range 0 to 90")
        })?;
        Ok(())
    }
    fn set_yaw_max(&mut self, v: f64) -> Result<()> {
        self.yaw_max = thunderclap::bound(v, Some(self.yaw_min), Some(90.0), |v, _| {
            format!(
                "Maximum yaw {v} must be between yaw_min ({}) and 90",
                self.yaw_min
            )
        })?;
        Ok(())
    }
    fn set_poly_degree(&mut self, v: usize) -> Result<()> {
        self.poly_degree = thunderclap::bound(v, Some(2), Some(12), |v, _| {
            format!("The polynomial degree {v} should be between 2 and 12 for reliability",)
        })?;
        Ok(())
    }

    fn set_closeness(&mut self, closeness: f64) -> Result<()> {
        self.closeness = closeness;
        Ok(())
    }
    fn set_yaw_error(&mut self, v: f64) -> Result<()> {
        self.yaw_error = thunderclap::bound(v, Some(0.0), Some(1.0), |v, _| {
            format!("The maximum yaw error {v} must be between 0 and 1 degree",)
        })?;
        Ok(())
    }
    fn set_within(&mut self, v: f64) -> Result<()> {
        self.within = thunderclap::bound(v, Some(0.0), Some(90.0), |v, _| {
            format!("The 'within' yaw {v} must be between 0 and 90 degree",)
        })?;
        Ok(())
    }
    fn set_brightness(&mut self, v: f32) -> Result<()> {
        self.brightness = thunderclap::bound(v, Some(0.0), Some(16.0), |v, _| {
            format!("Brightness (magnitude of stars) {v} must be between 0 and 16",)
        })?;
        Ok(())
    }

    fn use_pts(&self, n: usize) -> usize {
        if self.use_pts != 0 {
            n.min(self.use_pts)
        } else {
            n
        }
    }
}

//ip CmdArgs - Argument handling
impl CmdArgs {
    //fp add_args_write_camera
    fn add_args_write_camera(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "write_camera",
            None,
            "File to write the final camera JSON to",
            None,
            CmdArgs::set_write_camera,
            false,
        );
    }

    //fp add_args_write_mapping
    fn add_args_write_mapping(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "write_mapping",
            None,
            "File to write a derived mapping JSON to",
            None,
            CmdArgs::set_write_mapping,
            false,
        );
    }

    //fp add_args_write_polys
    fn add_args_write_polys(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "write_polys",
            None,
            "File to write a derived polynomials JSON to",
            None,
            CmdArgs::set_write_polys,
            false,
        );
    }

    //fp add_args_poly_degree
    fn add_args_poly_degree(build: &mut CommandBuilder<Self>) {
        build.add_arg_usize(
            "poly_degree",
            None,
            "Degree of polynomial to use for the lens calibration (5 for 50mm)",
            Some("5"),
            CmdArgs::set_poly_degree,
            false,
        );
    }

    //fp add_args_num_pts
    fn add_args_num_pts(build: &mut CommandBuilder<Self>) {
        build.add_arg_usize(
            "num_pts",
            Some('n'),
            "Number of points to use (from start of mapping); if not specified, use all",
            None,
            CmdArgs::set_use_pts,
            false,
        );
    }

    //fp add_args_yaw_min_max
    fn add_args_yaw_min_max(
        build: &mut CommandBuilder<Self>,
        min: Option<&'static str>,
        max: Option<&'static str>,
    ) {
        build.add_arg_f64(
            "min_yaw",
            None,
            "Minimim yaw to use for calibration in degrees",
            min,
            CmdArgs::set_yaw_min,
            false,
        );
        build.add_arg_f64(
            "max_yaw",
            None,
            "Maximim yaw to use for calibration",
            max,
            CmdArgs::set_yaw_max,
            false,
        );
    }

    //fp add_args_yaw_error
    fn add_args_yaw_error(build: &mut CommandBuilder<Self>) {
        build.add_arg_f64(
            "yaw_error",
            None,
            "Maximum relative error in yaw to permit a closest match for",
            Some("0.03"),
            CmdArgs::set_yaw_error,
            false,
        );
    }

    //fp add_args_within
    fn add_args_within(build: &mut CommandBuilder<Self>) {
        build.add_arg_f64(
            "within",
            None,
            "Only use catalog stars Within this angle (degrees) for mapping",
            Some("15"),
            CmdArgs::set_within,
            false,
        );
    }

    //fp add_args_closeness
    fn add_args_closeness(build: &mut CommandBuilder<Self>) {
        build.add_arg_f64(                                    "closeness", None,
                                    "Closeness (degrees) to find triangles of stars or degress for calc cal mapping, find stars, map_stars etc",
                                    Some("0.2"),
                                    CmdArgs::set_closeness,
                                                           false);
    }
    //fp add_args_star_mapping
    fn add_args_star_mapping(build: &mut CommandBuilder<Self>) {
        build.add_arg(
            Arg::new("star_mapping")
                .required(true)
                .help("File mapping sensor coordinates to catalog identifiers")
                .action(ArgAction::Set),
            Box::new(CmdArgs::arg_star_mapping),
        );
    }

    //fp add_args_star_catalog
    fn add_args_star_catalog(build: &mut CommandBuilder<Self>) {
        build.add_arg(
            Arg::new("star_catalog")
                .long("catalog")
                .required(true)
                .help("Star catalog to use")
                .action(ArgAction::Set),
            Box::new(CmdArgs::arg_star_catalog),
        );
    }

    //fp add_args_brightness
    fn add_args_brightness(build: &mut CommandBuilder<Self>) {
        build.add_arg_f32(
            "brightness",
            None,
            "Maximum brightness of stars to use in the catalog",
            Some("5.0"),
            CmdArgs::set_brightness,
            false,
        );
    }

    //fp arg_star_mapping
    fn arg_star_mapping(args: &mut CmdArgs, matches: &ArgMatches) -> Result<()> {
        let filename = matches.get_one::<String>("star_mapping").unwrap();
        let json = json::read_file(filename)?;
        args.star_mapping = StarMapping::from_json(&json)?;
        Ok(())
    }

    //fp arg_star_catalog
    fn arg_star_catalog(args: &mut CmdArgs, matches: &ArgMatches) -> Result<()> {
        let catalog_filename = matches.get_one::<String>("star_catalog").unwrap();
        let mut catalog = Catalog::load_catalog(&catalog_filename, 99.)?;
        catalog.derive_data();
        args.star_catalog = Some(Box::new(catalog));
        Ok(())
    }
}

//ip CmdArgs - Operations
impl CmdArgs {
    fn setup(&self) -> (NamedPointSet, PointMappingSet) {
        let v = self.mapping.as_ref().unwrap().get_xyz_pairings();
        let mut nps = NamedPointSet::default();
        let mut pms = PointMappingSet::default();

        //cb Add calibrations to NamedPointSet and PointMappingSet
        for (n, (model_xyz, pxy_abs)) in v.into_iter().enumerate() {
            let name = n.to_string();
            let color = [255, 255, 255, 255].into();
            nps.add_pt(name.clone(), color, Some(model_xyz), 0.);
            pms.add_mapping(&nps, &name, &pxy_abs, 0.);
        }
        (nps, pms)
    }

    //mp draw_image
    pub fn draw_image(&self, pts: &[ImagePt]) -> Result<()> {
        if self.read_img.is_empty() || self.write_img.is_none() {
            return Ok(());
        }
        let mut img = ImageRgb8::read_image(&self.read_img[0])?;
        for p in pts {
            p.draw(&mut img);
        }
        img.write(self.write_img.as_ref().unwrap())?;
        Ok(())
    }

    //mp show_step
    fn show_step<S>(&self, s: S)
    where
        S: std::fmt::Display,
    {
        if self.verbose {
            eprintln!("\n{s}");
        }
    }

    //mp if_verbose
    fn if_verbose<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        if self.verbose {
            f()
        }
    }

    //mp output_camera
    fn output_camera(&self) -> Result<()> {
        let s = self.camera.to_json()?;
        if let Some(filename) = &self.write_camera {
            let mut f = std::fs::File::create(filename)?;
            f.write_all(s.as_bytes())?;
        } else {
            println!("{s}");
        }
        Ok(())
    }

    //mp output_mapping
    fn output_mapping(&self) -> Result<()> {
        let s = self.mapping.as_ref().unwrap().clone().to_json()?;
        if let Some(filename) = &self.write_mapping {
            let mut f = std::fs::File::create(filename)?;
            f.write_all(s.as_bytes())?;
        } else {
            println!("{s}");
        }
        Ok(())
    }

    //mp output_star_mapping
    fn output_star_mapping(&self) -> Result<()> {
        let s = self.star_mapping.clone().to_json()?;
        if let Some(filename) = &self.write_mapping {
            let mut f = std::fs::File::create(filename)?;
            f.write_all(s.as_bytes())?;
        } else {
            println!("{s}");
        }
        Ok(())
    }

    //mp output_polynomials
    fn output_polynomials(&self) -> Result<()> {
        let s = self.camera.lens().polys().to_json()?;
        if let Some(filename) = &self.write_polys {
            let mut f = std::fs::File::create(filename)?;
            f.write_all(s.as_bytes())?;
        } else {
            println!("{s}");
        }
        Ok(())
    }
}

//ip CommandArgs for CmdArgs
impl CommandArgs for CmdArgs {
    type Error = ic_base::Error;
    type Value = ();
}

//a Calibrate
//fi calibrate_cmd
fn calibrate_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("calibrate")
        .about("Read image and draw crosses on grid coordinates")
        .long_about(
            "This uses the camera calibration JSON file in conjunction with a camera body/lens and focus distance to generate the correct focal length and tan-tan mapping for the lens as world-to-screen (and vice-versa) polynomials. The camera calibration JSON file includes 'mappings' that is a list of (grid xmm, grid ymm, x pixel, y pixel) tuples each being the mapping of a grid x,y to a frame pixel x,y on an image. If read and write imnages are provided then the immage is read and red crosses superimposed on the image at the post-calibrated points using the provided grid x,y points as sources (so they should align with the actual grid points on the image)")
        .version("0.1.0");

    let mut build = CommandBuilder::new(command, Some(Box::new(calibrate_fn)));

    ic_cmdline::camera::add_arg_calibration_mapping(&mut build, CmdArgs::set_mapping, true);
    ic_cmdline::image::add_arg_read_img(&mut build, CmdArgs::set_read_img, false, Some(1));
    ic_cmdline::image::add_arg_write_img(&mut build, CmdArgs::set_write_img, false);
    build
}

//fi calibrate_fn
fn calibrate_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let calibrate = cmd_args.mapping.as_ref().unwrap();

    let v = calibrate.get_pairings(&cmd_args.camera);
    let mut world_yaws = vec![];
    let mut camera_yaws = vec![];
    for (n, (grid, camera_rel_xyz, pxy_ry)) in v.iter().enumerate() {
        let camera_rel_txty: TanXTanY = camera_rel_xyz.into();
        let camera_rel_ry: RollYaw = camera_rel_txty.into();
        world_yaws.push(camera_rel_ry.yaw());
        camera_yaws.push(pxy_ry.yaw());
        cmd_args.if_verbose(|| {
            eprintln!(
                "{n} {grid} : {camera_rel_xyz} : {camera_rel_ry} : {pxy_ry} : camera_rel_ty {} : pxy_ty {}",
                camera_rel_ry.tan_yaw(),
                pxy_ry.tan_yaw()
            );
        });
    }

    let poly_degree = 5;
    let wts = polynomial::min_squares_dyn(poly_degree, &world_yaws, &camera_yaws);
    let stw = polynomial::min_squares_dyn(poly_degree, &camera_yaws, &world_yaws);

    let (max_sq_err, max_n, mean_err, mean_sq_err, variance_err) =
        polynomial::error_in_y_stats(&wts, &world_yaws, &camera_yaws);
    let sd_err = variance_err.sqrt();
    let max_err = max_sq_err.sqrt();
    eprintln!(" err: mean {mean_err:.4e} mean_sq {mean_sq_err:.4e} sd {sd_err:.4e} abs max {max_err:.4e} max_n {max_n}");

    if false {
        for i in 0..world_yaws.len() {
            let wy = world_yaws[i];
            let cy = camera_yaws[i];
            eprintln!(
                "{i} {wy} : {} : {cy} : {} : {wy}",
                wts.calc(wy),
                stw.calc(cy)
            );
        }
    }
    eprintln!(" wts: {wts:?}");
    eprintln!(" stw: {stw:?}");

    eprintln!("cal camera {}", cmd_args.camera);
    let mut camera = cmd_args.camera.clone();
    let mut camera_lens = camera.lens().clone();
    camera_lens.set_polys(LensPolys::new(stw, wts));
    camera.set_lens(camera_lens);
    cmd_args.camera = camera;
    let camera = &cmd_args.camera;

    //    let m: Point3D = camera.camera_xyz_to_world_xyz([0., 0., -calibrate.distance()].into());
    //    let w: Point3D = camera.world_xyz_to_camera_xyz([0., 0., 0.].into());
    //    eprintln!("Camera {camera} focused on {m} world origin in camera {w}");

    let pxys = calibrate.get_pxys();
    let mut pts = vec![];
    let n = 30;
    let n_f = n as f64;
    let c_f = n_f / 2.0;
    for y in 0..=n {
        let y_f = (y as f64 - c_f) * 10.;
        for x in 0..=n {
            let x_f = (x as f64 - c_f) * 10.;
            let pt: Point3D = [x_f, y_f, 0.].into();
            let rgba = [255, 255, 255, 255].into();
            pts.push((pt, rgba));
        }
    }

    if !cmd_args.read_img.is_empty() && cmd_args.write_img.is_some() {
        let mut img = ImageRgb8::read_image(&cmd_args.read_img[0])?;
        let c = &[255, 0, 0, 0].into();
        for p in pxys.into_iter() {
            img.draw_cross(p, 5.0, c);
        }
        for (p, c) in &pts {
            let mapped = camera.map_model(*p);
            if false {
                let xyz = camera.world_xyz_to_camera_xyz(*p);
                let txy = camera.world_xyz_to_camera_txty(*p);
                eprintln!("{mapped} {xyz} {txy} {p} {c:?}");
            }
            img.draw_cross(mapped, 5.0, c);
        }
        img.write(cmd_args.write_img.as_ref().unwrap())?;
    }
    Ok(())
}

//a Locate
//fi locate_cmd
fn locate_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("locate")
        .about("Determine an optimal location from a calibration description")
        .long_about(LOCATE_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(locate_fn)));
    ic_cmdline::camera::add_arg_calibration_mapping(&mut build, CmdArgs::set_mapping, true);
    CmdArgs::add_args_num_pts(&mut build);
    CmdArgs::add_args_write_camera(&mut build);

    build
}

//fi locate_fn
fn locate_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    //cb Reset the camera position and orientation, defensively
    cmd_args.camera.set_position([0., 0., 0.].into());
    cmd_args.camera.set_orientation(Quat::default());

    //cb Set up HashMaps and collections
    let (_nps, pms) = cmd_args.setup();
    let n = cmd_args.use_pts(pms.mappings().len());
    let closest_n: Vec<usize> = (0..n).into_iter().collect();

    //cb For required pairings, display data
    cmd_args.show_step("Using the following mappings ([n] [world] : [pxy] : [world_dir]");
    for pm_n in &closest_n {
        let pm = &pms.mappings()[*pm_n];
        let n = pm.name();
        let grid_xyz = pm.model();
        // Px Abs -> Px Rel -> TxTy -> lens mapping
        let pxy_abs = pm.screen();
        let txty = cmd_args.camera.px_abs_xy_to_camera_txty(pxy_abs);
        let grid_dir = txty.to_unit_vector();
        cmd_args.if_verbose(|| {
            eprintln!(">> {n} {grid_xyz} : {pxy_abs} : {grid_dir}",);
        });
    }

    //cb Create ModelLineSet
    let mut mls = ModelLineSet::new(&cmd_args.camera);

    for n0 in &closest_n {
        let pm0 = &pms.mappings()[*n0];
        let dir0 = cmd_args
            .camera
            .px_abs_xy_to_camera_txty(pm0.screen())
            .to_unit_vector();
        for n1 in &closest_n {
            if n0 == n1 {
                continue;
            }
            let pm1 = &pms.mappings()[*n1];
            let dir1 = cmd_args
                .camera
                .px_abs_xy_to_camera_txty(pm1.screen())
                .to_unit_vector();
            let cos_theta = dir0.dot(&dir1);
            let angle = cos_theta.acos();
            let _ = mls.add_line_of_models(pm0.model(), pm1.model(), angle);
        }
    }

    //cb Find best position given ModelLineSet
    // Find best location 'p' for camera
    let (best_cam_pos, e) = mls.find_best_min_err_location(&|p| p[2] > 0., 1000, 1000);
    cmd_args.if_verbose(|| {
        eprintln!("{best_cam_pos} {e}",);
    });

    cmd_args.camera.set_position(best_cam_pos);
    cmd_args.output_camera()?;
    Ok(())
}

//a Orient
//fi orient_cmd
fn orient_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("orient")
        .about("Determine an optimal orientation from a calibration description")
        .long_about(ORIENT_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(orient_fn)));
    ic_cmdline::camera::add_arg_calibration_mapping(&mut build, CmdArgs::set_mapping, true);
    CmdArgs::add_args_num_pts(&mut build);
    CmdArgs::add_args_write_camera(&mut build);

    build
}

//fi orient_fn
fn orient_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let calibrate = cmd_args.mapping.as_ref().unwrap();

    //cb Set up 'cam' as the camera; use its position (unless otherwise told?)
    let mut camera = cmd_args.camera.clone();
    camera.set_orientation(Quat::default());

    //cb Set up HashMaps and collections
    let n = cmd_args.use_pts(calibrate.len());
    let closest_n: Vec<usize> = (0..n).into_iter().collect();
    let (_nps, pms) = cmd_args.setup();

    //cb For required pairings, display data
    cmd_args.show_step("All the following mappings ([n] [world] : [pxy] : [world_dir]");
    cmd_args.if_verbose(|| {
        for pm in pms.mappings() {
            let n = pm.name();
            let grid_xyz = pm.model();
            // Px Abs -> Px Rel -> TxTy -> lens mapping
            let pxy_abs = pm.screen();
            let txty = camera.px_abs_xy_to_camera_txty(pxy_abs);
            let grid_dir = txty.to_unit_vector();
            eprintln!("{n} {grid_xyz} : {pxy_abs} : {grid_dir}",);
        }
    });

    //cb Find best orientation given position
    // We can get N model direction vectors given the camera position,
    // and for each we have a camera direction vector
    cmd_args.show_step("Derive orientations from *specified* mappings");
    let best_cam_pos = camera.position();
    let mut qs = vec![];

    for n0 in &closest_n {
        let pm0 = &pms.mappings()[*n0];
        let di_c = -camera
            .px_abs_xy_to_camera_txty(pm0.screen())
            .to_unit_vector();
        let di_m = (best_cam_pos - pm0.model()).normalize();
        let z_axis: Point3D = [0., 0., 1.].into();
        let qi_c: Quat = quat::rotation_of_vec_to_vec(&di_c.into(), &z_axis.into()).into();
        let qi_m: Quat = quat::rotation_of_vec_to_vec(&di_m.into(), &z_axis.into()).into();
        for n1 in &closest_n {
            if n0 == n1 {
                continue;
            }
            let pm1 = &pms.mappings()[*n1];
            let dj_c = -camera
                .px_abs_xy_to_camera_txty(pm1.screen())
                .to_unit_vector();
            let dj_m = (best_cam_pos - pm1.model()).normalize();

            let dj_c_rotated: Point3D = quat::apply3(qi_c.as_ref(), dj_c.as_ref()).into();
            let dj_m_rotated: Point3D = quat::apply3(qi_m.as_ref(), dj_m.as_ref()).into();

            let theta_dj_m = dj_m_rotated[0].atan2(dj_m_rotated[1]);
            let theta_dj_c = dj_c_rotated[0].atan2(dj_c_rotated[1]);
            let theta = theta_dj_m - theta_dj_c;
            let theta_div_2 = theta / 2.0;
            let cos_2theta = theta_div_2.cos();
            let sin_2theta = theta_div_2.sin();
            let q_z = Quat::of_rijk(cos_2theta, 0.0, 0.0, sin_2theta);

            // At this point, qi_m * di_m = (0,0,1)
            //
            // At this point, q_z.conj * qi_m * di_m = (0,0,1)
            //                q_z.conj * qi_m * dj_m = dj_c_rotated
            //
            let q = qi_c.conjugate() * q_z * qi_m;

            // dc_i === quat::apply3(q.as_ref(), di_m.as_ref()).into();
            // dc_j === quat::apply3(q.as_ref(), dj_m.as_ref()).into();
            //            eprintln!(
            //                "di_c==q*di_m? {di_c} ==? {:?}",
            //                quat::apply3(q.as_ref(), di_m.as_ref())
            //            );
            //            eprintln!(
            //                "dj_c==q*dj_m? {dj_c} ==? {:?}",
            //                quat::apply3(q.as_ref(), dj_m.as_ref())
            //            );
            cmd_args.if_verbose(|| {
                eprintln!("{q}");
            });

            qs.push((1., q.into()));
        }
    }

    cmd_args.show_step("Calculate average orientation");
    let qr: Quat = quat::weighted_average_many(qs.iter().copied()).into();
    camera.set_orientation(qr);
    cmd_args.if_verbose(|| {
        eprintln!("{camera}");
    });

    cmd_args.output_camera()?;
    Ok(())
}

//a Lens calibrate
//fi lens_calibrate_cmd
fn lens_calibrate_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("lens_calibrate")
        .about("From calibrate_from_grid")
        .long_about(LENS_CALIBRATE_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(lens_calibrate_fn)));

    ic_cmdline::camera::add_arg_calibration_mapping(&mut build, CmdArgs::set_mapping, true);
    CmdArgs::add_args_yaw_min_max(&mut build, Some("1.0"), Some("20.0"));
    CmdArgs::add_args_num_pts(&mut build);
    CmdArgs::add_args_poly_degree(&mut build);
    CmdArgs::add_args_write_polys(&mut build);

    build
}

//fi lens_calibrate_fn
fn lens_calibrate_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let calibrate = cmd_args.mapping.as_ref().unwrap();
    let camera = &cmd_args.camera;

    let yaw_range_min = cmd_args.yaw_min.to_radians();
    let yaw_range_max = cmd_args.yaw_max.to_radians();
    let num_pts = cmd_args.use_pts(calibrate.len());

    //cb Set up HashMaps and collections
    let (_nps, pms) = cmd_args.setup();

    //cb Calculate Roll/Yaw for each point given camera
    let mut world_yaws = vec![];
    let mut camera_yaws = vec![];
    for pm in pms.mappings().iter().take(num_pts) {
        let model_txty = camera.world_xyz_to_camera_txty(pm.model());
        let cam_txty = camera.px_abs_xy_to_camera_txty(pm.screen());

        let model_ry: RollYaw = model_txty.into();
        let cam_ry: RollYaw = cam_txty.into();

        if cam_ry.yaw() > yaw_range_max {
            continue;
        }

        if cam_ry.yaw() > yaw_range_min {
            world_yaws.push(model_ry.yaw());
            camera_yaws.push(cam_ry.yaw());
        }
    }

    //cb Calculate Polynomials for camera-to-world and vice-versa
    // encourage it to go through the origin
    for _ in 0..10 {
        world_yaws.push(0.);
        camera_yaws.push(0.);
    }

    let mut wts;
    let mut stw;
    loop {
        let n = world_yaws.len();
        stw = polynomial::min_squares_dyn(cmd_args.poly_degree, &camera_yaws, &world_yaws);
        stw[0] = 0.0;
        let (max_sq_err, max_n, mean_err, mean_sq_err, variance_err) =
            polynomial::error_in_y_stats(&stw, &camera_yaws, &world_yaws);
        let sd_err = variance_err.sqrt();
        let max_err = max_sq_err.sqrt();
        eprintln!(" {n} err: mean {mean_err:.4e} mean_sq {mean_sq_err:.4e} sd {sd_err:.4e} abs max {max_err:.4e} max_n {max_n}");

        let dmin = sd_err * 3.0;
        let dmax = sd_err * 3.0;
        let outliers = polynomial::find_outliers(&stw, &camera_yaws, &world_yaws, dmin, dmax);
        if outliers.is_empty() {
            break;
        }
        for n in outliers.iter().rev() {
            world_yaws.remove(*n);
            camera_yaws.remove(*n);
        }
    }

    let mut sensor_yaw = vec![];
    let mut world_yaw = vec![];
    for s in &camera_yaws {
        sensor_yaw.push(*s);
        world_yaw.push(stw.calc(*s));
    }
    wts = polynomial::min_squares_dyn(7, &world_yaw, &sensor_yaw);
    wts[0] = 0.0;

    let mut camera_lens = cmd_args.camera.lens().clone();
    camera_lens.set_polys(LensPolys::new(stw, wts));
    cmd_args.camera.set_lens(camera_lens);

    cmd_args.output_polynomials()?;

    Ok(())
}

//a Yaw plot
//fi yaw_plot_cmd
fn yaw_plot_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("yaw_plot")
        .about("Plot yaw")
        .long_about(YAW_PLOT_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(yaw_plot_fn)));

    ic_cmdline::camera::add_arg_calibration_mapping(&mut build, CmdArgs::set_mapping, true);
    CmdArgs::add_args_yaw_min_max(&mut build, Some("1.0"), Some("20.0"));
    CmdArgs::add_args_num_pts(&mut build);

    build
}

//fi yaw_plot_fn
fn yaw_plot_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let calibrate = cmd_args.mapping.as_ref().unwrap();
    let camera = &cmd_args.camera;
    let mut camera_linear = camera.clone();
    let mut lens_linear = camera.lens().clone();
    lens_linear.set_polys(LensPolys::default());
    camera_linear.set_lens(lens_linear);

    let yaw_range_min = cmd_args.yaw_min.to_radians();
    let yaw_range_max = cmd_args.yaw_max.to_radians();
    let num_pts = cmd_args.use_pts(calibrate.len());

    let f_world = |w: f64, s: f64| (w.to_degrees(), s.to_degrees());
    let f_rel_error = |w: f64, s: f64| (w.to_degrees(), s / w - 1.0);
    let plot_f = {
        if false {
            f_world
        } else {
            f_rel_error
        }
    };

    //cb Set up HashMaps and collections
    let (_nps, pms) = cmd_args.setup();

    //cb Calculate Error in yaw/Yaw for each point given camera
    let mut pts = [vec![], vec![], vec![], vec![]];
    for pm in pms.mappings().iter().take(num_pts) {
        let model_txty = camera_linear.world_xyz_to_camera_txty(pm.model());
        let cam_txty = camera_linear.px_abs_xy_to_camera_txty(pm.screen());

        let model_ry: RollYaw = model_txty.into();
        let cam_ry: RollYaw = cam_txty.into();

        if cam_ry.yaw() > yaw_range_max {
            continue;
        }

        if (model_ry.yaw() / cam_ry.yaw()) > 1.2 {
            continue;
        }
        let mut quad = 0;
        if cam_ry.cos_roll() < 0.0 {
            // X < 0
            quad += 1;
        }
        if cam_ry.sin_roll() < 0.0 {
            // Y < 0
            quad += 2;
        }
        if cam_ry.yaw() > yaw_range_min {
            pts[quad].push(plot_f(model_ry.yaw(), cam_ry.yaw()));
        }
    }

    //cb Plot 4 graphs for quadrants and one for the polynomial
    use poloto::prelude::*;
    use tagu::prelude::*;
    let theme = poloto::render::Theme::light();
    let theme = theme.append(tagu::build::raw(".poloto_scatter{stroke-width:3px;}"));
    let theme = theme.append(tagu::build::raw(
        ".poloto_text.poloto_legend{font-size:10px;}",
    ));
    let theme = theme.append(tagu::build::raw(
        ".poloto_line{stroke-dasharray:2;stroke-width:2;}",
    ));

    let plots = poloto::build::origin();

    let mut wts_poly_pts = vec![];
    for i in 0..=100 {
        let model_yaw = (i as f64) / 100.0 * (yaw_range_max - yaw_range_min) + yaw_range_min;
        let model_ry = RollYaw::of_yaw(model_yaw);
        let frame_ry = camera.ry_camera_to_ry_frame(model_ry);
        wts_poly_pts.push(plot_f(model_yaw, frame_ry.yaw()));
    }

    let mut stw_poly_pts = vec![];
    for i in 0..=400 {
        let frame_yaw = (i as f64) / 400.0 * (yaw_range_max - yaw_range_min) + yaw_range_min;
        let frame_ry = RollYaw::of_yaw(frame_yaw);
        let model_ry = camera.ry_frame_to_ry_camera(frame_ry);
        if model_ry.yaw() > yaw_range_min && model_ry.yaw() < yaw_range_max {
            stw_poly_pts.push(plot_f(model_ry.yaw(), frame_yaw));
        }
    }

    let mut linear_pts = vec![];
    let mut stereographic_pts = vec![];
    let mut equiangular_pts = vec![];
    let mut equisolid_pts = vec![];
    let mut orthographic_pts = vec![];
    for i in 0..=100 {
        let world_yaw = (i as f64) / 100.0 * (yaw_range_max - yaw_range_min) + yaw_range_min;
        linear_pts.push(plot_f(world_yaw, world_yaw));
        stereographic_pts.push(plot_f(world_yaw, ((world_yaw / 2.0).tan() * 2.0).atan()));
        equiangular_pts.push(plot_f(world_yaw, world_yaw.atan()));
        equisolid_pts.push(plot_f(world_yaw, (world_yaw / 2.0).sin().atan()));
        orthographic_pts.push(plot_f(world_yaw, world_yaw.sin().atan()));
    }

    let plot = poloto::build::plot("Linear");
    let plot = plot.line(linear_pts.iter());
    let plots = plots.chain(plot);

    let plot = poloto::build::plot("Stereographic");
    let plot = plot.line(stereographic_pts.iter());
    let plots = plots.chain(plot);

    let plot = poloto::build::plot("Equiangular");
    let plot = plot.line(equiangular_pts.iter());
    let plots = plots.chain(plot);

    //    let plot = poloto::build::plot("Equisolid");
    //    let plot = plot.line(equisolid_pts.iter());
    //    let plots = plots.chain(plot);

    let plot = poloto::build::plot("Orthographic");
    let plot = plot.line(orthographic_pts.iter());
    let plots = plots.chain(plot);

    let plot = poloto::build::plot("Camera wts mapping");
    let plot = plot.line(wts_poly_pts.iter());
    let plots = plots.chain(plot);

    let plot = poloto::build::plot("Camera stw mapping");
    let plot = plot.line(stw_poly_pts.iter());
    let plots = plots.chain(plot);

    let plot = poloto::build::plot("Quad x>0 y>0");
    let plot = plot.scatter(pts[0].iter());
    let plots = plots.chain(plot);
    let plot = poloto::build::plot("Quad x<0 y>0");
    let plot = plot.scatter(pts[1].iter());
    let plots = plots.chain(plot);
    let plot = poloto::build::plot("Quad x>0 y<0");
    let plot = plot.scatter(pts[2].iter());
    let plots = plots.chain(plot);
    let plot = poloto::build::plot("Quad x<0 y<0");
    let plot = plot.scatter(pts[3].iter());
    let plots = plots.chain(plot);

    let plot_initial = poloto::frame_build()
        .data(plots)
        // .build_and_label(("Relative Yaw Error v Yaw", "World yaw / °", "(w-c)/w"))
        .build_and_label(("Sensor yaw", "World yaw / °", "(w-c)/w"))
        .append_to(poloto::header().append(theme))
        .render_string()
        .map_err(|e| format!("{e:?}"))?;
    println!("{plot_initial}");

    Ok(())
}

//a Roll plot
//fi roll_plot_cmd
fn roll_plot_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("roll_plot")
        .about("Plot roll of model versus roll of camera")
        .long_about(ROLL_PLOT_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(roll_plot_fn)));

    ic_cmdline::camera::add_arg_calibration_mapping(&mut build, CmdArgs::set_mapping, true);
    CmdArgs::add_args_yaw_min_max(&mut build, Some("1.0"), Some("20.0"));
    CmdArgs::add_args_num_pts(&mut build);

    build
}

//fi roll_plot_fn
fn roll_plot_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let calibrate = cmd_args.mapping.as_ref().unwrap();
    let camera = &cmd_args.camera;

    let num_pts = cmd_args.use_pts(calibrate.len());

    //cb Set up HashMaps and collections
    let (_nps, pms) = cmd_args.setup();

    //cb Calculate Roll/Yaw for each point given camera
    let mut pts = vec![];
    for pm in pms.mappings().iter().take(num_pts) {
        let model_txty = camera.world_xyz_to_camera_txty(pm.model());
        let cam_txty = camera.px_abs_xy_to_camera_txty(pm.screen());

        let model_ry: RollYaw = model_txty.into();
        let cam_ry: RollYaw = cam_txty.into();

        pts.push((
            (cam_ry.yaw() - model_ry.yaw()).to_degrees(),
            (cam_ry.roll() - model_ry.roll()).to_degrees(),
        ));
    }

    //cb Plot 4 graphs for quadrants and one for the polynomial
    use poloto::build::PlotIterator;
    let plots = poloto::build::origin();
    let plot = poloto::build::plot("Roll ");
    let plot = plot.scatter(pts.iter());
    let plots = plots.chain(plot);

    let plot_initial = poloto::frame_build()
        .data(plots)
        .build_and_label(("Roll diff v Yaw diff", "Yaw C-W / °", "Roll C-W / °"))
        .append_to(poloto::header().light_theme())
        .render_string()
        .map_err(|e| format!("{e:?}"))?;
    println!("{plot_initial}");

    Ok(())
}

//a Grid image
fn grid_image_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("grid_image")
        .about("From calibrate_from_grid")
        .long_about(GRID_IMAGE_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(grid_image_fn)));

    ic_cmdline::camera::add_arg_calibration_mapping(&mut build, CmdArgs::set_mapping, true);
    ic_cmdline::image::add_arg_read_img(&mut build, CmdArgs::set_read_img, true, Some(1));
    ic_cmdline::image::add_arg_write_img(&mut build, CmdArgs::set_write_img, true);
    CmdArgs::add_args_num_pts(&mut build);
    build
}

//fi grid_image_fn
fn grid_image_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let calibrate = cmd_args.mapping.as_ref().unwrap();
    let camera = &cmd_args.camera;

    //cb Set up HashMaps and collections
    let num_pts = cmd_args.use_pts(calibrate.len());
    let (_nps, pms) = cmd_args.setup();

    //cb Create points for crosses for output image
    let mut pts = vec![];
    let n = 30;
    let n_f = n as f64;
    let c_f = n_f / 2.0;
    let rgba: Color = { [0, 0, 0, 255] }.into();
    for y in 0..=n {
        let y_f = (y as f64 - c_f) * 10.;
        for x in 0..=n {
            let x_f = (x as f64 - c_f) * 10.;
            let pt: Point3D = [x_f, y_f, 0.].into();
            pts.push((pt, rgba));
        }
    }
    let rgba: Color = { [100, 255, 100, 255] }.into();
    for pm in pms.mappings().iter().take(num_pts) {
        pts.push((pm.model(), rgba));
    }

    //cb Read source image and draw on it, write output image
    let pxys = calibrate.get_pxys();
    let mut img = ImageRgb8::read_image(&cmd_args.read_img[0])?;
    let c = &[255, 0, 0, 0].into();
    for p in pxys {
        img.draw_cross(p, 5.0, c);
    }
    for (p, c) in &pts {
        let mapped = camera.map_model(*p);
        if mapped[0] < -10000.0 || mapped[0] > 10000.0 {
            continue;
        }
        if mapped[1] < -10000.0 || mapped[1] > 10000.0 {
            continue;
        }
        img.draw_cross(mapped, 5.0, c);
    }
    img.write(cmd_args.write_img.as_ref().unwrap())?;

    Ok(())
}

//a Star subcommand with its commands
fn star_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("star")
        .about("Calibrate a lens using stars")
        .long_about(STAR_LONG_HELP);

    let mut build = CommandBuilder::<CmdArgs>::new(command, None);
    CmdArgs::add_args_closeness(&mut build);
    CmdArgs::add_args_star_mapping(&mut build);
    CmdArgs::add_args_star_catalog(&mut build);
    CmdArgs::add_args_brightness(&mut build);

    build.add_subcommand(star_show_mapping_cmd());
    build.add_subcommand(star_find_stars_cmd());
    build.add_subcommand(star_orient_cmd());
    build.add_subcommand(star_calibrate_desc_cmd());
    build.add_subcommand(star_update_mapping_cmd());

    build
}
//a Star find_initial_orientation
//fp star_find_stars_cmd
fn star_find_stars_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("find_stars")
        .about("Find a camera orientation using two star triangles from an image")
        .long_about(STAR_FIND_STARS_LONG_HELP);
    let mut build = CommandBuilder::new(command, Some(Box::new(star_find_stars_fn)));
    CmdArgs::add_args_write_camera(&mut build);
    build
}

//fp star_find_stars_fn
fn star_find_stars_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let closeness = cmd_args.closeness;

    cmd_args
        .star_catalog
        .as_mut()
        .unwrap()
        .set_filter(StarFilter::brighter_than(cmd_args.brightness));

    let angle_orientations = cmd_args.star_mapping.find_orientation_from_triangles(
        cmd_args.star_catalog.as_ref().unwrap(),
        &cmd_args.camera,
        closeness.to_radians() as f32,
    )?;
    let mut best_match = (angle_orientations[0].1, angle_orientations[0].0, usize::MAX);
    for (i, (x, q)) in angle_orientations.iter().enumerate() {
        cmd_args.camera.set_orientation(*q);
        let (num_unmapped, total_error) = cmd_args.star_mapping.update_star_mappings(
            cmd_args.star_catalog.as_ref().unwrap(),
            &cmd_args.camera,
            cmd_args.closeness,
            0.03,
            40.0,
        );
        let Ok(orientation) = cmd_args
            .star_mapping
            .find_orientation_from_all_mapped_stars(
                cmd_args.star_catalog.as_ref().unwrap(),
                &cmd_args.camera,
                10.0, // brightness,
            )
        else {
            continue;
        };
        cmd_args.camera.set_orientation(orientation);
        let (num_unmapped, total_error) = cmd_args.star_mapping.update_star_mappings(
            cmd_args.star_catalog.as_ref().unwrap(),
            &cmd_args.camera,
            cmd_args.closeness,
            0.03,
            40.0,
        );
        if num_unmapped < best_match.2 {
            best_match = (orientation, *x, num_unmapped);
        }
        eprintln!(
            "Candidate {i} {} unmapped {num_unmapped} total_error {total_error} {q}",
            x.to_degrees()
        );
    }
    eprintln!(
        "Using candidate with triangle angle error {} and {} unmapped stars out of {}",
        best_match.1.to_degrees(),
        best_match.2,
        cmd_args.star_mapping.mappings().len()
    );

    cmd_args.camera.set_orientation(best_match.0);
    //cmd_args.if_verbose(|| {
    eprintln!("{}", &cmd_args.camera);
    //});

    cmd_args.output_camera()?;
    Ok(())
}

//a Star orient
//fp star_orient_cmd
fn star_orient_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("orient")
        .about("Orient on all of the mapped stars")
        .long_about(STAR_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(star_orient_fn)));
    CmdArgs::add_args_write_camera(&mut build);
    build
}

//fp star_orient_fn
fn star_orient_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let brightness = cmd_args.brightness;
    let orientation = cmd_args
        .star_mapping
        .find_orientation_from_all_mapped_stars(
            cmd_args.star_catalog.as_ref().unwrap(),
            &cmd_args.camera,
            brightness,
        )?;
    cmd_args.camera.set_orientation(orientation);
    cmd_args.output_camera()?;
    Ok(())
}

//a Star update_star_mapping
//fp star_update_mapping_cmd
fn star_update_mapping_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("update_star_mapping")
        .about(
            "Generate an updated mapping of stars from the catalog to with ids frmom the catalog",
        )
        .long_about(STAR_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(star_update_mapping_fn)));
    CmdArgs::add_args_yaw_error(&mut build);
    CmdArgs::add_args_within(&mut build);
    CmdArgs::add_args_write_mapping(&mut build);
    build
}

//fp star_update_mapping_fn
fn star_update_mapping_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    cmd_args
        .star_catalog
        .as_mut()
        .unwrap()
        .set_filter(StarFilter::brighter_than(cmd_args.brightness));

    //cb Show the star mappings
    let (num_unmapped, total_error) = cmd_args.star_mapping.update_star_mappings(
        cmd_args.star_catalog.as_ref().unwrap(),
        &cmd_args.camera,
        cmd_args.closeness,
        cmd_args.yaw_error,
        cmd_args.within,
    );
    eprintln!(
        "{num_unmapped} stars were not mapped out of {}, total error of mapped stars {total_error:.4e}|",
        cmd_args.star_mapping.mappings().len(),
    );
    cmd_args.output_star_mapping()?;
    Ok(())
}

//a Star show_star_mapping
//fp star_show_mapping_cmd
fn star_show_mapping_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("show_star_mapping")
        .about("Show the mapped stars onto an output image")
        .long_about(STAR_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(star_show_mapping_fn)));
    ic_cmdline::image::add_arg_read_img(&mut build, CmdArgs::set_read_img, false, Some(1));
    ic_cmdline::image::add_arg_write_img(&mut build, CmdArgs::set_write_img, false);
    CmdArgs::add_args_within(&mut build);
    build
}

//fp star_show_mapping_fn
fn star_show_mapping_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let within = cmd_args.within;

    cmd_args
        .star_catalog
        .as_mut()
        .unwrap()
        .set_filter(StarFilter::brighter_than(cmd_args.brightness));

    //cb Show the star mappings
    let _ = cmd_args
        .star_mapping
        .show_star_mappings(cmd_args.star_catalog.as_ref().unwrap(), &cmd_args.camera);

    let mut mapped_pts = vec![];

    // Mark the points with blue-grey crosses
    cmd_args.star_mapping.img_pts_add_cat_index(
        cmd_args.star_catalog.as_ref().unwrap(),
        &cmd_args.camera,
        &mut mapped_pts,
        1,
        cmd_args.brightness,
    )?;

    // Mark the mapping points with small purple crosses
    cmd_args
        .star_mapping
        .img_pts_add_mapping_pxy(&mut mapped_pts, 0)?;

    // Mark the catalog stars with yellow Xs
    cmd_args.star_mapping.img_pts_add_catalog_stars(
        cmd_args.star_catalog.as_ref().unwrap(),
        &cmd_args.camera,
        &mut mapped_pts,
        within,
        2,
    )?;

    // Draw a circle of radius yaw_max
    for yaw in [5, 10, 15, 20, 25, 30, 35, 40, 45, 50] {
        let yaw = (yaw as f64).to_radians();
        for i in 0..3600 {
            let angle = ((i as f64) / 10.0).to_radians();
            let s = angle.sin();
            let c = angle.cos();
            let world_ry = RollYaw::from_roll_yaw(s, c, yaw);
            let world_txty = world_ry.into();
            let pxy = cmd_args.camera.camera_txty_to_px_abs_xy(&world_txty);
            mapped_pts.push((pxy, 3).into());
        }
    }

    cmd_args.draw_image(&mapped_pts)?;

    Ok(())
}

//a Star calibrate_desc
//fp star_calibrate_desc_cmd
fn star_calibrate_desc_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("calibrate_desc")
        .about("Generate a calibration description")
        .long_about(STAR_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(star_calibrate_desc_fn)));
    CmdArgs::add_args_write_mapping(&mut build);
    build
}

//fp star_calibrate_desc_fn
fn star_calibrate_desc_fn(cmd_args: &mut CmdArgs) -> Result<()> {
    let pc = cmd_args
        .star_mapping
        .create_calibration_mapping(cmd_args.star_catalog.as_ref().unwrap());
    cmd_args.set_mapping(pc)?;
    cmd_args.output_mapping()?;
    Ok(())
}

//a Main
//fi main
fn main() -> Result<()> {
    let command = Command::new("camera_calibrate")
        .about("Camera calibration tool")
        .long_about(CAMERA_CALIBRATE_LONG_HELP)
        .version("0.1.0");

    let mut build = CommandBuilder::new(command, None);
    ic_cmdline::add_arg_verbose(&mut build, CmdArgs::set_verbose);
    ic_cmdline::camera::add_arg_camera_database(&mut build, CmdArgs::set_cdb, false);
    ic_cmdline::camera::add_arg_camera(&mut build, CmdArgs::get_cdb, CmdArgs::set_camera, false);

    build.add_subcommand(calibrate_cmd());
    build.add_subcommand(locate_cmd());
    build.add_subcommand(orient_cmd());
    build.add_subcommand(lens_calibrate_cmd());
    build.add_subcommand(yaw_plot_cmd());
    build.add_subcommand(roll_plot_cmd());
    build.add_subcommand(grid_image_cmd());
    build.add_subcommand(star_cmd());
    //    build.add_subcommand(grid_calibrate_cmd());

    let mut cmd_args = CmdArgs::default();
    let mut command = build.main(true, true);
    command.execute_env(&mut cmd_args)?;
    Ok(())
}
