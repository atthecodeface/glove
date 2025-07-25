//a Documentation
//! Allow for calibrating using stars (and a Star Catalog)
//!
//! This is expected to use camera images taken at night on (e.g.) a
//! 30 second exposure at f/2 at ISO 400, or thereabouts. Shorter
//! exposures may be used with higher ISO, or to record just brighter
//! stars
//!
//! The notion is that the *position* of the camera is fixed (as 'on
//! the earth') and the stars are sufficiently distant that where one
//! is on the earth is irrelevant. Hence the camera orientation is all
//! that needs to be determined, in order to then calibrate the camera
//! and lens.
//!
//! # Finding an initial orientation
//!
//! The orientation of the camera can be determined from a perfect
//! lens calibration from a triangle of stars, using the angle betwen
//! them (i.e stars A, B and C, and the angle as seen by the camera
//! between A and B, between B and C, and between C and A).
//!
//! Given a non-perfect lens calibration (as one starts off with...!)
//! the will be error in the angles measured between the stars; hence
//! using a star catalog of reasonably bright stars one may find
//! multiple sets of triangles of stars that match, to some error, the
//! angles as measured from the camera.
//!
//! Given a *second* (independent) set of three stars, we can get a
//! second independent guess on the orietnation of the camera.
//!
//! We can compare all the pairs of orientations derived from the
//! triangle candidates to see if they match - i.e. if q1 = q2, or q1
//! . q2' = (1,0,0,0). Hence multiplying the quaternion for the one
//! candidate orientation by the conjugate of the quaternion for the
//! other candidate s and using the value of the real term (cos of
//! angle of rotation about some axis) as a measure of how well
//! matched the quaternions are (the difference between this value and
//! one).
//!
//! The best pair of candidate triangles can be used to evaluate a
//! camera orientations (by averaging the two candidate orientations);
//! using this orientation, we can find the direction to the other
//! stars in the calibration description; from these we can find the
//! closest stars in the catalog for these directions, and update the
//! calibration description with such mappings.
//!
//! # Mapping stars on an image
//!
//! From a full calibration description - where each visible star on
//! the image is recorded (px,py) in the calibration file *with* a
//! corresponding star catalog id, an updated camera orientation can
//! be calculated.
//!
//! For each position in the calibration file we know its catalog
//! direction vector, and we can calculate a camera relative direction
//! vector (i.e. ignoreing the camera orientation).
//!
//! For each pair of star positions A and B we can thus generate a
//! quaternion that maps camera relative direction A to star catalog
//! direction A *and* that maps camera relative direction B to
//! (approximately) star catalog direction B. The approximation comes
//! from the errors in the sensor image positions and the camera
//! calibration, which lead to an error in the camera-relative angle
//! between A and B.
//!
//! We can calculate quaternions for every pair (A,B) - including the
//! reverse (B,A), but not (A,A) - and gnerate an average (so for N
//! positions in the calibration file there are N*(N-1) quaternions).
//!
//! This quaternion provides the best guess for the orientation for
//! the camera.

//a Modules
mod star_mapping;
pub use star_mapping::StarMapping;
