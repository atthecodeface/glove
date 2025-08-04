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
//! # Star mappings
//!
//! The library uses the notion of a *star mapping*; this is a mapping
//! from the pixel XY on the sensor of the (centre of the) image of a
//! star to a catalog ID within a star catalog. The actual star
//! mapping includes a fourth value, which is used to indicate how to
//! use it for finding orientations and so on. An initial *star
//! mapping* will be update once the orientation and calibration of a
//! camera have been performed; the initial *star mapping* only
//! requires the pixel XY of the star centres, and for six stars an
//! indication that they are bright enough and central enough to use
//! for initial operation. After the whole calibration process is
//! complete, all of the *star mapping* entries should associate a
//! real star catalog id with each pixel XY - but the tool/library
//! does these updates.
//!
//! # Finding an initial orientation
//!
//! The orientation of the camera can be determined from a perfect
//! lens calibration from a triangle of stars, using the angle betwen
//! them (i.e stars A, B and C, and the angle as seen by the camera
//! between A and B, between B and C, and between C and A).
//!
//! Given a non-perfect lens calibration (starting, for example with a
//! linear calibration) the will be error in the angles measured
//! between the stars; hence using a star catalog of reasonably bright
//! stars one may find multiple sets of triangles of stars that match,
//! to some error, the angles as measured from the camera.
//!
//! Given a *second* (independent) set of three stars, we can get a
//! second independent guess on the orientation of the camera.
//!
//! We can compare all the pairs of orientations derived from the
//! triangle candidates to see if they match - i.e. if q1 = q2, or q1
//! . q2' = (1,0,0,0). Hence multiplying the quaternion for the one
//! candidate orientation by the conjugate of the quaternion for the
//! other candidate s and using the value of the real term (cos of
//! angle of rotation about some axis) as a measure of how well
//! matched the quaternions are (the difference between this value and
//! the identity quaternion).
//!
//! The best pair of candidate triangles can be used to evaluate an
//! initial camera orientation (by averaging the two candidate
//! orientations).
//!
//! # Updating a star mapping
//!
//! Given a camera orientation (and some lens calibration), all of the
//! pixel XY values in a *star mapping* can be converted into real
//! world direction vectors. The closest star in the star catalog to a
//! direction vector, provided it is a close enough mapping, is
//! presumably the start that the pixel XY corresponds to, and so the
//! *star mapping* can be updated with the catalog id for that star.
//!
//! # Better camera orientation
//!
//! From a full *star mapping* - where each visible star on
//! the image is recorded (px,py) in the *star mapping* file *with* a
//! corresponding star catalog id, an updated camera orientation can
//! be calculated.
//!
//! For each catalog id in the *star mapping* we can retrieve the star
//! direction vector for the star in the catalog, as a camera relative
//! direction (ignoring the current camera orientation).
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
//! positions in the *star mapping* there are N*(N-1) quaternions).
//!
//! This quaternion provides the best guess for the orientation for
//! the camera.
//!
//! # Calibrating the lens
//!
//! A *star mapping* can be converted to a regular camera calibration
//! *mapping* quite simple. The former is a mapping from sensor pixel
//! XY coordinates to stars in a catalog; the latter is a mapping from
//! sensor pixel XY coordinates to camera-position-relative 3D
//! points. The actual mapping for a star could have the star at
//! infinity - so a unit direction vector for each star could be
//! calculated, and the star projected to a 3D position at an
//! arbitrarily large distance. However, given that the camera is
//! defined to be at (0,0,0), all the 3D points in the *mapping*
//! provide is a direction vector - so the corresponding *mapping* for
//! a *star mapping* is the pixel XY mapped to the 3D unit direcion
//! vector to the star in the catalog.
//!
//! Stars in a *star mapping* that do not identify a star in the
//! catalog are not written into a *mapping*.
//!
//! With a *mapping* file, the standard camera calibration methods (see elsewhere) can be used.
//!
//! # The calibration process
//!
//! The calibration process starts with a linear lens calibration, and
//! then is simply to find an initial orientation using six bright
//! stars near the centre of the image (as two triangles), then to
//! update the star mapping using this initial orientation to map a
//! reasonable number of stars (close to the centre usually), and then
//! reorient using these mapped stars.
//!
//! This initial stage, for a wide angle lens, may only work on stars
//! near the centre of the image where the mapping from sensor to
//! world space is approximately linear.
//!
//! Given some lens calibration and star mapping, though, valid for up
//! to a certain 'yaw' value (the angle around the centre for which
//! the calibration holds well), the proces is to generate an
//! orientation based on the calibration and star mapping; update the
//! star mapping to a slightly larger 'yaw' value, generate a 3D
//! *mapping* file, generate a lens calibration valid for this
//! slightly wider *yaw* value, and then update the star mapping.
//!
//! At this point there should be an improved lesn calibration and
//! star mapping, and those steps can be repeated to further widen the
//! 'yaw' value - i.e. to more accurately map more stars around the
//! centre of the image, in a wider region than before.
//!
//! The process is repeated until all of the stars on the image are
//! included for mapping; this should give a precise orientation, and
//! the most precise lens calibration.
//!

//a Modules
mod star_mapping;
pub use star_mapping::StarMapping;
