//a Documentation
//! # Camera calibration tool
//!
//! This tool is used to convert *mappings*, generated either by hand
//! or automatically, into lens calibrations.
//!
//! A *mapping* can be a correspondence betwen pixel XY values from an
//! image to 3D world positions (such as a graph paper grid placed
//! nominally in the Z=0 plane), or a *star mapping* that is a
//! correspondence between pixel XY values and known stars (from
//! hipparcos catalog, normally).
//!
//! A lens calibration is a pair of polynomials that map from apparent
//! sensor 'yaw' (the angle that a pixel XY is off-centre) to an
//! apparent world 'yaw' (the angle that the actual point or star is
//! offset from the direction of the image). This lens calibration
//! should be specific to the lens (in a removable lens camera) and
//! should apply to whatever camera body the lens is used in; the
//! distance that the lens was focused on *does* impact the
//! calibration, so images should alwayws be used with the same focus
//! distance.
//!
//! ## Grid calibration
//!
//! A photograph of graph paper should be taken, focused on
//! 'infinity', with as wide an aperture as possible, so that the
//! image fills the screen. Ideally the photograph should be straight
//! on to the graph paper, but this need not be precise. The location
//! of the camera relative to the image need not be recorded.
//!
//! In the base example used to develop the tool, the camera is
//! face-on to a grid (which is graph paper); the approximate
//! intersections of 550 grid lines were captured as sensor pixel
//! coordinates XY and graph paper mm XY pairings; the graph paper was
//! assumed to be at Z=0. Hence a *mapping* file from PXY to 'world'
//! 3D coordinates of (x,y,z) in mm could be produced.
//!
//! ## Grid calibration first step - locate the camera
//!
//! The first step is to locate the camera, given the camera body, an
//! uncalibrated lens (i.e. something with an identity sensor to world
//! mapping), and with a given focusing distance for
//! the image and *mapping*.
//!
//! This is performed using the 'locate' command, using the first few
//! N (e.g. N=6) mapping points (which should not all be in a line,
//! and should be near the centre of the image ideally); it converts
//! the pixel XY values for these mappings into approximate world
//! directions, assuming a linear lens mapping, and hence it estimates
//! the angular delta between every N^2-1 pairs; for every pair of
//! points in world space and an angular delta for that pair there is
//! a locus of points (a surface) in world space where for all the
//! locus points see that angle when looking at the pair. Any three
//! mapping pairs will provide three loci (each of which is a surface)
//! and the intersection of three surfaces is a single point - which
//! must be the location of the camera. Hence an initial approximation
//! for the location of the camera can be determined using three pairs
//! from the N^2-1 pairs; this can be improved by using all N^2-1
//! pairs to find a location of least error.
//!
//! Note that for this to operate well the first N points in the
//! *mapping* should be near the centre of the image (maybe 1/4 of the
//! height of the image from the centre), and spread roughly evenly
//! around the centre (perhaps in a circle).
//!
//! ## Camera location determination, programmatically
//!
//! The first 'N' mappings are used to create a [ModelLineSet], which is
//! a set of ModelLines (between grid points, hence all in the plane
//! Z=0) and the angles subtended by the camera, given by the sensor
//! pixel coordinates through the camera/lens model from the database
//! (i.e. this includes the lens mapping)
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
//! *mappings* (or for the first N pairings) by:
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
//! # Refinement
//!
//! The initial location of the camera uses an incorrect lens
//! calibration, and this is also used for an initial orientation.
//! From these values a first lens calibration can be determined.
//!
//! The lens calibration can be refined by repeating the process, this
//! time using the lens calibration to generate a better location for
//! the camera, and hence a better orientation and new lens
//! calibration.
//!
//! Ths *first* stage can use a small value of 'N' for the camera
//! positioning - a value of 6 is quite fast; the final calibration
//! should use a much larger value of 'N' (and the mappings should be
//! provided so that the first 'N' values are not heavily biased to
//! one sector of the image).
//!
//! # Generating a grid test image
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

use clap::Command;
use thunderclap::CommandBuilder;

use ic_base::Result;

//a Mods
mod cmd;
pub use cmd::{cmd_ok, CmdArgs, CmdResult};
mod calibration;
mod image_analyze;
mod image_process;
mod named_points;
mod point_mapping;
mod project;
mod star;

//a Main
//fi main
fn main() -> Result<()> {
    let command = Command::new("photogram")
        .about("Photogrammetry tool")
        .version("0.1.0");

    let mut build = CommandBuilder::new(command, None);

    CmdArgs::add_arg_verbose(&mut build);

    // Camera database and project are really mutually exclusive, as the project includes a camera database
    CmdArgs::add_arg_camera_database(&mut build, false);
    CmdArgs::add_arg_project(&mut build, false);
    CmdArgs::add_arg_camera(&mut build, false);

    build.add_subcommand(project::project_cmd());
    build.add_subcommand(image_process::image_process_cmd());
    build.add_subcommand(image_analyze::image_analyze_cmd());
    build.add_subcommand(star::star_cmd());
    build.add_subcommand(calibration::calibration_cmd());
    build.add_subcommand(point_mapping::cip_cmd());
    build.add_subcommand(named_points::named_points_cmd());

    let mut cmd_args = CmdArgs::default();
    let mut command = build.main(true, true);
    command.execute_env(&mut cmd_args)?;
    Ok(())
}
